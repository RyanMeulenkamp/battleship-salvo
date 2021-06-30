use serde::{Deserialize, Serialize};
use crate::model::class::Class::{Carrier, Battleship, Destroyer, Submarine, PatrolBoat};
use strum_macros::EnumIter;
use std::fmt::{Display, Formatter};
use core::fmt;
use crate::model::occupation::Occupation;

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Clone, Copy, EnumIter)]
pub enum Class {
    Carrier([bool; 5]),
    Battleship([bool; 4]),
    Destroyer([bool; 3]),
    Submarine([bool; 3]),
    PatrolBoat([bool; 2]),
}

impl Into<String> for Class {
    fn into(self) -> String {
        self.name()
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Into<Class> for &str {
    fn into(self) -> Class {
        match self {
            "carrier" => Carrier([false; 5]),
            "battleship" => Battleship([false; 4]),
            "destroyer" => Destroyer([false; 3]),
            "submarine" => Submarine([false; 3]),
            "patrolboat" => PatrolBoat([false; 2]),
            _ => "carrier".into()
        }
    }
}

impl Default for Class {
    fn default() -> Self {
        Battleship([false; 4])
    }
}

impl Class {
    pub(crate) fn size(&self) -> u8 {
        let length = match self {
            Carrier(places) => places.len(),
            Battleship(places) => places.len(),
            Destroyer(places) => places.len(),
            Submarine(places) => places.len(),
            PatrolBoat(places) => places.len(),
        };
        length as u8
    }

    pub fn name(&self) -> String {
        String::from(match self {
            Carrier(_) => "carrier",
            Battleship(_) => "battleship",
            Destroyer(_) => "destroyer",
            Submarine(_) => "submarine",
            PatrolBoat(_) => "patrolboat",
        })
    }

    pub fn map_token(&self) -> &str {
        match self {
            Class::Carrier(_) => "C",
            Class::Battleship(_) => "B",
            Class::Destroyer(_) => "D",
            Class::Submarine(_) => "S",
            Class::PatrolBoat(_) => "P",
        }
    }

    fn get_place<const COUNT: usize>(places: &[bool; COUNT], x: usize) -> Option<bool> {
        if x >= COUNT {
            None
        } else {
            Some(places[x])
        }
    }

    fn place(&self, x: usize) -> Option<bool> {
        match self {
            Carrier(places) => Self::get_place(places, x),
            Battleship(places) => Self::get_place(places, x),
            Destroyer(places) => Self::get_place(places, x),
            Submarine(places) => Self::get_place(places, x),
            PatrolBoat(places) => Self::get_place(places, x),
        }
    }

    pub fn probe(&self, x: usize) -> Occupation {
        if let Some(spot) = self.place(x) {
            if self.is_sunk() {
                Occupation::Sunk(self.clone())
            } else {
                Occupation::Ship(self.clone(), spot)
            }
        } else {
            Occupation::Empty
        }
    }

    fn mark<const COUNT: usize>(places: &[bool; COUNT], x: u8) -> [bool; COUNT] {
        if x >= COUNT as u8 {
            return places.clone()
        }
        let mut places = places.clone();
        places[x as usize] = true;
        places.clone()
    }

    pub(crate) fn shoot(&self, x: u8) -> Class {
        match self {
            Carrier(places) => Carrier(Class::mark(places, x)),
            Battleship(places) => Battleship(Class::mark(places, x)),
            Destroyer(places) => Destroyer(Class::mark(places, x)),
            Submarine(places) => Submarine(Class::mark(places, x)),
            PatrolBoat(places) => PatrolBoat(Class::mark(places, x)),
        }
    }

    fn all_hit<const COUNT: usize>(places: &[bool; COUNT]) -> bool {
        for index in 0..COUNT {
            if !places[index] {
                return false
            }
        }
        true
    }

    pub(crate) fn is_sunk(&self) -> bool {
        match self {
            Carrier(places) => Class::all_hit(places),
            Battleship(places) => Class::all_hit(places),
            Destroyer(places) => Class::all_hit(places),
            Submarine(places) => Class::all_hit(places),
            PatrolBoat(places) => Class::all_hit(places),
        }
    }
}
