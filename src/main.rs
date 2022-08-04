use serde::Deserialize;
use serenity;
use serenity::model::prelude::MessageId;
use mcping;
use toml::from_str;
use std::fs::read_to_string;
use std::process::exit;


#[derive(Deserialize)]
struct Config {
    token: String,
    message: MessageId,
    server: String,
    port: Option<u16>,
    up: String,
    down: String
}

fn main() {
    let filename = "discst.toml";
    println!("Hello, {}!",filename);

    let contents = match read_to_string(filename) {
        Ok(h) => h,
        Err(_a) => {
            eprintln!("failed to read {} ut oh", filename);
            exit(69);
        }
    };

    let config: Config = match from_str(&contents) {
        Ok(h) => h,
        Err(_a) => {
            eprintln!("failed to parse {}. did you put all the required options?", filename);
            exit(69);
        }
    };

    println!("the heccin port is {} lol", config.port.unwrap_or(25565));
}
