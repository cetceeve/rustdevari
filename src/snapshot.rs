use crate::rsm::RSMCommand;
use crate::types::*;
use std::collections::HashMap;
use omnipaxos_core::storage::Snapshot;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OPSnapshot {
    pub snapshotted: HashMap<Key, Value>
}

impl Snapshot<RSMCommand> for OPSnapshot {
    fn create(entries: &[RSMCommand]) -> Self {
        let mut snapshotted = HashMap::new();
        for cmd in entries {
            match cmd {
                RSMCommand::Put((_, kv)) => { snapshotted.insert(kv.key.clone(), kv.value.clone()); },
                RSMCommand::CAS((_, kv, exp_val)) => {
                    if let Some(old_val) = snapshotted.get(&kv.key) {
                        if *old_val == *exp_val {
                            snapshotted.insert(kv.key.clone(), kv.value.clone());
                        }
                    }
                },
                RSMCommand::Delete((_, key)) => { snapshotted.remove(key); },
                RSMCommand::LinearizableRead(_) => (),
            }
        }
        Self { snapshotted }
    }

    fn merge(&mut self, delta: Self) {
        for (k, v) in delta.snapshotted {
            self.snapshotted.insert(k, v);
        }
    }

    fn use_snapshots() -> bool {
        true
    }
}
