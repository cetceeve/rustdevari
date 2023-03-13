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
- [ ] snapshots
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
