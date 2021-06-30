use serde::{Deserialize, Serialize};
use crate::model::orientation::Orientation::{Vertical, Horizontal};
use std::fmt::{Display, Formatter};
use core::fmt;

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Clone, Copy)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

impl Display for Orientation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Orientation {
    pub fn transposed(&self) -> Orientation {
        match self {
            Horizontal => Vertical,
            Vertical => Horizontal,
        }
    }
}
