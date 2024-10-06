mod live;
mod schedule;

pub use schedule::Node;

use crate::http::{ContenType, UserType};

pub trait ICommunicator {
    fn request_user<T>(&self, id: T) -> UserType
    where
        T: AsRef<str>;

    fn request_content(&self, id: u64) -> ContenType;
}
