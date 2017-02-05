use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::Mutex;

// YO JIM
// CHECK OUT THE ARC BRANCH
// WHY DIDN'T I PUSH TO THIS ONE?
// IDK LOL

#[macro_use] extern crate lazy_static;
lazy_static! {
  static ref GLOBAL_USERS: Mutex<Vec<User>> = Mutex::new(Vec::new());
  static ref GLOBAL_CHANNELS: Mutex<Vec<Channel>> = Mutex::new(Vec::new());
}

struct User {
  username: String,
  hostname: String,
  servername: String,
  realname: String,
}

impl User {
  fn new(name: &str, host: &str, servername: &str, realname: &str) -> 
    User {User {username: String::from(name),
                hostname: String::from(host), 
                servername: String::from(servername),
                realname: String::from(realname)
               }
         }
}

struct Channel {
  channel_name: String,
  topic: String,
  users: Vec<User>,
}

impl Channel {
  fn new(c_name: &str, t_name: &str) -> 
    Channel {Channel {channel_name: String::from(c_name),
                      topic: String::from(t_name),
                      users: Vec::new(),
                    } 
            }
}

fn handle_user(cmd: Vec<&str>, ref mut stream: &TcpStream) {
  println!("recieved USER command");
  let mut guard = GLOBAL_USERS.lock().unwrap();
  guard.push(User::new(cmd[1], cmd[1], cmd[2], cmd[3]));
  let response = format!(":RustIRC Welcome to RustIRC!\r\n");
  let _ = stream.write(response.as_bytes());
}

fn handle_nick(cmd: Vec<&str>, ref mut stream: &TcpStream) {
  println!("recieved NICK command");
  let response = format!(":{0} NICK {0}\r\n", cmd[1]);
  let _ = stream.write(response.as_bytes());
}

fn handle_list(ref mut stream: &TcpStream) {
  println!("recieved LIST command");
  let guard = GLOBAL_CHANNELS.lock().unwrap(); //this line makes it so line 71 does not work
  let mut response : String = "".into();
  for channel in guard.iter() {
    response = response + "#" + channel.channel_name.as_str() + " ";
    let ref users = channel.users;
    for user in users {
      response = response + user.username.as_str() + ","
    }
    response = response + ": " + channel.topic.as_str() + "\r\n";
  }

  let _ = stream.write("alwl".as_bytes());
}

fn handle_command(cmd: &[u8], ref mut stream: &TcpStream) {
  let tmp = String::from_utf8_lossy(cmd);
  let command: Vec<&str> = tmp.split_whitespace().collect();

  match command[0] {
      "NICK" => handle_nick(command, stream),
      "USER" => handle_user(command, stream),
      "LIST" => handle_list(stream),
      _ => println!("unknown command {}", command[0])
  }
  // stream
}

fn handle_client(mut stream: TcpStream) {
  let mut buf;
  loop {
    buf = [0; 512];
    let _ = match stream.read(&mut buf) {
      Err(e) => panic!("Got error: {}", e),
      Ok(m) => {
        if m == 0 {
          println!("User at address {} disconnected\n", stream.peer_addr().unwrap());
          break;
        }
        m
      },
    };

    handle_command(&buf, &stream);
    println!("User at address {} said {}\n", stream.peer_addr().unwrap(), String::from_utf8_lossy(&buf) );
  }
}


// YO JIM
// CHECK OUT THE ARC BRANCH
// WHY DIDN'T I PUSH TO THIS ONE?
// IDK LOL

fn main(){
  let listener = TcpListener::bind("127.0.0.1:6667").unwrap();
  let mut guard = GLOBAL_CHANNELS.lock().unwrap();
  guard.push(Channel::new("memes", "where to find all the dankest memes"));
  guard.push(Channel::new("anim00_garbage", "where all the Otaku talk about anime titties"));

  for stream in listener.incoming() {
    match stream {
      Err(e) => { println!("failed: {}", e)}
      Ok(stream) => {
        thread::spawn(move || {
          handle_client(stream)
        });
      }
    }
  }
}