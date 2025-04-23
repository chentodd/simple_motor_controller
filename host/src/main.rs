use std::time::Duration;

use tokio::time::interval;
use host::client::Client;

#[tokio::main]
pub async fn main() {
    let client = Client::new();

    tokio::select! {
        // _ = client.wait_closed() => {
        //     println!("Client is closed, exiting...");
        // }
        _ = run(&client) => {
            println!("App is done")
        }
    }
}

async fn run(client: &Client) {
    let mut ticker = interval(Duration::from_millis(250));

    for i in 0..10 {
        ticker.tick().await;
        print!("Pinging with {i}... ");
        let res = client.ping(i).await;
        println!("got {res:?}!");
        // assert_eq!(res, i);
    }
}