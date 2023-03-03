# ID2203-Distributed-Systems-Final-Project

# Consistency
Like etcd, our implementation guarantees sequential consistency by default with all operations. This comes by default with omnipaxos.
We also support linearizable reads at a separate endpoint, by deciding the read before returning a value from local storage.
