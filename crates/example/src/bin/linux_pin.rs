use async_trait::async_trait;
use example::switch::{hardware::ZenohOutputPinSwitch, Switch, SwitchState};
use linux_embedded_hal::{
    gpio_cdev::{Chip, LineRequestFlags},
    CdevPin,
};
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;
use zenoh::prelude::r#async::*;

pub struct DynSwitch {
    inner: Arc<dyn Switch>,
}
impl DynSwitch {
    fn new(inner: Arc<dyn Switch>) -> Self {
        Self { inner }
    }
}
#[async_trait]
impl Switch for DynSwitch {
    async fn get_state(&self) -> SwitchState {
        self.inner.get_state().await
    }
    async fn set_state(&self, new_state: SwitchState) {
        self.inner.set_state(new_state).await
    }
    async fn wait_for_change(&self) -> SwitchState {
        self.wait_for_change().await
    }
}
#[tokio::main]
async fn main() {
    let mut chip = Chip::new("/dev/gpiochip0").unwrap();

    // Request a line (replace 17 with the GPIO pin number you want to use)
    let handle = chip
        .get_line(17)
        .unwrap()
        .request(LineRequestFlags::OUTPUT, 0, "example")
        .unwrap();

    // Wrap the line handle in a CdevPin
    let pin = CdevPin::new(handle).unwrap();

    // Create a Zenoh session
    let session = Arc::new(zenoh::open(config::peer()).res().await.unwrap());

    // represent this switch in zenoh
    let s = ZenohOutputPinSwitch::new(session.clone(), "switch".to_string(), pin, SwitchState::On)
        .await;

    let h = DynSwitch::new(s);

    h.set_state(SwitchState::Off).await;

}
