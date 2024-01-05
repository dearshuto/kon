mod definition;
mod html_parser;
mod scheduler;

use std::collections::HashMap;

pub use definition::{RoomMatrix, Schedule};
pub use html_parser::HtmlParser;
pub use scheduler::{Condition, Scheduler};

use crate::{BandId, UserId};

pub struct BandSchedule {
    pub name: String,
    pub is_available: Vec<bool>,
    pub members: Vec<String>,
}

pub struct LiveInfo {
    user_ids: Vec<UserId>,
    user_identifier_table: HashMap<UserId, String>,
    band_ids: Vec<BandId>,
    band_hash_table: HashMap<BandId, u128>,
    band_member_table: HashMap<BandId, Vec<UserId>>,
}

impl LiveInfo {
    pub fn new() -> Self {
        Self {
            user_ids: Vec::default(),
            user_identifier_table: HashMap::default(),
            band_ids: Vec::default(),
            band_hash_table: HashMap::default(),
            band_member_table: HashMap::default(),
        }
    }

    pub fn user_ids(&self) -> &[UserId] {
        &self.user_ids
    }

    pub fn user_identifier(&self, id: UserId) -> Option<&str> {
        let Some(identifier) = self.user_identifier_table.get(&id) else {
            return None;
        };

        Some(identifier)
    }

    pub fn band_ids(&self) -> &[BandId] {
        &self.band_ids
    }

    pub fn band_hash(&self, id: BandId) -> Option<u128> {
        let Some(hash) = self.band_hash_table.get(&id) else {
            return None;
        };

        Some(*hash)
    }

    pub fn band_member_ids(&self, id: BandId) -> Option<&[UserId]> {
        let Some(user_ids) = self.band_member_table.get(&id) else {
            return None;
        };

        Some(user_ids)
    }
}
