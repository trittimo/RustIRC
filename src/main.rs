use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;

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

    println!("User at address {} said {}\n", stream.peer_addr().unwrap(), String::from_utf8_lossy(&buf) );
    match stream.write(&buf) {
      Err(_) => break,
      Ok(_) => continue,
    }
  }
}
// use std::io::Read;
// use std::io::Write;

fn main(){
  let listener = TcpListener::bind("127.0.0.1:8888").unwrap();
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