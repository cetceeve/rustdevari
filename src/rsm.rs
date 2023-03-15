use crate::types::{KeyValue, Key, Value};
use crate::snapshot::OPSnapshot;

use omnipaxos_core::{omni_paxos::{OmniPaxos, OmniPaxosConfig}, messages::Message, util::{NodeId, LogEntry}};

#[cfg(not(feature = "crash_recovery"))]
use omnipaxos_storage::memory_storage::*;
#[cfg(feature = "crash_recovery")]
use omnipaxos_storage::persistent_storage::*;
use serde::{Serialize, Deserialize};
use tokio::time;
use axum::extract::Json;
use reqwest;
use std::{env, sync::{Arc, Mutex}, collections::HashMap};


#[cfg(all(feature = "pl", feature = "crash_recovery"))]
compile_error!("features `pl` and `crash_recovery` are mutually exclusive");

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

    static ref PEER_DOMAINS: Vec<String> = if let Ok(var) = env::var("PEER_DOMAINS") {
        var.split(",").map(|x| x.to_owned()).collect()
    } else {
        panic!("missing PEERS env var")
    };
}

static mut INSTANCE: Option<Arc<Mutex<RSM>>> = None;
static mut COMMAND_COUNTER: Option<Arc<Mutex<u64>>> = None;
#[cfg(feature = "pl")]
static mut OUT_MSG_COUNTER: Option<Arc<Mutex<u64>>> = None;

/// Generates a globally unique id for an RSMCommand
#[cfg(not(feature = "crash_recovery"))]
fn generate_cmd_id() -> (u64, u64) {
    unsafe {
        let unlocked = if let Some(ref x) = COMMAND_COUNTER {
            x.clone()
        } else {
            let x = Arc::new(Mutex::new(0));
            COMMAND_COUNTER = Some(x.clone());
            x
        };
        let mut counter = unlocked.lock().unwrap();
        *counter += 1;
        (*PID, *counter)
    }
}

/// Generates a locally unique id for use as a sequence_id on SequencePaxos Messages
#[cfg(feature = "pl")]
fn generate_sequence_id() -> u64 {
    unsafe {
        let unlocked = if let Some(ref x) = OUT_MSG_COUNTER {
            x.clone()
        } else {
            let x = Arc::new(Mutex::new(0));
            OUT_MSG_COUNTER = Some(x.clone());
            x
        };
        let mut counter = unlocked.lock().unwrap();
        *counter += 1;
        *counter
    }
}

/// Generates a globally unique id for an RSMCommand
#[cfg(feature = "crash_recovery")]
fn generate_cmd_id() -> (u64, u64) {
    unsafe {
        let unlocked = if let Some(ref x) = COMMAND_COUNTER {
            x.clone()
        } else {
            let val = if let Ok(texta) = std::fs::read_to_string("/data/etcd_cmd_id_a") {
                if let Ok(n) = texta.parse() {
                    n
                } else {
                    if let Ok(textb) = std::fs::read_to_string("/data/etcd_cmd_id_b") {
                        if let Ok(n) = textb.parse() { n } else { 0 }
                    } else { 0 }
                }
            } else { 0 };
            let x = Arc::new(Mutex::new(val));
            COMMAND_COUNTER = Some(x.clone());
            x
        };
        let mut counter = unlocked.lock().unwrap();
        *counter += 1;
        // we write redundant files, in case we crash while writing one of them
        std::fs::write("/data/etcd_cmd_id_a", counter.to_string()).unwrap();
        std::fs::write("/data/etcd_cmd_id_b", counter.to_string()).unwrap();
        (*PID, *counter)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RSMCommand{
    Put(((u64, u64), KeyValue)),
    LinearizableRead((u64, u64)),
    CAS(((u64, u64), KeyValue, Value)),
    Delete(((u64, u64), Key)),
    Clear((u64, u64)),
}

impl RSMCommand {
    pub fn get_id(&self) -> (u64, u64) {
        match self {
            Self::Put((id, _)) => *id,
            Self::Delete((id, _)) => *id,
            Self::CAS((id, _, _)) => *id,
            Self::LinearizableRead(id) => *id,
            Self::Clear(id) => *id,
        }
    }

    pub fn new_put(kv: KeyValue) -> Self {
        Self::Put((generate_cmd_id(), kv))
    }

    pub fn new_linearizable_read() -> Self {
        Self::LinearizableRead(generate_cmd_id())
    }

    pub fn new_delete(key: Key) -> Self {
        Self::Delete((generate_cmd_id(), key))
    }

    pub fn new_clear() -> Self {
        Self::Clear(generate_cmd_id())
    }

    pub fn new_cas(key: Key, new_v: Value, exp_v: Value) -> Self {
        Self::CAS((generate_cmd_id(), KeyValue{key, value: new_v}, exp_v))
    }
}

type OmniPaxosMessage = Message<RSMCommand, OPSnapshot>;
#[cfg(not(feature = "crash_recovery"))]
type OmniPaxosStorage = MemoryStorage<RSMCommand, OPSnapshot>;
#[cfg(feature = "crash_recovery")]
type OmniPaxosStorage = PersistentStorage<RSMCommand, OPSnapshot>;
type OmniPaxosType = OmniPaxos<RSMCommand, OPSnapshot, OmniPaxosStorage>;

pub struct RSM {
    pub omnipaxos: OmniPaxosType,
    addrs: HashMap<NodeId, String>,
    #[cfg(not(feature = "pl"))]
    connected: HashMap<NodeId, bool>,
    #[cfg(feature = "pl")]
    outgoing_buffer: Vec<(u64, String, OmniPaxosMessage)>,
    #[cfg(feature = "pl")]
    delivered_msgs: HashMap<NodeId, Vec<u64>>,
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
                    configuration_id: 1,
                    peers: PEERS.clone(),
                    ..Default::default()
                };

                #[cfg(not(feature = "crash_recovery"))]
                let omnipaxos = op_config.build(MemoryStorage::default());
                #[cfg(feature = "crash_recovery")]
                let omnipaxos = {
                    let mut storage_config = PersistentStorageConfig::default();
                    storage_config.set_path("/data/op_storage".to_string());
                    let storage = PersistentStorage::open(storage_config);
                    op_config.build(storage)
                };

                let mut addrs = HashMap::default();
                #[cfg(not(feature = "pl"))]
                let mut connected = HashMap::default();
                #[cfg(feature = "pl")]
                let mut delivered_msgs = HashMap::default();
                for i in 0..PEERS.len() {
                    addrs.insert(PEERS[i], PEER_DOMAINS[i].clone());
                    #[cfg(not(feature = "pl"))]
                    connected.insert(PEERS[i], false);
                    #[cfg(feature = "pl")]
                    delivered_msgs.insert(PEERS[i], vec![]);
                }
                #[cfg(feature = "pl")]
                let rsm = Arc::new(Mutex::new(RSM{
                    omnipaxos,
                    addrs,
                    outgoing_buffer: vec![],
                    delivered_msgs,
                }));
                #[cfg(not(feature = "pl"))]
                let rsm = Arc::new(Mutex::new(RSM{
                    omnipaxos,
                    addrs,
                    connected,
                }));
                INSTANCE = Some(rsm.clone());
                rsm
            }
        }
    }
}

/// Appends an entry and waits until it is decided
/// returns the index of the decided entry on success
pub async fn append(cmd: RSMCommand) -> Result<u64, ()> {
    let start_decided_idx;
    {
        let unlocked = RSM::instance();
        let mut rsm = unlocked.lock().unwrap();
        start_decided_idx = rsm.omnipaxos.get_decided_idx();
        if let Err(_) = rsm.omnipaxos.append(cmd.clone()) {
            return Err(());
        }
    }

    // wait until decided
    loop {
        time::sleep(time::Duration::from_millis(1)).await;
        if let Some(entries) = RSM::instance().lock().unwrap().omnipaxos.read_decided_suffix(start_decided_idx) {
            for (i, entry) in entries.iter().enumerate() {
                match entry {
                    LogEntry::Decided(new) => {
                        if new.get_id() == cmd.get_id() {
                            return Ok(start_decided_idx+i as u64);
                        }
                    },
                    _ => (),
                }
            }
        }
    }
}

#[cfg(not(feature = "pl"))]
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
        let url = format!("http://{}/omnipaxos", msg.1);
        match reqwest::Client::new().post(url).json(&msg.0).send().await {
            Ok(_) => {
                let unlocked = RSM::instance();
                let mut rsm = unlocked.lock().unwrap();
                if let Some(false) = rsm.connected.insert(msg.0.get_receiver(), true) {
                    rsm.omnipaxos.reconnected(msg.0.get_receiver());
                }
            },
            Err(_) => {
                let unlocked = RSM::instance();
                let mut rsm = unlocked.lock().unwrap();
                rsm.connected.insert(msg.0.get_receiver(), false);
            },
        }
    }
}

#[cfg(feature = "pl")]
async fn send_outgoing_msgs() {
    let mut ble_msg_buf = vec![];

    { // open a new scope, so we can drop the lock on RSM before we start actually sending messages
        let unlocked = RSM::instance();
        let mut rsm = unlocked.lock().unwrap();
        let msgs = rsm.omnipaxos.outgoing_messages();
        msgs.into_iter().for_each(|msg| {
            let receiver_id = msg.get_receiver();
            let addr = rsm.addrs.get(&receiver_id).unwrap().to_owned();
            match msg {
                OmniPaxosMessage::SequencePaxos(_) => {
                    let sequence_id = generate_sequence_id();
                    rsm.outgoing_buffer.push((sequence_id, addr, msg));
                },
                OmniPaxosMessage::BLE(_) => {
                    ble_msg_buf.push((0, addr, msg));
                },
            }
        });
    }

    // actually send the messages, first new BLE messages, then SequencePaxos Messages via our FIFO PL
    let mut num_successfully_sent = 0;
    let out_buf = RSM::instance().lock().unwrap().outgoing_buffer.clone();
    for (sequence_id, addr, msg) in ble_msg_buf.clone().into_iter().chain(out_buf.into_iter()) {
        let url = format!("http://{}/omnipaxos", addr);
        match reqwest::Client::new().post(url).json(&(sequence_id, msg)).send().await {
            Ok(_) => { num_successfully_sent += 1 },
            Err(_) => break,
        }
    }
    // remove sent SequencePaxos messages from our PL buffer
    if num_successfully_sent > ble_msg_buf.len() {
        let num_to_remove = num_successfully_sent - ble_msg_buf.len();
        let unlocked = RSM::instance();
        let mut rsm = unlocked.lock().unwrap();
        for _ in 0..num_to_remove {
            rsm.outgoing_buffer.remove(0);
        }
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
            else => {},
        }
    }
}

/// Receives an omnipaxos message and delivers it
#[cfg(not(feature = "pl"))]
pub async fn handle_msg_http(Json(msg): Json<OmniPaxosMessage>) {
    let unlocked = RSM::instance();
    let mut rsm = unlocked.lock().unwrap();
    if let Message::SequencePaxos(ref x) = msg {
        println!("{:?}", x);
    }
    rsm.omnipaxos.handle_incoming(msg.clone());
}

/// Receives an omnipaxos message and delivers it exactly once
#[cfg(feature = "pl")]
pub async fn handle_msg_http(Json((sequence_id, msg)): Json<(u64, OmniPaxosMessage)>) {
    let unlocked = RSM::instance();
    let mut rsm = unlocked.lock().unwrap();
    if let Message::SequencePaxos(ref x) = msg {
        println!("{}: {:?}", sequence_id, x);
    }
    if let OmniPaxosMessage::SequencePaxos(_) = msg {
        let delivered_msgs = rsm.delivered_msgs.get_mut(&msg.get_sender()).unwrap();
        if delivered_msgs.contains(&sequence_id) {
            return
        } else {
            delivered_msgs.push(sequence_id);
        }
    }
    rsm.omnipaxos.handle_incoming(msg.clone());
}
