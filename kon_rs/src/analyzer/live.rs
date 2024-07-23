use crate::{Band, User};

pub struct Live {}

impl Live {
    pub fn members(&self) -> &[User] {
        &[]
    }

    pub fn bands(&self) -> &[Band] {
        &[]
    }
}
