use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::io::{Result, Write, Read};
use time;
use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian};

const BUFFER_SIZE: usize = 1024 * 1024;

pub struct TestServer {
    //port: u16,
    //listen_address: String,
    listener: TcpListener
}

impl TestServer {

    pub fn new(port: u16, listen_address: &str) -> TestServer {
        TestServer {
            //port: port,
            //listen_address: listen_address.to_string(),
            listener: TcpListener::bind((listen_address as &str, port)).unwrap()
        }
    }

    pub fn listen(self) {

        for stream in self.listener.incoming() {
            self.new_connection(stream.unwrap());
        }
    }

    fn new_connection(&self, stream: TcpStream) {
        match stream.peer_addr() {
            Ok(addr) => {
                println!("Incoming connection from {}", addr);
                let addr_ = addr.clone();
                thread::spawn(move || {
                    let mut con = Connection::new(stream);
                    match con.handle() {
                        Ok(_) => println!("Connection from {} closed", addr_),
                        Err(x) => println!("Error while reading from connection from {}: {}", addr_, x)
                    };
                });
            },
            Err(x) => {
                println!("Could not retrieve peer address: {}", x)
            }
        }
    }
}

struct Connection {
    stream: TcpStream,
    sender_commander: Sender<u64>
}

impl Connection {

    fn new<'b>(stream: TcpStream) -> Connection {
        let (tx, rx) = channel::<u64>();
        let s = stream.try_clone().unwrap();
        thread::spawn(|| {
            let peer_addr =  s.peer_addr().unwrap().clone();
            match Connection::sender_runner(rx, s) {
                Ok(_) => {}
                Err(x) => println!("Error while writing to connection from {}: {}", peer_addr, x)
            };
        });
        Connection {
            stream: stream,
            sender_commander: tx
        }
    }

    pub fn handle(&mut self) -> Result<()> {
        let mut sink = [0; BUFFER_SIZE];
        loop {
            let cmd = try!(self.stream.read_u8());
            match cmd {
                0 => {
                    try!(self.stream.read_exact(&mut sink));
                },
                1 => {
                    // Request for Payload
                    let ms = try!(self.stream.read_u64::<BigEndian>());
                    self.sender_commander.send(ms).unwrap();
                },
                2 => {} // End of test, client only
                3 => {
                    // Pingtest
                    try!(self.stream.write_u8(3u8));
                }
                255 => {
                    // Disconnect
                    return Ok(());
                },
                _ => {}
            };
        }
    }

    fn sender_runner(rx: Receiver<u64>, mut stream: TcpStream) -> Result<()> {

        let buf = [0u8; BUFFER_SIZE];

        loop {
            match rx.recv() {
                Ok(time) => {
                    let start = time::precise_time_ns();
                    loop {

                        try!(stream.write_u8(0u8));
                        try!(stream.write(&buf));
                        try!(stream.flush());

                        if (time::precise_time_ns() - start) / 1_000_000 >= time {
                            break;
                        }
                    }
                    try!(stream.write_u8(2))
                },
                Err(_) => return Ok(())
            }
        }
    }
}
