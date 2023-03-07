from requests_futures.sessions import FuturesSession
from concurrent.futures import wait, FIRST_EXCEPTION
from time import sleep, time
from pprint import pprint


MAX_TIMEOUT = 5
TIMEOUT = 3
proxies = {"http": "http://localhost:5000"}


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
    """ The Wing & Gong linearizability checker algorithm, implemented for Key Value stores """
    class S:
        def __init__(self):
            self.state = {}
        def perform(self, event):
            op = event["op"]
            if op == "put":
                key = event["input"]["key"]
                val = event["input"]["value"]
                result = {"prev_kv": None}
                if not key in self.state:
                    self.state[key] = [val]
                else:
                    self.state[key].append(val)
                    if len(self.state[key]) > 1:
                        result["prev_kv"] = {"key": key, "value": self.state[key][-2]}
                return result
            if op == "read":
                key = event["input"]["key"]
                if not key in self.state or len(self.state[key]) == 0:
                    return None
                else:
                    return {"key": key, "value": self.state[key][-1]}
        def undo(self, event):
            op = event["op"]
            if op in ["read"]:
                return
            if op == "put":
                key = event["input"]["key"]
                del self.state[key][-1]
            
    def search(h, s):
        if len(h) == 0:
            return True
        min_end = h[0]["end"]
        for event in h:
            if event["start"] >= min_end:
                break
            res = event["result"]
            check_res = s.perform(event)
            new_h = h.copy()
            new_h.remove(event)
            if (event["end"] == float("inf") or check_res == res) and search(new_h, s):
                return True
            else:
                print(f"want: {check_res}, got: {res}")
            s.undo(event)
        return False

    list.sort(results_list, key=lambda x: x["end"])
    return search(results_list, S())

futures_list = []
def read(node, key):
    future = session.get(f"http://etcd{node}:8080/linearizable/get/{key}", proxies=proxies, timeout=MAX_TIMEOUT)
    future.start = time()
    future.input = {"key": key}
    future.op = "read"
    futures_list.append(future)
    return future

def put(node, key, val):
    future = session.put(f"http://etcd{node}:8080/put", json={"key":key, "value":val, "lease":0, "prev_kv": True, "ignore_value": False, "ignore_lease": True}, proxies=proxies, timeout=MAX_TIMEOUT)
    future.start = time()
    future.input = {"key": key, "value": val}
    future.op = "put"
    futures_list.append(future)
    return future


result_events = []
try:
    def timing(r, *args, **kwargs):
        r.end = time()
    session = FuturesSession()
    session.hooks["response"] = timing

    # start recording
    # session.get("http://localhost:8000/start_recording").result()

    # partition the network
    session.get(f"http://localhost:8000/block_config?&mode=partitions&partitions=[[\"etcd1\"],[\"etcd2\",\"etcd3\"]]").result()

    # write something
    put(1, "1", "1")
    put(2, "1", "2")

    sleep(0.5)

    # try to read it
    read(1, "1")
    read(2, "1")

    sleep(0.5)

    # un-partition the network
    session.get(f"http://localhost:8000/block_config?&mode=partitions&partitions=[[\"etcd1\",\"etcd2\",\"etcd3\"]]").result()

    sleep(0.5)
    
    result_events = collect_results(futures_list)

    # end recording
    # session.get("http://localhost:8000/end_recording").result()

except:
    pass

# wait for futures to resolve
print(wing_gong(result_events))
