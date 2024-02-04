use std::collections::HashMap;

use crate::{BandId, RoomId};

pub struct Evaluator;

impl Evaluator {
    // 部屋をどれくらい使い切れてるかの判定
    // 点数が高いほど優秀な部屋割り
    pub fn evaluate_room_density(_room_assign: &HashMap<RoomId, Vec<BandId>>) -> i32 {
        0
    }

    // 部屋移動の手間の少なさを判定
    // 連続するメンバーがいると高得点
    pub fn evaluate_user_coherency(
        room_assign: &HashMap<RoomId, Vec<BandId>>,
        band_hash_table: &HashMap<BandId, u64>,
    ) -> u32 {
        let mut score = 0;
        for band_ids in room_assign.values() {
            for (index, band_id) in band_ids.iter().skip(1).enumerate() {
                // イテレーターを 1 スキップしてるので 1 足すと実際のインデックスになる
                let previous_band_id = band_ids[index + 1];
                let previous_band_hash = band_hash_table.get(&previous_band_id).unwrap();
                let band_hash = band_hash_table.get(band_id).unwrap();
                let coherency = (previous_band_hash & band_hash).count_ones();
                score += coherency;
            }
        }
        score
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{BandId, RoomId};

    use super::Evaluator;

    #[test]
    fn empty() {
        let room_assign = HashMap::default();
        let band_table = HashMap::default();
        let score = Evaluator::evaluate_user_coherency(&room_assign, &band_table);
        assert_eq!(score, 0);
    }

    // ひとつの部屋にひとつのバンドが連続して破り当たった場合
    #[test]
    fn one_band_coherency() {
        let band_id = BandId::new();
        let room_assign = HashMap::from([(RoomId::new(), vec![band_id, band_id])]);
        let band_table = HashMap::from([(band_id, 0x00FFu64)]);
        let score = Evaluator::evaluate_user_coherency(&room_assign, &band_table);
        assert_eq!(score, 0x00FFu64.count_ones());
    }

    // 連続するメンバーがいると一貫性が上がる
    #[test]
    fn simple_coherency() {
        let band_id = BandId::new();
        let room_assign = HashMap::from([(RoomId::new(), vec![band_id, band_id])]);
        let band_table = HashMap::from([(band_id, 0x0004u64)]);
        let score = Evaluator::evaluate_user_coherency(&room_assign, &band_table);
        assert!(0 < score);
    }

    #[test]
    fn simple() {
        let band_id0 = BandId::new();
        let band_id1 = BandId::new();
        let band_id2 = BandId::new();

        // 同じ部屋割りなら空き時間は詰まってる方が優秀
        let good_room_assign =
            HashMap::from([(RoomId::new(), vec![BandId::invalid(), band_id2, band_id1])]);
        let bad_room_assign =
            HashMap::from([(RoomId::new(), vec![band_id0, BandId::invalid(), band_id1])]);

        // 未実装なので一旦コメントアウト
        let _good_room_assign_score = Evaluator::evaluate_room_density(&good_room_assign);
        let _bad_room_assign_score = Evaluator::evaluate_room_density(&bad_room_assign);
        // assert!(good_room_assign_score < bad_room_assign_score);
    }
}
