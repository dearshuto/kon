use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::algorithm::{pruning_decorators::ITraverseDecorator, LiveInfo};
use crate::algorithm::{IScheduleCallback, RoomMatrix, TraverseOperation};
use crate::{BandId, BlockId, RoomId};

use super::permutation_treverser::PermutationTraverser;

#[allow(unused)]
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
    #[allow(unused)]
    pub fn new(decorator: TDecorator, callback: TCallback) -> Self {
        Self {
            decorator: Arc::new(decorator),
            callback: Arc::new(Mutex::new(callback)),
        }
    }

    #[allow(unused)]
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

    #[allow(unused)]
    pub async fn assign_async(
        &mut self,
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
        let mut traverer = PermutationTraverser::new(band_count, band_count);
        let mut sub_tree = traverer.allocate().unwrap();

        let mut task = Vec::new();
        while let Some(permutation) = sub_tree.next() {
            let decorator_local = self.decorator.clone();
            // let callback_local = self.callback.clone();
            let room_matrix_local = room_matrix.clone();
            let live_info_local = live_info.clone();
            let handle = tokio::spawn(async move {
                decorator_local.invoke_with_room_matrix(
                    permutation.current(),
                    &room_matrix_local,
                    &live_info_local,
                )
            });
            task.push(handle);

            // タスクが 64 個以上になったらどれか終わるまで待つ
            while 64 < task.len() {
                tokio::time::sleep(Duration::from_millis(1000));

                for index in (0..task.len()).rev() {
                    if !task[index].is_finished() {
                        continue;
                    }

                    let finished_task = task.swap_remove(index);
                    let traverse_operation = finished_task.await.unwrap();
                    match traverse_operation {
                        TraverseOperation::Next => {
                            // ここで部屋割に変換する
                        }
                        TraverseOperation::Pruning => {
                            // もう走査しても結果が得られないのでここで打ち切る
                            break;
                        }
                        TraverseOperation::Skip(index) => {
                            // 部分木内の特定の部分木で可能性がなくなったので別の部分木まで飛ばす
                            sub_tree.skip(index)
                        }
                    }
                }
            }
        }

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
