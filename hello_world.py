import requests
from time import sleep

resp = requests.put("http://localhost:8081/put", json={"key":"1", "value":"Hello World!", "lease":0, "prev_kv": False, "ignore_value": False, "ignore_lease": False})
print(resp.text)

resp = requests.get("http://localhost:8082/get/1")
print(resp.text)

resp = requests.put("http://localhost:8083/put", json={"key":"1", "value":"Test!", "lease":0, "prev_kv": False, "ignore_value": False, "ignore_lease": False})
print(resp.text)
