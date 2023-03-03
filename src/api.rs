use crate::{types::*, store};
use axum::extract::{Json, Path};
use hyper::StatusCode;

/// Sequentially consistent read
pub async fn handle_get(Path(key): Path<Key>) -> Json<GetResponse> {
    let value = store::get(&key);
    Json(GetResponse{key, value})
}

/// Linearizable read
pub async fn handle_linearizable_get(Path(key): Path<Key>) -> Json<GetResponse> {
    todo!()
}

/// Write and return previous value
pub async fn handle_put(Json(req): Json<PutRequest>) -> (StatusCode, Json<PutResponse>) {
    let kv = KeyValue{key: req.key.clone(), value: req.value};
    if let Ok(prev_kv) = store::put(kv).await {
        return (StatusCode::OK, Json(PutResponse{ prev_kv }))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(PutResponse{ prev_kv: None }))
    }
}
