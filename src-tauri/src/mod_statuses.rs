use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone, Copy)]
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

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone)]
#[repr(u8)]
pub enum RequestType {
    Test,
    Start,
    Stop,
    Login = 0xF0,
    KeepAlive = 0xFF,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone, Copy)]
#[repr(u8)]
pub enum ResponseType {
    EffectRequest,
    Login = 0xF0,
    KeepAlive = 0xFF,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub id: u32,
    pub status: EffectResult,
    pub message: Option<String>,
    pub time_remaining: i64,
    #[serde(rename = "type")]
    pub t: ResponseType,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FromServer {
    pub player_id: String,
    pub code: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ToServer {
    pub player_id: String,
    pub code: String,
    pub time: i64,
    pub sender: String,
}
