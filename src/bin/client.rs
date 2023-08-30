#[path = "../select.rs"]
mod select;

extern crate libc;

use std::env;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::os::unix::io::AsRawFd;

struct Client {
    username: String,
    stream: TcpStream,
}

impl Client {
    fn new(server_addr: &str, username: &str) -> io::Result<Client> {
        let stream = TcpStream::connect(server_addr)?;
        stream.set_nonblocking(true)?;
        Ok(Client {
            username: username.to_string(),
            stream,
        })
    }

    fn prompt(&self) {
        print!("{}> ", self.username);
        io::stdout().flush().unwrap();
    }

    fn receive_message(&mut self) -> io::Result<()> {
        let mut buf = [0u8; 1024];
        match self.stream.read(&mut buf) {
            Ok(n) => {
                print!("\r\x1b[K"); // Clear the line
                print!("{}", String::from_utf8_lossy(&buf[0..n]));
                self.prompt();
            }
            Err(e) => {
                eprintln!("Error reading from stream: {}", e);
            }
        }
        Ok(())
    }

    fn send_message(&mut self) -> io::Result<()> {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        print!("\r\x1b[K\x1b[A\r\x1b[K"); // Clear the line that was just written
        let formatted_message = format!("{}: {}", self.username, input);
        self.stream.write_all(formatted_message.as_bytes())?;
        self.prompt();
        Ok(())
    }

    fn run(&mut self) -> io::Result<()> {
        let stdin_fd = io::stdin().as_raw_fd();
        let stream_fd = self.stream.as_raw_fd();

        self.prompt();

        loop {
            let fds = vec![stream_fd, stdin_fd];
            let select_iter = select::Select::new(fds)?;

            for fd in select_iter {
                if fd == stream_fd {
                    self.receive_message()?;
                } else if fd == stdin_fd {
                    self.send_message()?;
                }
            }
        }
    }
}

fn main() -> io::Result<()> {
    let mut username = "Anonymous".to_string();
    let mut server_addr = "127.0.0.1:4000".to_string();

    let args: Vec<String> = env::args().collect();
    let mut iter = args.iter();

    let _ = iter.next(); // Skip the first argument, which is the program name

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--username" => {
                username = iter
                    .next()
                    .cloned()
                    .unwrap_or_else(|| "Anonymous".to_string());
            }
            "--server" => {
                server_addr = iter
                    .next()
                    .cloned()
                    .unwrap_or_else(|| "127.0.0.1:4000".to_string());
            }
            _ => {
                eprintln!("Unknown option: {}", arg);
                eprintln!("Usage: <program> [--username <username>] [--server <address>]");
                return Ok(());
            }
        }
    }

    let mut client = Client::new(&server_addr, &username)?;
    client.run()?;
    Ok(())
}
