#!/usr/bin/python3

# 1. [5 points] Receive connections from a client, and send them the motd
# 2. [20 points] Receive heartbeats from the client, and disconnect them if a heartbeat is not received within a certain interval
# 3. [20 points] Receive messages from the client
# 4. [20 points] Forward messages to everyone in the channel
# 5. [20 points] Support for multiple channels
# 6. [15 points] Support for whisper messages

import unittest
import threading
import socket
import signal
import time

HOSTNAME = "localhost"
PORT = int(6667)

def send(s, content):
	s.sendall(content.encode())

def recv(s):
	return s.recv(4096).decode("ascii").replace('\x00','').strip()

class RustIRCTest(unittest.TestCase):
	"""Tests for the RustIRC"""

	@classmethod
	def setUpClass(self):
		self.sockets = []
		for i in range(0, 2):
			s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
			s.connect((HOSTNAME, PORT))
			self.sockets.append(s)
			name = "client" + str(i)
			send(s, "NICK " + name + "\r\n")
			recv(s)
			send(s, "USER " + name + " localhost localhost\r\n")
			recv(s)
			recv(s)

	@classmethod
	def tearDownClass(self):
		print("Attempting to shut down RustIRCTest")
		for s in self.sockets:
			send(s, "PART #general :done\r\n")
			time.sleep(1)
			send(s, "QUIT\r\n")
			time.sleep(1)
			s.close()

	def runTest(self):
		print("Setting nick to 'trittimo'")
		send(self.sockets[0], "NICK trittimo\r\n")
		resp = recv(self.sockets[0])
		self.assertEqual(resp, ":trittimo NICK trittimo")
		print("\tNICK command successful")

		print("Setting the user to 'trittimo localhost localhost'")
		send(self.sockets[0], "USER trittimo localhost localhost\r\n")
		resp = recv(self.sockets[0])
		self.assertEqual(resp, "PING :3813401942")
		resp = recv(self.sockets[0])
		self.assertEqual(resp, ":localhost 001 trittimo :Welcome to RustIRC!")
		print("\tUSER command successful")

		print("Listing the available channels")
		send(self.sockets[0], "LIST\r\n")
		resp = recv(self.sockets[0])
		self.assertEqual(resp, ":localhost 321 RustIRC Channel :Users  Name\r\n:localhost 322 RustIRC #general 0 :Anything goes\r\n:localhost 322 RustIRC #rust 0 :Complain about rust here\r\n:localhost 323 RustIRC :End of /LIST")
		print("\tLIST command successful")

		print("Having 'trittimo' join #general")
		send(self.sockets[0], "JOIN #general\r\n")
		resp = recv(self.sockets[0])
		self.assertEqual(resp, ":localhost 332 trittimo #general :Anything goes")
		resp = recv(self.sockets[0])
		self.assertEqual(resp, "trittimo")
		print("\tJOIN command successful")

		print("Listing the available channels from client1")
		send(self.sockets[1], "LIST\r\n")
		resp = recv(self.sockets[1])
		self.assertEqual(resp, ":localhost 321 RustIRC Channel :Users  Name\r\n:localhost 322 RustIRC #general 1 :Anything goes\r\n:localhost 322 RustIRC #rust 0 :Complain about rust here\r\n:localhost 323 RustIRC :End of /LIST")
		print("\tLIST command successful")

		print("Having 'client1' join #general")
		send(self.sockets[1], "JOIN #general\r\n")
		resp = recv(self.sockets[1])
		self.assertEqual(resp, ":localhost 332 client1 #general :Anything goes")
		resp = recv(self.sockets[1])
		self.assertEqual(resp, "client1 trittimo")
		print("\tJOIN command successful")

		print("Sending a message from 'trittimo' to the channel")
		send(self.sockets[0], "PRIVMSG #general hello world\r\n")
		resp = recv(self.sockets[1])
		self.assertEqual(resp, ":trittimo PRIVMSG #general hello world")
		print("\tPRIVMSG to #general works")

		print("Sending a message from 'trittimo' to 'client1'")
		send(self.sockets[0], "PRIVMSG client1 hello world\r\n")
		resp = recv(self.sockets[1])
		self.assertEqual(resp, ":trittimo PRIVMSG client1 hello world")
		print("\tPRIVMSG from trittimo to client1")

		print("Testing ping")
		send(self.sockets[0], "PING trittimo")
		resp = recv(self.sockets[0])
		self.assertEqual(resp, "PONG :trittimo")
		print("\tPING/PONG work")

		print("Testing hearbeat -- if we ignore for 10 seconds the stream should be shutdown")
		time.sleep(30)
		try:
			send(self.sockets[0], "LIST")
			resp = recv(self.sockets[1])
		except ConnectionAbortedError:
			print("\tHeartbeats work")

if __name__ == '__main__':
	signal.signal(signal.SIGINT, lambda x,y: sys.exit(0))
	unittest.main()