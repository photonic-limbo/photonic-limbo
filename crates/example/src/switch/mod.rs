pub mod r#virtual;
pub mod hardware;
pub mod client;

use std::ops::Not;

use async_trait::async_trait;
use zenoh::value::Value;


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SwitchState {
    On,
    Off,
}

#[async_trait]
pub trait Switch : Sync + Send + 'static {
    async fn get_state(&self) -> SwitchState;
    async fn set_state(&self, new_state: SwitchState);
    async fn wait_for_change(&self) -> SwitchState;
}

impl Not for SwitchState {
    type Output = SwitchState;

    fn not(self) -> Self::Output {
        match self {
            SwitchState::On => SwitchState::Off,
            SwitchState::Off => SwitchState::On,
        }
    }
}



impl From<i8> for SwitchState {
    fn from(value: i8) -> Self {
        if value == 1 {
            SwitchState::On
        } else {
            SwitchState::Off
        }
    }
}

impl From<SwitchState> for i8 {
    fn from(state: SwitchState) -> Self {
        match state {
            SwitchState::On => 1,
            SwitchState::Off => 0,
        }
    }
}

impl From<Value> for SwitchState {
    fn from(value: Value) -> Self {
        let i8_value: i8 = value.try_into().unwrap();
        i8_value.into()
    }
}
impl From<SwitchState> for Value {
    fn from(state: SwitchState) -> Self {
        let i8_value: i8 = state.into();
        i8_value.into()
    }
}
