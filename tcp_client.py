import socket
import time
import sched
from threading import Thread


def req(socket):
    send_data = "client send data"
    socket.send(send_data.encode("gbk"))
    recv_data = socket.recv(1024)
    print('recv data:', recv_data.decode("gbk"))
    loop_monitor(socket)
        

def loop_monitor(socket):
    s = sched.scheduler(time.time, time.sleep)  # 生成调度器
    s.enter(1, 1, req, (socket,))
    s.run()

def connect():
    tcp_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server_addr = ("127.0.0.1", 6201)
    tcp_socket.connect(server_addr)
    print("connect success!") 
    loop_monitor(tcp_socket)



threads = []
threads_num = 8
for i in range(threads_num):
    t = Thread(target=connect)
    threads.append(t)
    t.start()

for i in range(threads_num):
    threads[i].join()

