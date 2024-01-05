use futures::future::join_all;
use itertools::Itertools;

use super::LiveInfo;

pub struct Scheduler;

impl Scheduler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn assign(&self, rooms: &[u32], live_info: &LiveInfo) -> Result<Vec<Vec<usize>>, ()> {
        let available_rooms: u32 = rooms.iter().sum();
        if available_rooms < live_info.band_ids().len() as u32 {
            return Err(());
        }

        let mut result: Vec<Vec<usize>> = Vec::default();

        // スケジュールの全組み合わせを調査
        let band_count = live_info.band_ids().len();
        for perm in (0..band_count).permutations(band_count) {
            let Ok(schedule) = Self::assign_impl(perm, rooms, live_info) else {
                continue;
            };

            result.push(schedule);
        }

        Ok(result)
    }

    pub async fn assign_async(
        &self,
        rooms: &[u32],
        live_info: &LiveInfo,
    ) -> Result<Vec<Vec<usize>>, ()> {
        // TODO: バンド数の決め打ちをやめて動的に設定するようにする
        // 4 バンドとして 4 スレッド起動。各スレッド内で下位 3 桁を総当たりで検索。
        let task0 = Self::assign_impl_async(vec![0, 1, 2, 3], 3, rooms, live_info);
        let task1 = Self::assign_impl_async(vec![1, 0, 2, 3], 3, rooms, live_info);
        let task2 = Self::assign_impl_async(vec![2, 0, 1, 3], 3, rooms, live_info);
        let task3 = Self::assign_impl_async(vec![3, 0, 1, 2], 3, rooms, live_info);

        // 全検索の結果を取得
        // メモ：バンド数が多いと組み合わせ爆発が起きてメモリーが枯渇するかもしれない
        let results = join_all(vec![task0, task1, task2, task3]).await;
        let final_result: Vec<_> = results.into_iter().flatten().collect();

        // 適切なスケジュールが存在しなければ失敗とする
        if final_result.is_empty() {
            Err(())
        } else {
            Ok(final_result)
        }
    }

    fn assign_impl(
        band_indicies: Vec<usize>,
        rooms: &[u32],
        live_info: &LiveInfo,
    ) -> Result<Vec<usize>, ()> {
        let mut current_head = 0;
        for count in rooms {
            let mut conflict_hash: u128 = 0;
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

    async fn assign_impl_async(
        band_indicies: Vec<usize>,
        permutation_digit: u8,
        rooms: &[u32],
        live_info: &LiveInfo,
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

            let Ok(result) = Self::assign_impl(band_indicies, rooms, live_info) else {
                continue;
            };
            results.push(result);
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::algorithm::create_live_info;

    use super::Scheduler;

    #[test]
    fn simple() {
        let band_table = HashMap::from([
            ("band_a".to_string(), vec!["aaa_aaa".to_string()]),
            ("band_b".to_string(), vec!["aaa_aaa".to_string()]),
        ]);
        let live_info = create_live_info(&band_table);

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
        let live_info = create_live_info(&band_table);

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
        let live_info = create_live_info(&band_table);

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
        let live_info = create_live_info(&band_table);

        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(async {
                let scheduler = Scheduler::new();
                let result = scheduler.assign_async(&rooms, &live_info).await.unwrap();
                assert_eq!(result.len(), 4);
            });
    }
}
