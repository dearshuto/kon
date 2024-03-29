use std::ops::Range;

use super::{LiveInfo, TraverseOperation};

pub trait ITraverseDecorator {
    fn invoke(
        &self,
        indicies: &[i32],
        room_assign: &[Range<usize>],
        live_info: &LiveInfo,
    ) -> TraverseOperation;
}

// なにもしない
#[derive(Default)]
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
}

// バンドのスケジュールが合わないとき枝刈り
pub struct BandScheduleTraverseDecorator<T: ITraverseDecorator> {
    decorator: T,
}

impl<T: ITraverseDecorator> ITraverseDecorator for BandScheduleTraverseDecorator<T> {
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
}

impl<T: ITraverseDecorator> BandScheduleTraverseDecorator<T> {
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
}

// 同時刻のメンバー衝突の枝刈り
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
}
