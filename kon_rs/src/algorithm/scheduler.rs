use std::collections::{HashMap, HashSet};

use crate::BandId;

use super::definition::{RoomMatrix, Schedule};

pub struct Condition {
    // バンドのスケジュール
    pub band_schedule: HashMap<BandId, HashSet<u8>>,

    pub room_matrix: RoomMatrix,
}

pub struct Scheduler;

impl Scheduler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn assign(&self, _condition: &Condition) -> Result<Schedule, ()> {
        Err(())
    }
}
