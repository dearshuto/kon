use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use crate::algorithm::{IScheduleCallback, LiveInfo, RoomMatrix, TraverseOperation};
use crate::{BandId, BlockId, RoomId};

use super::permutation_treverser::PermutationTraverser;
use super::pruning_decorators::ITraverseDecorator;
use super::PartialPermutation;

pub struct SchedulerImplBuilder {
    task_count_max: usize,
    sub_tree_depth: usize,
}

impl SchedulerImplBuilder {
    pub fn build<TDecorator, TCallback>(
        self,
        decorator: TDecorator,
        callback: TCallback,
    ) -> SchedulerImpl<TDecorator, TCallback>
    where
        TDecorator: ITraverseDecorator + Send + Sync + Clone + 'static,
        TCallback: IScheduleCallback + Send + Sync + Clone + 'static,
    {
        SchedulerImpl::new(
            decorator,
            callback,
            self.task_count_max,
            self.sub_tree_depth,
        )
    }

    pub fn with_task_count_max(mut self, count: usize) -> Self {
        self.task_count_max = count;
        self
    }

    pub fn with_sub_tree_depth(mut self, depth: usize) -> Self {
        self.sub_tree_depth = depth;
        self
    }
}

pub struct SchedulerImpl<
    TDecorator: ITraverseDecorator + Send + Sync + Clone + 'static,
    TCallback: IScheduleCallback + Send + Sync + Clone + 'static,
> {
    decorator: TDecorator,
    callback: TCallback,
    task_count_max: usize,
    sub_tree_depth: usize,
}

impl<TDecorator, TCallback> SchedulerImpl<TDecorator, TCallback>
where
    TDecorator: ITraverseDecorator + Send + Sync + Clone + 'static,
    TCallback: IScheduleCallback + Send + Sync + Clone + 'static,
{
    pub fn builder() -> SchedulerImplBuilder {
        SchedulerImplBuilder {
            task_count_max: 64,
            sub_tree_depth: 8,
        }
    }

    fn new(
        decorator: TDecorator,
        callback: TCallback,
        task_count_max: usize,
        sub_tree_depth: usize,
    ) -> Self {
        Self {
            decorator,
            callback,
            task_count_max,
            sub_tree_depth,
        }
    }

    pub fn assign(
        &mut self,
        room_matrix: &RoomMatrix,
        live_info: &LiveInfo,
    ) -> Result<HashMap<BandId, BlockId>, ()> {
        // そもそも部屋数が足りてなければ失敗
        let available_rooms = room_matrix.blocks().len();
        if available_rooms < live_info.band_ids().len() {
            self.callback.on_completed();
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
                    let table = Self::convert(permutation.current(), room_matrix, live_info);
                    self.callback.on_assigned(&table, live_info);
                }
                TraverseOperation::Pruning => {
                    break;
                }
                TraverseOperation::Skip(index) => sub_tree.skip(index),
            }
        }

        self.callback.on_completed();

        Ok(Default::default())
    }

    pub async fn assign_async<TRoomMatrix, TLiveInfo>(
        &mut self,
        room_matrix: TRoomMatrix,
        live_info: TLiveInfo,
    ) -> Result<HashMap<BandId, RoomId>, ()>
    where
        TRoomMatrix: AsRef<RoomMatrix> + Sync + Send + Clone + 'static,
        TLiveInfo: AsRef<LiveInfo> + Sync + Send + Clone + 'static,
    {
        // そもそも部屋数が足りてなければ失敗
        let available_rooms = room_matrix.as_ref().blocks().len();
        if available_rooms < live_info.as_ref().band_ids().len() {
            return Err(());
        }

        // スケジュールの全組み合わせを調査
        let band_count = live_info.as_ref().band_ids().len();
        let mut traverer =
            PermutationTraverser::new(band_count, band_count.min(self.sub_tree_depth));
        let _current_head = Arc::new(RwLock::new(PartialPermutation::new(
            band_count,
            band_count.min(self.sub_tree_depth),
        )));

        let mut task_queue = TaskQueue::new(self.task_count_max);
        while let Some(mut sub_tree) = traverer.allocate() {
            let decorator_local = self.decorator.clone();
            let room_matrix_local = room_matrix.clone();
            let live_info_local = live_info.clone();

            let handle = tokio::spawn(async move {
                let mut results = Vec::new();
                while let Some(permutation) = sub_tree.next() {
                    let operation = decorator_local.invoke_with_room_matrix(
                        permutation.current(),
                        room_matrix_local.as_ref(),
                        live_info_local.as_ref(),
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
                for result in results {
                    for permutation in result {
                        let table = Self::convert(
                            permutation.current(),
                            room_matrix.as_ref(),
                            live_info.as_ref(),
                        );
                        self.callback.on_assigned(&table, live_info.as_ref());
                    }
                }
            }
        }

        let results = task_queue.wait().await;
        {
            for result in results {
                for permutation in result {
                    let table = Self::convert(
                        permutation.current(),
                        room_matrix.as_ref(),
                        live_info.as_ref(),
                    );
                    self.callback.on_assigned(&table, live_info.as_ref());
                }
            }
        }

        self.callback.on_completed();

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
