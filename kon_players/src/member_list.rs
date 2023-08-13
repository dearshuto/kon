use crate::{InstrumentType, Member};

pub struct MemberList {
    members: Vec<Member>,

    instrument_filter: InstrumentType,
}

impl MemberList {
    pub fn from_csv(data: &str) -> Self {
        let members = crate::deserialize(data);
        Self {
            members,
            instrument_filter: Default::default(),
        }
    }

    pub fn members(&self) -> &[Member] {
        &self.members
    }

    pub fn filter(&self) -> InstrumentType {
        self.instrument_filter
    }

    pub fn add_filter(&mut self, filter: InstrumentType) {
        self.instrument_filter = self.instrument_filter.union(filter);
    }

    pub fn remove_filter(&mut self, filter: InstrumentType) {
        self.instrument_filter.remove(filter);
    }

    pub fn clear_instrument_filter(&mut self) {
        self.instrument_filter = self.instrument_filter.intersection(Default::default());
    }
}
