use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum EffectResult {
    Success,
    Failure,
    Unavailable,
    Retry,
    Queue,
    Running,
    Paused,
    Resumed,
    Finished,
    NotReady,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum RequestType {
    Test,
    Start,
    Stop,
    Login = 0xF0,
    KeepAlive = 0xFF,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum ResponseType {
    EffectRequest,
    Login = 0xF0,
    KeepAlive = 0xFF,
}
