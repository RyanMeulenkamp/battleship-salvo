use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use core::fmt;

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Clone, Copy)]
pub struct Point {
    pub(crate) x: u8,
    pub(crate) y: u8,
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[{}; {}]", self.x, self.y)
    }
}

impl Point {
    pub fn new(x: u8, y: u8) -> Self {
        Point {
            x: x, y: y,
        }
    }

    pub(crate) fn transposed(self) -> Self {
        Point::new(self.y, self.x)
    }
}
