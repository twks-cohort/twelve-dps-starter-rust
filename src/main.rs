use std::env;
use anyhow::Result;
use serenity::{async_trait, http::Http, model::{channel::Message, gateway::Ready, id::{ChannelId}}, prelude::*};
use tokio::time::{interval, Duration};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

struct Responder;

#[async_trait]
impl EventHandler for Responder {
    /// React to messages
    /// The actual listening is handled with some Serenity magic.
    /// Here, we just match on the message content and react .
    async fn message(&self, ctx: Context, msg: Message) {
        match msg.content.as_ref() {
            "!check" => match Updater::react(&msg.channel_id, &ctx).await {
                Ok(_) => {}
                Err(err) => println!("{}", err),
            },
            "lorem ipsum" => {},
            _ => {}
        }
    }
    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

pub struct Updater {
    token: String,
    channel_id: ChannelId,
}

impl Updater {
    pub fn updater_new(token: String, channel_id: ChannelId) -> Result<Self> {
        Ok(
        Self{
            token, channel_id
        })
    }

    pub fn updater_from_config(token: String, channel_id: ChannelId) -> Result<Self> {
        Self::updater_new(token, channel_id)
    }

    // TODO - msg passing
    async fn react(channel_id: &ChannelId, ctx: &Context) -> Result<Message> {
        let msgContent = "Hello!";
        let msg = channel_id.say(&ctx.http, &msgContent).await?;
        Ok(msg)
    }

    pub async fn update_loop(&self) -> Result<()> {
        let http = &Http::new_with_token(&self.token);
        let mut interval = interval(Duration::from_secs(10));

        // Set to true to enable looping messages to the channel
        //   TODO - this can go away when we build in a proper debug fn and environment variable
        let debug_loop_enabled = false;
        loop {
            interval.tick().await;
            if debug_loop_enabled {
                self.channel_id.say(http, "Hey preview deployement!").await?;
            }
        }
    }
}

pub async fn try_responder_client_and_updater_from_config(token: String) -> Result<(Client, Updater)> {
        let responder = Client::builder(&token)
            .event_handler(Responder)
            .await
            .expect("Err creating client");

        let channel_id = env::var("CHANNEL_ID").expect("Expected a channel_id in the environment");
        let channel_u64 = channel_id.parse::<u64>().unwrap();
        let channel_id = ChannelId(channel_u64);
        let updater = Updater::updater_from_config(token, channel_id)?;
        Ok((responder, updater))
}

#[tokio::main]
async fn main() {

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let (mut responder, updater) = try_responder_client_and_updater_from_config(token).await.expect("Create bot failed.");

    tokio::select! {
        _ = updater.update_loop() => {
            println!("The updater stopped unexpectedly")
        }
        _ = responder.start() => {
            println!("The responder stopped unexpectedly")
        }
    };
}


