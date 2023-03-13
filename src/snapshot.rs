use crate::rsm::RSMCommand;
use crate::types::*;
use std::collections::HashMap;
use omnipaxos_core::storage::Snapshot;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OPSnapshot {
    pub snapshotted: HashMap<Key, Vec<RSMCommand>>
}

impl Snapshot<RSMCommand> for OPSnapshot {
    fn create(entries: &[RSMCommand]) -> Self {
        let mut snapshotted = HashMap::new();
        for cmd in entries {
            match cmd {
                RSMCommand::LinearizableRead(_) => (),
                RSMCommand::Put((_, kv)) => { snapshotted.insert(kv.key.clone(), vec![cmd.clone()]); },
                RSMCommand::Delete((_, key)) => { snapshotted.insert(key.clone(), vec![cmd.clone()]); },
                RSMCommand::CAS((_, kv, _)) => {
                    if let Some(x) = snapshotted.get_mut(&kv.key) {
                        x.push(cmd.clone());
                    } else {
                        snapshotted.insert(kv.key.clone(), vec![cmd.clone()]);
                    }
                },
                RSMCommand::Clear(_) => { snapshotted.clear(); },
            }
        }
        Self { snapshotted }
    }

    fn merge(&mut self, delta: Self) {
        for (k, v) in delta.snapshotted {
            for cmd in v {
                match cmd {
                    RSMCommand::LinearizableRead(_) => (),
                    RSMCommand::Put(_) => { self.snapshotted.insert(k.clone(), vec![cmd.clone()]); },
                    RSMCommand::Delete(_) => { self.snapshotted.insert(k.clone(), vec![cmd.clone()]); },
                    RSMCommand::CAS(_) => {
                        if let Some(x) = self.snapshotted.get_mut(&k) {
                            x.push(cmd.clone());
                        } else {
                            self.snapshotted.insert(k.clone(), vec![cmd.clone()]);
                        }
                    },
                    RSMCommand::Clear(_) => { self.snapshotted.clear(); },
                }
            }
        }
    }

    fn use_snapshots() -> bool {
        false
    }
}
