from time import sleep
from util import (
    read,
    put,
    partition,
    clear,
    new_session,
    snapshot,
    collect_results,
    wing_gong,
    get_availability,
    print_availability,
)


futures_list = []
trace = []

try:
    session = new_session()
    # partition the network
    partition(session, [["etcd1"],["etcd2","etcd3"]])
    sleep(1)
    put(session, futures_list, 2, "k1", "v1")
    put(session, futures_list, 3, "k2", "v2")
    sleep(1)
    snapshot(session, 2)
    sleep(1)
    # un-partition the network
    partition(session, [["etcd1","etcd2","etcd3"]])
    sleep(1)
    # try to read it
    read(session, futures_list, 1, "k1")
    read(session, futures_list, 2, "k2")
    sleep(1)
    # cleanup
    clear(session, futures_list, 3)
    sleep(3)

    trace = collect_results(futures_list)

except Exception as e:
    print(e)
    pass

# check linearizability
if wing_gong(trace):
    exit(1)
