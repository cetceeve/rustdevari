from time import sleep
from pprint import pprint
from random import random
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
)

def initial_broad_test_case(session, futures_list):
    """The initial testing case"""

    # partition the network
    partition(session, [["etcd1"],["etcd2","etcd3"]])
    # write something
    put(session, futures_list, 1, "1", "1")
    put(session, futures_list, 2, "1", "2")
    # sleep to make sure the writes are done
    sleep(0.5)
    # crash everything
    crash(session, 3)
    crash(session, 2)
    crash(session, 1)
    sleep(1)
    # try to read it
    read(session, futures_list, 1, "1")
    read(session, futures_list, 2, "1")
    sleep(1)
    # snapshot(session, 3)
    # un-partition the network
    partition(session, [["etcd1","etcd2","etcd3"]])
    sleep(1)
    cas(session, futures_list, 1, "1", "3", "2")
    cas(session, futures_list, 3, "1", "4", "1")
    sleep(0.1)
    read(session, futures_list, 2, "1")
    # clear and see if it's clear
    clear(session, futures_list, 3)
    sleep(0.1)
    read(session, futures_list, 1, "1")
    sleep(1)
    # cleanup
    clear(session, futures_list, 2)
    sleep(3)


def write_sleep_read(session, futures_list):
    """Write, sleep for 0.5-1 seconds, then read"""
    put(session, futures_list, 1, "1", "1")
    put(session, futures_list, 2, "1", "2")
    sleep(random())
    read(session, futures_list, 1, "1")
    read(session, futures_list, 2, "1")

def writee_overwrite_sleep_read(session, futures_list):
    """write, overwrite, sleep, read"""
    put(session, futures_list, 1, "1", "1")
    put(session, futures_list, 2, "1", "2")
    sleep(random())
    read(session, futures_list, 3, "1")

def write_sleep_cas_sleep_read(session, futures_list):
    """write, sleep, cas, sleep, read"""
    put(session, futures_list, 1, "1", "1")
    sleep(random())
    cas(session, futures_list, 2, "1", "2", "1")
    sleep(random())
    read(session, futures_list, 3, "1")

def no_sleep(session, futures_list):
    """scip the sleeping to see how the system reacts"""
    put(session, futures_list, 1, "1", "1")
    cas(session, futures_list, 2, "1", "2", "1")
    read(session, futures_list, 3, "1")
    put(session, futures_list, 1, "2", "1")
    cas(session, futures_list, 2, "2", "2", "1")
    read(session, futures_list, 3, "2")

def run_test(test_function):
    """Given a function as input, run the test function and evaluate the result"""
    futures_list = []
    result_events = []
    session = new_session()
    test_function(session, futures_list)
    result_events = collect_results(futures_list)
    wing_gong(result_events)


def main():
    print("Recommendation: run this script with \"python test.py > log.txt\"")
    sleep(1)
    # run and evaluate the initial test case
    run_test(initial_broad_test_case)
    
    # run the new test cases 2 times
    # just for the sake of testing
    for _ in range(2):
        run_test(write_sleep_read)
        run_test(writee_overwrite_sleep_read)
        run_test(write_sleep_cas_sleep_read)
        run_test(no_sleep)

if __name__ == "__main__":
    main()