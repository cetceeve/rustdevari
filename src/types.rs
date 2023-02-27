use serde::{Serialize, Deserialize};

pub type Key = String;
pub type Value = String; // TODO: different type?, maybe json?

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyValue {
    key: Key,
    value: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetResponse {
    key: Key,
    prev_value: Option<Value>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PutRequest {
    key: Key,
    value: Value,
    lease: i64,
    prev_kv: bool,
    ignore_value: bool,
    ignore_lease: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PutResponse {
    prev_kv: Option<KeyValue>
}
