# ID2203-Distributed-Systems-Final-Project
This project is an implementation of the core key-value functionality of a distributed coordination service
like etcd. Unlike the current industry standard coordination services however, it is based around the Omnipaxos
consensus protocol, instead of Raft or ZAB.

## How to run and test
The easiest way to spin up a local cluster on docker-compose for testing is like this:
```sh
make full  # using our Makefile

# or if that doesn't work for you...
docker build -t op-etcd .
docker-compose up -V
```
If you would like to build on your local machine, you can do the following instead.
This also gives you more control over which features to use. We support two mutually exclusive
feature flags: `crash_recovery` to enable persistent storage and `pl` to enable a Perfect Link channel
implementation.
```sh
# pick your build command
cargo build --release
cargo build --release --features crash_recovery
cargo build --release --features pl

# and then build the image and start the cluster
docker build -f DevDockerfile -t op-etcd .
docker-compose up -V
```
If you take a look at the docker-compose file, you will notice, that we start not only the instances of our
service, but also an instance of the ditm testing proxy. We route all network traffik between nodes through this proxy
to allow us to simulate arbitrary network partitions.
Our testing setup makes heavy use of this. You can run it like the following.
```sh
# first start the cluster like above
# then run the test script
python random_test.py
```
This script will run random command sequences against the service, while injecting random crashes and network
partitions and checking the generated traces for linearizability. It can be configured via a few constants at the top of the file.

We also have a special test case that can demonstate a bug in the current version of the Omnipaxos library.
To reproduce the bug, go into the `Cargo.toml` file of this project, switch the commented Omnipaxos dependencies and run the following.
```sh
# first start the cluster like above
# then run the test script
python snapshot_test.py
```

## Consistency
Like etcd, our implementation guarantees sequential consistency by default with all operations. This comes by default with omnipaxos.
We also support linearizable reads at a separate endpoint, by deciding the read before returning a value from local storage. All other
key-value operations are linearizable by default, since they are decided before returning.

## TODO
#### Features
##### Need
- [x] sequentially consistent read/write
- [x] linearizable reads
- [x] CAS operation
- [x] Delete operation
- [x] perfect link (omission model)
- [x] crash recovery
##### Want
- [x] snapshots
- [ ] configuration changes
#### Testing
- [x] linearizability checker
- [x] measure availability / show that progress is being made during partitions
- [x] make crash recovery testable
- [x] randomized testing with failure injection

