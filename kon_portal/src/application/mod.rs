mod client;
mod workspace;

pub use client::IClient;
pub use workspace::Workspace;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    Members,
    Schedule,
}
