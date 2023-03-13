use crate::api::*;
use axum::{routing::{get, post, put, delete}, Router};
use hyper::StatusCode;
use tokio::time::sleep;
use std::{env, net::{SocketAddr, IpAddr, Ipv4Addr}, process::exit, time::Duration};

mod types;
mod api;
mod rsm;
mod snapshot;
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

static mut CRASH: bool = false;

/// Request handler that crashes the server when called
pub async fn handle_crash() -> StatusCode {
    unsafe { CRASH = true; }
    StatusCode::OK
}

#[tokio::main]
async fn main() {
    let router = Router::new()
        .route("/omnipaxos", post(rsm::handle_msg_http))
        .route("/crash", post(handle_crash))
        .route("/print_log", get(handle_print_log))
        .route("/put", put(handle_put))
        .route("/cas", post(handle_cas))
        .route("/get/:key", get(handle_get))
        .route("/delete/:key", delete(handle_delete))
        .route("/linearizable/get/:key", get(handle_linearizable_get))
        .route("/snapshot", post(handle_snapshot));

    rsm::RSM::instance();

    // start event loop
    tokio::spawn(rsm::run());

    // this is used to simulate server crashes
    tokio::spawn(async {
        loop {
            sleep(Duration::from_millis(1)).await;
            if unsafe { CRASH } {
                sleep(Duration::from_millis(4)).await;
                exit(1);
            }
        }
    });

    println!("Started etcd");

    // start web server
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), *PORT);
    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}
