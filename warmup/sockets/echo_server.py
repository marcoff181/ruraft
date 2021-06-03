import time
from socket import socket, AF_INET, SOCK_STREAM


def main():
    sock = socket(AF_INET, SOCK_STREAM)
    sock.bind(('',12345))
    sock.listen()
    while True:
        client, addr = sock.accept()
        print('Connection from', addr)
        response = client.recv(1000)
        client.sendall(response)
        client.close()

main()
