use std::io::Write;
use std::net;
use std::sync::mpsc::{self, TryRecvError};

mod data;
mod input_capture;

fn main() {
    let (sender, receiver) = mpsc::channel::<data::KeyEvent>();

    input_capture::init(sender);
    
    let listener = net::TcpListener::bind("0.0.0.0:6069").unwrap();
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
