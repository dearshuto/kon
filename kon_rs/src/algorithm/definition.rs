use std::collections::HashMap;

use crate::BandId;

pub struct RoomId;

pub struct RoomMatrix {}

impl RoomMatrix {
    pub fn new(_rooms: &[u8]) -> Self {
        Self {}
    }
}

pub struct Schedule {
    pub assign_table: HashMap<BandId, RoomId>,
}
