from requests_futures.sessions import FuturesSession
from concurrent.futures import wait, FIRST_EXCEPTION
from time import sleep, time


MAX_TIMEOUT = 10
TIMEOUT = 3
proxies = {"http": "http://localhost:5000"}


def collect_results(futures_list):
    results_list = []
    (done, not_done) = wait(futures_list, timeout=TIMEOUT, return_when=FIRST_EXCEPTION)
    for f in done:
        r = f.result()
        if r.ok:
            results_list.append({"start": f.start, "end": r.end, "response": r.json()})
        else:
            results_list.append({"start": f.start, "end": float("inf")})

    for f in not_done:
        results_list.append({"start":f.start, "end": float("inf")})
        f.cancel()
    return results_list


futures_list = []
def schedule(future):
    future.start = time()
    futures_list.append(future)

def timing(r, *args, **kwargs):
    r.end = time()


try:
    session = FuturesSession()
    session.hooks["response"] = timing

    # start recording
    # session.get("http://localhost:8000/start_recording").result()

    # partition the network
    session.get(f"http://localhost:8000/block_config?&mode=partitions&partitions=[[\"etcd1\"],[\"etcd2\",\"etcd3\"]]").result()

    # write something
    schedule(session.put("http://etcd1:8080/put", json={"key":"1", "value":"1", "lease":0, "prev_kv": False, "ignore_value": False, "ignore_lease": False}, proxies=proxies, timeout=MAX_TIMEOUT))
    schedule(session.put("http://etcd2:8080/put", json={"key":"1", "value":"2", "lease":0, "prev_kv": False, "ignore_value": False, "ignore_lease": False}, proxies=proxies, timeout=MAX_TIMEOUT))

    sleep(0.5)

    # try to read it
    schedule(session.get(f"http://etcd1:8080/linearizable/get/1", proxies=proxies, timeout=MAX_TIMEOUT))
    schedule(session.get(f"http://etcd2:8080/linearizable/get/1", proxies=proxies, timeout=MAX_TIMEOUT))

    sleep(0.5)

    # un-partition the network
    session.get(f"http://localhost:8000/block_config?&mode=partitions&partitions=[[\"etcd1\",\"etcd2\",\"etcd3\"]]").result()

    sleep(0.5)

    # wait for futures to resolve
    print(collect_results(futures_list))

    # end recording
    # session.get("http://localhost:8000/end_recording").result()

except:
    pass
