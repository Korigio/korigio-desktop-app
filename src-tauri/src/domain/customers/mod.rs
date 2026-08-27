pub mod constants;
pub mod repository;
pub mod service;
pub mod types;
pub mod validation;

#[cfg(test)]
mod tests;

pub use service::{
    archive_customer, create_customer, get_customer, list_customers, unarchive_customer,
    update_customer,
};
pub use types::{Customer, CustomerInput, CustomerListQuery, CustomerListResult};
