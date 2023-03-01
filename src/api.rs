use crate::{types::*, rsm::RSM, store::Store};
use axum::{extract::{Json, Path}, response::IntoResponse};
use hyper::StatusCode;

pub async fn handle_get(Path(key): Path<Key>) -> Json<GetResponse> {
    // TODO: what to do it we are partitioned from the network and our state is outdated?
    let value = Store::instance().lock().unwrap().get(&key);
    Json(GetResponse{key, value})
}

pub async fn handle_put(Json(req): Json<PutRequest>) -> impl IntoResponse {
    println!("HELLO");
    let prev_value = Store::instance().lock().unwrap().get(&req.key);
    let prev_kv = if let Some(val) = prev_value {
        Some(KeyValue{key: req.key.clone(), value: val})
    } else {
        None
    };

    let kv = KeyValue{key: req.key.clone(), value: req.value};
    if let Err(_) = RSM::instance().lock().unwrap().omnipaxos.append(kv) {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(PutResponse{ prev_kv: None })).into_response()
    }
    Json(PutResponse{ prev_kv }).into_response()
}
