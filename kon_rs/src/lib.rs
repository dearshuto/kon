use uuid::Uuid;

pub mod algorithm;
pub mod http;
pub mod rest_type;

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, Default)]
    pub struct InstrumentType: u32 {
        const VOCAL           = 0b000000000001;
        const ELECTRIC_GUITAR = 0b000000000010;
        const ACOUSTIC_GUITAR = 0b000000000100;
        const ELECTRIC_BASS   = 0b000000001000;
        const TENOR_SAXPHONE  = 0b000000010000;
        const KEYBOARD        = 0b000000100000;
        const PIANO           = 0b000001000000;
        const TROMBONE        = 0b000010000000;
        const DRUMS           = 0b000100000000;
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct BandId {
    uuid: Uuid,
}

impl BandId {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
        }
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct UserId {
    uuid: Uuid,
}

impl UserId {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Band {
    pub member_ids: Vec<String>,
}

#[derive(Clone)]
pub struct User {
    pub name: String,
    pub instrument_type: InstrumentType,
}

pub struct LiveProgram {
    // 出演者一覧
    pub user_names: Vec<String>,

    // 出演バンド
    pub item: Vec<String>,
}
