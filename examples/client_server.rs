use std::{net::{TcpStream, TcpListener}, mem::MaybeUninit, thread};
use std::io::Write;
use env_logger::Builder;

use simple_parse::{SpRead, SpWrite};

#[derive(SpRead, SpWrite)]
pub enum Message {
    Ping,
    Pong,
    Chat(String),
    Key {
        private: Vec<u8>,
        public: Vec<u8>,
    },
    Disconnect,
}

pub fn main() {
    let mut builder = Builder::from_default_env();
    builder
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .init();
        
    let listener = TcpListener::bind("127.0.0.1:0").expect("[server] Failed to bind");
    let port = listener.local_addr().unwrap().port();

    // Spawn a "client"
    let t1 = thread::spawn(move || client_thread(port));

    let mut dst = MaybeUninit::uninit();
    let mut sock = listener.accept().expect("[server] Failed to accept connection").0;

    loop {
        // Receive & parse bytes from the socket as a `Message` using SpRead
        let msg = Message::from_reader(&mut sock, &mut dst).expect("[server] Failed to receive message");

        match msg {
            Message::Ping => {
                println!("[server] Got Ping ! Sending Pong...");
                // Respond with a Pong using SpWrite
                (Message::Pong).to_writer(&mut sock).expect("[server] Failed to send Pong");
            },
            Message::Pong => println!("[server] got pong !"),
            Message::Chat(s) => println!("[server] Received chat : '{s}'"),
            Message::Key{private, public} => println!("[server] got keys : {private:X?}:{public:X?}"),
            Message::Disconnect => break,
        }
    }

    t1.join().unwrap();
}

fn client_thread(port: u16) {
    let mut sock = TcpStream::connect(&format!("127.0.0.1:{port}")).expect("Failed to connect");

    println!("[client] Sending Ping !");
    Message::Ping.to_writer(&mut sock).expect("[client] Failed to send Ping");
    
    // Wait for the pong
    let mut dst = MaybeUninit::uninit();
    let msg = Message::from_reader(&mut sock, &mut dst).expect("[client] Failed to recv Pong");
    if !matches!(msg, Message::Pong) {
        panic!("[client] Server did not send a pong ?!");
    }
    println!("[client] Got Pong !");
    
    // Send a chat !
    let msg = Message::Chat("Hello from client !!".to_string());
    msg.to_writer(&mut sock).expect("[client] Failed to send Chat");

    // Disconnect
    Message::Disconnect.to_writer(&mut sock).expect("[client] Failed to send Disconnect");
}