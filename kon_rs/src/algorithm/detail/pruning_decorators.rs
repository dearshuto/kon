use std::ops::Range;

use crate::algorithm::{LiveInfo, RoomMatrix, TraverseOperation};

pub trait ITraverseDecorator {
    fn invoke(
        &self,
        indicies: &[i32],
        room_assign: &[Range<usize>],
        live_info: &LiveInfo,
    ) -> TraverseOperation;

    fn invoke_with_room_matrix(
        &self,
        indicies: &[i32],
        room_matrix: &RoomMatrix,
        live_info: &LiveInfo,
    ) -> TraverseOperation;
}

// なにもしない
#[derive(Default, Clone)]
pub struct TreeTraverser;
impl ITraverseDecorator for TreeTraverser {
    fn invoke(
        &self,
        _data: &[i32],
        _room_assign: &[Range<usize>],
        _live_info: &LiveInfo,
    ) -> TraverseOperation {
        TraverseOperation::Next
    }

    fn invoke_with_room_matrix(
        &self,
        _indicies: &[i32],
        _room_matrix: &RoomMatrix,
        _live_info: &LiveInfo,
    ) -> TraverseOperation {
        TraverseOperation::Next
    }
}

// バンドのスケジュールが合わないとき枝刈り
#[derive(Clone)]
pub struct BandScheduleTraverseDecorator<T: ITraverseDecorator + Clone> {
    decorator: T,
}

impl<T: ITraverseDecorator + Clone> ITraverseDecorator for BandScheduleTraverseDecorator<T> {
    fn invoke(
        &self,
        data: &[i32],
        room_assign: &[Range<usize>],
        live_info: &LiveInfo,
    ) -> TraverseOperation {
        match self.decorator.invoke(data, room_assign, live_info) {
            TraverseOperation::Pruning => TraverseOperation::Pruning,
            TraverseOperation::Skip(index) => TraverseOperation::Skip(index),
            TraverseOperation::Next => self.invoke_impl(data, room_assign, live_info),
        }
    }

    fn invoke_with_room_matrix(
        &self,
        indicies: &[i32],
        room_matrix: &RoomMatrix,
        live_info: &LiveInfo,
    ) -> TraverseOperation {
        match self
            .decorator
            .invoke_with_room_matrix(indicies, room_matrix, live_info)
        {
            TraverseOperation::Pruning => TraverseOperation::Pruning,
            TraverseOperation::Skip(index) => TraverseOperation::Skip(index),
            TraverseOperation::Next => {
                self.invoke_impl_with_room_matrix(indicies, room_matrix, live_info)
            }
        }
    }
}

impl<T: ITraverseDecorator + Clone> BandScheduleTraverseDecorator<T> {
    pub fn new(decorator: T) -> Self {
        Self { decorator }
    }

    fn invoke_impl(
        &self,
        indicies: &[i32],
        room_assign: &[Range<usize>],
        live_info: &LiveInfo,
    ) -> TraverseOperation {
        // バンドスケジュールが合致しなかったら走査をやめる
        for (index, range) in room_assign.iter().enumerate() {
            for band_index in range.clone().into_iter() {
                let band_count = live_info.band_ids().len();
                if band_index >= band_count {
                    continue;
                }

                let actual_index = indicies[band_index];
                if actual_index >= band_count as i32 {
                    continue;
                }
                let band_id = live_info.band_ids()[actual_index as usize];

                let Some(is_available) = live_info.band_schedule(band_id, index as i32) else {
                    let band_name = live_info.band_name(band_id);
                    println!("{}", band_name);
                    panic!();
                };
                if !is_available {
                    return TraverseOperation::Skip(band_index);
                }
            }
        }

        TraverseOperation::Next
    }

    fn invoke_impl_with_room_matrix(
        &self,
        indicies: &[i32],
        room_matrix: &RoomMatrix,
        live_info: &LiveInfo,
    ) -> TraverseOperation {
        let mut current_band_index = 0;
        for block_id in room_matrix.blocks() {
            let actual_index = indicies[current_band_index] as usize;
            let band_id = live_info.band_ids()[actual_index];

            let is_available = live_info.confirm_assignable(*block_id, band_id);
            if is_available {
                current_band_index += 1;
                continue;
            }

            if current_band_index == live_info.band_ids().len() - 1 {
                // 最後まで到達したら枝かり
                return TraverseOperation::Pruning;
            } else {
                // 途中で失敗したらスキップ
                return TraverseOperation::Skip(current_band_index + 1);
            }
        }

        TraverseOperation::Next
    }
}

// 同時刻のメンバー衝突の枝刈り
#[derive(Clone)]
pub struct MemberConflictTraverseDecorator<T: ITraverseDecorator> {
    decorator: T,
}

impl<T: ITraverseDecorator> ITraverseDecorator for MemberConflictTraverseDecorator<T> {
    fn invoke(
        &self,
        data: &[i32],
        room_assign: &[Range<usize>],
        live_info: &LiveInfo,
    ) -> TraverseOperation {
        match self.decorator.invoke(data, room_assign, live_info) {
            TraverseOperation::Pruning => TraverseOperation::Pruning,
            TraverseOperation::Skip(index) => TraverseOperation::Skip(index),
            TraverseOperation::Next => self.invoke_impl(data, room_assign, live_info),
        }
    }

    fn invoke_with_room_matrix(
        &self,
        indicies: &[i32],
        room_matrix: &RoomMatrix,
        live_info: &LiveInfo,
    ) -> TraverseOperation {
        match self
            .decorator
            .invoke_with_room_matrix(indicies, room_matrix, live_info)
        {
            TraverseOperation::Pruning => TraverseOperation::Pruning,
            TraverseOperation::Skip(index) => TraverseOperation::Skip(index),
            TraverseOperation::Next => {
                self.invoke_impl_with_room_matrix(indicies, room_matrix, live_info)
            }
        }
    }
}

impl<T: ITraverseDecorator> MemberConflictTraverseDecorator<T> {
    pub fn new(decorator: T) -> Self {
        Self { decorator }
    }

    fn invoke_impl(
        &self,
        indicies: &[i32],
        room_assign: &[Range<usize>],
        live_info: &LiveInfo,
    ) -> TraverseOperation {
        for range in room_assign {
            let mut band_hash_intersect = 0;
            let mut debug_buffer = Vec::default();
            for band_index in range.clone().into_iter() {
                // 空き部屋対応
                let band_count = live_info.band_ids().len();
                if band_index >= band_count {
                    continue;
                }
                let actual_index = indicies[band_index] as usize;
                if actual_index >= band_count {
                    continue;
                }

                let band_id = live_info.band_ids()[actual_index];
                let band_hash = live_info.band_hash(band_id).unwrap();

                debug_buffer.push(live_info.band_name(band_id));

                if (band_hash_intersect & band_hash) != 0 {
                    return TraverseOperation::Skip(band_index);
                } else {
                    band_hash_intersect |= band_hash;
                }
            }
        }

        TraverseOperation::Next
    }

    fn invoke_impl_with_room_matrix(
        &self,
        indicies: &[i32],
        room_matrix: &RoomMatrix,
        live_info: &LiveInfo,
    ) -> TraverseOperation {
        let mut current_band_index = 0;
        for span_id in room_matrix.spans() {
            let mut band_hash_intersect = 0;
            for _block_id in room_matrix.iter_span_blocks(*span_id) {
                let actual_index = indicies[current_band_index];
                let band_id = live_info.band_ids()[actual_index as usize];
                let band_hash = live_info.band_hash(band_id).unwrap();

                if (band_hash_intersect & band_hash) == 0 {
                    band_hash_intersect |= band_hash;
                    current_band_index += 1;
                    continue;
                }

                if current_band_index == live_info.band_ids().len() - 1 {
                    return TraverseOperation::Pruning;
                } else {
                    return TraverseOperation::Skip(current_band_index + 1);
                }
            }
        }

        TraverseOperation::Next
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::algorithm::{create_live_info, RoomMatrix, TraverseOperation};

    use super::{BandScheduleTraverseDecorator, MemberConflictTraverseDecorator, TreeTraverser};

    #[test]
    fn schedule_simple() {
        let decorator = BandScheduleTraverseDecorator::new(TreeTraverser::default());

        let room_matrix = RoomMatrix::builder().push_room(1).build();
        let live_info = create_live_info(
            &HashMap::from([("band_a".to_string(), vec!["a".to_string()])]),
            &HashMap::from([("band_a".to_string(), vec![true])]),
            &room_matrix,
        );

        let TraverseOperation::Next =
            decorator.invoke_impl_with_room_matrix(&[0], &room_matrix, &live_info)
        else {
            panic!();
        };
    }

    #[test]
    fn schedule_multi_simple() {
        let decorator = BandScheduleTraverseDecorator::new(TreeTraverser::default());

        let room_matrix = RoomMatrix::builder().push_room(2).build();
        let live_info = create_live_info(
            &HashMap::from([
                ("band_a".to_string(), vec!["a".to_string()]),
                ("band_b".to_string(), vec!["a".to_string()]),
            ]),
            &HashMap::from([
                ("band_a".to_string(), vec![true, true]),
                ("band_b".to_string(), vec![true, true]),
            ]),
            &room_matrix,
        );

        let TraverseOperation::Next =
            decorator.invoke_impl_with_room_matrix(&[0, 1], &room_matrix, &live_info)
        else {
            panic!();
        };

        let TraverseOperation::Next =
            decorator.invoke_impl_with_room_matrix(&[1, 0], &room_matrix, &live_info)
        else {
            panic!();
        };
    }

    #[test]
    fn schedule_conflict() {
        let decorator = BandScheduleTraverseDecorator::new(TreeTraverser::default());

        let room_matrix = RoomMatrix::builder().push_room(1).build();
        let live_info = create_live_info(
            &HashMap::from([("band_a".to_string(), vec!["a".to_string()])]),
            &HashMap::from([("band_a".to_string(), vec![false])]),
            &room_matrix,
        );

        let TraverseOperation::Pruning =
            decorator.invoke_impl_with_room_matrix(&[0], &room_matrix, &live_info)
        else {
            panic!();
        };
    }

    #[test]
    fn member_conflict_pass() {
        let decorator = MemberConflictTraverseDecorator::new(TreeTraverser::default());

        let room_matrix = RoomMatrix::builder().push_room(1).build();
        let live_info = create_live_info(
            &HashMap::from([("band_a".to_string(), vec!["a".to_string()])]),
            &HashMap::from([("band_a".to_string(), vec![false])]),
            &room_matrix,
        );

        let TraverseOperation::Next =
            decorator.invoke_impl_with_room_matrix(&[0], &room_matrix, &live_info)
        else {
            panic!();
        };
    }

    #[test]
    fn member_conflict() {
        let decorator = MemberConflictTraverseDecorator::new(TreeTraverser::default());

        let room_matrix = RoomMatrix::builder().push_room(1).push_room(1).build();
        let live_info = create_live_info(
            &HashMap::from([
                ("band_a".to_string(), vec!["a".to_string()]),
                ("band_b".to_string(), vec!["a".to_string()]),
            ]),
            &HashMap::from([
                ("band_a".to_string(), vec![true]),
                ("band_b".to_string(), vec![true]),
            ]),
            &room_matrix,
        );

        let TraverseOperation::Pruning =
            decorator.invoke_impl_with_room_matrix(&[0, 1], &room_matrix, &live_info)
        else {
            panic!();
        };

        let TraverseOperation::Pruning =
            decorator.invoke_impl_with_room_matrix(&[1, 0], &room_matrix, &live_info)
        else {
            panic!();
        };
    }
}
