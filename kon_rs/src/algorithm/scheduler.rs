use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use uuid::Uuid;

use crate::{BandId, BlockId};

use super::detail::{
    BandScheduleTraverseDecorator, MemberConflictTraverseDecorator, TreeTraverser,
};
use super::{detail::SchedulerImpl, LiveInfo, RoomMatrix};

pub struct SchedulerInfo {
    /// 走査総数
    pub count: usize,
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct TaskId {
    uuid: Uuid,
}

impl TaskId {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
        }
    }
}

pub struct TaskInfo {}

pub trait IScheduleCallback {
    fn on_started(&mut self, _scheduler_info: &SchedulerInfo);

    fn on_progress(&mut self, _task_id: TaskId, _task_info: &TaskInfo);

    fn on_assigned(
        &mut self,
        table: &HashMap<BlockId, BandId>,
        room_matrix: &RoomMatrix,
        live_info: &LiveInfo,
    );

    fn on_completed(&mut self);
}

pub struct Scheduler<T> {
    callback: T,
}

impl Scheduler<()> {
    pub fn new() -> Self {
        Self { callback: () }
    }

    pub fn assign(
        &self,
        room_matrix: &RoomMatrix,
        live_info: &LiveInfo,
    ) -> Vec<HashMap<BlockId, BandId>> {
        // 枝刈り
        let decorator = TreeTraverser::default();
        let decorator = BandScheduleTraverseDecorator::new(decorator);
        let decorator = MemberConflictTraverseDecorator::new(decorator);

        let schedule_callback = Arc::new(Mutex::new(ScheduleCallbackMock::new()));
        let mut scheduler_impl = SchedulerImpl::new(decorator, Arc::clone(&schedule_callback));
        let _ = scheduler_impl.assign(room_matrix, live_info);

        let x = schedule_callback.lock().unwrap().assigned.clone();
        x
    }

    pub async fn assign_async(
        &self,
        room_matrix: Arc<RoomMatrix>,
        live_info: Arc<LiveInfo>,
    ) -> Vec<HashMap<BlockId, BandId>> {
        // 枝刈り
        let decorator = TreeTraverser::default();
        let decorator = BandScheduleTraverseDecorator::new(decorator);
        let decorator = MemberConflictTraverseDecorator::new(decorator);

        let schedule_callback = Arc::new(Mutex::new(ScheduleCallbackMock::new()));
        let mut scheduler_impl = SchedulerImpl::new(decorator, Arc::clone(&schedule_callback));
        let _ = scheduler_impl.assign_async(room_matrix, live_info).await;

        let x = schedule_callback.lock().unwrap().assigned.clone();
        x
    }
}

impl<T>
    Scheduler<
        SchedulerImpl<
            MemberConflictTraverseDecorator<BandScheduleTraverseDecorator<TreeTraverser>>,
            T,
        >,
    >
where
    T: IScheduleCallback + Send + Sync + Clone + 'static,
{
    pub fn new_with_callback(callback: T) -> Self {
        // 枝刈り
        let decorator = TreeTraverser::default();
        let decorator = BandScheduleTraverseDecorator::new(decorator);
        let decorator = MemberConflictTraverseDecorator::new(decorator);

        let scheduler_impl = SchedulerImpl::new(decorator, callback);
        Self {
            callback: scheduler_impl,
        }
    }

    pub fn assign(&mut self, room_matrix: &RoomMatrix, live_info: &LiveInfo)
    where
        T: IScheduleCallback + Send + Sync + 'static,
    {
        let _ = self.callback.assign(room_matrix, live_info);
    }

    // #[cfg(not(target_arch = "wasm32"))]
    pub async fn assign_async(
        &mut self,
        room_matrix: Arc<RoomMatrix>,
        live_info: Arc<LiveInfo>,
        sub_tree_depth: usize,
        task_count: usize,
    ) {
        let _ = self
            .callback
            .assign_async_with_params(room_matrix, live_info, sub_tree_depth, task_count)
            .await;
    }
}

struct ScheduleCallbackMock {
    assigned: Vec<HashMap<BlockId, BandId>>,
}

impl ScheduleCallbackMock {
    pub fn new() -> Self {
        Self {
            assigned: Default::default(),
        }
    }
}

impl IScheduleCallback for Arc<Mutex<ScheduleCallbackMock>> {
    fn on_started(&mut self, _scheduler_info: &SchedulerInfo) {}

    fn on_progress(&mut self, _task_id: TaskId, _task_info: &TaskInfo) {}

    fn on_assigned(
        &mut self,
        table: &HashMap<BlockId, BandId>,
        _room_matrix: &RoomMatrix,
        _live_info: &LiveInfo,
    ) {
        self.lock().unwrap().assigned.push(table.clone());
    }

    fn on_completed(&mut self) {}
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use crate::algorithm::{create_live_info, RoomMatrix, Scheduler};

    #[test]
    fn simple() {
        let band_table = HashMap::from([
            ("band_a".to_string(), vec!["aaa_aaa".to_string()]),
            ("band_b".to_string(), vec!["aaa_aaa".to_string()]),
        ]);
        let band_schedule: HashMap<String, Vec<bool>> = band_table
            .keys()
            .map(|key| (key.to_string(), vec![true; 16]))
            .collect();
        let room_matrix = RoomMatrix::builder().push_room(2).build();
        let live_info = create_live_info(&band_table, &band_schedule, &room_matrix);

        let scheduler = Scheduler::new();
        let result = scheduler.assign(&room_matrix, &live_info);
        assert_eq!(result.len(), 2);
    }

    // そもそも部屋数が足りない場合をテスト
    #[test]
    fn exhaustion() {
        let band_table = HashMap::from([
            ("band_a".to_string(), vec!["aaa_aaa".to_string()]),
            ("band_b".to_string(), vec!["aaa_aaa".to_string()]),
        ]);
        let band_schedule: HashMap<String, Vec<bool>> = band_table
            .keys()
            .map(|key| (key.to_string(), vec![true; 16]))
            .collect();
        let room_matrix = RoomMatrix::builder().push_room(1).build();
        let live_info = create_live_info(&band_table, &band_schedule, &room_matrix);

        let scheduler = Scheduler::new();
        let result = scheduler.assign(&room_matrix, &live_info);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn simple_parallel() {
        // 以下の 2 通りのスケジュールがある
        // (band_b, band_c) => (band_a)
        // (band_c, band_b) => (band_a)
        let band_table = HashMap::from([
            (
                "band_a".to_string(),
                vec!["aaa_aaa".to_string(), "ccc".to_string()],
            ),
            (
                "band_b".to_string(),
                vec!["aaa_aaa".to_string(), "ddd".to_string()],
            ),
            (
                "band_c".to_string(),
                vec!["bbb_bbb".to_string(), "ccc".to_string()],
            ),
        ]);
        let band_schedule: HashMap<String, Vec<bool>> = band_table
            .keys()
            .map(|key| (key.to_string(), vec![true; 16]))
            .collect();
        let room_matrix = RoomMatrix::builder().push_room(2).push_room(1).build();
        let live_info = create_live_info(&band_table, &band_schedule, &room_matrix);

        let scheduler = Scheduler::new();
        let result = scheduler.assign(&room_matrix, &live_info);
        assert_eq!(result.len(), 2);
    }

    // バンドの候補日が歯抜けな状態でスケジューリングするテスト
    #[test]
    fn simple_parallel_with_schedule() {
        // 以下の 2 通りのスケジュールがある
        // (band_b, band_c) => (band_a)
        // (band_c, band_b) => (band_a)
        let band_table = HashMap::from([
            ("band_a".to_string(), vec!["a".to_string()]),
            ("band_b".to_string(), vec!["b".to_string()]),
            ("band_c".to_string(), vec!["c".to_string()]),
        ]);
        let band_schedule = HashMap::from([
            ("band_a".to_string(), vec![false, true]),
            ("band_b".to_string(), vec![true, true]),
            ("band_c".to_string(), vec![true, false]),
        ]);
        let room_matrix = RoomMatrix::builder().push_room(2).push_room(1).build();
        let live_info = create_live_info(&band_table, &band_schedule, &room_matrix);

        let scheduler = Scheduler::new();
        let result = scheduler.assign(&room_matrix, &live_info);
        assert_eq!(result.len(), 2);
    }

    // tokio ランタイムで並列実行するテスト
    #[test]
    fn simple_parallel_on_runtime() {
        // 以下の 4 通りのスケジュールがある
        // (band_b, band_c) => (band_a) => (band_d)
        // (band_c, band_b) => (band_a) => (band_d)
        // (band_b, band_c) => (band_d) => (band_a)
        // (band_c, band_b) => (band_d) => (band_a)
        let band_table = HashMap::from([
            (
                "band_a".to_string(),
                vec!["aaa_aaa".to_string(), "ccc".to_string()],
            ),
            (
                "band_b".to_string(),
                vec!["aaa_aaa".to_string(), "ddd".to_string()],
            ),
            (
                "band_c".to_string(),
                vec!["bbb_bbb".to_string(), "ccc".to_string()],
            ),
            (
                "band_d".to_string(),
                vec![
                    "aaa_aaa".to_string(),
                    "bbb_bbb".to_string(),
                    "ccc".to_string(),
                ],
            ),
        ]);
        let band_schedule: HashMap<String, Vec<bool>> = band_table
            .keys()
            .map(|key| (key.to_string(), vec![true; 16]))
            .collect();
        let room_matrix = Arc::new(RoomMatrix::builder().push_room(3).push_room(1).build());
        let live_info = create_live_info(&band_table, &band_schedule, &room_matrix);
        let live_info = Arc::new(live_info);

        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(async {
                let scheduler = Scheduler::new();
                let result = scheduler.assign_async(room_matrix, live_info).await;
                // let result = scheduler.assign(&room_matrix, &live_info);
                assert_eq!(result.len(), 4);
            });
    }

    // tokio ランタイムで並列実行するテスト
    #[test]
    fn heavy_on_runtime() {
        let band_table = HashMap::from([
            ("band_a".to_string(), vec!["a".to_string()]),
            ("band_b".to_string(), vec!["b".to_string()]),
            ("band_c".to_string(), vec!["c".to_string()]),
            ("band_d".to_string(), vec!["d".to_string()]),
            ("band_e".to_string(), vec!["e".to_string()]),
            ("band_f".to_string(), vec!["f".to_string()]),
            ("band_g".to_string(), vec!["a".to_string()]),
            ("band_h".to_string(), vec!["b".to_string()]),
            ("band_i".to_string(), vec!["c".to_string()]),
            ("band_j".to_string(), vec!["d".to_string()]),
            ("band_k".to_string(), vec!["e".to_string()]),
            // ("band_l".to_string(), vec!["f".to_string()]),
        ]);
        let band_schedule: HashMap<String, Vec<bool>> = band_table
            .keys()
            .map(|key| (key.to_string(), vec![true; 16]))
            .collect();
        let room_matrix = Arc::new(
            RoomMatrix::builder()
                .push_room(2)
                .push_room(2)
                .push_room(2)
                .push_room(2)
                .push_room(2)
                .push_room(1)
                .build(),
        );
        let live_info = create_live_info(&band_table, &band_schedule, &room_matrix);
        let live_info = Arc::new(live_info);

        // 並列実行
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(async {
                let scheduler = Scheduler::new();
                let result = scheduler.assign_async(room_matrix, live_info).await;
                // let result = scheduler.assign(&room_matrix, &live_info);
                assert_eq!(result.len(), 2764800);
            });
    }
}
