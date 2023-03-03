// horribly messy little script thrown together to
// send a message to a channel

use std::env;

use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        let channelid = env::var("DISCORD_CHANNEL")
            .expect("Expected a DISCORD_CHANNEL in the environment")
            .parse::<u64>();
        ctx.http
            .get_channel(channelid.expect("invalid channel id"))
            .await
            .expect("h")
            .guild()
            .expect("goold")
            .say(&ctx.http, "h")
            .await
            .expect("failed to send");
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a DISCORD_TOKEN in the environment");
    let intents = GatewayIntents::empty();

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
