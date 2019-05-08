extern crate chrono;

use chrono::{DateTime, Utc};
use std::io;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::time::SystemTime;

fn handle_client(stream: &mut TcpStream) {
    let greeting = "hello, you made it!\n".as_bytes();
    stream.write(greeting);
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080")?;
    for stream in listener.incoming() {
        handle_client(&mut stream?);
    }
    Ok(())
}
