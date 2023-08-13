pub mod clients;
mod member_list;

pub use member_list::MemberList;
use std::collections::HashMap;

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

#[derive(Debug, serde::Deserialize)]
pub struct Record {
    name: String,
    property_name: String,
    value: String,
}

pub fn deserialize(str: &str) -> Vec<Member> {
    let mut rdr = csv::Reader::from_reader(str.as_bytes());
    let records = rdr.deserialize::<Record>().map(|x| x.unwrap());
    let mut table = HashMap::<String, Member>::default();
    for record in records {
        if !table.contains_key(&record.name) {
            table.insert(
                record.name.clone(),
                Member {
                    name: record.name.clone(),
                    instruments: Default::default(),
                },
            );
        }

        match &record.property_name as &str {
            "instrument" => {
                match &record.value as &str {
                    "Vocal" => {
                        table.get_mut(&record.name).unwrap().instruments |= InstrumentType::VOCAL
                    }
                    "ElectricBass" => {
                        table.get_mut(&record.name).unwrap().instruments |=
                            InstrumentType::ELECTRIC_BASS
                    }
                    "Keyboard" => {
                        table.get_mut(&record.name).unwrap().instruments |= InstrumentType::KEYBOARD
                    }
                    "ElectricGuitar" => {
                        table.get_mut(&record.name).unwrap().instruments |=
                            InstrumentType::ELECTRIC_GUITAR
                    }
                    "Drums" => {
                        table.get_mut(&record.name).unwrap().instruments |= InstrumentType::DRUMS
                    }
                    _ => {}
                };
            }
            _ => {}
        }
    }

    let mut members = Vec::new();
    for member in table.values() {
        members.push(member.clone());
    }
    members
}

#[derive(Debug, Default, Clone)]
pub struct Member {
    name: String,
    instruments: InstrumentType,
}

impl Member {
    pub fn new(name: &str, instruments: InstrumentType) -> Self {
        Self {
            name: name.to_string(),
            instruments,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn instruments(&self) -> InstrumentType {
        self.instruments
    }
}
