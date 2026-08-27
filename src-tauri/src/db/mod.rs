mod connection;
mod migrate;
pub mod repository;

#[cfg(test)]
mod tests;

pub use connection::{Db, DbHealth, DbState};

#[allow(unused_imports)] // re-exported for tests and future domain code
pub use migrate::MIGRATIONS;
