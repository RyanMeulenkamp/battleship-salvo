use crate::mqtt::{Mosquitto, MosquittoArc};
use log::{info, error};

mod attack;
mod grab;
mod orientation;
mod mqtt;
mod point;
mod translate;
mod turn;

use std::{thread, io};
use futures::executor::block_on;
use std::time::Duration;
use tokio::time::sleep;
use std::future::Future;
use std::io::{Read, BufRead};
use crate::orientation::Orientation::{Horizontal, Vertical};
use std::fmt::{Display, Formatter, format};
use core::fmt;
use crate::grab::{grab_number, grab_string, grab_coordinates, grab_orientation};
use translate::{
    deserialize,
    TranslationError
};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::{mpsc, Mutex, Arc};

use crate::turn::Turn;

enum State {
    Lobby,
    Underway,
    Over
}

struct Game {
    name: String,
    players: Vec<String>,
    // state: GameState,
    mqtt: MosquittoArc,
    ships: usize,
}

impl Game {
    pub async fn is_defeated(mqtt: &mut MosquittoArc, player: &String) -> bool {
        mqtt.await_topic(format!("/players/{}/defeated", player))
            .await
            .1
            .parse::<bool>()
            .unwrap_or(false)
    }
}

impl Iterator for Game {
    type Item = Turn;

    fn next(&mut self) -> Option<Turn> {
        let handle = tokio::runtime::Handle::current();
        let mqtt = self.mqtt.clone();
        let game_state = handle.block_on(mqtt.clone().await_topic("/game/state")).1;
        if game_state == "lobby" {
            return None;
        }

        loop {
            let current_player = handle.block_on(mqtt.clone().await_topic("/game/current")).1;
            if current_player == self.name {
                let mqtt = &mut mqtt.clone();
                return Some(Turn::new(
                    self.players.iter()
                        .filter(|player| **player != self.name)
                        .filter(|player| handle.block_on(Game::is_defeated(mqtt, player)))
                        .cloned()
                        .collect(),
                    self.ships,
                ));
            }

            let game_state = handle.block_on(self.mqtt.await_topic("/game/state")).1;
            if game_state == "over" {
                return None;
            }
        }
    }
}

#[tokio::main(worker_threads = 12)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    println!("Please put in a playername: ");
    let player = grab_string();
    // println!("Enter a password: ");
    // let secret = grab_string();
    println!("Enter your team channel: ");
    let prefix = grab_string();
    println!("{} playing on channel {}", player, prefix);

    let mut mqtt = MosquittoArc::new("nvs0495", 1883, &prefix, &player);

    mqtt.subscribe(
        format!("/{}/players/{}/ships/+/error", &prefix, &player),
        |topic, payload| info!("Received: {} on topic: {}", payload, topic)
    );

    let mut player_list = String::new();
    while !player_list.contains(&player) {
        player_list = mqtt.await_response(
            format!("/{}/game/request", &prefix),
            format!("{{ \"name\": \"{}\", \"secret\": \"Cockadoodledoo\"}}", &player),
            format!("/{}/players/list", &prefix)
        ).await.1;
        sleep(Duration::from_millis(1)).await;
    }

    mqtt.subscribe(
        format!("/{}/players/count", &prefix),
        |topic, payload| println!("Number of players: {}", payload)
    );

    sleep(Duration::from_millis(1)).await;
    for (_, ship) in [ "carrier", "battleship", "destroyer", "submarine", "patrolboat" ].iter().enumerate() {
        let mut success = false;
        while !success {
            println!("Enter coordinates [0 - 9] for {}:", ship);
            let (x, y) = grab_coordinates();

            println!("Enter orientation [0 = Horizontal, 1 = Vertical]: ");
            let orientation = grab_orientation();

            println!("Requesting placement at [{}; {}], oriented {}.", x, y, orientation);
            mqtt.publish(
                format!("/{}/players/{}/ships/{}/place", prefix, &player, ship),
                format!("{{ \"coordinates\": {{ \"x\": {}, \"y\": {} }}, \"orientation\": \"{}\" }}", x, y, orientation)
            );

            mqtt.await_topic(format!("/{}/players/{}/ships/{}/place", prefix, &player, ship)).await;
            let (topic, payload) = mqtt.await_topic(format!("/{}/players/{}/ships/{}/+", prefix, &player, ship)).await;

            if topic.ends_with("approved") && payload == "true" {
                println!("Placed {} successfully", ship);
                success = true;
            } else if topic.ends_with("error") {
                if payload == format!("{} class ship has already been placed!", ship) {
                    success = true;
                } else {
                    eprintln!("Error received: {}", payload);
                }
            } else {
                eprintln!("Received on another topic {}: {}", topic, payload);
            }
        }
    }

    println!("Waiting for game to start.");
    loop {
        let state = mqtt.await_topic(format!("/{}/game/state", &prefix)).await.1;
        println!("Gamestate:");
        // task::sleep(Duration::from_secs(1)).await;
    }

    let (sender, receiver): (Sender<Vec<String>>, Receiver<Vec<String>>) = mpsc::channel();
    let henkie = Arc::new(Mutex::new(vec!["a"]));
    let henkie_for_thread = henkie.clone();
    mqtt.subscribe(
        format!("/{}/players/list", &prefix),
        move |topic, payload| {
            let result: Result<Vec<String>, TranslationError> = deserialize(payload);
            if let Ok(result) = result {
                sender.send(result);
            }
            let mut henkie = henkie_for_thread.lock().unwrap();
        }
    );
    let players = receiver.recv().unwrap();

    let mut boats = 5;
    let target = loop {
        println!("Choose player to attack [{:?}]:", players.iter().enumerate());
        let player_index = grab_number();
        if player_index < players.len() as u8 {
            break players[player_index as usize].clone();
        }
    };

    loop {
        println!("Put in some coordinates to fire at:");
        let (x, y) = grab_coordinates();
        let shots_fired = mqtt.await_response(
            format!("/{}/players/{}/fire", &prefix, &target),
            format!("{{ \"x\": \"{}\", \"y\": \"{}\"}}", &x, &y),
            format!("/{}/game/fired_shots", &prefix),
        ).await.1;
        let shots_fired: u8 = shots_fired.parse().unwrap();
        if shots_fired == boats {
            break;
        }
    }



    println!("Game on!");

    Ok(())
}

