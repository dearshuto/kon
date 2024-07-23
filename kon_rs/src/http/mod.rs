mod confluence;
mod detail;
mod rest_type;

pub use confluence::Confluence;
pub use rest_type::{BodyType, ContenType, UserType, ViewType};

pub struct User {
    internal: UserType,
}

impl User {
    pub fn name(&self) -> &str {
        &self.internal.username
    }

    pub fn display_name(&self) -> &str {
        &self.internal.displayName
    }
}

pub struct Content {
    id: u64,
    title: String,
    raw_content: String,
}

impl Content {
    pub(crate) fn new(content_type: &ContenType) -> Self {
        Self {
            id: content_type.id.parse().unwrap(),
            title: content_type.title.clone(),
            raw_content: content_type.body.view.value.clone(),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn raw_content(&self) -> &str {
        &self.raw_content
    }
}
