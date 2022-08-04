use mc_query::status;
use serde::Deserialize;
use serenity;
use serenity::model::prelude::MessageId;
use std::env::args;
use std::fs::read_to_string;
use std::process::exit;
use toml::from_str;

#[derive(Deserialize)]
struct Config {
    token: String,
    message: MessageId,
    server: String,
    port: Option<u16>,
    up: String,
    down: String,
}

fn main() {
    let filename = &match args().nth(1) {
        Some(h) => h,
        None => {
            eprintln!(
                "usage: {} file",
                args().nth(0).expect("how did you do that")
            );
            exit(1);
        }
    };

    let contents = match read_to_string(filename) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("failed to read {}. {}", filename, e);
            exit(69);
        }
    };

    let config: Config = match from_str(&contents) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("failed to parse {}. {}", filename, e);
            exit(420);
        }
    };

    println!("the heccin port is {} lol", config.port.unwrap_or(25565));
}
