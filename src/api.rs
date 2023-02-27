use crate::types::*;
use axum::extract::{Json, Path};


pub async fn handle_get(Path(key): Path<Key>) -> Json<GetResponse> {
    todo!()
}

pub async fn handle_put(Json(req): Json<PutRequest>) -> Json<PutResponse> {
    todo!()
}
