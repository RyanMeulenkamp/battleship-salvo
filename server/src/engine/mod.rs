
use crate::model::{
    game::GameArc,
    player::Player,
    class::Class,
    ship::Ship,
    gameerror::GameError,
    gamestate::GameState::{
        self, Underway
    },
    point::Point,
    impact::Impact,
    size::Size,
};
use crate::messaging::{
    translate,
    translate::deserialize,
    translate::serialize,
    translate::decrypt,
    mqtt::MosquittoArc,
};
use log::info;
use strum::IntoEnumIterator;
use rand::Rng;

pub async fn start_engine(size: Size, prefix: impl Into<String>, host: impl Into<String>, port: u16, user: impl Into<String>) {
    let prefix = prefix.into();
    let game = GameArc::new(size, prefix.clone());
    let (mqtt, future) = MosquittoArc::new(prefix + "-server", host, port, user);

    subscribe_player(game.clone(), mqtt.clone());
    mqtt.publish(format!("/{}/game/server", game.prefix()), "up");

    let (result_1, _result_2, result_3) = future.await;
    result_1.unwrap();
    result_3.unwrap();
}

fn subscribe_player(mut game_arc: GameArc, mut mqtt_arc: MosquittoArc) {
    let mqtt = mqtt_arc.clone();
    mqtt_arc.subscribe(format!("/{}/game/request", game_arc.prefix()),  move |topic, payload| {
        info!("Received: {} on topic {}", payload, topic);
        let result: translate::Result<Player> = deserialize(payload);
        match result {
            Ok(player) => {
                let playername = player.name.clone();
                game_arc.update_player(player);
                for class in Class::iter() {
                    info!("Create ship placement endpoint: {}", class);
                    place_ship(game_arc.clone(), mqtt.clone(), playername.clone(), class.name());
                }
            },
            Err(error) => info!("Error occured when deserializing player message: {:?}", error),
        }
        mqtt.retain(format!("/{}/players/count", game_arc.prefix()), format!("{}", game_arc.player_count()));

        match serialize(&game_arc.player_list()) {
            Ok(list) => mqtt.retain(format!("/{}/players/list", game_arc.prefix()), list),
            _ => {}
        };
    })
}

fn place_ship(
    mut game_arc: GameArc, mut mqtt_arc: MosquittoArc, playername: String, class: String
) {
    let mqtt = mqtt_arc.clone();
    mqtt_arc.subscribe(format!("/{}/players/{}/ships/{}/place", game_arc.prefix(), &playername, class), move |topic, payload| {
        assert_eq!(&format!("/{}/players/{}/ships/{}/place", game_arc.prefix(), playername, class), topic);
        info!("Received ship placement {} on topic {}", payload, topic);

        let game = &mut game_arc;
        let player = match game.find_player(&playername) {
            Some((_, player)) => player,
            _ => return,
        };
        //
        // let payload = match decrypt(&player.secret, payload) {
        //     Ok(payload) => payload,
        //     _ => return,
        // };

        let ship: Ship = match Ship::from_json(payload, class.as_str().into()) {
            Ok(ship) => ship,
            Err(error) => {
                mqtt.publish(
                    format!("/{}/players/{}/ships/{}/error", game_arc.prefix(), &playername, &class),
                    format!("Deserializing ship failed: {:?}", error)
                );
                return;
            }
        };

        match player.place_ship(ship) {
            Ok(player) => {
                mqtt.retain(
                    format!("/{}/players/{}/ships/count", game.prefix(), &playername),
                    &player.active_ships().to_string()
                );
                game.update_player(player);
                mqtt.retain(
                    format!("/{}/players/{}/ships/{}/approved", game.prefix(), &playername, &class),
                    "true"
                );
                print!("{} has successfully placed {}.", playername, class);

                if game.ready_player_count() == game.player_count() {
                    start_game(game_arc.clone(), mqtt.clone())
                }
            }
            Err(error) => {
                mqtt.publish(
                    format!("/{}/players/{}/ships/{}/error", game.prefix(), &playername, &class),
                    format!("{}", error)
                );
            }
        }
    });
}

fn next_turn(game: &mut GameArc, mqtt: &mut MosquittoArc) {
    game.next_turn();
    if let Underway((_index, player), fired_shots, _hits) = game.state() {
        mqtt.retain(format!("/{}/game/fired_shots", game.prefix()), format!("{}", fired_shots));
        mqtt.retain(format!("/{}/game/current", game.prefix()), &player);
    }
}

fn start_game(mut game: GameArc, mut mqtt: MosquittoArc) {
    mqtt.unsubscribe(format!("/{}/game/request", game.prefix()));
    for player in game.players() {
        for class in Class::iter() {
            mqtt.clear(format!("/{}/players/{}/ships/{}/approved", game.prefix(), &player, &class));
            mqtt.unsubscribe(format!("/{}/players/{}/ships/{}/place", game.prefix(), player, class));
        }
    }
    game.start(|size| rand::thread_rng().gen_range(0..size));
    let gamestate: String = game.state().into();
    mqtt.retain(format!("/{}/game/state", game.prefix()), &gamestate);
    next_turn(&mut game, &mut mqtt);
    for player in game.players() {
        perform_salvo(game.clone(), mqtt.clone(), player);
    }
}

fn perform_salvo(mut game_arc: GameArc, mut mqtt_arc: MosquittoArc, target_player: String) {
    let mut mqtt = mqtt_arc.clone();
    mqtt_arc.subscribe(format!("/{}/players/{}/fire", game_arc.prefix(), &target_player), move |topic, payload| {
        assert!(format!("/{}/players/{}/fire", game_arc.prefix(), &target_player) == *topic);
        info!("Received shot on topic {}, coordinates: {}", topic, payload);

        let mut game = &mut game_arc;

        let (current_player, fired_shots) = match game.state() {
            GameState::Underway((_, player), fired_shots, _) => (player, fired_shots),
            _ => return,
        };

        if current_player == target_player {
            info!("You shootn' yourself there!?");
        }

        let current_player = match game.find_player(current_player) {
            Some((_, current_player)) => current_player,
            _ => return,
        };

        // let coordinates: Point = match translate::verify(payload, &current_player.secret) {
        //     Ok(payload) => payload,
        //     _ => return,
        // };

        let coordinates: Point = match deserialize(payload) {
            Ok(coordinates) => coordinates,
            _ => return,
        };

        let target_player = match game.find_player(&target_player) {
            Some((_, player)) => player,
            _ => return,
        };

        if let Ok(json) = serialize(&coordinates) {
            mqtt.publish(format!("/{}/players/{}/hit", game.prefix(), &target_player.name), json);
        }

        match target_player.shoot(&coordinates) {
            Impact::Miss => info!("That's a miss!"),
            Impact::Hit((updated_target_player, hit_ship)) => {
                info!(
                    "That's a hit! {} hit {}'s {}", &current_player.name, &target_player.name,
                    hit_ship.class
                );
                let target_player = updated_target_player.clone();
                game.update_player(updated_target_player);

                if hit_ship.is_sunk() {
                    info!("Player {}'s {} sunk!", target_player.name, hit_ship.class);
                    mqtt.publish(
                        format!("/{}/players/{}/ships/{}/sunk", game.prefix(), target_player.name, hit_ship.class),
                        "true"
                    );
                    mqtt.retain(
                        format!("/{}/players/{}/ships/count", game.prefix(), target_player.name),
                        target_player.active_ships().to_string()
                    );

                    if target_player.is_defeated() {
                        info!("Player {}'s is now defeated!", target_player.name);
                        mqtt.publish(format!("/{}/players/{}/defeated", game.prefix(), target_player.name), "true");

                        if game.active_player_count() == 1 {
                            game_over(game, &mut mqtt);
                            return;
                        }
                    }
                }
            }
        };

        mqtt.retain(format!("/{}/game/fired_shots", game.prefix()), format!("{}", fired_shots));

        if fired_shots >= current_player.active_ships() as u8 {
            next_turn(&mut game, &mut mqtt);
        }
    });
}

fn game_over(game: &mut GameArc, mqtt: &mut MosquittoArc) {

    for player in game.players() {
        mqtt.clear(format!("/{}/players/{}/ships/count", game.prefix(), &player));
        mqtt.unsubscribe(format!("/{}/players/{}/fire", game.prefix(), &player));
    }

    mqtt.clear(format!("/{}/game/state", game.prefix()));
    mqtt.clear(format!("/{}/players/count", game.prefix()));
    mqtt.clear(format!("/{}/players/list", game.prefix()));
    mqtt.clear(format!("/{}/game/fired_shots", game.prefix()));
    mqtt.clear(format!("/{}/game/current", game.prefix()));

    game.game_over();
    let state = game.state();
    mqtt.publish(format!("/{}/game/state", game.prefix()), format!("{}", &state));

    if let GameState::Over(winner) = &state {
        mqtt.publish(format!("/{}/game/winner", game.prefix()), winner);
    }
}
