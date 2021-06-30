use std::fmt::{Display, Formatter};
use core::fmt;

pub(crate) enum Orientation {
    Horizontal,
    Vertical
}

impl Display for Orientation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f, "{}", match self {
                Orientation::Horizontal => "Horizontal",
                Orientation::Vertical => "Vertical",
            }
        )
    }
}
