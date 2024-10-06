use url::Url;

use super::{ContenType, Content, User, UserType};

pub struct Confluence {
    base_url: Url,
}

impl Confluence {
    pub fn new(base_url: Url) -> Self {
        Self { base_url }
    }

    pub async fn fetch_user<T>(&self, id: T) -> User
    where
        T: AsRef<str>,
    {
        let mut rest_url = self.base_url.clone();
        rest_url.set_path(&format!("{}/rest/api/user", self.base_url.path()));
        rest_url.set_query(Some(&format!("username={}", id.as_ref())));

        let response = reqwest::get(rest_url.as_str())
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let user_type = serde_json::from_str::<UserType>(&response).unwrap();
        User {
            internal: user_type,
        }
    }

    pub async fn fetch_content(&self, page_id: u64) -> Content {
        let mut rest_url = self.base_url.clone();
        rest_url.set_path(&format!(
            "{}/rest/api/content/{}",
            self.base_url.path(),
            page_id
        ));
        rest_url.set_query(Some("expand=body.view"));

        let response = reqwest::get(rest_url.as_str())
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let content_type = serde_json::from_str::<ContenType>(&response).unwrap();
        Content::new(&content_type)
    }
}
