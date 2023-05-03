import socket
import time
import sched
from threading import Thread, Lock

global_num = 0
lock = Lock()

def req(socket):
    send_data = "client send data"
    socket.send(send_data.encode("gbk"))
    recv_data = socket.recv(1024)
    print('recv data:', recv_data.decode("gbk"))
    loop_monitor(socket)
        

def loop_monitor(socket):
    s = sched.scheduler(time.time, time.sleep)  # 生成调度器
    s.enter(0.05, 1, req, (socket,))
    s.run()

def connect(index):
    global global_num
    tcp_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    time.sleep(0.2 * index)
    server_addr = ("127.0.0.1", 6201)
    tcp_socket.connect(server_addr)

    send_data = "connect ok?"
    tcp_socket.send(send_data.encode("gbk"))
    recv_data = tcp_socket.recv(1024)

    print('recv connect result:', recv_data.decode("gbk"))
    if recv_data.decode("gbk") == "connect ok":
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

