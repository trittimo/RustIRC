use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::Mutex;

#[macro_use] extern crate lazy_static;
lazy_static! {
  static ref GLOBAL_USERS: Mutex<Vec<User>> = Mutex::new(Vec::new());
}


struct User {
  username: String,
  hostname: String,
  servername: String,
  realname: String,
}

impl User{
  fn new(name: &str, host: &str, servername: &str, realname: &str) -> 
    User {User {username: String::from(name),
      hostname: String::from(host), 
      servername: String::from(servername),
      realname: String::from(realname)} }
  
}

fn handle_nick(cmd: Vec<&str>, ref mut stream: &TcpStream) {
  println!("recieved NICK command");
  let response = format!(":{0} NICK {0}\r\n", cmd[1]);
  let _ = stream.write(response.as_bytes());
}

fn handle_user(cmd: Vec<&str>, ref mut stream: &TcpStream) {
  println!("recieved USER command");
  let mut guard = GLOBAL_USERS.lock().unwrap();
  guard.push(User::new(cmd[1], cmd[1], cmd[2], cmd[3]));
  let response = format!(":RustIRC Welcome to RustIRC!\r\n");
  let _ = stream.write(response.as_bytes());
}

fn handle_command(cmd: &[u8], ref mut stream: &TcpStream) {
  let tmp = String::from_utf8_lossy(cmd);
  let command: Vec<&str> = tmp.split_whitespace().collect();

  match command[0] {
      "NICK" => handle_nick(command, stream),
      "USER" => handle_user(command, stream),
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

fn main(){
  let listener = TcpListener::bind("127.0.0.1:6667").unwrap();
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