use kon_rs::{Band, User};

pub trait IClient {
    fn fetch_users(&mut self) -> Vec<String>;

    fn fetch_user(&mut self, id: &str) -> Result<User, ()>;

    fn fetch_bands(&mut self) -> Vec<Band>;
}
