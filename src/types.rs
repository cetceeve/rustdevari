use serde::{Serialize, Deserialize};

pub type Key = String;
pub type Value = String; // TODO: different type?, maybe json?

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyValue {
    pub key: Key,
    pub value: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetResponse {
    pub key: Key,
    pub value: Option<Value>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PutRequest {
    pub key: Key,
    pub value: Value,
    pub lease: i64,
    pub prev_kv: bool,
    pub ignore_value: bool,
    pub ignore_lease: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PutResponse {
    pub prev_kv: Option<KeyValue>
}
