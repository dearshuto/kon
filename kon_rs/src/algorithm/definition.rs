use std::collections::HashMap;

#[derive(Hash, PartialEq, Eq)]
pub struct BandId;

impl BandId {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct RoomId;

pub struct RoomMatrix {}

impl RoomMatrix {
    pub fn new(_rooms: &[u8]) -> Self {
        Self {}
    }
}

pub struct Band {
    pub member_ids: Vec<String>,
}

pub struct Schedule {
    pub assign_table: HashMap<BandId, RoomId>,
}
