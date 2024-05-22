use super::{Switch, SwitchState};
use async_trait::async_trait;
use embedded_hal::digital::OutputPin;
use std::sync::{Arc, Mutex};
use tokio::sync::watch;
use zenoh::prelude::r#async::*;

pub struct ZenohOutputPinSwitch<P: OutputPin + Sync + Send + 'static> {
    
    state: Arc<Mutex<SwitchState>>,
    high_state: SwitchState,
    pin: Arc<Mutex<P>>,
    session: Arc<Session>,
    resource_path: String,
    notifier_tx: watch::Sender<SwitchState>,
    notifier_rx: watch::Receiver<SwitchState>,
}

impl<P: OutputPin + Sync + Send + 'static> ZenohOutputPinSwitch<P> {
    pub async fn new(
        session: Arc<Session>,
        resource_path: String,
        pin: P,
        high_state: SwitchState,
    ) -> Arc<Self> {
        let (notifier_tx, notifier_rx) = watch::channel(SwitchState::Off);
        let state = Arc::new(Mutex::new(SwitchState::Off));
        let pin = Arc::new(Mutex::new(pin));

        let resource = Arc::new(ZenohOutputPinSwitch {
            state,
            high_state,
            pin,
            session,
            resource_path: resource_path.clone(),
            notifier_tx,
            notifier_rx,
        });

        resource.setup_get().await;
        resource.setup_set().await;
        resource
    }

    pub async fn set_state(&self, new_state: SwitchState) {
        Self::_set_state(
            self.state.clone(),
            self.high_state,
            new_state,
            self.pin.clone(),
            self.notifier_tx.clone(),
            self.resource_path.clone(),
            self.session.clone(),
        )
        .await;

    }
    // handle get - reading the switch state - just reads from the local state
    async fn setup_get(&self) {
        let state = self.state.clone();
        let session = self.session.clone();
        let resource_path = self.resource_path.clone();

        tokio::spawn(async move {
            let queryable = session
                .declare_queryable(resource_path.clone())
                .res()
                .await
                .unwrap();

            while let Ok(query) = queryable.recv_async().await {
                let state = { state.lock().unwrap().clone() };
                query
                    .reply(Ok(Sample::try_from(resource_path.clone(), state).unwrap()))
                    .res()
                    .await
                    .unwrap();
            }
        });
    }

    // handle set - setting the switch state
    async fn setup_set(&self) {
        let state = self.state.clone();
        let high_state = self.high_state;
        let session = self.session.clone();
        let resource_path = self.resource_path.clone();
        let pin = self.pin.clone();
        let notifier_tx = self.notifier_tx.clone();

        tokio::spawn(async move {
            // Subscribe to the resource path to listen for state change commands
            let sub = session
                .declare_subscriber(resource_path.clone())
                .res()
                .await
                .unwrap();

            while let Ok(sample) = sub.recv_async().await {
                let new_state: SwitchState = sample.value.try_into().unwrap();
                Self::_set_state(
                    state.clone(),
                    high_state,
                    new_state,
                    pin.clone(),
                    notifier_tx.clone(),
                    resource_path.clone(),
                    session.clone(),
                )
                .await;
            }
        });
    }

    // - sets the internal state
    // - updates the pin
    // - notifies zenoh
    // - notifies the local watchers
    async fn _set_state(
        state: Arc<Mutex<SwitchState>>,
        high_state: SwitchState,
        new_state: SwitchState,
        pin: Arc<Mutex<P>>,
        notifier_tx: watch::Sender<SwitchState>,
        resource_path: String,
        session: Arc<Session>,
    ) {
        *state.lock().unwrap() = new_state;

        notifier_tx.send_replace(new_state); // Notify the state change

        if new_state == high_state {
            pin.lock().unwrap().set_high().unwrap();
        } else {
            pin.lock().unwrap().set_low().unwrap();
        }

        session
            .put(resource_path.clone(), new_state)
            .res()
            .await
            .unwrap();
    }
}

#[async_trait]
impl<P: OutputPin + Send + Sync + 'static> Switch for ZenohOutputPinSwitch<P> {
    async fn get_state(&self) -> SwitchState {
        *self.state.lock().unwrap()
    }

    async fn set_state(&self, new_state: SwitchState) {
        ZenohOutputPinSwitch::_set_state(
            self.state.clone(),
            self.high_state,
            new_state,
            self.pin.clone(),
            self.notifier_tx.clone(),
            self.resource_path.clone(),
            self.session.clone(),
        )
        .await;
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

// #[async_trait]
// impl<P : embedded_hal::digital::OutputPin + Send + Sync + 'static> Switch for Arc<ZenohOutputPinSwitch<P>> {
//     async fn get_state(&self) -> SwitchState {
//         self.get_state().await
//     }
//     async fn set_state(&self, new_state: SwitchState) {
//         self.set_state(new_state).await
//     }
//     async fn wait_for_change(&self) -> SwitchState {
//         self.wait_for_change().await
//     }
// }