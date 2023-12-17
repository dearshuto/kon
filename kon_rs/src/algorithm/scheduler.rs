use std::collections::{HashMap, HashSet};

use crate::BandId;

use super::definition::{RoomId, RoomMatrix, Schedule};

pub struct AssignParams {
    // バンドのスケジュール
    pub band_schedule: HashMap<BandId, HashSet<u8>>,

    pub room_matrix: RoomMatrix,
}

pub struct Scheduler;

impl Scheduler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn assign(&self, _condition: &AssignParams) -> Result<Schedule, ()> {
        let schedule = Schedule {
            assign_table: HashMap::from([(BandId::new(), RoomId::new())]),
        };
        Ok(schedule)
    }
}
