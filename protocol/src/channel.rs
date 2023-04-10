use crate::discord::{MessageType, Msg};
use async_channel::{bounded, Receiver, Sender};
use color_eyre::Result;
use std::time::Duration;
use tokio::time::timeout;

#[derive(Clone)]
pub struct Channel {
    pub one: (Sender<Msg>, Receiver<Msg>),
    pub two: (Sender<Msg>, Receiver<Msg>),
}

impl Default for Channel {
    fn default() -> Self {
        let (tx1, rx1): (Sender<Msg>, Receiver<Msg>) = bounded(4);
        let (tx2, rx2): (Sender<Msg>, Receiver<Msg>) = bounded(4);

        Channel {
            one: (tx1, rx1),
            two: (tx2, rx2),
        }
    }
}

impl Channel {
    pub async fn send_message(&mut self, msg: Msg) -> Result<()> {
        self.two.0.send(msg).await?;

        Ok(())
    }

    pub async fn receive_with_timeout(
        &mut self,
        timeout_duration: Duration,
        target: &str,
        packet_type: Option<MessageType>,
    ) -> Result<Option<Msg>> {
        timeout(timeout_duration, self.receive(target, packet_type)).await?
    }

    pub async fn receive(
        &mut self,
        target: &str,
        packet_type: Option<MessageType>,
    ) -> Result<Option<Msg>> {
        while let Ok(data) = self.one.1.recv().await {
            if data.sender.as_str().eq(target) {
                if let Some(p) = packet_type.clone() {
                    if p == data.message_type {
                        return Ok(Some(data));
                    }
                }

                return Ok(Some(data));
            }
        }

        Ok(None)
    }
}
