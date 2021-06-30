use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use core::fmt;

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Clone, Copy)]
pub struct Size {
    pub(crate) width: u8,
    pub(crate) height: u8,
}

impl Display for Size {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[{} X {}]", self.width, self.height)
    }
}

impl Default for Size {
    fn default() -> Self {
        Self::new(10, 10)
    }
}

impl Size {
    pub fn new(width: u8, height: u8) -> Self {
        Size {
            width: width, height: height,
        }
    }

    pub(crate) fn transposed(self) -> Self {
        Size::new(self.height, self.width)
    }
}
