use crate::types::*;
use crate::{rsm, rsm::RSM};
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
                    LogEntry::Decided(kv) => { self.map.insert(kv.key, kv.value); },
                    LogEntry::Undecided(x) => { panic!("read undecided log entry: {:?}", x)},
                    LogEntry::StopSign(_) => { todo!() },
                    LogEntry::Snapshotted(_) => { todo!() },
                    LogEntry::Trimmed(_) => { todo!() },
                }
            }
        }
    }
}

pub fn get(key: &Key) -> Option<Value> {
    let unlocked = Store::instance();
    let mut store = unlocked.lock().unwrap();
    store.apply_decided_entries();
    store.map.get(key).map(|x| x.to_owned())
}

/// Inserts into the replicated store
/// returns the previous value of this key on success
pub async fn put(kv: KeyValue) -> Result<Option<KeyValue>,()> {
    let prev_idx = RSM::instance().lock().unwrap().omnipaxos.get_decided_idx();
    let mut prev_value = get(&kv.key);
    let idx = rsm::append(kv.clone()).await?;

    // read entries that were decided in the meantime, to get latest previous value
    if let Some(entries) = RSM::instance().lock().unwrap().omnipaxos.read_decided_suffix(prev_idx+1) {
        for (i, entry) in entries.iter().enumerate() {
            if prev_idx+1+i as u64 == idx {
                break // only read until the new entry's index
            }
            if let LogEntry::Decided(old) = entry {
                if old.key == kv.key {
                    prev_value = Some(old.value.clone());
                }
            }
        }
    }
    Ok(prev_value.map(|value| KeyValue{key: kv.key, value}))
}
