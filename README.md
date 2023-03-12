# ID2203-Distributed-Systems-Final-Project

## TODO
### Features
#### Need
- [x] sequentially consistent read/write
- [x] linearizable reads
- [x] CAS operation
- [x] Delete operation
- [x] perfect link (omission model)
- [ ] crash recovery
#### Want
- [x] snapshots
- [ ] configuration changes
### Testing
- [x] linearizability checker
- [ ] measure availability / show that progress is being made during partitions
- [ ] test crash recovery
- [ ] test cases for all partial connectivity scenarios
- [ ] sequential consistency checker?



## Consistency
Like etcd, our implementation guarantees sequential consistency by default with all operations. This comes by default with omnipaxos.
We also support linearizable reads at a separate endpoint, by deciding the read before returning a value from local storage.
