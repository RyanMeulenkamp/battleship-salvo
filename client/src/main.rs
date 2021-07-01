use crate::mqtt::{Mosquitto, MosquittoArc};
use log::{info, error};

mod grab;
mod orientation;
mod mqtt;
mod translate;

use std::{thread, io};
use futures::executor::block_on;
use std::time::Duration;
use tokio::time::sleep;
use std::future::Future;
use std::io::{Read, BufRead};
use crate::orientation::Orientation::{Horizontal, Vertical};
use std::fmt::{Display, Formatter, format};
use core::fmt;
use crate::grab::{grab_string, grap_coordinates, grab_orientation};

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
            let (x, y) = grap_coordinates();

            println!("Enter orientation [0 = Horizontal, 1 = Vertical]: ");
            let orientation = grab_orientation();

            println!("Requesting placement at [{}; {}], oriented {}.", x, y, orientation);
            mqtt.publish(
                format!("/{}/players/{}/ships/{}/place", prefix, &player, ship),
                format!("{{ \"coordinates\": {{ \"x\": {}, \"y\": {} }}, \"orientation\": \"{}\" }}", x, y, orientation)
                // translate::encrypt(format!("{{ \"coordinates\": {{ \"x\": {}, \"y\": {} }}, \"orientation\": \"{}\" }}", x, y, orientation), &secret)
            );

            mqtt.await_topic(format!("/{}/players/{}/ships/{}/place", prefix, &player, ship)).await;
            let (topic, payload) = mqtt.await_topic(format!("/{}/players/{}/ships/{}/+", prefix, &player, ship)).await;

            if topic.ends_with("approved") && payload == "true" {
                println!("Placed {} successfully", ship);
                success = true;
            } else if topic.ends_with("error") {
                eprintln!("Error received: {}", payload);
            } else {
                eprintln!("Received on another topic {}: {}", topic, payload);
            }
        }
    }

    while mqtt.await_topic(format!("/{}/game/state", &prefix)).await.1.as_str() != "Ongoing" {

    }



    println!("Game on!");

    Ok(())
}

