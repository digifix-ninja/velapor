//! The shared vocabulary: how serious something is, which part of the machine it belongs to,
//! and the finding that tells the person about it.

mod domain;
mod finding;
mod severity;
pub mod units;

pub use domain::Domain;
pub use finding::{Evidence, Finding};
pub use severity::Severity;
