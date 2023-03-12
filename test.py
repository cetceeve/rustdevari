from time import sleep
from pprint import pprint
from util import (
    sc_read,
    read,
    put,
    cas,
    delete,
    partition,
    crash,
    new_session,
    collect_results,
    wing_gong,
)


futures_list = []
result_events = []

try:
    session = new_session()

    # partition the network
    partition(session, [["etcd1"],["etcd2","etcd3"]])

    # write something
    put(session, futures_list, 1, "1", "1")
    put(session, futures_list, 2, "1", "2")

    sleep(0.5)

    # crash(session, 2)
    # crash(session, 1)

    # try to read it
    read(session, futures_list, 1, "1")
    read(session, futures_list, 2, "1")

    sleep(0.5)

    # un-partition the network
    partition(session, [["etcd1","etcd2","etcd3"]])


    sleep(1)

    cas(session, futures_list, 1, "1", "3", "2")
    cas(session, futures_list, 3, "1", "4", "1")
    sleep(0.1)
    read(session, futures_list, 2, "1")

    sleep(1)

    # cleanup
    delete(session, futures_list, 1, "1")

    sleep(3)
    
    result_events = collect_results(futures_list)

except Exception as e:
    print(e)
    pass

# check linearizability
wing_gong(result_events)
