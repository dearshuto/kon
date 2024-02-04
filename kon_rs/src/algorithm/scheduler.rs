// 非同期実行用に必要なクレート
// wasm32 ビルドでは非同期ランタイムが非対応なのでオフにしておく
#[cfg(not(target_arch = "wasm32"))]
use futures::future::join_all;
use std::ops::Range;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;

use itertools::Itertools;

use super::{
    pruning_decorators::{
        BandScheduleTraverseDecorator, ITraverseDecorator, MemberConflictTraverseDecorator,
        TreeTraverser,
    },
    traverse::TraverseOperation,
    traverse_all, ITreeCallback, LiveInfo,
};

pub trait IScheduleCallback {
    fn assigned(&mut self, indicies: &[usize], live_info: &LiveInfo);
}

#[derive(Default)]
struct ScheduleStoreCallback {
    stored: Vec<Vec<usize>>,
}
impl IScheduleCallback for ScheduleStoreCallback {
    fn assigned(&mut self, indicies: &[usize], _live_info: &LiveInfo) {
        self.stored.push(indicies.to_vec());
    }
}

pub struct Scheduler;

impl Scheduler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn assign(&self, rooms: &[u32], live_info: &LiveInfo) -> Result<Vec<Vec<usize>>, ()> {
        let mut callback = ScheduleStoreCallback::default();
        self.assign_with_callback(rooms, live_info, &mut callback);
        if callback.stored.is_empty() {
            Err(())
        } else {
            Ok(callback.stored)
        }
    }

    pub fn assign_with_callback<T: IScheduleCallback>(
        &self,
        rooms: &[u32],
        live_info: &LiveInfo,
        callback: &mut T,
    ) {
        let available_rooms: u32 = rooms.iter().sum();
        if available_rooms < live_info.band_ids().len() as u32 {
            return;
        }

        // 枝刈り
        let decorator = TreeTraverser::default();
        let decorator = BandScheduleTraverseDecorator::new(decorator);
        let decorator = MemberConflictTraverseDecorator::new(decorator);

        // スケジュールの全組み合わせを調査
        let band_count = live_info.band_ids().len();
        let mut band_indicies: Vec<i32> =
            (0..band_count.max(available_rooms as usize) as i32).collect();
        let room_assign: Vec<Range<usize>> = rooms
            .iter()
            .scan((0, 0), |(_start, end), room_count| {
                let start = *end;
                *end += *room_count;
                Some((start as usize, *end))
            })
            .map(|(start, end)| Range {
                start,
                end: end as usize,
            })
            .collect();
        let mut callback = TraverseCallback::new(decorator, callback, &room_assign, live_info);
        traverse_all(&mut band_indicies, &mut callback);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn assign_async(
        &self,
        rooms: &[u32],
        live_info: Arc<LiveInfo>,
    ) -> Result<Vec<Vec<usize>>, ()> {
        let band_count = live_info.band_ids().len();

        // N バンドとして N スレッド起動。
        // 各スレッド内で下位 N-1 桁を総当たりで検索。
        let tasks: Vec<_> = (0..band_count)
            .map(|band_index| {
                let digit = band_count;
                let mut buffer: Vec<_> = (0..digit).collect();
                for index in 0..band_index {
                    for j in (0..index as usize).rev() {
                        buffer.swap(j, j + 1);
                    }
                }

                let r: Vec<u32> = rooms.iter().map(|x| *x).collect();
                let live_info = Arc::clone(&live_info);
                tokio::spawn(async move {
                    Self::assign_impl_async(buffer, (digit.clone() - 1) as u8, &r, live_info).await
                })
            })
            .collect();

        // 全検索の結果を取得
        // メモ：バンド数が多いと組み合わせ爆発が起きてメモリーが枯渇するかもしれない
        let results = join_all(tasks).await;
        let final_result: Vec<_> = results.into_iter().flatten().flatten().collect();

        // 適切なスケジュールが存在しなければ失敗とする
        if final_result.is_empty() {
            Err(())
        } else {
            Ok(final_result)
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn assign_impl_async(
        band_indicies: Vec<usize>,
        permutation_digit: u8,
        rooms: &[u32],
        live_info: Arc<LiveInfo>,
    ) -> Vec<Vec<usize>> {
        let skip = (band_indicies.len() - permutation_digit as usize).max(0);
        let permutation_digit = band_indicies.len() - skip;
        let head: Vec<usize> = band_indicies.iter().take(skip).map(|x| *x).collect();
        let tail: Vec<usize> = band_indicies.iter().skip(skip).map(|x| *x).collect();

        let mut results = Vec::default();
        for permutation in tail.iter().permutations(permutation_digit) {
            let band_indicies = {
                let mut head = head.clone();
                for tail in permutation {
                    head.push(*tail);
                }
                head
            };

            // ジェネリクス指定しているが、実装に依存はないので実はなんでもよい
            let Ok(result) = TraverseCallback::<TreeTraverser, ScheduleStoreCallback>::assign_impl(
                band_indicies,
                rooms,
                &live_info,
            ) else {
                continue;
            };
            results.push(result);
        }

        results
    }
}

struct TraverseCallback<'a, T, TScheduleCallback>
where
    T: ITraverseDecorator,
    TScheduleCallback: IScheduleCallback,
{
    traverse_decorator: T,

    schedule_callback: &'a mut TScheduleCallback,

    // 走査途中で見つけた最高スコア
    #[allow(dead_code)]
    score: u32,

    live_info: &'a LiveInfo,

    room_assign: &'a [Range<usize>],
}

impl<'a, T, TScheduleCallback> TraverseCallback<'a, T, TScheduleCallback>
where
    T: ITraverseDecorator,
    TScheduleCallback: IScheduleCallback,
{
    pub fn new(
        traverse_decorator: T,
        schedule_callback: &'a mut TScheduleCallback,
        room_assign: &'a [Range<usize>],
        live_info: &'a LiveInfo,
    ) -> Self {
        Self {
            traverse_decorator,
            schedule_callback,
            score: 0,
            live_info,
            room_assign,
        }
    }

    pub fn assign_impl(
        band_indicies: Vec<usize>,
        rooms: &[u32],
        live_info: &LiveInfo,
    ) -> Result<Vec<usize>, ()> {
        // まずバンドの候補時間に参加できるかを判定
        let is_available = band_indicies.iter().enumerate().all(|(index, band_index)| {
            let band_id = live_info.band_ids()[*band_index];

            // 時間帯と部屋数からバンドがどの時間帯に割り振られるか判定
            let room_count_scan: Vec<u32> = rooms
                .iter()
                .scan(0, |sum, room_count| {
                    *sum += room_count;
                    Some(*sum)
                })
                .collect();
            let (time_index, _room_count) = room_count_scan
                .iter()
                .enumerate()
                .find(|(_index, room_sum)| index < **room_sum as usize)
                .unwrap();

            // 割り振られる予定の時間帯に参加できるか判定
            live_info.band_schedule(band_id, time_index as i32).unwrap()
        });
        if !is_available {
            return Err(());
        }

        let mut current_head = 0;
        for count in rooms {
            let mut conflict_hash: u64 = 0;
            for index in band_indicies
                .iter()
                .skip(current_head)
                .take(*count as usize)
            {
                let band_id = live_info.band_ids()[*index];
                let band_hash = live_info.band_hash(band_id).unwrap();

                if (conflict_hash & band_hash) != 0 {
                    // 衝突があったらスケジューリング失敗
                    return Err(());
                }

                conflict_hash |= band_hash;
            }

            current_head += *count as usize;
        }

        Ok(band_indicies)
    }
}

impl<'a, T: ITraverseDecorator, TScheduleCallback: IScheduleCallback> ITreeCallback
    for TraverseCallback<'a, T, TScheduleCallback>
{
    fn invoke(&mut self, indicies: &[i32]) -> TraverseOperation {
        let invoke_result =
            self.traverse_decorator
                .invoke(indicies, self.room_assign, self.live_info);
        match invoke_result {
            TraverseOperation::Next => {
                let indicies: Vec<usize> = indicies.iter().map(|x| *x as usize).collect();
                self.schedule_callback.assigned(&indicies, self.live_info);
                TraverseOperation::Next
            }
            _ => invoke_result,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use crate::algorithm::create_live_info;

    use super::Scheduler;

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
        let live_info = create_live_info(&band_table, &band_schedule);

        let scheduler = Scheduler::new();
        let result = scheduler.assign(&[1, 1], &live_info).unwrap();
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
        let live_info = create_live_info(&band_table, &band_schedule);

        let scheduler = Scheduler::new();
        let result = scheduler.assign(&[1], &live_info);
        assert!(result.is_err());
    }

    #[test]
    fn simple_parallel() {
        // 以下の 2 通りのスケジュールがある
        // (band_b, band_c) => (band_a)
        // (band_c, band_b) => (band_a)
        let rooms = [2, 1];
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
        let live_info = create_live_info(&band_table, &band_schedule);

        let scheduler = Scheduler::new();
        let result = scheduler.assign(&rooms, &live_info).unwrap();
        assert_eq!(result.len(), 2);
    }

    // バンドの候補日が歯抜けな状態でスケジューリングするテスト
    #[test]
    fn simple_parallel_with_schedule() {
        // 以下の 2 通りのスケジュールがある
        // (band_b, band_c) => (band_a)
        // (band_c, band_b) => (band_a)
        let rooms = [2, 1];
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
        let live_info = create_live_info(&band_table, &band_schedule);

        let scheduler = Scheduler::new();
        let result = scheduler.assign(&rooms, &live_info).unwrap();
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
        let rooms = [2, 1, 1];
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
        let live_info = create_live_info(&band_table, &band_schedule);
        let live_info = Arc::new(live_info);

        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(async {
                let scheduler = Scheduler::new();
                let result = scheduler.assign_async(&rooms, live_info).await.unwrap();
                assert_eq!(result.len(), 4);
            });
    }

    // tokio ランタイムで並列実行するテスト
    #[test]
    fn heavy_on_runtime() {
        let rooms = [6, 5];
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
            //          ("band_l".to_string(), vec!["f".to_string()]),
        ]);
        let band_schedule: HashMap<String, Vec<bool>> = band_table
            .keys()
            .map(|key| (key.to_string(), vec![true; 16]))
            .collect();
        let live_info = create_live_info(&band_table, &band_schedule);
        let live_info = Arc::new(live_info);

        // 動作比較用の同期実行
        // let scheduler = Scheduler::new();
        // let result = scheduler.assign(&rooms, &live_info).unwrap();
        // assert_eq!(result.len(), 2764800);

        // 並列実行
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(async {
                let scheduler = Scheduler::new();
                let result = scheduler.assign_async(&rooms, live_info).await.unwrap();
                assert_eq!(result.len(), 2764800);
            });
    }
}
