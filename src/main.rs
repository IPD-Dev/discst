use mc_query::status;
use serde::Deserialize;
use std::env::args;
use std::fs::read_to_string;
use std::process::exit;
use std::sync::Arc;
use tokio;
use toml::from_str;

use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::model::prelude::MessageId;
use serenity::prelude::*;

#[derive(Deserialize)]
struct Config {
    token: String,
    message: MessageId,
    server: String,
    port: Option<u16>,
    up: String,
    down: String,
}

fn readconfig() -> Config {
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

    return config;
}

struct ConfigContainer;
impl TypeMapKey for ConfigContainer {
    type Value = Arc<Config>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} connected", ready.user.name);
        let config = {
            let data_read = ctx.data.read().await;
            data_read.get::<ConfigContainer>().expect("gimme the config").clone()
        };
        println!("{}", config.server);
    }
}

#[tokio::main]
async fn main() {
    let config = readconfig();

    let intents = GatewayIntents::empty();

    let mut client = Client::builder(&config.token, intents).event_handler(Handler).await.expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ConfigContainer>(Arc::new(config));
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
