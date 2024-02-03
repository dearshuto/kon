use super::{LiveInfo, TraverseOperation};

pub trait ITraverseDecorator {
    fn invoke(&self, indicies: &[i32], rooms: &[u32], live_info: &LiveInfo) -> TraverseOperation;
}

// なにもしない
#[derive(Default)]
pub struct TreeTraverser;
impl ITraverseDecorator for TreeTraverser {
    fn invoke(&self, _data: &[i32], _rooms: &[u32], _live_info: &LiveInfo) -> TraverseOperation {
        TraverseOperation::Next
    }
}

// バンドのスケジュールが合わないとき枝刈り
pub struct BandScheduleTraverseDecorator<T: ITraverseDecorator> {
    decorator: T,
}

impl<T: ITraverseDecorator> ITraverseDecorator for BandScheduleTraverseDecorator<T> {
    fn invoke(&self, data: &[i32], rooms: &[u32], live_info: &LiveInfo) -> TraverseOperation {
        match self.decorator.invoke(data, rooms, live_info) {
            TraverseOperation::Pruning => TraverseOperation::Pruning,
            TraverseOperation::Skip(index) => TraverseOperation::Skip(index),
            TraverseOperation::Next => self.invoke_impl(data, rooms, live_info),
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
        rooms: &[u32],
        live_info: &LiveInfo,
    ) -> TraverseOperation {
        // バンドスケジュールが合致しなかったら走査をやめる
        for index in 0..indicies.len() {
            let band_index = indicies[index];
            let band_id = live_info.band_ids()[band_index as usize];

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

            let is_available = live_info.band_schedule(band_id, time_index as i32).unwrap();
            if !is_available {
                return TraverseOperation::Skip(index);
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
    fn invoke(&self, data: &[i32], rooms: &[u32], live_info: &LiveInfo) -> TraverseOperation {
        match self.decorator.invoke(data, rooms, live_info) {
            TraverseOperation::Pruning => TraverseOperation::Pruning,
            TraverseOperation::Skip(index) => TraverseOperation::Skip(index),
            TraverseOperation::Next => self.invoke_impl(data, rooms, live_info),
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
        rooms: &[u32],
        live_info: &LiveInfo,
    ) -> TraverseOperation {
        let i: Vec<(usize, u32)> = rooms
            .iter()
            .scan((0, 0), |(_start, end), room_count| {
                let start = *end;
                *end += *room_count;
                Some((start as usize, *end))
            })
            .collect();
        for (start, end) in i {
            let mut band_hash_intersect = 0;
            let band_count = live_info.band_ids().len();
            let end = (end as usize).min(band_count);
            for band_index in start..end {
                let actual_index = indicies[band_index] as usize;
                let band_id = live_info.band_ids()[actual_index];
                let band_hash = live_info.band_hash(band_id).unwrap();
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
