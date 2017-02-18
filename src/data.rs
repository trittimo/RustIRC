use std::io::prelude::*;
use std::net::{TcpListener, TcpStream, Shutdown, SocketAddr};
use std::thread;
use std::sync::{Arc, Mutex};

pub struct User {
  username: String,
  realname: String,
  hostname: String,
  servername: String,
  address: SocketAddr
}

pub struct Channel {
  name: String,
  description: String
}

pub struct IRCData {
  channels: Vec<Channel>,
  users: Vec<User>
}

impl IRCData {
  pub fn new() -> IRCData {
    IRCData {
      channels: Vec::new(),
      users: Vec::new()
    }
  }
}