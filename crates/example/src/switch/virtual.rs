use zenoh::Session;

use super::{Switch, SwitchState};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use tokio::sync::watch;
use zenoh::prelude::r#async::*;


pub struct ZenohVirtualSwitch {
    session: Arc<Session>,
    resource_path: String,
    notifier_rx: watch::Receiver<SwitchState>,
    notifier_tx: watch::Sender<SwitchState>,
    state: Arc<Mutex<SwitchState>>,
}

impl ZenohVirtualSwitch {
    pub async fn new(session: Arc<Session>, resource_path: String) -> Self {
        let (notifier_tx, notifier_rx) = watch::channel(SwitchState::Off);

        let resource_path_clone = resource_path.clone();
        let state = Arc::new(Mutex::new(SwitchState::Off));

        // handle gets - read the state of the switch
        let queryable = session
            .declare_queryable(&resource_path_clone)
            .res()
            .await
            .unwrap();

        tokio::task::spawn({
            let state = state.clone();
            let resource_path_clone = resource_path_clone.clone();
            async move {
                while let Ok(query) = queryable.recv_async().await {
                    let v = {
                        state.lock().unwrap().clone()
                    };
                    query
                        .reply(Ok(Sample::try_from(resource_path_clone.clone(), v).unwrap()))
                        .res()
                        .await
                        .unwrap();
                }
            }
        });
        

        // handle puts - command to turn the switch on or off
        let sub = session
            .declare_subscriber(&resource_path_clone)
            .res()
            .await
            .unwrap();

        tokio::spawn({
            let state = state.clone();
            let notifier_tx = notifier_tx.clone();
            async move {
                while let Ok(sample) = sub.recv_async().await {
                    let data: Result<SwitchState, _> = sample.value.try_into();
                    if let Ok(data) = data {
                        let mut state_lock = state.lock().unwrap();
                        *state_lock = data;
                        notifier_tx.send(data).unwrap(); // Notify the state change

                    }
                }
            }
        });

        ZenohVirtualSwitch {
            session,
            resource_path,
            notifier_tx,
            notifier_rx,
            state,
        }
    }
}


#[async_trait]
impl Switch for ZenohVirtualSwitch {

    async fn get_state(&self) -> SwitchState {
        *self.state.lock().unwrap()
    }

    async fn set_state(&self, new_state: SwitchState) {
        {
            *self.state.lock().unwrap() = new_state;
        }
        // notify zenoh subscribers
        self.session
            .put(&self.resource_path, new_state)
            .res()
            .await
            .unwrap();
        
        // notify local subscribers
        self.notifier_tx.send(new_state).unwrap();
    }

    async fn wait_for_change(&self) -> SwitchState {
        let mut n = self.notifier_rx.clone();

        let initial = n.borrow().clone();
        n.changed().await.unwrap();

        // have to ignore the initial value
        if initial == *n.borrow() {
            n.changed().await.unwrap();
        }

        let v = n.borrow().clone();
        v
    }
}
