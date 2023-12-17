use std::collections::HashMap;

use kon_rs::User;

use crate::application::IClient;

pub struct MockClient {
    user_ids: Vec<String>,
    users: HashMap<String, User>,
}

impl MockClient {
    pub fn new() -> Self {
        Self {
            user_ids: vec!["shikama_shuto".to_string()],
            users: HashMap::from([(
                "shikama_shuto".to_string(),
                User {
                    name: "鹿間脩斗".to_string(),
                },
            )]),
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
