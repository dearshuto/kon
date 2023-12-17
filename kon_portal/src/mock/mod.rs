use std::collections::HashMap;

use kon_rs::{InstrumentType, User};

use crate::application::IClient;

pub struct MockClient {
    user_ids: Vec<String>,
    users: HashMap<String, User>,
}

impl MockClient {
    pub fn new() -> Self {
        Self {
            user_ids: vec![
                "shikama_shuto".to_string(),
                "edogawa_conan".to_string(),
                "hattori_heiji".to_string(),
            ],
            users: HashMap::from([
                (
                    "shikama_shuto".to_string(),
                    User {
                        name: "鹿間脩斗".to_string(),
                        instrument_type: InstrumentType::ELECTRIC_BASS,
                    },
                ),
                (
                    "edogawa_conan".to_string(),
                    User {
                        name: "江戸川コナン".to_string(),
                        instrument_type: InstrumentType::VOCAL,
                    },
                ),
                (
                    "hattori_heiji".to_string(),
                    User {
                        name: "服部平次".to_string(),
                        instrument_type: InstrumentType::TROMBONE | InstrumentType::TENOR_SAXPHONE,
                    },
                ),
            ]),
        }
    }
}

impl IClient for MockClient {
    fn fetch_users(&mut self) -> Vec<String> {
        self.user_ids.clone()
    }

    fn fetch_user(&mut self, id: &str) -> Result<User, ()> {
        let Some(user) = self.users.get(id) else {
            return Err(());
        };

        Ok(user.clone())
    }
}
