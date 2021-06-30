use serde::{Deserialize, Serialize};
use crate::model::status::Status::Requested;

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Clone, Copy)]
pub enum Status {
    Requested,
    Ready,
    Preparing,
    Idle,
    Due,
    Defeated
}

impl Default for Status {
    fn default() -> Self {
        Requested
    }
}
