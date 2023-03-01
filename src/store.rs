use crate::types::*;
use crate::rsm::RSM;
use omnipaxos_core::util::LogEntry;
use std::{sync::{Arc, Mutex}, collections::HashMap};

static mut INSTANCE: Option<Arc<Mutex<Store>>> = None;

pub struct Store { map: HashMap<Key, Value>, applied_log_index: u64,
}

impl Store {
    /// Get the singleton Store instance
    pub fn instance() -> Arc<Mutex<Self>> {
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
    pub fn apply_decided_entries(&mut self) {
        if let Some(entries) = RSM::instance().lock().unwrap().omnipaxos.read_decided_suffix(self.applied_log_index) {
            for entry in entries {
                self.applied_log_index += 1;
                match entry {
                    LogEntry::Decided(kv) => { self.map.insert(kv.key, kv.value); },
                    LogEntry::Undecided(_) => {},
                    LogEntry::StopSign(_) => { todo!() },
                    LogEntry::Snapshotted(_) => { todo!() },
                    LogEntry::Trimmed(_) => { todo!() },
                }
            }
        }
    }

    pub fn get(&mut self, key: &Key) -> Option<Value> {
        self.apply_decided_entries();
        self.map.get(key).map(|x| x.to_owned())
    }
}

