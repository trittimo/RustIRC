use std::io::prelude::*;
use std::net::{TcpListener, TcpStream, Shutdown, SocketAddr};
use std::{thread, time};
use std::sync::{Arc, Mutex};
#[macro_use]
extern crate lazy_static;


lazy_static!{
  static ref USERS: Arc<Mutex<Vec<User>>>  = Arc::new(Mutex::new(Vec::new()));
  static ref CHANNELS: Arc<Mutex<Vec<Channel>>> = Arc::new(Mutex::new(Vec::new()));
}

#[derive(Clone)]
struct User {
  username: String,
  realname: String,
  hostname: String,
  servername: String,
  address: SocketAddr,
  last_pong: i32
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
      address: address,
      last_pong: 0
    }
  }
  fn refresh(&mut self) {
    self.last_pong = 0;
  }
  fn increment(&mut self) {
    self.last_pong += 1;
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

fn handle_user(cmd: Vec<&str>, mut stream: &TcpStream) {
  println!("recieved USER command");
  let ref mut users = USERS.lock().unwrap();
  users.push(User::new(cmd[1], cmd[1], cmd[2], cmd[3], stream.peer_addr().unwrap()));
  let mut response = format!("PING :3813401942\r\n");
  let _ = stream.write(response.as_bytes());

  response = String::from(":localhost 001 jeem :Welcome to RustIRC!\r\n");

  let _ = stream.write(response.as_bytes());
}

fn handle_nick(cmd: Vec<&str>, mut stream: &TcpStream) {
  println!("recieved NICK command");
  let response = format!(":{0} NICK {0}\r\n", cmd[1]);
  let _ = stream.write(response.as_bytes());
}

fn handle_list(mut stream: &TcpStream) {
  println!("recieved LIST command");
  let ref mut channels = CHANNELS.lock().unwrap();
  let mut response : String = String::from(":localhost 321 jeem Channel :Users  Name\r\n");
  for channel in channels.iter() {
    response = response + format!(":localhost 322 jeem {0} {1} :{2}\r\n", 
                                    channel.name.as_str(), 
                                    channel.users.len(), 
                                    channel.topic.as_str()).as_str();
  }
  response = response + ":localhost 323 jeem :End of /LIST\r\n";

  let _ = stream.write(response.as_bytes());
}

fn handle_join(cmd: Vec<&str>, mut stream: &TcpStream) {
  println!("recieved JOIN command");

  let ref mut channels = CHANNELS.lock().unwrap();
  let ref users = USERS.lock().unwrap();

  let addr = stream.peer_addr().unwrap();
  let mut user: Option<&User> = None;
  for x in users.iter() {
    if x.address.ip() == addr.ip() && x.address.port() == addr.port() {
      user = Some(x);
    }
  }
  let mut channel: Option<&mut Channel> = None;
  for x in channels.iter_mut() {
    if x.name == cmd[1] {
      channel = Some(x);
    }
  }

  match (user, channel) {
    (Some(u), Some(c)) => {
      let response = format!(":localhost 332 {0} {1} {2}\r\n",
                              u.username, cmd[1], c.topic);
      let _ = stream.write(response.as_bytes());
      c.users.push(u.clone());
      let current_users = c.users.iter().fold("".to_string(), |acc, x| {
        x.username.clone() + " " + &acc
      });
      let _ = stream.write(current_users.as_bytes());
    },
    (Some(u), _) => {
      // No such channel exists
      // TODO
    },
    _ => {
      // Unknown state
      // TODO
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

fn handle_quit(stream: &TcpStream) {
  let addr = stream.peer_addr().unwrap();

  let mut users = USERS.lock().unwrap();
  for i in 0..users.len() {
    let user = users[i].clone();
    if user.address.ip() == addr.ip() && user.address.port() == addr.port() {
      users.remove(i);
      
    }
  }
  let _ = stream.shutdown(Shutdown::Both);
  println!("User at address {} disconnected\n", stream.peer_addr().unwrap());
  //TODO: remove user from their channels
}

fn handle_pong() {
  println!("received PONG");
}

// returns a copy of the user if it does exist
fn addr_to_user(stream: &TcpStream) -> Option<User> {
  let users = USERS.lock().unwrap();
  let addr = stream.peer_addr().unwrap();
  for x in users.iter() {
    if x.address.ip() == addr.ip() && x.address.port() == addr.port() {
      return Some(x.clone());
    } else {
      println!("ip {:?} != {:?}\naddr {:?} !+ {:?}\n", x.address.ip(), addr.ip(), x.address.port(), addr.port());
    }
  }
  None
}

fn handle_privmsg(cmd: Vec<&str>, mut stream: &TcpStream) {
  println!("recieved PRIVMSG\n");
  let channel_name = cmd[1];
  let channels =  CHANNELS.lock().unwrap();

  for c in channels.iter() {
    if c.name == channel_name {
      let user = addr_to_user(stream);
      match user {
        Some(u) => {
          let response = String::from(format!(":{0} PRIVMSG {1} {2}", u.username, channel_name, cmd[3]));
          let _ = stream.write(response.as_bytes());
        },
        None => {
          println!("who dafaq is dis?");
        }
      }

    }
  }

  println!("broadcasting to {}", channel_name);
}

fn refresh(stream: &TcpStream) {
  let addr = stream.peer_addr().unwrap();
  let mut users = USERS.lock().unwrap();
  for user in users.iter_mut() {
    if user.address.ip() == addr.ip() && user.address.port() == addr.port() {
      user.refresh();
    }
  }
}

fn handle_command(cmd: &[u8], stream: &TcpStream) {
  let tmp = String::from_utf8_lossy(cmd);
  let command: Vec<&str> = tmp.split_whitespace().collect();
  refresh(stream);
  match command[0] {
      "NICK" => handle_nick(command, stream),
      "USER" => handle_user(command, stream),
      "LIST" => handle_list(stream),
      "JOIN" => handle_join(command, stream),
      "PING" => handle_ping(command, stream),
      "PONG" => handle_pong(),
      "CAP" => handle_cap(stream),
      "PRIVMSG" => handle_privmsg(command, stream),
      "QUIT" => handle_quit(stream),
      _ => println!("unknown command {}", command[0])
  }
}

fn increment(stream: &TcpStream) {
  let addr = stream.peer_addr().unwrap();
  let mut users = USERS.lock().unwrap();
  for user in users.iter_mut() {
    if user.address.ip() == addr.ip() && user.address.port() == addr.port() {
      user.increment();
    }
  }
}

fn handle_client(mut stream: TcpStream) {
  let mut clone = stream.try_clone().unwrap();
  
  thread::spawn(move || {
    let to_sleep = time::Duration::from_secs(5);
    thread::sleep(time::Duration::from_secs(10));
    loop {
      println!("PINGING");
      thread::sleep(to_sleep);
      let _ = clone.write("PING\r\n".as_bytes());
      increment(&clone);
      match addr_to_user(&clone) {
        Some(user) => {
          println!("{:?}", user.last_pong);
          if user.last_pong > 5 {
            // println!("Disconnecting user because they didn't respond");
            handle_quit(&clone);
            return;
          }
        },
        _ => {
          return;
        }
      }
    }
  });

  let mut buf;
  loop {
    buf = [0; 512];
    match stream.read(&mut buf) {
      Err(e) => panic!("Error handling client: {}", e),
      Ok(m) => {
        if m == 0 {
          handle_quit(&stream);
          break;
        }
      }
    }

    handle_command(&buf, &stream);
  }
}

fn main() {
  // let state = Arc::new(Mutex::new(IRCState::new()));
  let  listener = TcpListener::bind("127.0.0.1:6667").unwrap();
  {
    let mut channels = CHANNELS.lock().unwrap();

    channels.push(Channel::new("#general", "Anything goes"));
    channels.push(Channel::new("#rust", "Complain about rust here"));
  }

  for stream in listener.incoming() {
    match stream {
      Err(e) => { println!("Error in stream: {}", e)}
      Ok(stream) => {
        // let st = irc.clone();
        thread::spawn(move || {
          handle_client(stream);
        });
      }
    }
  }
}