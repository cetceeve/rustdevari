import requests
from time import sleep

# resp = requests.put("http://localhost:8081/put", json={"key":"1", "value":"Hello World!"})
# print(resp.text)

# resp = requests.get("http://localhost:8082/snapshot")
# print(resp.text)

# resp = requests.get("http://localhost:8082/linearizable/get/1")
# print(resp.text)

# resp = requests.post("http://localhost:8081/cas", json={"key":"1", "expected_value":"Hello World!", "new_value":"CAS Wrote"})
# print(resp.text)

# resp = requests.put("http://localhost:8083/delete", json={"key":"1", "value":"Test!"})
# print(resp.text)

# resp = requests.post("http://localhost:8082/clear")
# print(resp.text)

# sleep(1)
# resp = requests.get("http://localhost:8082/linearizable/get/1")
# print(resp.text)



resp = requests.post("http://localhost:8082/clear")
requests.get(f"http://localhost:8000/block_config?&mode=partitions&partitions=[[\"etcd1\"], [\"etcd2\", \"etcd3\"]]")
resp = requests.put("http://localhost:8082/put", json={"key":"1", "value":"Hello World!"})
print(resp.text)
resp = requests.get("http://localhost:8082/linearizable/get/1")
print(resp.text)
resp = requests.get("http://localhost:8081/get/1")
print(resp.text)
try:
    resp = requests.get("http://localhost:8081/linearizable/get/1", timeout=1)
    print(resp.text)
except:
    print("timed out")
requests.get(f"http://localhost:8000/block_config?&mode=partitions&partitions=[[\"etcd1\", \"etcd2\", \"etcd3\"]]")
resp = requests.post("http://localhost:8082/clear")
