use crate::rsm::RSMCommand;
use crate::types::*;
use crate::{rsm, rsm::RSM};
use omnipaxos_core::omni_paxos::CompactionErr;
use omnipaxos_core::util::LogEntry;
use std::{sync::{Arc, Mutex}, collections::HashMap};

static mut INSTANCE: Option<Arc<Mutex<Store>>> = None;

#[derive(Debug, Clone)]
struct Store {
    map: HashMap<Key, Value>,
    applied_log_index: u64,
}

impl Store {
    /// Get the singleton Store instance
    fn instance() -> Arc<Mutex<Self>> {
        unsafe {
            if let Some(ref store) = INSTANCE {
                store.clone()
            } else {
                let store = Arc::new(Mutex::new(Store{
                    map: HashMap::new(),
                    applied_log_index: 0,
                }));
                INSTANCE = Some(store.clone());
                store
            }
        }
    }

    /// Call this before every read to stay up to date
    fn apply_decided_entries(&mut self) {
        if let Some(entries) = RSM::instance().lock().unwrap().omnipaxos.read_decided_suffix(self.applied_log_index) {
            for entry in entries {
                self.applied_log_index += 1;
                match entry {
                    LogEntry::Decided(cmd) => { 
                        match cmd {
                            RSMCommand::Put((_, kv)) => { self.map.insert(kv.key, kv.value); },
                            RSMCommand::CAS((_, kv, exp_val)) => {
                                if let Some(old_val) = self.map.get(&kv.key) {
                                    if *old_val == exp_val {
                                        self.map.insert(kv.key, kv.value);
                                    }
                                }
                            },
                            RSMCommand::Delete((_, key)) => { self.map.remove(&key); },
                            RSMCommand::LinearizableRead(_) => (),
                        }
                    },
                    LogEntry::Snapshotted(entry) => {
                        for (key, val) in entry.snapshot.snapshotted.into_iter() {
                            self.map.insert(key, val);
                        }
                    },
                    LogEntry::Undecided(x) => { panic!("read undecided log entry: {:?}", x)},
                    LogEntry::StopSign(_) => { todo!() },
                    LogEntry::Trimmed(_) => { todo!() },
                }
            }
        }
    }
}

/// Sequentially consistent read
pub fn get(key: &Key) -> Option<Value> {
    let unlocked = Store::instance();
    let mut store = unlocked.lock().unwrap();
    store.apply_decided_entries();
    store.map.get(key).map(|x| x.to_owned())
}

/// linearizable read
pub async fn linearizable_get(key: &Key) -> Result<Option<Value>, ()> {
    rsm::append(RSMCommand::new_linearizable_read()).await?;
    Ok(get(key))
}

/// Takes a previous value that was read before an operation and updates it with
/// the new commands that were decided during the operation
fn get_prev_value_after_decide(key: &Key, mut prev_val: Option<Value>, prev_decided_idx: u64, new_decided_idx: u64) -> Option<Value> {
    if let Some(entries) = RSM::instance().lock().unwrap().omnipaxos.read_decided_suffix(prev_decided_idx) {
        for (i, entry) in entries.iter().enumerate() {
            if prev_decided_idx + (i as u64) == new_decided_idx {
                break // only read until the operation's entry's index
            }
            match entry {
                LogEntry::Decided(cmd) => {
                    match cmd {
                        RSMCommand::LinearizableRead(_) => (),
                        RSMCommand::Put((_, kv)) => {
                            if kv.key == *key {
                                prev_val = Some(kv.value.clone());
                            }
                        },
                        RSMCommand::Delete((_, del_key)) => {
                            if *del_key == *key {
                                prev_val = None;
                            }
                        },
                        RSMCommand::CAS((_, kv, exp_val)) => {
                            if kv.key == *key && kv.value == *exp_val {
                                prev_val = Some(kv.value.clone());
                            }
                        },
                    }
                },
                LogEntry::Snapshotted(snap) => {
                    prev_val = snap.snapshot.snapshotted.get(key).map(|x| x.to_owned());
                },
                _ => (),
            }
        }
    }
    prev_val
}

/// Inserts into the replicated store
/// returns the previous value of this key on success
pub async fn put(kv: KeyValue) -> Result<Option<KeyValue>,()> {
    let prev_idx = RSM::instance().lock().unwrap().omnipaxos.get_decided_idx();
    let mut prev_value = get(&kv.key);
    let idx = rsm::append(RSMCommand::new_put(kv.clone())).await?;

    // read entries that were decided in the meantime, to get latest previous value
    prev_value = get_prev_value_after_decide(&kv.key, prev_value, prev_idx, idx);
    Ok(prev_value.map(|value| KeyValue{key: kv.key, value}))
}

/// Inserts into the replicated store
/// returns the previous value of this key on success
pub async fn delete(key: Key) -> Result<Option<KeyValue>,()> {
    let prev_idx = RSM::instance().lock().unwrap().omnipaxos.get_decided_idx();
    let mut prev_value = get(&key);
    let idx = rsm::append(RSMCommand::new_delete(key.clone())).await?;

    // read entries that were decided in the meantime, to get latest previous value
    prev_value = get_prev_value_after_decide(&key, prev_value, prev_idx, idx);
    Ok(prev_value.map(|value| KeyValue{key, value}))
}

/// Performs linearizable CAS operation
/// returns the previous value of this key on success
pub async fn cas(key: Key, new_value: Value, expected_value: Value) -> Result<Option<KeyValue>,()> {
    let prev_idx = RSM::instance().lock().unwrap().omnipaxos.get_decided_idx();
    let mut prev_value = get(&key);
    let idx = rsm::append(RSMCommand::new_cas(key.clone(), new_value.clone(), expected_value.clone())).await?;

    // read entries that were decided in the meantime, to get latest previous value
    prev_value = get_prev_value_after_decide(&key, prev_value, prev_idx, idx);
    Ok(prev_value.map(|value| KeyValue{key, value}))
}

pub async fn snapshot() -> Result<(), CompactionErr> {
    RSM::instance().lock().unwrap().omnipaxos.snapshot(None, false)?;
    Ok(())
}
