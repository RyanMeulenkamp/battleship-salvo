use serde::{Deserialize, Serialize};
use core::cmp;

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Clone, Copy)]
pub struct Range {
    pub(crate) min: u8,
    pub(crate) max: u8
}

impl Range {
    pub fn new(min: u8, max: u8) -> Range {
        Range {
            min: cmp::min(min, max),
            max: cmp::max(min, max)
        }
    }

    pub fn within(&self, x: u8) -> bool {
        x >= self.min && x <= self.max
    }

    pub fn overlap(&self, other: Range) -> bool {
        self.max >= other.min && self.min <= other.max
    }
}
