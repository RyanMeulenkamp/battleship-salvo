#[macro_use]
#[cfg(test)]
mod tests {
    use crate::model::{
        ship::Ship, point::Point, range::Range, player::Player, game::Game
    };
    use crate::model::class::Class::{Carrier, Battleship, Destroyer, Submarine, PatrolBoat};
    use lazy_static::lazy_static;
    use std::ops::Deref;
    use crate::model::orientation::Orientation::{Horizontal, Vertical};
    use crate::model::status::Status::Requested;

    lazy_static! {
        static ref CARRIER: Ship = Ship::new(
            Point::new(1, 1), Vertical, Carrier([false; 5])
        );
        static ref BATTLESHIP: Ship = Ship::new(
            Point::new(2, 2), Horizontal, Battleship([false; 4])
        );
        static ref DESTROYER: Ship = Ship::new(
            Point::new(2, 2), Vertical, Destroyer([false; 3])
        );
        static ref SUBMARINE: Ship = Ship::new(
            Point::new(3, 1), Vertical, Submarine([false; 3])
        );
        static ref PATROL_BOAT: Ship = Ship::new(
            Point::new(5,2), Horizontal, PatrolBoat([false; 2])
        );
        static ref PLAYER: Player = Player::new(
            "Henkie".to_string(),
            "Cockadoodledoo".to_string(),
            [Some(*BATTLESHIP), None, None, None, None],
            Requested
        );
        static ref GAME: Game = Game::from(vec![PLAYER.clone()]);
    }

    #[test]
    fn test_transposing_ships() {
        assert_eq!(
            PATROL_BOAT.transposed_to(Horizontal),
            *PATROL_BOAT
        );
        assert_eq!(
            PATROL_BOAT.transposed_to(Vertical),
            Ship {
                coordinates: Point {
                    x: 2,
                    y: 5,
                },
                orientation: Vertical,
                class: PatrolBoat([false; 2])
            }
        );
    }

    #[test]
    fn test_tail_end() {
        assert_eq!(CARRIER.tail_end(), 5);
        assert_eq!(PATROL_BOAT.tail_end(), 6);
    }

    #[test]
    fn test_range() {
        assert!(Range::new(2, 5).within(2));
        assert!(Range::new(2, 5).within(3));
        assert!(!Range::new(2, 5).within(1));
        assert!(!Range::new(2, 5).within(6));

        assert!(Range::new(2, 5).overlap(Range::new(5, 8)));
        assert!(Range::new(4, 7).overlap(Range::new(2, 5)));
        assert!(!Range::new(2, 5).overlap(Range::new(6, 8)));
        assert!(!Range::new(5, 7).overlap(Range::new(2, 4)));
    }

    #[test]
    fn test_overlapping_ships() {
        let ship_pairs = [
            (*CARRIER, *BATTLESHIP, false, "Non overlapping"),
            (*BATTLESHIP, *DESTROYER, true, "Identical coordinates with different orientations"),
            (*BATTLESHIP, *BATTLESHIP, true, "Identical coordinates with same orientation"),
            (*BATTLESHIP, *SUBMARINE, true, "Overlapping with different orientations"),
            (*BATTLESHIP, *PATROL_BOAT, true, "Overlapping with same orientation"),
        ];

        for (one, other, expected, message) in ship_pairs.iter() {
            let one_result = one.overlap(other);
            let other_result = other.overlap(one);
        //
        //     info!(
        //         "{}:\n  One: {:?}, result: {}\n  Another: {:?}, result: {}",
        //         message, one, one_result, other, other_result
        //     );
        //
        //     assert_eq!(one_result, *expected);
        //     assert_eq!(one_result, other_result);
        }
    }

    #[test]
    fn test_placement_existing_class() {
        assert!(!PLAYER.check_placement(
            &Ship {
                coordinates: Point {
                    x: 4,
                    y: 7,
                },
                orientation: Horizontal,
                class: Battleship([false; 4]),
            }
        ))
    }

    #[test]
    fn test_placement_outside_field() {
        assert!(!PLAYER.check_placement(
            &Ship {
                coordinates: Point {
                    x: 4,
                    y: 7,
                },
                orientation: Vertical,
                class: Carrier([false; 5]),
            }
        ))
    }

    #[test]
    fn test_overlapping_placement() {
        assert!(!PLAYER.check_placement(DESTROYER.deref()))
    }

    #[test]
    fn test_correct_placement() {
        assert!(PLAYER.check_placement(CARRIER.deref()))
    }

    #[test]
    fn test_shoot_and_hit() {
        let mut game = GAME.deref().clone();
        assert!(match game.fire("Henkie", &Point { x: 4, y: 2 }) {
            Ok(result) => result,
            Err(error) => false,
        });
    }

    #[test]
    fn test_shoot_and_miss() {
        let mut game = GAME.deref().clone();
        assert!(!match game.fire(String::from("Henkie"), &Point { x: 4, y: 3 }) {
            Ok(result) => result,
            Err(error) => false,
        });
    }
}
