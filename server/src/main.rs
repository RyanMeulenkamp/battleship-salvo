#![feature(repr128, async_closure)]

mod engine;
mod messaging;
mod model;

use crate::engine::start_engine;
use simple_log::LogConfigBuilder;
use model::size::Size;
use std::env;
use std::num::ParseIntError;
use futures::future::err;
use log::error;

#[tokio::main(worker_threads = 6)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = LogConfigBuilder::builder()
        .level("info")
        .output_console()
        .build();
    simple_log::new(config)?;

    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        error!("Please pass hostname, port, username and a game name as a command line argument!");
    } else {
        match args[2].parse() {
            Ok(port) => start_engine(Size::default(), &args[4], &args[1], port, &args[3]).await,
            Err(error) => error!("Unable to parse port: {:?}", error)
        }
    }
    Ok(())
}
