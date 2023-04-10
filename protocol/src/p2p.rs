use std::time::Duration;
use color_eyre::eyre::format_err;
use rand::distributions::{Alphanumeric, DistString};
use crate::channel::Channel;
use crate::discord::{MessageType, Msg, new_discord_protocol};
use color_eyre::Result;

pub struct P2P {
    pub other_end: Option<String>,
    channel: Channel,
    id: String
}

impl P2P {
    pub fn new(token: String, channel_id: String) -> P2P {
        let p2p = P2P {
            other_end: None,
            channel: Channel::default(),
            id: Alphanumeric.sample_string(&mut rand::thread_rng(), 8)
        };

        let ch = p2p.channel.clone();

        tokio::spawn(async move {
            new_discord_protocol(token, channel_id, ch.clone().one.0, ch.clone().two.1).await.unwrap().start().await.unwrap();
        });

        p2p
    }

    pub async fn connect(&mut self, id: String) -> Result<()> {
                let payload = Msg {
            message_type: MessageType::ConnectionRequest,
            content: Some(id.clone()),
            sender: self.id.to_string()
        };

        self.channel.send_message(payload).await?;

        self.channel.receive_with_timeout(Duration::from_secs(15), id.clone().as_str().trim(), Some(MessageType::ConnectionAccepted)).await?;

        self.other_end = Some(id);

        Ok(())
    }

    pub fn get_id(&self) -> String {
        self.id.to_string()
    }

    pub fn disconnect(&mut self) -> Result<()> {
        if self.other_end.is_none() {
            return Err(format_err!("You are not connected to someone!"))
        }

        self.other_end = None;

        Ok(())
    }

    pub async fn receive(&mut self, timeout: Option<Duration>) -> Result<Option<String>> {
        if self.other_end.is_none() {
            return Err(format_err!("You are not connected to someone!"))
        }

        let data = {
            if let Some(timeout) = timeout {
                self.channel.receive_with_timeout(timeout, self.other_end.clone().unwrap().as_str().trim(), None).await?
            } else {
                self.channel.receive(self.other_end.clone().unwrap().as_str().trim(), None).await?
            }
        };

        if let Some(data) = data {
            return Ok(data.content)
        }

        Ok(None)
    }

    pub async fn send(&mut self, message: String) -> Result<()> {
        if self.other_end.is_none() {
            return Err(format_err!("You are not connected to someone!"))
        }

        self.channel.send_message(Msg { message_type: MessageType::Message, content: Some(message), sender: self.id.clone() }).await
    }
}