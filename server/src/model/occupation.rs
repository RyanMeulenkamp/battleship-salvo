use crate::model::class::Class;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use core::fmt;

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Clone, Copy)]
pub enum Occupation {
    Ship(Class, bool),
    Sunk(Class),
    Empty,
}

impl Display for Occupation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f, "{}",
            match self {
                Occupation::Ship(class, hit) => {
                    if *hit {
                        "[s]"
                    } else {
                        " s "
                    }.replace(
                        "s",
                        class.map_token()
                    )
                }
                Occupation::Sunk(class) => String::from("↓") + class.map_token() + "↓",
                Occupation::Empty => String::from("   ")
            }
        )
    }
}
