use std::io::{Read, Write};
use std::net;
use std::sync::mpsc::{self, TryRecvError};

use clap::Parser;

mod data;
mod input_capture;
mod input_injection;

#[derive(Parser, Debug)]
enum Args {
    Client { address: net::Ipv4Addr, port: u16 },
    Server { port: u16 },
}

fn main() {
    let args = Args::parse();

    match args {
        Args::Client { address, port } => run_client(address, port),
        Args::Server { port } => run_server(port),
    }
}

fn run_client(address: net::Ipv4Addr, port: u16) {
    let mut injector = input_injection::InputInjector::new();
    let mut stream = net::TcpStream::connect((address, port)).unwrap();

    loop {
        let mut buffer = [0; 4];
        stream.read_exact(&mut buffer).unwrap();

        let event = data::KeyEvent::from_bytes(buffer);
        injector.emit(event);
    }
}

fn run_server(port: u16) {
    let (sender, receiver) = mpsc::channel::<data::KeyEvent>();

    input_capture::init(sender);

    let listener = net::TcpListener::bind(("0.0.0.0", port)).unwrap();
    println!("Waiting for client...");
    let (mut client, addr) = listener.accept().unwrap();
    println!("client connected from {}", addr);

    // clear anything that was buffered before a client connected
    let try_recv_error = loop {
        match receiver.try_recv() {
            Ok(_) => {}
            Err(e) => break e,
        }
    };
    match try_recv_error {
        TryRecvError::Empty => {}
        err => panic!("Channel error {}", err),
    }

    loop {
        let event = receiver.recv().unwrap();
        client.write_all(&event.to_bytes()).unwrap();
    }
}
