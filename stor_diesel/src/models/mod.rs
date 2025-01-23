pub mod model_project;
pub mod xrn_registry;

use chrono::{DateTime, FixedOffset};
pub use xrn_registry::*;

pub type StorDate = DateTime<FixedOffset>;
