from requests_futures.sessions import FuturesSession
from concurrent.futures import wait, FIRST_EXCEPTION
from pprint import pprint
from time import time
import copy

MAX_TIMEOUT = 8
TIMEOUT = 3
proxies = {"http": "http://localhost:5000"}

def new_session():
    def timing(r, *args, **kwargs):
        r.end = time()
    session = FuturesSession()
    session.hooks["response"] = timing
    return session

def read(session, futures_list, node, key):
    future = session.get(f"http://etcd{node}:8080/linearizable/get/{key}", proxies=proxies, timeout=MAX_TIMEOUT)
    future.start = time()
    future.input = {"key": key}
    future.op = "read"
    futures_list.append(future)
    return future

def put(session, futures_list, node, key, val):
    future = session.put(f"http://etcd{node}:8080/put", json={"key":key, "value":val, "lease":0, "prev_kv": True, "ignore_value": False, "ignore_lease": True}, proxies=proxies, timeout=MAX_TIMEOUT)
    future.start = time()
    future.input = {"key": key, "value": val}
    future.op = "put"
    futures_list.append(future)
    return future

def collect_results(futures_list):
    results_list = []
    (done, not_done) = wait(futures_list, timeout=TIMEOUT, return_when=FIRST_EXCEPTION)
    for f in done:
        r = f.result()
        if r.ok:
            results_list.append({
                "start": f.start,
                "end": r.end,
                "op": f.op,
                "input": f.input,
                "result": r.json(),
            })
    for f in not_done:
        results_list.append({
            "start":f.start,
            "end": float("inf"),
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
            if op == "read":
                key = event["input"]["key"]
                if not key in self.state:
                    return {"key": key, "value": None}
                else:
                    return {"key": key, "value": self.state[key]}
            
    def search(h, s):
        print(f"state: {s.state}, events:")
        pprint(h)
        if len(h) == 0:
            return True
        min_end = h[0]["end"]
        for event in h:
            if event["start"] >= min_end:
                break
            new_s = copy.deepcopy(s)
            res = new_s.perform(event)
            new_h = copy.copy(h)
            new_h.remove(event)
            if (event["end"] == float("inf") or res == event["result"]) and search(new_h, new_s):
                return True
            else:
                print(f"want: {res}, got: {event['result']}")
        return False

    list.sort(results_list, key=lambda x: x["end"])
    if search(results_list, S()):
        print("The execution is linearizable.")
    else:
        print("The execution is NOT linearizable.")
