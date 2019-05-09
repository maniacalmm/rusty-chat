extern crate chrono;
extern crate bytes;

use chrono::{DateTime, Utc};
use std::io;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::time::SystemTime;
use std::env;
use std::vec::*;
use bytes::{BytesMut, BufMut};
use std::ops::Add;
use std::thread;

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use std::sync::mpsc::{self, Receiver, Sender};

type RX = Receiver<String>;
type TX = Sender<String>;

static STR_DELIMITER: &'static str = "\r\n";

struct SharedChat {
    // all other people in this shared chat
    peers: HashMap<SocketAddr, TX>,
}

impl SharedChat {
    pub fn new() -> Self {
        SharedChat {
            peers: HashMap::new(),
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut buf: [u8; 1024] = [0; 1024];
    println!("incoming: {:?}", &stream);
    loop {
        let read_size = stream.read(&mut buf).unwrap();
        println!("read_size: {}", read_size);
        let mut buf_vec: Vec<u8> = buf.to_vec();
        buf_vec.drain(read_size..buf.len());

        let whole_content: String = String::from_utf8(buf_vec).unwrap();
        let idx = whole_content.find(STR_DELIMITER).unwrap();
        let whole_content_wo_delimiter = whole_content[..idx].to_owned();

        match &whole_content_wo_delimiter[..]  {
            ":q" => break,
            _    => {
                let output = whole_content_wo_delimiter.add(" ((right back at ya\n");
                stream.write(output.as_bytes());
            }
        }

    }
}

// to represent a connected client
struct Client {
    name: String,
    chat: String,
    shared_chat: Arc<Mutex<SharedChat>>,
    rx: RX,
    addr: SocketAddr,
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let port = args.get(0).unwrap();

    println!("starting server on {}", port);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;
    for stream in listener.incoming() {
        thread::spawn(move || {
            handle_client(stream.unwrap());
        });
    }
    Ok(())
}
