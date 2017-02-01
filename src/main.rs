use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;

fn handle_command(command: &[u8], ref mut stream: &TcpStream) {
  let command = String::from_utf8_lossy(command);
  if command.starts_with("NICK") {
    println!("recieved NICK command");
    let _ = stream.write(":jeem NICK jeem\r\n".as_bytes());
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

    // match stream.write(&buf) {
    //   Err(_) => break,
    //   Ok(_) => continue,
    // }
  }
}
// use std::io::Read;
// use std::io::Write;

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