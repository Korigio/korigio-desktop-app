//! Global search across customers, devices, and repairs.

pub mod constants;
pub mod repository;
pub mod service;
pub mod types;

#[cfg(test)]
mod tests;

pub use service::global_search;
pub use types::{GlobalSearchQuery, GlobalSearchResult};
