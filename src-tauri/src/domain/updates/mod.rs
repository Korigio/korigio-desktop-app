pub mod constants;
pub mod service;
pub mod types;
pub mod validation;

#[cfg(test)]
mod tests;

pub use service::{app_version, check_app_update, ReqwestLatestFetcher};
pub use types::{AppVersion, UpdateCheck};
pub use validation::validate_open_url;
