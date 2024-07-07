use std::collections::HashMap;

use crate::algorithm::pruning_decorators::{
    BandScheduleTraverseDecorator, MemberConflictTraverseDecorator, TreeTraverser,
};
use crate::algorithm::{pruning_decorators::ITraverseDecorator, ITreeCallback, LiveInfo};
use crate::algorithm::{traverse_all, RoomMatrix, TraverseOperation};
use crate::{BandId, RoomId};

#[allow(unused)]
pub struct SchedulerImpl<TDecorator: ITraverseDecorator, TCallback: ITreeCallback> {
    decorator: TDecorator,
    callback: TCallback,
}

impl<TDecorator, TCallback> SchedulerImpl<TDecorator, TCallback>
where
    TDecorator: ITraverseDecorator,
    TCallback: ITreeCallback,
{
    #[allow(unused)]
    pub fn new(decorator: TDecorator, callback: TCallback) -> Self {
        Self {
            decorator,
            callback,
        }
    }

    #[allow(unused)]
    pub fn assign(
        &mut self,
        room_matrix: &RoomMatrix,
        live_info: &LiveInfo,
    ) -> Result<HashMap<BandId, RoomId>, ()> {
        // そもそも部屋数が足りてなければ失敗
        let available_rooms = room_matrix.blocks().len();
        if available_rooms < live_info.band_ids().len() {
            return Err(());
        }

        // スケジュールの全組み合わせを調査
        let band_count = live_info.band_ids().len();
        let mut band_indicies: Vec<i32> = (0..band_count.max(available_rooms) as i32).collect();

        let decorator = TreeTraverser::default();
        let decorator = BandScheduleTraverseDecorator::new(decorator);
        let decorator = MemberConflictTraverseDecorator::new(decorator);
        let mut callback = TreeCallbackAdapter::new(decorator, room_matrix, live_info);
        traverse_all(&mut band_indicies, &mut callback);

        Ok(callback.result())
    }
}

struct TreeCallbackAdapter<'a, TDecorator: ITraverseDecorator> {
    decorator: TDecorator,

    room_matrix: &'a RoomMatrix,

    live_info: &'a LiveInfo,

    traverse_result: HashMap<BandId, RoomId>,
}

impl<'a, TDecorator: ITraverseDecorator> TreeCallbackAdapter<'a, TDecorator> {
    pub fn new(
        decorator: TDecorator,
        room_matrix: &'a RoomMatrix,
        live_info: &'a LiveInfo,
    ) -> Self {
        Self {
            decorator,
            room_matrix,
            live_info,
            traverse_result: Default::default(),
        }
    }

    pub fn result(self) -> HashMap<BandId, RoomId> {
        self.traverse_result
    }
}

impl<'a, TDecorator: ITraverseDecorator> ITreeCallback for TreeCallbackAdapter<'a, TDecorator> {
    fn invoke(&mut self, indicies: &[i32]) -> TraverseOperation {
        let operation =
            self.decorator
                .invoke_with_room_matrix(indicies, self.room_matrix, self.live_info);

        match operation {
            TraverseOperation::Next => {
                // TODO; ここで結果を格納する
            }
            _ => {}
        };

        operation
    }
}
