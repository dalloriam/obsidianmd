//! Opinionated library to parse and interact with an obsidian.md vault.

pub mod markdown;
mod note;
mod section;
mod vault;

pub use note::Note;
pub use vault::Vault;

// Re-exports
pub use time::Date;
