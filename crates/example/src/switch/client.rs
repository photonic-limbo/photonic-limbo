use super::{Switch, SwitchState};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::watch;
use zenoh::prelude::r#async::*;

// ZenohClientSwitch struct
pub struct ZenohClientSwitch {
    session: Arc<Session>,
    resource_path: String,
    notifier_rx: watch::Receiver<SwitchState>,
}

impl ZenohClientSwitch {
    pub async fn new(session: Arc<Session>, resource_path: String) -> Self {
        let (notifier_tx, notifier_rx) = watch::channel(SwitchState::Off);

        // subscribe to all changes to the resource, notifying any local subscribers
        let sub = session
            .declare_subscriber(resource_path.clone())
            .res()
            .await
            .unwrap();
        tokio::task::spawn(async move {
            while let Ok(sample) = sub.recv_async().await {
                let v: SwitchState = sample.value.try_into().unwrap();
                notifier_tx.send(v).unwrap();
            }
        });

        ZenohClientSwitch {
            session,
            resource_path,
            notifier_rx,
        }
    }
}

#[async_trait]
impl Switch for ZenohClientSwitch {
    async fn get_state(&self) -> SwitchState {
        let replies = self.session.get(&self.resource_path).res().await.unwrap();
        while let Ok(reply) = replies.recv_async().await {
            let v: SwitchState = reply.sample.unwrap().value.try_into().unwrap();
            return v;
        }
        return SwitchState::Off;
    }

    async fn set_state(&self, new_state: SwitchState) {
        self.session
            .put(&self.resource_path, new_state)
            .res()
            .await
            .unwrap();
    }

    async fn wait_for_change(&self) -> SwitchState {
        let mut n = self.notifier_rx.clone();

        let initial = n.borrow().clone();
        n.changed().await.unwrap();

        // have to ignore the initial value since the notifier always starts with the initial value
        if initial == *n.borrow() {
            n.changed().await.unwrap();
        }

        let v = n.borrow().clone();
        v
    }
}
