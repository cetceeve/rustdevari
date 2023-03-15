from time import sleep
from random import randint
from util import (
    sc_read,
    read,
    put,
    cas,
    delete,
    partition,
    crash,
    clear,
    new_session,
    snapshot,
    print_log,
    collect_results,
    wing_gong,
    get_availability,
    print_availability,
)

N = 10
CRASH = True
NODES = [1, 2, 3]
KEYS = ["k1", "k2"]
VALUES = ["v1", "v2", "v3", "v5", "v6", "v7", "v8", "v9"]

### Helper functions

def random_node():
    return NODES[randint(0, len(NODES)-1)]

def random_key():
    return KEYS[randint(0, len(KEYS)-1)]

def random_value():
    return VALUES[randint(0, len(VALUES)-1)]

def random_set_of_nodes():
    nodes = NODES.copy()
    for _ in range(randint(0, len(NODES))):
        try:
            nodes.remove(random_node())
        except:
            pass
    return nodes

### Failures

def partition_at_random(session):
    partitions = []
    for _ in range(randint(2, 4)):
        p = [f"etcd{node}" for node in random_set_of_nodes()]
        partitions.append(p)
    partition(session, partitions)
    
def unpartition(session):
    partitions = [[f"etcd{node}" for node in NODES]]
    partition(session, partitions)

def crash_random_nodes(session):
    for node in random_set_of_nodes():
        crash(session, node)

def random_failure(session):
    if randint(0, 2) > 0 and CRASH:
        crash_random_nodes(session)
    else:
        partition_at_random(session)

### Operations

def random_put(session, futures_list):
    put(session, futures_list, random_node(), random_key(), random_value())

def random_delete(session, futures_list):
    delete(session, futures_list, random_node(), random_key())

def random_cas(session, futures_list):
    cas(session, futures_list, random_node(), random_key(), random_value(), random_value())

def random_read(session, futures_list):
    read(session, futures_list, random_node(), random_key())
    
def random_clear(session, futures_list):
    clear(session, futures_list, random_node())

def random_snapshot(session, _):
    snapshot(session, random_node())

def random_operation(session, futures_list):
    operations = [random_put, random_delete, random_cas, random_read, random_clear, random_snapshot]
    operations[randint(0, len(operations)-1)](session, futures_list)

### Putting it together

def run_single_random_test(availability):
    trace = []
    try:
        futures_list = []
        session = new_session()
        # ensure clean state
        random_clear(session, futures_list)
        sleep(1)
        # write a value, so we are actually testing something interesting
        random_put(session, futures_list)
        sleep(0.5)
        # now perform some random operations and inject random failures
        for _ in range(3):
            if randint(0, 2) > 0:
                random_failure(session)
            else:
                unpartition(session)
            sleep(0.5)
            random_operation(session, futures_list)
            random_operation(session, futures_list)
            sleep(0.5)
            random_operation(session, futures_list)
            sleep(0.5)
        unpartition(session)
        sleep(1)

        trace = collect_results(futures_list)
    except Exception as e:
        print(e)
    get_availability(trace, availability=availability)
    return wing_gong(trace)

### Run n rounds, or until we encounter a non-linearizable trace

availability = {}
is_linearizable = True
n = 0
while is_linearizable and n < N:
    is_linearizable = run_single_random_test(availability)
    n += 1
print_availability(availability)

