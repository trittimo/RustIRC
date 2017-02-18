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

struct Channel<'a> {
  name: String,
  topic: String,
  users: Vec<&'a User>
}

impl<'a> Channel<'a> {
  fn new(name: &str, topic: &str) -> Channel<'a> {
    Channel {
      name: String::from(name),
      topic: String::from(topic),
      users: Vec::new()
    }
  }
}

struct IRCState<'a> {
  users: Vec<User>,
  channels: Vec<Channel<'a>>,
}

impl<'a> IRCState<'a> {
  fn new() -> IRCState<'a> {
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


fn handle_user(cmd: Vec<&str>, mut stream: &TcpStream, state: &Arc<Mutex<IRCState>>) {
  println!("recieved USER command");
  let ref mut users = state.lock().unwrap().users;
  users.push(User::new(cmd[1], cmd[1], cmd[2], cmd[3], stream.peer_addr().unwrap()));
  let mut response = format!("PING :3813401942\r\n");
  let _ = stream.write(response.as_bytes());

  response = String::from(":localhost 001 jeem :Welcome to RustIRC!\r\n");

  let _ = stream.write(response.as_bytes());
}

fn handle_nick(cmd: Vec<&str>, mut stream: &TcpStream, state: &Arc<Mutex<IRCState>>) {
  println!("recieved NICK command");
  let response = format!(":{0} NICK {0}\r\n", cmd[1]);
  let _ = stream.write(response.as_bytes());
}

fn handle_list(mut stream: &TcpStream, state: &Arc<Mutex<IRCState>>) {
  println!("recieved LIST command");
  let ref mut channels = state.lock().unwrap().channels;
  let mut response : String = ":localhost 321 jeem Channel :Users  Name\r\n".into();
  for channel in channels.iter() {
    response = response + format!(":localhost 322 jeem #{0} {1} :{2}\r\n", 
                                    channel.name.as_str(), 
                                    channel.users.len(), 
                                    channel.topic.as_str()).as_str();
  }
  response = response + ":localhost 323 jeem :End of /LIST\r\n";
  let _ = stream.write(response.as_bytes());
}

fn handle_join(cmd: Vec<&str>, mut stream: &TcpStream, state: &Arc<Mutex<IRCState>>) {
  println!("recieved JOIN command");
  let users = &mut state.lock().unwrap().users;
  let channels = &mut state.lock().unwrap().channels;
  let addr = stream.peer_addr().unwrap();
  let mut user: Option<&mut User> = None;
  for x in users {
    if x.address.ip() == addr.ip() && x.address.port() == addr.port() {
      user = Some(x);
    }
  }
  let mut channel: Option<&mut Channel> = None;
  for x in channels {
    if x.name == cmd[1] {
      channel = Some(x);
    }
  }

  match user {
    Some(u) => {
      match channel {
        Some(c) => {
          let mut response = format!(":localhost 332 {0} {1} {2}\r\n",
                              u.username, cmd[1], c.topic);
          let _ = stream.write(response.as_bytes());
          c.users.push(u);
          let current_users = c.users.iter().fold("".to_string(), |acc, x| {
            x.username + " " + &acc
          });
        }
        None => {
          // That particular channel doesn't exist: inform the user of that
          // TODO
          return;
        }
      }
    },
    None => {
      // The user hasn't connected? This is a strange case
      return;
    }
  }
}

fn handle_ping(cmd: Vec<&str>, mut stream: &TcpStream) {
  let response : String = String::from("PONG :") + cmd[1];
  let _ = stream.write(response.as_bytes()); 
}

fn handle_cap(mut stream: &TcpStream) {
  let response = "CAP * LS :multi-prefix sasl=EXTERNAL";
  let _ = stream.write(response.as_bytes());
}

fn handle_quit(cmd: Vec<&str>, stream: &TcpStream) {
  stream.shutdown(Shutdown::Both);
  //TODO: add logic to remove that user
}

fn handle_command(cmd: &[u8], mut stream: &TcpStream, state: &Arc<Mutex<IRCState>>) {
  let tmp = String::from_utf8_lossy(cmd);
  let command: Vec<&str> = tmp.split_whitespace().collect();

  match command[0] {
      "NICK" => handle_nick(command, stream, state),
      "USER" => handle_user(command, stream, state),
      "LIST" => handle_list(stream, state),
      "JOIN" => handle_join(command, stream, state),
      "PING" => handle_ping(command, stream),
      "CAP" => handle_cap(stream),
      "QUIT" => handle_quit(command, stream),
      _ => println!("unknown command {}", command[0])
  }
}

fn handle_client(mut stream: TcpStream, state: Arc<Mutex<IRCState>>) {
  let mut buf;
  loop {
    buf = [0; 512];
    match stream.read(&mut buf) {
      Err(e) => panic!("Error handling client: {}", e),
      Ok(m) => {
        if m == 0 {
          // TODO remove user from channel
          println!("User at address {} disconnected\n", stream.peer_addr().unwrap());
          break;
        }
      }
    }

    handle_command(&buf, &stream, &state);
  }
}

fn listen(irc: IRCState, listener: TcpListener) {

  let state = Arc::new(Mutex::new(irc));
  for stream in listener.incoming() {
    match stream {
      Err(e) => { println!("Error in stream: {}", e)}
      Ok(stream) => {
        let st = state.clone();
        thread::spawn(move || {
          handle_client(stream, st);
        });
      }
    }
  }
}

fn main() {
  let mut irc = IRCState::new();
  let listener = TcpListener::bind("127.0.0.1:6667").unwrap();
  setupChannels(&mut irc);
  listen(irc, listener);
}