use std::io::prelude::*;
use std::net::{TcpListener, TcpStream, Shutdown, SocketAddr};
use std::{thread, time};
use std::sync::{Arc, Mutex};
#[macro_use]
extern crate lazy_static;


lazy_static!{
  static ref USERS: Arc<Mutex<Vec<User>>>  = Arc::new(Mutex::new(Vec::new()));
  static ref CHANNELS: Arc<Mutex<Vec<Channel>>> = Arc::new(Mutex::new(Vec::new()));
  static ref USER_STREAMS: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));
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
  users: Vec<User>,
  user_streams: Vec<TcpStream>
}

impl Channel {
  fn new(name: &str, topic: &str) -> Channel {
    Channel {
      name: String::from(name),
      topic: String::from(topic),
      users: Vec::new(),
      user_streams: Vec::new()
    }
  }

  fn remove_user(&mut self, name: String) {
      let ref mut users = self.users;
      let ref mut streams = self.user_streams;
      println!("removing {}", name);

      for i in 0..users.len() {
        if users[i].username == name {
          for j in 0..streams.len() {
            if users[i].address.ip() == streams[j].peer_addr().unwrap().ip() && 
               users[i].address.port() == streams[j].peer_addr().unwrap().port() {
              users.remove(i);
              streams.remove(j);
              return;
           }
        }
      }
    }
  }

  fn get_streams(&mut self) -> &mut Vec<TcpStream>{
    return &mut self.user_streams;
  }
}

fn handle_user(cmd: Vec<&str>, mut stream: &TcpStream) {
  println!("recieved USER command");
  let client = addr_to_user(stream).unwrap();

  {
    let mut streams = USER_STREAMS.lock().unwrap();
    streams.push(stream.try_clone().unwrap());
  }

  let ref mut users = USERS.lock().unwrap();
  for user in users.iter_mut() {
    if user.address == client.address {

      user.realname = String::from(cmd[1]);
      user.hostname = String::from(cmd[2]);
      user.servername = String::from(cmd[3]);
    }
  }

  // users.push(User::new(cmd[1], cmd[1], cmd[2], cmd[3], stream.peer_addr().unwrap()));
  let mut response = format!("PING :3813401942\r\n");
  let _ = stream.write(response.as_bytes());

  response = String::from(format!(":localhost 001 {} :Welcome to RustIRC!\r\n", cmd[1]));

  let _ = stream.write(response.as_bytes());
}

fn handle_nick(cmd: Vec<&str>, mut stream: &TcpStream) {
  println!("recieved NICK command");

  let client = addr_to_user(stream);
  let mut users = USERS.lock().unwrap();

  match client {
      Some(c) => {
        for user in users.iter_mut() {
          if user.address == c.address {
            user.username = String::from(cmd[1]);
          }
        }
      },
      None =>{
        users.push(User::new(cmd[1], "", "", "", stream.peer_addr().unwrap()));
      },
  }

  

  let response = format!(":{0} NICK {0}\r\n", cmd[1]);
  let _ = stream.write(response.as_bytes());
}

fn handle_list(mut stream: &TcpStream) {
  println!("recieved LIST command");
  let ref mut channels = CHANNELS.lock().unwrap();
  let mut response : String = String::from(":localhost 321 RustIRC Channel :Users  Name\r\n");
  for channel in channels.iter() {
    response = response + format!(":localhost 322 RustIRC {0} {1} :{2}\r\n", 
                                    channel.name.as_str(), 
                                    channel.users.len(), 
                                    channel.topic.as_str()).as_str();
  }
  response = response + ":localhost 323 RustIRC :End of /LIST\r\n";

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
      let response = format!(":localhost 332 {0} {1} :{2}\r\n",
                              u.username, cmd[1], c.topic.as_str());
      let _ = stream.write(response.as_bytes());
      c.users.push(u.clone());
      c.user_streams.push(stream.try_clone().unwrap());
      let current_users = c.users.iter().fold("".to_string(), |acc, x| {
        x.username.clone() + " " + &acc
      });
      let response = current_users + "\r\n";
      let _ = stream.write(response.as_bytes());
    },
    (Some(_), _) => {
      // No such channel exists
      // TODO
    },
    _ => {
      // Unknown state
      // TODO
    }
  }
}

fn handle_part(cmd: Vec<&str>, mut stream: &TcpStream) {
  let mut channels = CHANNELS.lock().unwrap();

  for channel in channels.iter_mut() {
    if channel.name == cmd[1] {
      let ref client = addr_to_user(stream).unwrap();
      channel.remove_user(String::from(client.clone().username));
      println!("removing user");
    }
  }
  let client = addr_to_user(stream).unwrap();
  let mut msg = String::from(cmd[2]);
  for i in 3..cmd.len(){
    msg += " ";
    msg += cmd[i];
  }
  let response = format!(":{0} PART {1} {2}\r\n", client.username, cmd[1], msg);
  let _ = stream.write(response.as_bytes());
}

fn handle_ping(cmd: Vec<&str>, mut stream: &TcpStream) {
  let response : String = format!("PONG :{}\r\n", cmd[1]);
  let _ = stream.write(response.as_bytes()); 
}

fn handle_cap(mut stream: &TcpStream) {
  let response = "CAP * LS :multi-prefix sasl=EXTERNAL\r\n";
  let _ = stream.write(response.as_bytes());
}

fn handle_quit(stream: &TcpStream) {
  let addr = stream.peer_addr().unwrap();

  let mut channels = CHANNELS.lock().unwrap();
  let mut users = USERS.lock().unwrap();
  for i in 0..users.len() {
    let user = users[i].clone();
    if user.address.ip() == addr.ip() && user.address.port() == addr.port() {
      for j in 0..channels.len(){
        for k in 0..channels[j].users.len(){
          if channels[j].users[k].username == user.username {
            channels[j].users.remove(k);
            break;
          }
        }
      }
      users.remove(i);
      break;
    }
  }
  let mut streams = USER_STREAMS.lock().unwrap();
  for i in 0..streams.len() {
    match streams[i].peer_addr() {
      Err(e) => return,
      _ => {},
    }
    if streams[i].peer_addr().unwrap().ip() == addr.ip() && streams[i].peer_addr().unwrap().port() == addr.port() {
      streams.remove(i);
      break;
    }
  }
  println!("User at address {} disconnected", stream.peer_addr().unwrap());
  let _ = stream.shutdown(Shutdown::Both);



  //TODO: remove user from their channels
}

fn handle_pong() {
  println!("received PONG");
}

// returns a copy of the user if it does exist
fn addr_to_user(stream: &TcpStream) -> Option<User> {
  match stream.peer_addr() {
    Err(_) => return None,
    _ => {}
  } 

  let users = USERS.lock().unwrap();
  let addr = stream.peer_addr().unwrap();
  for x in users.iter() {
    if x.address.ip() == addr.ip() && x.address.port() == addr.port() {
      return Some(x.clone());
    } 
  }
  None
}

fn handle_privmsg(cmd: Vec<&str>, stream: &TcpStream) {
  println!("recieved PRIVMSG\n");
  let channel_name = cmd[1];

  if channel_name.as_bytes()[0] != "#".as_bytes()[0] {
    println!("pm to a person");
    let sender_name = addr_to_user(stream).unwrap().username;
    let mut msg = String::from(cmd[2]);
    for i in 3..cmd.len() {
      msg += " ";
      msg += cmd[i];
    }

    let response = format!(":{0} PRIVMSG {1} {2}\r\n", sender_name, cmd[1], msg);

    let mut streams = USER_STREAMS.lock().unwrap();
    let users = USERS.lock().unwrap();

    for user in users.iter() {
      if user.username == channel_name {
        for mut strm in streams.iter_mut() {
          if strm.peer_addr().unwrap().ip() == user.address.ip() && 
             strm.peer_addr().unwrap().port() == user.address.port() {
              let _ = strm.write(response.as_bytes());
              return;
           }
        }
      }
    }

    return;
  }


  let mut channels =  CHANNELS.lock().unwrap();

  let user = addr_to_user(stream).unwrap();

  for c in channels.iter_mut() {
    if c.name == channel_name {
      println!("broadcasting to {}", channel_name);
      for mut ustream in c.get_streams() {
        if ustream.peer_addr().unwrap().ip() != stream.peer_addr().unwrap().ip() || 
           ustream.peer_addr().unwrap().port() != stream.peer_addr().unwrap().port() {

          let mut msg = String::from(cmd[2]);
          for i in 3..cmd.len() {
            msg += " ";
            msg += cmd[i];
          }
          let response = String::from(format!(":{0} PRIVMSG {1} {2}\r\n", user.username, channel_name, msg));
          let _ = ustream.write(response.as_bytes());
        }
      }
    }
  }
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
      "PART" => handle_part(command, stream),
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
    let to_sleep = time::Duration::from_secs(10);
    // thread::sleep(time::Duration::from_secs(10));
    loop {
      thread::sleep(to_sleep);
      match addr_to_user(&clone) {
        Some(user) => {
          println!("{:?}", user.last_pong);
          println!("PINGING");
          let _ = clone.write("PING\r\n".as_bytes());
          increment(&clone);
          if user.last_pong > 2 {

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
      Err(e) => { println!("Error handling client: {}", e); return; },
      Ok(m) => {
        if m == 0 {
          handle_quit(&stream);
          return;
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