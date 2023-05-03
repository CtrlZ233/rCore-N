import socket
import time
import sched
from threading import Thread, Lock

import threading

RUN_TIME_LIMIT = 1000

global_num = 0
lock = Lock()

local = threading.local()

def req(socket):
    send_data = "client send data"
    socket.send(send_data.encode("utf8"))
    recv_data = socket.recv(1024)
    print('recv data:', recv_data.decode("utf8"))
    loop_monitor(socket)
        

def loop_monitor(socket):
    if hasattr(local, "start_time") == False:
        local.start_time = int(round(time.time() * 1000))
        print(local.start_time)

    if int(round(time.time() * 1000)) >= local.start_time + RUN_TIME_LIMIT:
        send_data = "close connection"
        socket.send(send_data.encode("utf8"))
        return
    s = sched.scheduler(time.time, time.sleep)  # 生成调度器
    s.enter(0.05, 1, req, (socket,))
    s.run()

def connect(index):
    global global_num
    tcp_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    time.sleep(0.1 * index)
    server_addr = ("127.0.0.1", 6201)
    tcp_socket.connect(server_addr)

    send_data = "connect ok?"
    tcp_socket.send(send_data.encode("utf8"))
    recv_data = tcp_socket.recv(1024)

    print('recv connect result:', recv_data.decode("utf8"))
    if recv_data.decode("utf8") == "connect ok":
        with lock:
            global_num += 1

    while True:
        with lock:
            if global_num == threads_num:
                break
    print("all threads connect success!")
    loop_monitor(tcp_socket)




threads = []
threads_num = 32
for i in range(threads_num):
    t = Thread(target=connect, args=(i,))
    threads.append(t)
    t.start()

for i in range(threads_num):
    threads[i].join()

