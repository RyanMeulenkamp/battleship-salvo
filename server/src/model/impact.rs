use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Clone, Copy)]
pub enum Impact<T> {
    Miss,
    Hit(T),
}
