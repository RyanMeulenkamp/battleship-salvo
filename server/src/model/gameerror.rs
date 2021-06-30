use crate::model::ship::Ship;
use std::fmt::{Display, Formatter};
use core::fmt;
use crate::model::class::Class;
use crate::model::orientation::Orientation;
use crate::model::point::Point;

#[derive(Debug, PartialOrd, PartialEq, Clone)]
pub enum GameError {
    ShipAlreadyPlaced(Class),
    ShipOutOfBounds(Point, Orientation, u8),
    ShipOverlaps(Ship),
}

impl Display for GameError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f, "{}",
            match self {
                GameError::ShipAlreadyPlaced(class) =>
                    format!("{} class ship has already been placed!", class),
                GameError::ShipOutOfBounds(coordinates, orientation, size) =>
                    format!(
                        "Ship is not placed (entirely) within the map! Coordinates: {}, orientation: {}, size: {}",
                        coordinates, orientation, size,
                    ),
                GameError::ShipOverlaps(ship) =>
                    format!(
                        "This ship overlaps with ship of class {} at {} (orientation: {})!",
                        ship.class, ship.coordinates, ship.orientation,
                    ),
            },
        )
    }
}
