use serde::Deserialize;
use std::env::args;
use std::fs::read_to_string;
use std::process::exit;
use std::sync::Arc;
use std::time::Duration;

use toml::from_str;

use serenity::async_trait;
use serenity::model::gateway::Activity;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

#[derive(Deserialize)]
struct Config {
    token: String,
    channel: u64,
    server: String,
    port: Option<u16>,
    interval: Option<u64>,
    minplayers: Option<u32>,
    up: String,
    down: String,
}

fn readconfig() -> Config {
    let filename = &match args().nth(1) {
        Some(h) => h,
        None => {
            eprintln!(
                "usage: {} file",
                args().next().expect("how did you do that")
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

    config
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

        set_playing_status(&ctx, &config.server).await;

        tokio::spawn(async move {
            start_ping_interval(config, ctx).await;
        });
    }
}

async fn set_playing_status(ctx: &Context, server: &String) {
    ctx.set_activity(Activity::playing(format!("on {}", server)))
        .await;
}

async fn start_ping_interval(config: Arc<Config>, ctx: Context) {
    let mut interval = tokio::time::interval(Duration::from_secs(config.interval.unwrap_or(30)));
    let mut last_message = "".to_string();
    let mut message;

    loop {
        interval.tick().await;
        message = (mcstatus_from_config(&config).await).to_string();

        if message != last_message {
            println!("updating to {}", &message);
            update_status_channel(&ctx, config.channel, message.to_string()).await;
        }

        // FIXME: this clone could probably be replaced by something faster
        last_message = message.clone();
    }
}

async fn update_status_channel(ctx: &Context, channelid: u64, text: String) {
    let discord_channel = match ctx.http.get_channel(channelid).await {
        Ok(h) => h,
        Err(e) => {
            println!("error getting channel: {}", e);
            return;
        }
    };
    let mut discord_channel = match discord_channel.guild() {
        Some(h) => h,
        None => {
            println!("im not in the guild lol");
            return;
        }
    };
    match discord_channel.edit(&ctx, |m| m.name(text)).await {
        Ok(h) => h,
        Err(e) => {
            println!("error editing channel: {}", e);
        }
    }
}

async fn mcstatus_from_config(config: &Arc<Config>) -> &String {
    let mut status = async_minecraft_ping::ConnectionConfig::build(&config.server);
    status = status.with_port(config.port.unwrap_or(25565));

    let connection = match status.connect().await {
        Ok(h) => h,
        Err(_) => return &config.down,
    };
    match connection.status().await {
        Ok(c) => {
            if c.status.players.online >= config.minplayers.unwrap_or(0) {
                &config.up
            } else {
                &config.down
            }
        }
        Err(_) => &config.down,
    }
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
