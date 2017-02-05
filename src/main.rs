use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct User {
  username: String,
  realname: String,
  hostname: String,
  servername: String,
}

impl User {
  fn new(username: &str, realname: &str,
         hostname: &str, servername: &str) -> User {
    User {
      username: String::from(username),
      realname: String::from(realname),
      hostname: String::from(hostname),
      servername: String::from(servername)
    }
  }
}

#[derive(Debug)]
struct Channel {
  name: String,
  topic: String,
  users: Vec<User>,
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

#[derive(Debug)]
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
  fn add_channel(&mut self, channel: Channel) {
    self.channels.push(channel);
  }
}

fn main() {
  let mut state = IRCState::new();
  let listener = TcpListener::bind("127.0.0.1:6667").unwrap();
  
  // Add our dank channels

  state.add_channel(Channel::new("memes", "Where to find all the dankest memes"));
  state.add_channel(Channel::new("anim00_garbage", "where all the Otaku talk about anime [censored]"));

  // Declare our shared state for threaded use
  let shared_state = Arc::new(Mutex::new(state));

  for stream in listener.incoming() {
    match stream {
      Err(e) => { println!("Error in stream: {}", e)}
      Ok(stream) => {
        let st = shared_state.clone();
        thread::spawn(move || {
          handle_client(stream, st);
        });
      }
    }
  }
}

fn handle_user(cmd: Vec<&str>, ref mut stream: &TcpStream, state: &Arc<Mutex<IRCState>>) {
  println!("recieved USER command");
  let ref mut users = state.lock().unwrap().users;
  users.push(User::new(cmd[1], cmd[1], cmd[2], cmd[3]));
  let mut response = format!("PING :3813401942\r\n");
  let _ = stream.write(response.as_bytes());

  response = String::from(":localhost 001 jeem :Welcome to RustIRC!\r\n");

  let _ = stream.write(response.as_bytes());
}

fn handle_nick(cmd: Vec<&str>, ref mut stream: &TcpStream, state: &Arc<Mutex<IRCState>>) {
  println!("recieved NICK command");
  let response = format!(":{0} NICK {0}\r\n", cmd[1]);
  let _ = stream.write(response.as_bytes());
}

fn handle_list(ref mut stream: &TcpStream, state: &Arc<Mutex<IRCState>>) {
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

fn handle_join(cmd: Vec<&str>, ref mut stream: &TcpStream, state: &Arc<Mutex<IRCState>>) {
  println!("recieved JOIN command");
  let mut response = ":jeem JOIN #memes";
  let _ = stream.write(response.as_bytes());
}

fn handle_ping(cmd: Vec<&str>, ref mut stream: &TcpStream) {
  let response : String = String::from("PONG :") + cmd[1];
  let _ = stream.write(response.as_bytes()); 
}

fn handle_cap(ref mut stream: &TcpStream) {
  let response = "CAP * LS :multi-prefix sasl=EXTERNAL";
  let _ = stream.write(response.as_bytes());
}

fn handle_command(cmd: &[u8], ref mut stream: &TcpStream, state: &Arc<Mutex<IRCState>>) {
  let tmp = String::from_utf8_lossy(cmd);
  let command: Vec<&str> = tmp.split_whitespace().collect();

  match command[0] {
      "NICK" => handle_nick(command, stream, &state),
      "USER" => handle_user(command, stream, &state),
      "LIST" => handle_list(stream, &state),
      "JOIN" => handle_join(command, stream, &state),
      "PING" => handle_ping(command, stream),
      "CAP" => handle_cap(stream),
      _ => println!("unknown command {}", command[0])
  }
  // stream 
}

fn handle_client(mut stream: TcpStream, state: Arc<Mutex<IRCState>>) {
  let mut buf;
  loop {
    buf = [0; 512];
    let _ = match stream.read(&mut buf) {
      Err(e) => panic!("Error handling client: {}", e),
      Ok(m) => {
        if m == 0 {
          // TODO remove user from channel
          println!("User at address {} disconnected\n", stream.peer_addr().unwrap());
          break;
        }
      },
    };

    handle_command(&buf, &stream, &state);
    // println!("\t{:?}\n\t", state.lock().unwrap().channels);
    println!("User at address {} said {}\n", stream.peer_addr().unwrap(), String::from_utf8_lossy(&buf));
  }
}