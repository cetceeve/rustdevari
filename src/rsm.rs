use crate::types::{Key, Value, KeyValue};

use omnipaxos_core::{omni_paxos::{OmniPaxos, OmniPaxosConfig}, messages::Message, util::NodeId};
use omnipaxos_storage::memory_storage::MemoryStorage;
use tokio::{sync::mpsc, time};
use axum::extract::Json;
use std::{env, sync::{Arc, Mutex}, collections::HashMap};


lazy_static! {
    static ref OUTGOING_INTERVAL: u64 = if let Ok(var) = env::var("OUTGOING_INTERVAL") {
        var.parse().expect("OUTGOING_INTERVAL must be u64 in millis")
    } else {
        10
    };

    static ref ELECTION_TIMEOUT: u64 = if let Ok(var) = env::var("ELECTION_TIMEOUT") {
        var.parse().expect("ELECTION_TIMEOUT must be u64 in millis")
    } else {
        100
    };

    static ref PID: NodeId = if let Ok(var) = env::var("PID") {
        let x = var.parse().expect("PIDs must be u64");
        if x == 0 { panic!("PIDs cannot be 0") } else { x }
    } else {
        panic!("missing PID env var")
    };

    static ref PEERS: Vec<NodeId> = if let Ok(var) = env::var("PEERS") {
        var.split(",").map(|s| {
            let x = s.parse().expect("PIDs must be u64");
            if x == 0 { panic!("PIDs cannot be 0") } else { x }
        }).collect()
    } else {
        panic!("missing PEERS env var")
    };
}

static mut INSTANCE: Option<Arc<Mutex<RSM>>> = None;

type OmniPaxosSnapShot = ();
type OmniPaxosMessage = Message<KeyValue, OmniPaxosSnapShot>;
type OmniPaxosStorage = MemoryStorage<KeyValue, OmniPaxosSnapShot>;
type OmniPaxosType = OmniPaxos<KeyValue, (), MemoryStorage<KeyValue, OmniPaxosSnapShot>>;

pub struct RSM {
    pub omnipaxos: OmniPaxosType,
    pub addrs: HashMap<NodeId, String>,
    // pub incoming: mpsc::Receiver<Message<KeyValue, ()>>,
}  

impl RSM {
    /// Get the singleton RSM instance
    pub fn instance() -> Arc<Mutex<Self>> {
        unsafe {
            if let Some(ref rsm) = INSTANCE {
                rsm.clone()
            } else {
                let op_config = OmniPaxosConfig{
                    pid: *PID,
                    configuration_id: 0, // TODO: what about crash recovery, or reconfiguration?
                    peers: PEERS.clone(),
                    ..Default::default()
                };

                let rsm = Arc::new(Mutex::new(RSM{
                    omnipaxos: op_config.build(OmniPaxosStorage::default()),
                    addrs: todo!(),
                }));
                INSTANCE = Some(rsm.clone());
                rsm
            }
        }
    }
}

async fn send_outgoing_msgs() {
    let messages: Vec<(OmniPaxosMessage, String)> = { // open a new scope, so we can drop the lock on RSM asap
        let unlocked = RSM::instance();
        let mut rsm = unlocked.lock().unwrap();
        let msgs = rsm.omnipaxos.outgoing_messages();
        msgs.into_iter().map(|msg| {
            let receiver_id = msg.get_receiver();
            (msg, rsm.addrs.get(&receiver_id).unwrap().to_owned())
        }).collect()
    };
    for msg in messages {
        todo!() // send via http
    }
}

/// Our main OmniPaxos event loop
pub async fn run() {
    let mut outgoing_interval = time::interval(time::Duration::from_millis(*OUTGOING_INTERVAL));
    let mut election_interval = time::interval(time::Duration::from_millis(*ELECTION_TIMEOUT));
    loop {
        tokio::select! {
            biased;
            _ = election_interval.tick() => { RSM::instance().lock().unwrap().omnipaxos.election_timeout(); },
            _ = outgoing_interval.tick() => { send_outgoing_msgs().await; },
            // TODO: handle incoming messages on this line
            else => {},
        }
    }
}

/// TODO: omnipaxos msg json serialization
/// Receives a message and queues it for the event loop
pub async fn handle_msg_http(Json(KeyValue): Json<KeyValue>) {
    todo!()
}
