use example::switch::client::ZenohClientSwitch;
use example::switch::r#virtual::ZenohVirtualSwitch;
use example::switch::{Switch, SwitchState};

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use zenoh::prelude::r#async::*;

#[tokio::main]
async fn main() {
    // Create a Zenoh session
    let session = Arc::new(zenoh::open(config::peer()).res().await.unwrap());

    // if the first argument is 'put'
    if std::env::args().nth(1).unwrap() == "put" {
        loop {
            session
                .put("test", "hello")
                .encoding(KnownEncoding::TextPlain)
                .res()
                .await
                .unwrap();
            eprintln!("put");
            sleep(Duration::from_secs(1)).await;
        }
    }

    // if the first argument is 'get'
    if std::env::args().nth(1).unwrap() == "get" {
        let sub = session.declare_subscriber("test").res().await.unwrap();
        tokio::task::spawn(async move {
            while let Ok(sample) = sub.recv_async().await {
                println!("Received: {:?}", sample);
            }
        })
        .await
        .unwrap();
    }

    if std::env::args().nth(1).unwrap() == "vswitch" {
        let virtual_switch =
            Arc::new(ZenohVirtualSwitch::new(session.clone(), "switch".to_string()).await);
        tokio::spawn(async move {
            println!("Virtual Switch Running");
            loop {
                let new_state = virtual_switch.wait_for_change().await;
                println!("Virtual Switch state changed to: {:?}", new_state);
            }
        });
        sleep(Duration::from_secs(9999)).await;
    }

    let client_switch = ZenohClientSwitch::new(session.clone(), "switch".to_string()).await;

    if std::env::args().nth(1).unwrap() == "client-toggle" {
        // Example: Set the state of the switch
        loop {
            let state = client_switch.get_state().await;
            match state {
                SwitchState::On => {
                    eprintln!("set off");
                    client_switch.set_state(SwitchState::Off).await;
                }
                SwitchState::Off => {
                    eprintln!("set on");
                    client_switch.set_state(SwitchState::On).await;
                }
            }
            sleep(Duration::from_secs(1)).await;
        }
    }

    if std::env::args().nth(1).unwrap() == "client-wait-for-change" {
        // // Example: Wait for the switch state to change
        loop {
            let new_state = client_switch.wait_for_change().await;
            println!("Switch state changed to: {:?}", new_state);
        }
    }

    if std::env::args().nth(1).unwrap() == "client-set-on" {
        // // Example: Wait for the switch state to change
        client_switch.set_state(SwitchState::On).await;
    }
    if std::env::args().nth(1).unwrap() == "client-set-off" {
        // // Example: Wait for the switch state to change
        client_switch.set_state(SwitchState::Off).await;
    }
}
