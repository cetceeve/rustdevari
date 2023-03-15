from requests_futures.sessions import FuturesSession
from concurrent.futures import wait, FIRST_EXCEPTION
from pprint import pprint
from time import time
import copy
import json

MAX_TIMEOUT = 8
TIMEOUT = 3

def new_session():
    def timing(r, *args, **kwargs):
        r.end = time()
    session = FuturesSession()
    session.hooks["response"] = timing
    return session

def partition(session, partitions):
    try:
        session.get(f"http://localhost:8000/block_config?&mode=partitions&partitions={json.dumps(partitions)}").result()
    except:
        print("partition")
        pass

def crash(session, node):
    try:
        session.post(f"http://localhost:808{node}/crash", timeout=1).result()
    except:
        pass

def snapshot(session, node):
    try:
        session.post(f"http://localhost:808{node}/snapshot", timeout=1).result()
    except:
        print("snapshot")
        pass

def print_log(session, node):
    try:
        session.get(f"http://localhost:808{node}/print_log", timeout=1).result()
    except:
        print("print_log")
        pass

def clear(session, futures_list, node):
    future = session.post(f"http://localhost:808{node}/clear", timeout=MAX_TIMEOUT)
    future.start = time()
    future.input = {}
    future.op = "clear"
    future.node = node
    futures_list.append(future)
    return future

def sc_read(session, futures_list, node, key):
    future = session.get(f"http://localhost:808{node}/get/{key}", timeout=MAX_TIMEOUT)
    future.start = time()
    future.input = {"key": key}
    future.op = "read"
    future.node = node
    futures_list.append(future)
    return future

def read(session, futures_list, node, key):
    future = session.get(f"http://localhost:808{node}/linearizable/get/{key}", timeout=MAX_TIMEOUT)
    future.start = time()
    future.input = {"key": key}
    future.op = "read"
    future.node = node
    futures_list.append(future)
    return future

def put(session, futures_list, node, key, val):
    future = session.put(f"http://localhost:808{node}/put", json={"key":key, "value":val}, timeout=MAX_TIMEOUT)
    future.start = time()
    future.input = {"key": key, "value": val}
    future.op = "put"
    future.node = node
    futures_list.append(future)
    return future

def cas(session, futures_list, node, key, new_val, expected_val):
    future = session.post(f"http://localhost:808{node}/cas", json={"key":key, "new_value":new_val, "expected_value":expected_val}, timeout=MAX_TIMEOUT)
    future.start = time()
    future.input = {"key": key, "new_value": new_val, "expected_value": expected_val}
    future.op = "cas"
    future.node = node
    futures_list.append(future)
    return future

def delete(session, futures_list, node, key):
    future = session.delete(f"http://localhost:808{node}/delete/{key}", timeout=MAX_TIMEOUT)
    future.start = time()
    future.input = {"key": key}
    future.op = "delete"
    future.node = node
    futures_list.append(future)
    return future

def collect_results(futures_list):
    results_list = []
    (done, not_done) = wait(futures_list, timeout=TIMEOUT, return_when=FIRST_EXCEPTION)
    for f in done:
        r = None
        try:
            r = f.result()
        except:
            not_done.add(f)
            continue
        if r.ok:
            results_list.append({
                "start": f.start,
                "end": r.end,
                "node": f.node,
                "op": f.op,
                "input": f.input,
                "result": r.json(),
            })
    for f in not_done:
        results_list.append({
            "start":f.start,
            "end": float("inf"),
            "node": f.node,
            "op": f.op,
            "input": f.input,
            "result": None,
        })
    return results_list


def wing_gong(results_list):
    """
    The Wing & Gong linearizability checker algorithm, implemented for Key Value stores.
    Takes an input trace in a format like the following
    [{
        "start": 1,
        "end": 2,
        "op": "put",
        "input": {"key": "1", "value": "2"},
        "result": {"prev_kv": None},
    }, ...]
    """
    class S:
        def __init__(self):
            self.state = {}
        def perform(self, event):
            op = event["op"]
            if op == "put":
                key = event["input"]["key"]
                val = event["input"]["value"]
                result = {"prev_kv": None}
                if key in self.state:
                    result["prev_kv"] = {"key": key, "value": self.state[key]}
                self.state[key] = val
                return result
            if op == "cas":
                key = event["input"]["key"]
                new_val = event["input"]["new_value"]
                exp_val = event["input"]["expected_value"]
                result = {"prev_kv": None}
                if key in self.state:
                    result["prev_kv"] = {"key": key, "value": self.state[key]}
                    if self.state[key] == exp_val:
                        self.state[key] = new_val
                return result
            if op == "delete":
                key = event["input"]["key"]
                result = {"prev_kv": None}
                if key in self.state:
                    result["prev_kv"] = {"key": key, "value": self.state[key]}
                    del self.state[key]
                return result
            if op == "read":
                key = event["input"]["key"]
                if not key in self.state:
                    return {"key": key, "value": None}
                else:
                    return {"key": key, "value": self.state[key]}
            if op == "clear":
                self.state.clear()
                return None
            
    def search(h, s):
        print(f"state: {s.state}, events left:")
        pprint(h)
        if len(h) == 0:
            return True
        min_end = h[0]["end"]
        for event in h:
            if event["start"] >= min_end:
                continue
            new_s = copy.deepcopy(s)
            res = new_s.perform(event)
            new_h = copy.copy(h)
            new_h.remove(event)
            if (event["end"] == float("inf") or res == event["result"]) and search(new_h, new_s):
                return True
        return False

    list.sort(results_list, key=lambda x: x["end"])
    if search(results_list, S()):
        print("The execution is linearizable.")
        return True
    else:
        print("The execution is NOT linearizable.")
        return False
