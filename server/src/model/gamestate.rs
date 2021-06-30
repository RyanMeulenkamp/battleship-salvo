use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use core::fmt;

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Clone)]
pub enum GameState {
    Lobby,
    Underway((usize, String), u8, u8),
    Over(String),
}

impl Into<String> for GameState {
    fn into(self) -> String {
        String::from(match self {
            GameState::Lobby => "lobby",
            GameState::Underway(_, _, _) => "underway",
            GameState::Over(_) => "over",
        })
    }
}

impl Display for GameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            GameState::Lobby => format!("Lobby"),
            GameState::Underway((_, player), shots, _hits) => format!("{}'s turn. {} shots to go.", player, 5 - shots),
            GameState::Over(winner) => format!("Game over. {} won!", winner),
        })
    }
}
//
// impl GameState {
//     pub fn timeout(&self) -> u32 {
//         match self {
//             GameState::Lobby => 60,
//             GameState::Underway(_, _, _) => 10,
//             GameState::Over(_) => 300,
//         }
//     }
// }
