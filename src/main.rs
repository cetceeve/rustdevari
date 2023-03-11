use crate::api::*;
use axum::{routing::{get, post, put}, Router};
use std::{env, net::{SocketAddr, IpAddr, Ipv4Addr}};

mod types;
mod api;
mod rsm;
mod store;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref PORT: u16 = if let Ok(var) = env::var("PORT") {
        var.parse().unwrap()
    } else {
        8080
    };
}

#[tokio::main]
async fn main() {
    let router = Router::new()
        .route("/omnipaxos", post(rsm::handle_msg_http))
        .route("/put", put(handle_put))
        .route("/cas", post(handle_cas))
        .route("/get/:key", get(handle_get))
        .route("/linearizable/get/:key", get(handle_linearizable_get));

    // start event loop
    tokio::spawn(rsm::run());

    println!("Started etcd");

    // start web server
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), *PORT);
    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}
