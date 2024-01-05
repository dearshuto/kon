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
}
