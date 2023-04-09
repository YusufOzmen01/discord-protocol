use std::env;
use std::io::stdin;
use std::sync::{Arc};
use color_eyre::Result;
use protocol::p2p::P2P;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<()>  {
    color_eyre::install()?;

    let p2p = Arc::new(Mutex::new(P2P::new(env::var("TOKEN")?, env::var("CHANNEL_ID")?)));

    println!("YOUR ID: {}", p2p.lock().await.get_id().await);

    let stdin = stdin();
    print!("Target ID: ");

    let mut buffer = String::new();
    stdin.read_line(&mut buffer).unwrap();

    println!("Requesting connection to {}...", buffer.trim());

    if p2p.lock().await.connect(buffer).await.is_err() {
        println!("Connection failed!");
    }

    println!("Connection successful!");

    tokio::spawn(async move {
        while let Ok(data) = p2p.lock().await.receive(None).await {
            if let Some(data) = data {
                println!("{}", data);
            }
        }
    }).await?;

    Ok(())
}
