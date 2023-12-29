mod definition;
mod html_parser;
mod scheduler;

pub use definition::{RoomMatrix, Schedule};
pub use html_parser::HtmlParser;
pub use scheduler::{Condition, Scheduler};

pub struct BandSchedule {
    pub name: String,
    pub is_available: Vec<bool>,
    pub members: Vec<String>,
}
