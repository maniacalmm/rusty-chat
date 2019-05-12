extern crate bytes;
extern crate chrono;

use bytes::{BufMut, BytesMut};
use chrono::{DateTime, Utc};
use std::env;
use std::io;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::ops::Add;
use std::thread;
use std::time::SystemTime;
use std::vec::*;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use std::sync::mpsc::{self, Receiver, Sender};

/// basic idea:
/// three main construct:
/// 1, chat_room -> stores all the user, and the tx channel to them
/// 2, client -> represent each connection, contains a tx channel to chat_room
/// 3, message -> indicate who sends the message, and the content of message
///
/// basic work flow:
/// -> user make the connection
/// -> we add this user to chat_room
/// -> user contain a tx to the chat_room
/// -> each user has two thread for detecting reading(it's blocking, that why we need a different thread) and writing.
/// -> each chat_room is a separate thread for detecting incoming message and broadcasting it to all the user
///
/// chat_room is a Arc<Mutex<ChatRoom>> since there are multiple thread writing and reading to it, which is not performant

type RX = Receiver<Message>;
type TX = Sender<Message>;

static STR_DELIMITER: &'static str = "\r\n";

#[derive(Clone, Debug)]
struct Message {
    from: SocketAddr,
    message: String,
}

impl Message {
    pub fn new(from: SocketAddr, message: String) -> Self {
        Message { from, message }
    }
}

struct ChatRoom {
    // all other people in this shared chat, with their addr, tx is so chat room can talk to each of them
    peers: HashMap<SocketAddr, TX>,
    // inbox of all connected client
    inbox: RX,
}

impl ChatRoom {
    pub fn new(inbox: RX) -> Self {
        ChatRoom {
            peers: HashMap::new(),
            inbox,
        }
    }

    pub fn add_client(&mut self, addr: SocketAddr, tx: TX) {
        self.peers.insert(addr, tx);
    }

    // receive message and broadcast to all peers
    fn broadcast(&mut self) {
        if let Ok(message) = self.inbox.try_recv() {
            for (addr, tx) in &self.peers {
                println!("broadcast message: {:?}, to: {}", &message, addr);
                if (&message.from).ne(addr) {
                    tx.send(message.clone());
                }
            }
        }
    }
}

fn handle_client(mut client: Client, inbox: Receiver<Message>) {
    let mut buf: [u8; 1024] = [0; 1024];
    println!("incoming: {:?}", &client.stream);

    let mut inbox_stream = client.stream.try_clone().unwrap();
    thread::spawn(move || {
        loop {
            println!("inbox try recv");
            if let Ok(message) = inbox.recv() {
//                println!("client inbox received: {:?}", message);
                inbox_stream.write(message.message.as_bytes());
            }
        }
    });

    loop {
        println!("about to read...");
        let read_size = client.stream.read(&mut buf).unwrap();
        let mut buf_vec: Vec<u8> = buf.to_vec();
        buf_vec.drain(read_size..buf.len());

        let whole_content: String = String::from_utf8(buf_vec).unwrap();
        let idx = whole_content.find(STR_DELIMITER).unwrap();
        let whole_content_wo_delimiter = whole_content[..idx].to_owned().add("\n");

        match &whole_content_wo_delimiter[..] {
            ":q" => break,
            _ => {
                // write content to chatting room first
                // lock here is inevitable, since every client need to modify the channel somehow, so
                println!(" ---- send data to chat room ----");
                let message = Message::new(client.addr, whole_content_wo_delimiter);
                client.send_to_chat_room(message);
            }
        }
    }
}

// to represent a connected client
struct Client {
    to_chat_room: TX,
//    inbox: RX,
    stream: TcpStream,
    addr: SocketAddr,
}

impl Client {
    pub fn new(to_chat_room: TX, stream: TcpStream) -> Self {
        let addr = stream.peer_addr().unwrap();
        Client {
            to_chat_room,
//            inbox,
            stream,
            addr,
        }
    }

    pub fn send_to_chat_room(&mut self, message: Message) {
        self.to_chat_room.send(message);
    }
}

// for chat room to receive messages
fn chat_room_broadcasting(chat_room: Arc<Mutex<ChatRoom>>) {
    thread::spawn(move || {
        // every broadcast also need to lock the chat_room?
        // is this optimum
        loop {
            chat_room.lock().unwrap().broadcast();
        }
    });
}

// the thing is that, this entire server revolves on shared struct, which is chat_room
fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let port = args.get(0).unwrap();
    // this represents the only chat room we have now
    let (tx, rx): (Sender<Message>, Receiver<Message>) = mpsc::channel();
    let chat_room = Arc::new(Mutex::new(ChatRoom::new(rx)));
    let chat_room_broadcast_ref = Arc::clone(&chat_room);

    thread::spawn(move || {
        chat_room_broadcasting(chat_room_broadcast_ref);
    });

    println!("starting server on {}", port);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;
    for stream in listener.incoming() {
        let (tx_private, rx_private): (Sender<Message>, Receiver<Message>) = mpsc::channel();

        let chat_room_ref: Arc<Mutex<ChatRoom>> = Arc::clone(&chat_room);
        let client = Client::new(tx.clone(), stream.unwrap());
        // add client to chat_room
        chat_room_ref
            .lock()
            .unwrap()
            .add_client(client.addr, tx_private);
        thread::spawn(move || {
            handle_client(client, rx_private);
        });
        println!("spawned new thread for new client");
    }
    Ok(())
}

