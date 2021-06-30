use serde::{Deserialize, Serialize};

use crate::model::player::Player;
use crate::model::gamestate::GameState;
use crate::model::gamestate::GameState::Lobby;
use std::sync::{Arc, Mutex};
use log::info;
use crate::model::size::Size;
use delegate::delegate;

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Clone)]
pub struct Game {
    pub state: GameState,
    pub players: Vec<Player>,
    pub size: Size,
    pub prefix: String,
}

impl Game {
    pub fn new(size: Size, prefix: String) -> Self {
        Game {
            state: Lobby,
            players: vec![],
            size,
            prefix
        }
    }

    pub fn prefix(&self) -> String {
        self.prefix.clone()
    }

    pub fn start(&mut self, dice: fn(usize) -> usize) {
        let first_player = dice(self.players.len());
        self.state = GameState::Underway((first_player, self.players[first_player].name.clone()), 0, 0);
    }

    pub fn player_count(&self) -> usize {
        self.players.len()
    }

    pub fn player_list(&self) -> Box<[String]> {
        self.players.iter().map(|player| &player.name).cloned().collect()
    }

    pub fn get_player(&self, index: usize) -> Option<Player> {
        if index < self.player_count() {
            Some(self.players[index].clone())
        } else {
            None
        }
    }

    pub fn find_player(&self, name: impl Into<String>) -> Option<(usize, Player)> {
        let name = name.into();
        self.players.clone()
            .into_iter()
            .enumerate()
            .find(|(_, player)| player.name == name)
    }

    pub fn update_player(&mut self, player: Player) {
        info!("{}", player);
        match self.find_player(&player.name) {
            None => self.players.push(player),
            Some((index, _)) => {
                self.players.remove(index);
                self.players.push(player);
            }
        }
    }

    pub fn ready_players(&self) -> usize {
        self.players.iter().filter(|player| player.is_fleet_complete()).count()
    }

    pub fn active_player_count(&self) -> usize {
        self.players.iter().filter(|player| player.is_defeated()).count()
    }

    pub fn next_turn(&mut self) {
        if let GameState::Underway((index, _), _, _) = &self.state {
            let mut next_player_index = (index + 1) % self.player_count();
            while next_player_index != *index {
                if match self.get_player(next_player_index) {
                    Some(player) => player.is_defeated(),
                    _ => false,
                } {
                    break;
                } else {
                    next_player_index += 1;
                    if next_player_index >= self.player_count() {
                        next_player_index = 0;
                    }
                }
            };
            if let Some(next_player) = self.get_player(next_player_index) {
                let next_player_name = next_player.name.clone();
                self.state = GameState::Underway((next_player_index, next_player_name), 0, 0);
            }
        }
    }

    pub fn game_over(&mut self) {
        if let Some(winner) = self.players.iter().find(|player| !player.is_defeated()) {
            self.state = GameState::Over(winner.name.clone());
        }
    }
}

pub struct GameArc {
    inner: Arc<Mutex<Game>>
}

impl GameArc {
    pub fn new(size: Size, prefix: String) -> GameArc {
        GameArc {
            inner: Arc::new(Mutex::new(Game::new(size, prefix))),
        }
    }

    delegate! {
        to  self.inner.lock().unwrap() {
            pub fn start(&mut self, dice: fn(usize) -> usize);
        }
    }

    pub fn prefix(&self) -> String {
        self.inner.lock().unwrap().prefix()
    }

    pub fn player_count(&self) -> usize {
        self.inner.lock().unwrap().player_count()
    }

    pub fn player_list(&self) -> Box<[String]> {
        self.inner.lock().unwrap().player_list()
    }

    pub fn find_player(&self, name: impl Into<String>) -> Option<(usize, Player)> {
        self.inner.lock().unwrap().find_player(name)
    }

    pub fn update_player(&mut self, player: Player) {
        self.inner.lock().unwrap().update_player(player)
    }

    pub fn ready_player_count(&self) -> usize {
        self.inner.lock().unwrap().ready_players()
    }

    pub fn active_player_count(&self) -> usize {
        self.inner.lock().unwrap().active_player_count()
    }

    pub fn next_turn(&mut self) {
        self.inner.lock().unwrap().next_turn()
    }

    pub fn game_over(&mut self) {
        self.inner.lock().unwrap().game_over()
    }

    pub fn state(&self) -> GameState {
        self.inner.lock().unwrap().state.clone()
    }

    pub fn players(&self) -> Vec<String> {
        self.inner.lock().unwrap().players.iter().map(|player| player.name.clone()).collect()
    }
}

impl Clone for GameArc {
    fn clone(&self) -> Self {
        GameArc {
            inner: Arc::clone(&self.inner),
        }
    }
}
