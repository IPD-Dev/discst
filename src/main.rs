use async_minecraft_ping;
use serde::Deserialize;
use std::env::args;
use std::fs::read_to_string;
use std::process::exit;
use std::sync::Arc;
use std::time::Duration;
use tokio;
use toml::from_str;

use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

#[derive(Deserialize)]
struct Config {
    token: String,
    message: u64,
    channel: u64,
    server: String,
    port: Option<u16>,
    interval: Option<u64>,
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
            data_read
                .get::<ConfigContainer>()
                .expect("gimme the config")
                .clone()
        };
        tokio::spawn(async move {
            start_ping_interval(config, ctx).await;
        });
    }
}

async fn start_ping_interval(config: Arc<Config>, ctx: Context) {
    let mut interval = tokio::time::interval(Duration::from_secs(config.interval.unwrap_or(30)));

    loop {
        interval.tick().await;
        update_status_msg(
            &ctx,
            config.channel,
            config.message,
            (mcstatus_from_config(&config).await).to_string(),
        )
        .await;
    }
}

async fn update_status_msg(ctx: &Context, channelid: u64, message: u64, text: String) {
    let mut discord_message = match ctx.http.get_message(channelid, message).await {
        Ok(h) => h,
        Err(e) => {
            println!("error getting message: {}", e);
            return;
        }
    };
    match discord_message.edit(&ctx, |m| m.content(text)).await {
        Ok(h) => h,
        Err(e) => {
            println!("error editing message: {}", e);
            return;
        }
    };
}

async fn mcstatus_from_config(config: &Arc<Config>) -> &String {
    let mut status = async_minecraft_ping::ConnectionConfig::build(&config.server);
    status = status.with_port(config.port.unwrap_or(25565));

    let connection = match status.connect().await {
        Ok(h) => h,
        Err(_) => return &config.down,
    };
    match connection.status().await {
        Ok(_) => return &config.up,
        Err(_) => return &config.down,
    };
}

#[tokio::main]
async fn main() {
    let config = readconfig();

    let intents = GatewayIntents::empty();

    let mut client = Client::builder(&config.token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ConfigContainer>(Arc::new(config));
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
