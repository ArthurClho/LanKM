use std::io::{Read, Write};
use std::net::{self, IpAddr, SocketAddr, TcpStream};
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

use clap::Parser;

mod event;
mod input_capture;
mod input_injection;

use event::{Event, KeyEvent, KeyEventKind, Modifiers};

#[derive(Parser, Clone, Debug)]
enum Command {
    Client { address: net::Ipv4Addr, port: u16 },
    Server { port: u16 },
}

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    command: Command,
    #[arg(short, long, required=false, action=clap::ArgAction::SetTrue)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    simple_logger::init_with_level(if args.verbose {
        log::Level::Debug
    } else {
        log::Level::Warn
    })
    .unwrap();

    match args.command {
        Command::Client { address, port } => run_client(address, port),
        Command::Server { port } => run_server(port),
    }
}

fn connect_to_server(address: net::Ipv4Addr, port: u16) -> net::TcpStream {
    let socket = SocketAddr::new(IpAddr::V4(address), port);
    loop {
        log::info!("Trying to connect");
        match net::TcpStream::connect_timeout(&socket, Duration::from_secs(2)) {
            Ok(stream) => {
                log::info!("Connected to server");
                return stream;
            }
            Err(e) => {
                log::info!("Could not connect ({}) Retrying...", e);
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
}

fn run_client(address: net::Ipv4Addr, port: u16) {
    let mut injector = input_injection::InputInjector::new();
    let mut stream: Option<net::TcpStream> = None;

    loop {
        let s = match stream.as_mut() {
            Some(s) => s,
            None => {
                stream = Some(connect_to_server(address, port));
                stream.as_mut().unwrap()
            }
        };

        let mut buffer = [0; 4];
        match s.read_exact(&mut buffer) {
            Ok(_) => {}
            Err(e) => {
                log::error!("Error reading from TcpStream: {}", e);
                stream = None;
                log::error!("Connection closed");
            }
        }

        let event = event::KeyEvent::from_bytes(buffer);
        injector.emit(event);
    }
}

fn wait_for_client(port: u16) -> TcpStream {
    // TODO: Maybe handle these unwraps gracefully
    let listener = net::TcpListener::bind(("0.0.0.0", port)).unwrap();
    let (client, addr) = listener.accept().unwrap();

    log::info!("client connected from {}", addr);
    client
}

fn run_server(port: u16) {
    let (sender, receiver) = mpsc::channel::<Event>();

    let mut sending = false;
    input_capture::init(move |e| match e {
        Event::Key(_) => {
            if sending {
                sender.send(e).unwrap();
                true
            } else {
                false
            }
        }
        Event::Hotkey => {
            sending = !sending;
            log::info!("Turned {} sending", if sending { "On" } else { "Off" });

            // Focus was brought back from the client, make sure it gets the hotkey release
            if !sending {
                let release = |hid| {
                    sender
                        .send(Event::Key(KeyEvent {
                            hid,
                            kind: KeyEventKind::Release,
                            mods: Modifiers::empty(),
                        }))
                        .unwrap();
                };

                release(0xE0);
                release(0xE4);
                release(0xE2);
                release(0xE6);
            }

            true
        }
    });

    let mut maybe_client = None;

    loop {
        let client = match maybe_client.as_mut() {
            Some(c) => c,
            None => {
                maybe_client = Some(wait_for_client(port));

                // clear anything that was buffered before a client connected
                loop {
                    match receiver.try_recv() {
                        Ok(_) => {}
                        Err(TryRecvError::Empty) => break,
                        Err(e) => panic!("Channel read error: {}", e),
                    }
                }

                maybe_client.as_mut().unwrap()
            }
        };

        let event = receiver.recv().unwrap();
        match client.write_all(&event.to_bytes()) {
            Ok(_) => {}
            Err(e) => {
                log::error!("Error writing to client: {}", e);
                maybe_client = None;
            }
        }
    }
}
