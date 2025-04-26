use std::{sync::Arc, time::Duration};

use host::client::Client;
use protocol::PositionCommand;
use tokio::time::interval;

#[tokio::main]
pub async fn main() {
    let client = Arc::new(Client::new("stm32-discovery"));

    tokio::join!(
        ping(client.clone()),
        subscribe(client.clone()),
        send_vel_cmd(client.clone()),
        send_pos_cmd(client.clone())
    );

    println!("Finished");
}

async fn ping(client: Arc<Client>) {
    println!("Check ping");

    let mut ticker = interval(Duration::from_millis(250));

    for i in 0..10 {
        ticker.tick().await;
        println!("ping {i}");
        let res = client.ping(i).await;
        println!("ping got {res:?}!");
    }
}

async fn subscribe(client: Arc<Client>) {
    println!("Check topic");

    let mut counter = 0_u32;
    let mut sub = client
        .client
        .subscribe_multi::<protocol::MotorProcessDataTopic>(8)
        .await
        .unwrap();

    loop {
        let msg = sub.recv().await.unwrap();
        counter += 1;
        
        if counter % 100 == 0 {
            counter = 0;
            println!("Got message: {msg:?}");
        }
    }
}

async fn send_vel_cmd(client: Arc<Client>) {
    println!("Check vel cmd");

    let vel = 500.0_f32;
    let mut ticker = interval(Duration::from_millis(50));

    for i in 0..10 {
        ticker.tick().await;
        let res = client
            .set_motor_cmd(
                protocol::MotorId::Left,
                protocol::MotorCommand::VelocityCommand(vel + i as f32 * 5.0),
            )
            .await;
        println!("send_vel_cmd got {res:?}!");
    }
}

async fn send_pos_cmd(client: Arc<Client>) {
    println!("Check pos cmd");

    let dummy_val = 500.0_f32;
    let mut ticker = interval(Duration::from_millis(50));

    for i in 0..10 {
        ticker.tick().await;

        let i = i as f32 + 1.0;
        let res = client
            .set_motor_cmd(
                protocol::MotorId::Left,
                protocol::MotorCommand::PositionCommand(PositionCommand {
                    displacement: dummy_val / i,
                    vel_max: dummy_val / i,
                    vel_end: dummy_val / i,
                }),
            )
            .await;
        println!("send_pos_cmd got {res:?}!");
    }
}
