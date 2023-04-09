use std::sync::Arc;
use serenity::{async_trait, Client};
use serenity::prelude::{Context, EventHandler, GatewayIntents, TypeMapKey};
use color_eyre::Result;
use serenity::model::gateway::Ready;
use serenity::model::prelude::Message;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{Mutex};
use crate::constant;

constant!(PACKET_CONNECTION_REQUEST, "0");
constant!(PACKET_CONNECTION_RESPONSE_ACCEPTED, "1");
constant!(PACKET_CONNECTION_RESPONSE_REJECTED, "2");
constant!(PACKET_PING, "3");
constant!(PACKET_PONG, "4");
constant!(PACKET_MESSAGE, "5");

#[derive(Clone)]
pub struct DiscordState {
    hub_channel_id: String,
    tx: Arc<Sender<Msg>>,
    rx: Arc<Mutex<Receiver<Msg>>>
}

#[derive(Debug, PartialEq, Clone)]
pub enum MessageType {
    ConnectionRequest,
    ConnectionAccepted,
    ConnectionRejected,
    Ping,
    Pong,
    Message
}

#[derive(Debug, Clone)]
pub struct Msg {
    pub message_type: MessageType,
    pub content: Option<String>,
    pub sender: String
}

struct DiscordHandler;

impl TypeMapKey for DiscordState {
    type Value = DiscordState;
}

#[async_trait]
impl EventHandler for DiscordHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        let mut write = ctx.data.write().await;

        if let Some(state) = write.get_mut::<DiscordState>() {
            if msg.channel_id.to_string() != state.hub_channel_id {
               return;
            }

            let message = msg.content.splitn(3, ' ');

            let out = message.collect::<Vec<_>>();

            if out.len() < 3 {
                println!("{:?}", out);

                return;
            }

            match out[1] {
                PACKET_CONNECTION_REQUEST => {
                    state.tx.send(Msg { message_type: MessageType::ConnectionRequest, content: None, sender: out[0].to_string() }).await.unwrap();
                },

                PACKET_CONNECTION_RESPONSE_ACCEPTED => {
                    state.tx.send(Msg { message_type: MessageType::ConnectionAccepted, content: None, sender: out[0].to_string() }).await.unwrap();
                },

                PACKET_CONNECTION_RESPONSE_REJECTED=> {
                    state.tx.send(Msg { message_type: MessageType::ConnectionRejected, content: None, sender: out[0].to_string() }).await.unwrap();
                },

                PACKET_MESSAGE => {
                    state.tx.send(Msg { message_type: MessageType::Message, content: Some(out[2].to_string()), sender: out[0].to_string() }).await.unwrap();
                }

                _ => {}
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let read = ctx.data.read().await;

        if let Some(state) = read.get::<DiscordState>() {
            let channel_id = state.hub_channel_id.clone();
            let rx = state.rx.clone();

            tokio::spawn(async move {
                let mut rx = rx.lock().await;

                let channel = ctx.http.get_channel(channel_id.parse().unwrap()).await.unwrap();

                while let Some(data) = rx.recv().await {
                    match data.message_type {
                        MessageType::ConnectionRequest => {
                            channel.id().say(&ctx.http, format!("{} {} {}", data.sender, PACKET_CONNECTION_REQUEST, data.content.unwrap())).await.unwrap();
                        },

                        MessageType::ConnectionAccepted => {
                            channel.id().say(&ctx.http, format!("{} {} {}", data.sender, PACKET_CONNECTION_RESPONSE_ACCEPTED, data.content.unwrap())).await.unwrap();
                        },

                        MessageType::ConnectionRejected => {
                            channel.id().say(&ctx.http, format!("{} {} {}", data.sender, PACKET_CONNECTION_RESPONSE_REJECTED, data.content.unwrap())).await.unwrap();
                        },

                        MessageType::Ping => {
                            channel.id().say(&ctx.http, format!("{} {}", data.sender, PACKET_PING)).await.unwrap();
                        }

                        MessageType::Pong => {
                            channel.id().say(&ctx.http, format!("{} {}", data.sender, PACKET_PONG)).await.unwrap();
                        }

                        MessageType::Message => {
                            channel.id().say(&ctx.http, format!("{} {} {}", data.sender, PACKET_CONNECTION_REQUEST, data.content.unwrap())).await.unwrap();
                        }
                    }
                }
            });
        }
    }
}

pub async fn new_discord_protocol(token: String, hub_channel_id: String, tx: Arc<Sender<Msg>>, rx: Arc<Mutex<Receiver<Msg>>>) -> Result<Client> {
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let client = Client::builder(&token, intents).event_handler(DiscordHandler).await?;

    let discord =  DiscordState {
        hub_channel_id,
        tx,
        rx
    };
    let mut data = client.data.write().await;

    data.insert::<DiscordState>(discord);

    drop(data);

    Ok(client)
}