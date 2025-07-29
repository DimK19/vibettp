import socket
import time

s = socket.socket()
s.connect(("127.0.0.1", 7878))
s.send(b"GET /hello HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: keep-alive\r\n")

## Simulate slowloris: send one header every few seconds
for i in range(20):
    time.sleep(10)  ## adjust to match server timeout
    try:
        s.send(b"X-Slowloris-Chunk: still-here\r\n")
        print(f"Sent partial header {i+1}")
    except:
        print("Connection closed by server.")
        break
