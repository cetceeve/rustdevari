from time import sleep
from pprint import pprint
from util import new_session, read, put, cas, collect_results, wing_gong


futures_list = []
result_events = []

try:
    session = new_session()

    # partition the network
    session.get(f"http://localhost:8000/block_config?&mode=partitions&partitions=[[\"etcd1\"],[\"etcd2\",\"etcd3\"]]").result()

    # write something
    put(session, futures_list, 1, "1", "1")
    put(session, futures_list, 2, "1", "2")

    sleep(0.5)

    # try to read it
    read(session, futures_list, 1, "1")
    read(session, futures_list, 2, "1")

    sleep(0.5)

    # un-partition the network
    session.get(f"http://localhost:8000/block_config?&mode=partitions&partitions=[[\"etcd1\",\"etcd2\",\"etcd3\"]]").result()

    sleep(1)

    cas(session, futures_list, 1, "1", "3", "2")
    cas(session, futures_list, 3, "1", "4", "1")
    sleep(0.1)
    read(session, futures_list, 2, "1")

    sleep(3)
    
    result_events = collect_results(futures_list)

except Exception as e:
    pass

# check linearizability
wing_gong(result_events)
