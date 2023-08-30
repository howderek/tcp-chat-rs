#[path = "../select.rs"]
mod select;

extern crate libc;

use libc::c_int;
use std::collections::HashMap;
use std::env;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::io::AsRawFd;

struct Server {
    listener: TcpListener,
    clients: HashMap<c_int, TcpStream>,
}

impl Server {
    fn new(addr: &str) -> std::io::Result<Server> {
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        Ok(Server {
            listener,
            clients: HashMap::new(),
        })
    }

    fn client_connect(&mut self) -> std::io::Result<()> {
        if let Ok((stream, _)) = self.listener.accept() {
            println!("client connected");
            let fd = stream.as_raw_fd() as c_int;
            stream.set_nonblocking(true)?;
            self.clients.insert(fd, stream);
        }
        Ok(())
    }

    fn client_disconnect(&mut self, fd: c_int) {
        println!("client disconnected");
        self.clients.remove(&fd);
    }

    fn broadcast(&mut self, message: &[u8]) -> std::io::Result<()> {
        for stream in self.clients.values_mut() {
            let _ = stream.write(message);
        }
        Ok(())
    }

    fn recieve(&mut self, fd: c_int) -> std::io::Result<()> {
        let stream = self.clients.get_mut(&fd).unwrap();
        let mut buf = [0u8; 1024];
        match stream.read(&mut buf) {
            Ok(0) => {
                self.client_disconnect(fd);
                Ok(())
            }
            Ok(n) => {
                let message = &buf[0..n];
                print!("{}", String::from_utf8_lossy(message));
                self.broadcast(message)?;
                Ok(())
            }
            Err(_) => Ok(()),
        }
    }

    fn run(&mut self) -> std::io::Result<()> {
        loop {
            let mut fds = vec![self.listener.as_raw_fd()];
            fds.extend(self.clients.keys().cloned());

            let select_iter = select::Select::new(fds)?;

            for fd in select_iter {
                if fd == self.listener.as_raw_fd() {
                    self.client_connect()?;
                } else {
                    self.recieve(fd)?;
                }
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    let mut addr = "127.0.0.1:4000".to_string();

    // Parsing command-line arguments
    let args: Vec<String> = env::args().collect();
    let mut iter = args.iter();

    let _ = iter.next(); // Skip the first argument, which is the program name

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--listen" => {
                addr = iter
                    .next()
                    .cloned()
                    .unwrap_or_else(|| "127.0.0.1:4000".to_string());
            }
            _ => {
                eprintln!("Unknown option: {}", arg);
                eprintln!("Usage: <program> [--listen <address>]");
                return Ok(());
            }
        }
    }

    let mut server = Server::new(&addr)?;
    server.run()?;
    Ok(())
}
