use std::sync::{Arc};
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use crate::discord::{MessageType, Msg};
use color_eyre::Result;
use tokio::time::timeout;

#[derive(Clone)]
pub struct Channel {
    pub one: (Arc<Sender<Msg>>, Arc<Mutex<Receiver<Msg>>>),
    pub two: (Arc<Sender<Msg>>, Arc<Mutex<Receiver<Msg>>>)
}

impl Default for Channel {
    fn default() -> Self {
        let (tx1, rx1): (Sender<Msg>, Receiver<Msg>) = mpsc::channel(4);
        let (tx2, rx2): (Sender<Msg>, Receiver<Msg>) = mpsc::channel(4);

        Channel {
            one: (Arc::new(tx1), Arc::new(Mutex::new(rx1))),
            two: (Arc::new(tx2), Arc::new(Mutex::new(rx2)))
        }
    }
}

impl Channel {
    pub async fn send_message(&mut self, msg: Msg) -> Result<()> {
        self.two.0.send(msg).await?;

        Ok(())
    }

    pub async fn receive_with_timeout(&mut self, timeout_duration: Duration, target: &str, packet_type: Option<MessageType>) -> Result<Option<Msg>> {
        timeout(timeout_duration, self.receive(target, packet_type)).await?
    }

    pub async fn receive(&mut self, target: &str, packet_type: Option<MessageType>) -> Result<Option<Msg>> {
        while let Some(data) = self.one.1.lock().await.recv().await {
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