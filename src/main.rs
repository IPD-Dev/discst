use serde::Deserialize;
use serenity;
use serenity::model::prelude::MessageId;
use mcping;
use toml;


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
    println!("Hello, world!");
}
