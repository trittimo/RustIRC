use std::io::prelude::*;
use std::net::{TcpListener, TcpStream, Shutdown, SocketAddr};
use std::thread;
use std::sync::{Arc, Mutex};

struct User {
  username: String,
  realname: String,
  hostname: String,
  servername: String,
  address: SocketAddr
}

impl User {
  fn new(username: &str,
         realname: &str,
         hostname: &str,
         servername: &str,
         address: SocketAddr) -> User {
    User {
      username: String::from(username),
      realname: String::from(realname),
      hostname: String::from(hostname),
      servername: String::from(servername),
      address: address
    }
  }
}

struct Channel {
  name: String,
  topic: String,
  users: Vec<User>
}

impl Channel {
  fn new(name: &str, topic: &str) -> Channel {
    Channel {
      name: String::from(name),
      topic: String::from(topic),
      users: Vec::new()
    }
  }
}

struct IRCState {
  users: Vec<User>,
  channels: Vec<Channel>,
}

impl IRCState {
  fn new() -> IRCState {
    IRCState {
      users: Vec::new(),
      channels: Vec::new(),
    }
  }
  fn add_channel(&mut self, name: &str, topic: &str) {
    self.channels.push(Channel::new(name, topic));
  }
}

fn setupChannels(irc: &mut IRCState) {
  irc.add_channel("#general", "Anything goes");
  irc.add_channel("#rust", "Complain about rust here");
}

fn listen(irc: &mut IRCState, listener: TcpListener) {
  let state = Arc::new(Mutex::new(irc));
  for stream in listener.incoming() {
    match stream {
      Err(e) => { println!("Error in stream: {}", e)}
      Ok(stream) => {
        let mut st = &state.clone();
        thread::spawn(move || {
          // handle_client(stream, st);
        });
      }
    }
  }
}

pub fn run(address: &str) {
  let irc = &mut IRCState::new();
  let listener = TcpListener::bind(address).unwrap();
  setupChannels(irc);
  listen(irc, listener);
}