# ID2203-Distributed-Systems-Final-Project

## TODO
### Features
#### Need
- [x] sequentially consistent read/write
- [x] linearizable reads
- [x] CAS operation
- [x] Delete operation
- [x] perfect link (omission model)
- [x] crash recovery
#### Want
- [x] snapshots
- [ ] configuration changes
### Testing
- [x] linearizability checker
- [ ] measure availability / show that progress is being made during partitions
- [x] make crash recovery testable
- [ ] test cases for all interesting failure scenarios
- [ ] sequential consistency checker?



## Consistency
Like etcd, our implementation guarantees sequential consistency by default with all operations. This comes by default with omnipaxos.
We also support linearizable reads at a separate endpoint, by deciding the read before returning a value from local storage.

## Notes
We seem to have found a problem with omnipaxos' snapshot capabilities.
Specifically, when logs get synced during the prepare phase,
if i.e. the leader has a decided_idx = 5 and compacted_idx = 5
and the follower has decided_idx = 2
the leader will not be able to construct an appropriate delta snapshot and will instead have to
send a complete snapshot in his AccSync message. Omnipaxos currently does not do this, and instead
crashes.
We think that we have fixed this on our fork of the library.
