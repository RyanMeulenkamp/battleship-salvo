use serde::{Deserialize, Serialize};
use crate::model::{
    ship::Ship,
    status::Status,
    class::Class,
    point::Point,
    impact::Impact,
    impact::Impact::{Hit, Miss},
    status::Status::Defeated,
    gameerror::GameError,
    gameerror::GameError::ShipAlreadyPlaced,
    occupation::Occupation
};
use std::fmt::{Display, Formatter};
use core::fmt;
use crate::model::size::Size;
use crate::model::orientation::Orientation;

type Fleet = [Option<Ship>; 5];

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Clone)]
pub struct Player {
    pub name: String,
    pub secret: String,
    #[serde(default = "default_fleet")]
    fleet: Fleet,
    #[serde(default = "Status::default")]
    status: Status,
    #[serde(skip)]
    field_size: Size,
}

fn default_fleet() -> [Option<Ship>; 5] {
    [None; 5]
}

fn ruler(width: u8, left: &str, inner: &str, border: &str, right: &str) -> String {
    let mut ruler = format!("{}", left);
    for _ in 0..=width - 1 {
        ruler = ruler + inner + border;
    }
    ruler + inner + right + "\n"
}

fn top_ruler(width: u8) -> String {
    ruler(width, "    ╔", "══╧══", "╤", "╗")
}

fn inner_ruler(width: u8) -> String {
    ruler(width, "    ╟", "─────", "┼", "╢")
}

fn bottom_ruler(width: u8) -> String {
    ruler(width, "    ╚", "═════", "╧", "╝")
}

impl Display for Player {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let last_x = self.field_size.width - 1;
        let last_y = self.field_size.height - 1;
        let mut output = format!("\n\n     ");
        for x in 0..=last_x {
            output += &*format!("  {}   ", x);
        }
        output += "\n";
        for y in 0..=last_y {
            output += &*if y == 0 {
                top_ruler(last_x)
            } else {
                inner_ruler(last_x)
            };
            output += &*format!("  {} ╢", y);
            for x in 0..=last_x {
                output += &*format!(" {} ", self.probe(&Point::new(x, y)));
                output += if x == 9 {
                    "║"
                } else {
                    "│"
                };
            }
            output += "\n";
        }
        output += &*bottom_ruler(last_x);
        write!(f, "{}", output)
    }
}

impl Player {

    pub fn new(
        name: String, secret: String, fleet: Fleet, status: Status, field_size: Size
    ) -> Player {
        Player {
            name, secret, fleet, status, field_size
        }
    }

    pub fn placed(&self, class: &Class) -> bool {
        for ship in self.fleet.iter() {
            if ship.is_some() && ship.unwrap().class == *class {
                return true
            }
        }
        false
    }

    pub fn inside_field(&self, ship: &Ship) -> bool {
        if ship.coordinates.x >= self.field_size.width {
            false
        } else if ship.coordinates.y >= self.field_size.height {
            false
        } else {
            ship.tail_end() <= match ship.orientation {
                Orientation::Horizontal => self.field_size.width,
                Orientation::Vertical => self.field_size.height,
            } - 1
        }
    }

    pub fn overlap(&self, ship: &Ship) -> Option<Ship> {
        for other in &self.fleet {
            if other.is_some() && ship.overlap(other.as_ref().unwrap()) {
                return other.clone()
            }
        }
        None
    }

    pub fn check_placement(&self, ship: &Ship) -> Result<(), GameError> {
        if self.placed(&ship.class) {
            Err(GameError::ShipAlreadyPlaced(ship.class))
        } else if !self.inside_field(&ship) {
            Err(GameError::ShipOutOfBounds(ship.coordinates, ship.orientation, ship.class.size()))
        } else if let Some(other) = self.overlap(&ship) {
            Err(GameError::ShipOverlaps(other))
        } else {
            Ok(())
        }
    }

    pub fn find_empty_spot(&self) -> Option<usize> {
        for (index, option) in self.fleet.iter().enumerate() {
            if option.is_none() {
                return Some(index);
            }
        }
        None
    }

    pub fn place_ship(&self, ship: Ship) -> Result<Player, GameError> {
        self.check_placement(&ship)?;
        let mut player = self.clone();
        match self.find_empty_spot() {
            None => return Err(ShipAlreadyPlaced(ship.class)),
            Some(empty_spot) => player.fleet[empty_spot] = Some(ship),
        }
        Ok(player)
    }

    pub fn fleet_size(&self) -> usize {
        self.fleet.iter().filter(|option| option.is_some()).count()
    }

    pub fn is_fleet_complete(&self) -> bool {
        self.fleet_size() == 5
    }

    pub fn probe(&self, coordinates: &Point) -> Occupation {
        for option in self.fleet {
            if let Some(ship) = option {
                if ship.is_hit(coordinates) {
                    return ship.probe(coordinates);
                }
            }
        }
        Occupation::Empty
    }

    pub fn active_ships(&self) -> usize {
        active_ships(self.fleet)
    }

    pub fn is_defeated(&self) -> bool {
        is_defeated(self.fleet)
    }

    pub fn shoot(&self, coordinates: &Point) -> Impact<(Player, Ship)> {
        for (index, ship) in self.fleet.iter().enumerate() {
            if ship.is_some() {
                match ship.as_ref().unwrap().shoot(coordinates) {
                    Hit(hit_ship) => {
                        let mut new_fleet = self.fleet;
                        new_fleet[index] = Some(hit_ship);
                        return Hit((
                            Player::new(
                                self.name.clone(), self.secret.clone(), new_fleet,
                                if is_defeated(new_fleet) {
                                    Defeated
                                } else {
                                    self.status
                                },
                                self.field_size
                            ), hit_ship
                        ))
                    },
                    Miss => {}
                }
            }
        }
        Miss
    }
}

pub fn active_ships(fleet: Fleet) -> usize {
    fleet.iter()
        .filter(|option| option.is_some())
        .map(|option| option.unwrap())
        .filter(|ship| !ship.is_sunk())
        .count()
}

pub fn is_defeated(fleet: Fleet) -> bool {
    active_ships(fleet) == 0
}
