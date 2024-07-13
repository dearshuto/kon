use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use crate::algorithm::{pruning_decorators::ITraverseDecorator, LiveInfo};
use crate::algorithm::{IScheduleCallback, RoomMatrix, TraverseOperation};
use crate::{BandId, BlockId, RoomId};

use super::permutation_treverser::PermutationTraverser;
use super::PartialPermutation;

pub struct SchedulerImpl<
    TDecorator: ITraverseDecorator + Send + Sync + 'static,
    TCallback: IScheduleCallback + Send + Sync + 'static,
> {
    decorator: Arc<TDecorator>,
    callback: Arc<Mutex<TCallback>>,
}

impl<TDecorator, TCallback> SchedulerImpl<TDecorator, TCallback>
where
    TDecorator: ITraverseDecorator + Send + Sync + 'static,
    TCallback: IScheduleCallback + Send + Sync + 'static,
{
    pub fn new(decorator: TDecorator, callback: TCallback) -> Self {
        Self {
            decorator: Arc::new(decorator),
            callback: Arc::new(Mutex::new(callback)),
        }
    }

    pub fn assign(
        &self,
        room_matrix: &RoomMatrix,
        live_info: &LiveInfo,
    ) -> Result<HashMap<BandId, BlockId>, ()> {
        // そもそも部屋数が足りてなければ失敗
        let available_rooms = room_matrix.blocks().len();
        if available_rooms < live_info.band_ids().len() {
            let mut callback = self.callback.lock().unwrap().on_completed();
            return Err(());
        }

        // スケジュールの全組み合わせを調査
        let band_count = live_info.band_ids().len();
        let mut traverer = PermutationTraverser::new(band_count, band_count);
        let mut sub_tree = traverer.allocate().unwrap();

        while let Some(permutation) = sub_tree.next() {
            let traverse_operation = self.decorator.invoke_with_room_matrix(
                permutation.current(),
                room_matrix,
                live_info,
            );

            match traverse_operation {
                TraverseOperation::Next => {
                    let mut callback = self.callback.lock().unwrap();
                    let table = Self::convert(permutation.current(), room_matrix, live_info);
                    callback.on_assigned(&table, live_info);
                }
                TraverseOperation::Pruning => {
                    break;
                }
                TraverseOperation::Skip(index) => sub_tree.skip(index),
            }
        }

        self.callback.lock().unwrap().on_completed();

        Ok(Default::default())
    }

    pub async fn assign_async(
        &self,
        room_matrix: Arc<RoomMatrix>,
        live_info: Arc<LiveInfo>,
    ) -> Result<HashMap<BandId, RoomId>, ()> {
        // そもそも部屋数が足りてなければ失敗
        let available_rooms = room_matrix.blocks().len();
        if available_rooms < live_info.band_ids().len() {
            return Err(());
        }

        // スケジュールの全組み合わせを調査
        let band_count = live_info.band_ids().len();
        let mut traverer = PermutationTraverser::new(band_count, band_count.min(8));
        let _current_head = Arc::new(RwLock::new(PartialPermutation::new(
            band_count,
            band_count.min(8),
        )));

        let mut task_queue = TaskQueue::new(64);
        while let Some(mut sub_tree) = traverer.allocate() {
            let decorator_local = self.decorator.clone();
            let room_matrix_local = room_matrix.clone();
            let live_info_local = live_info.clone();

            let handle = tokio::spawn(async move {
                let mut results = Vec::new();
                while let Some(permutation) = sub_tree.next() {
                    let operation = decorator_local.invoke_with_room_matrix(
                        permutation.current(),
                        &room_matrix_local,
                        &live_info_local,
                    );

                    match operation {
                        TraverseOperation::Next => results.push(permutation),
                        TraverseOperation::Pruning => return results,
                        TraverseOperation::Skip(index) => sub_tree.skip(index),
                    }
                }

                return results;
            });

            let results = task_queue.push_task(handle).await;
            {
                let mut callback = self.callback.lock().unwrap();
                for result in results {
                    for permutation in result {
                        let table = Self::convert(permutation.current(), &room_matrix, &live_info);
                        callback.on_assigned(&table, &live_info);
                    }
                }
            }
        }

        let results = task_queue.wait().await;
        {
            let mut callback = self.callback.lock().unwrap();
            for result in results {
                for permutation in result {
                    let table = Self::convert(permutation.current(), &room_matrix, &live_info);
                    callback.on_assigned(&table, &live_info);
                }
            }
        }

        self.callback.lock().unwrap().on_completed();

        Ok(Default::default())
    }

    fn convert(
        indicies: &[i32],
        room_matrix: &RoomMatrix,
        live_info: &LiveInfo,
    ) -> HashMap<BandId, BlockId> {
        (0..room_matrix.blocks().len())
            .map(|index| {
                let actual_index = indicies[index] as usize;
                let band_id = live_info.band_ids()[actual_index];
                let block_id = room_matrix.blocks()[index];
                (band_id, block_id)
            })
            .collect()
    }
}

struct TaskQueue<T> {
    tasks: Vec<JoinHandle<T>>,
    task_count_max: usize,
}

impl<T> TaskQueue<T> {
    pub fn new(max: usize) -> Self {
        Self {
            tasks: Vec::default(),
            task_count_max: max,
        }
    }

    pub async fn push_task(&mut self, handle: JoinHandle<T>) -> Vec<T> {
        if self.tasks.len() < self.task_count_max {
            self.tasks.push(handle);
            return Vec::default();
        }

        let results = self.wait_until(self.task_count_max).await;
        self.tasks.push(handle);
        results
    }

    pub async fn wait(&mut self) -> Vec<T> {
        self.wait_until(0).await
    }

    async fn wait_until(&mut self, count: usize) -> Vec<T> {
        let mut results = Vec::default();
        while count < self.tasks.len() {
            tokio::time::sleep(Duration::from_millis(50)).await;

            for index in (0..self.tasks.len()).rev() {
                if !self.tasks[index].is_finished() {
                    continue;
                }

                let finished_task = self.tasks.swap_remove(index);
                results.push(finished_task.await.unwrap());
            }
        }

        results
    }
}
