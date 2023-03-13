use crate::{types::*, store};
use axum::extract::{Json, Path};
use hyper::StatusCode;

/// Sequentially consistent read
pub async fn handle_get(Path(key): Path<Key>) -> Json<GetResponse> {
    let value = store::get(&key);
    Json(GetResponse{key, value})
}

/// Delete key from store
pub async fn handle_delete(Path(key): Path<Key>) -> (StatusCode, Json<PutResponse>) {
    if let Ok(prev_kv) = store::delete(key).await {
        return (StatusCode::OK, Json(PutResponse{ prev_kv }))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(PutResponse{ prev_kv: None }))
    }
}

/// Clear Store store
pub async fn handle_clear() -> (StatusCode, Json<Option<()>>) {
    //println!("Hello from handler");
    if let Ok(_) = store::clear().await {
        (StatusCode::OK, Json(None))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(None))
    }
}

/// Linearizable read
pub async fn handle_linearizable_get(Path(key): Path<Key>) -> (StatusCode, Json<GetResponse>) {
    if let Ok(value) = store::linearizable_get(&key).await {
        (StatusCode::OK, Json(GetResponse{key, value}))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(GetResponse{key, value: None}))
    }
}

/// Write and return previous value
pub async fn handle_put(Json(req): Json<PutRequest>) -> (StatusCode, Json<PutResponse>) {
    let kv = KeyValue{key: req.key.clone(), value: req.value};
    if let Ok(prev_kv) = store::put(kv).await {
        (StatusCode::OK, Json(PutResponse{ prev_kv }))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(PutResponse{ prev_kv: None }))
    }
}

/// Linearizable Compare and Swap
pub async fn handle_cas(Json(req): Json<CASRequest>) -> (StatusCode, Json<PutResponse>) {
    if let Ok(prev_kv) = store::cas(req.key, req.new_value, req.expected_value).await {
        (StatusCode::OK, Json(PutResponse{ prev_kv }))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(PutResponse{ prev_kv: None }))
    }
}

/// Linearizable Compare and Swap
pub async fn handle_snapshot() -> StatusCode {
    if let Ok(_) = store::snapshot().await {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
