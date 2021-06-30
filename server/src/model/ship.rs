use serde::{Deserialize, Serialize};
use crate::model::point::Point;
use crate::model::orientation::Orientation;
use crate::model::class::Class;
use crate::model::range::Range;
use crate::model::orientation::Orientation::{Horizontal, Vertical};
use crate::model::impact::Impact;
use crate::model::impact::Impact::{Hit, Miss};
use crate::messaging::translate::deserialize;
use crate::messaging::translate;
use crate::model::occupation::Occupation;

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Clone, Copy)]
pub struct Ship {
    pub(crate) coordinates: Point,
    pub(crate) orientation: Orientation,
    #[serde(default = "Class::default")]
    pub class: Class,
}

impl Ship {
    pub fn new(coordinates: Point, orientation: Orientation, class: Class) -> Self {
        Ship {
            coordinates: coordinates,
            orientation: orientation,
            class: class
        }
    }

    pub fn from_json(json: impl Into<String>, class: Class) -> translate::Result<Self> {
        let mut ship: Ship = deserialize(&json.into())?;
        ship.class = class;
        Ok(ship)
    }

    pub fn transposed_to(self, orientation: Orientation) -> Ship {
        if orientation == self.orientation {
            self
        } else {
            Ship::new(
                self.coordinates.transposed(),
                self.orientation.transposed(),
                self.class,
            )
        }
    }

    pub fn tail_end(&self) -> u8 {
        self.class.size() - 1 + match self.orientation {
            Horizontal => self.coordinates.x,
            Vertical => self.coordinates.y
        }
    }

    pub fn range(&self) -> Range {
        let min = match self.orientation {
            Horizontal => self.coordinates.x,
            Vertical => self.coordinates.y,
        };
        Range::new(min, min + self.class.size())
    }

    pub fn overlap(&self, other: &Ship) -> bool {
        if self.coordinates == other.coordinates {
            true
        } else {
            // Transpose ships to fixed orientations to reduce number of flows for this test
            let self_transposed = self.transposed_to(Horizontal);

            if self.orientation == other.orientation {
                let other_transposed = other.transposed_to(Horizontal);
                self_transposed.coordinates.y == other_transposed.coordinates.y
                    && self_transposed.range().overlap(other_transposed.range())
            } else {
                let other_transposed = other.transposed_to(Vertical);
                self_transposed.range().within(other_transposed.coordinates.x)
                    && other_transposed.range().within(self_transposed.coordinates.y)
            }
        }
    }

    pub fn is_hit(&self, coordinates: &Point) -> bool {
        match self.orientation {
            Horizontal => self.range().within(coordinates.x) && self.coordinates.y == coordinates.y,
            Vertical => self.range().within(coordinates.y) && self.coordinates.x == coordinates.x,
        }
    }

    pub fn probe(&self, coordinates: &Point) -> Occupation {
        if self.is_hit(coordinates) {
            self.class.probe(self.global_to_local(coordinates) as usize)
        } else {
            Occupation::Empty
        }
    }

    pub fn global_to_local(&self, coordinates: &Point) -> u8 {
        match self.orientation {
            Horizontal => coordinates.x - self.coordinates.x,
            Vertical => coordinates.y - self.coordinates.y,
        }
    }

    pub fn shoot(&self, coordinates: &Point) -> Impact<Ship> {
        if self.is_hit(coordinates) {
            Hit(Ship::new(
                self.coordinates, self.orientation,
                self.class.shoot(self.global_to_local(coordinates))
            ))
        } else {
            Miss
        }
    }

    pub fn is_sunk(&self) -> bool {
        self.class.is_sunk()
    }
}
