use std::env;
use std::io::{self, Write};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    print!("{}> ", username);

    let stream = TcpStream::connect(&server_addr).await?;
    let (mut reader, mut writer) = stream.into_split();

    let read_username = username.clone();
    let read_handle = tokio::spawn(async move {
        let mut buf = [0u8; 1024];
        loop {
            match reader.read(&mut buf).await {
                Ok(0) => {
                    println!("Connection closed by server.");
                    break;
                }
                Ok(n) => {
                    print!("\r\x1b[K"); // Clear the line
                    print!("{}", String::from_utf8_lossy(&buf[0..n]));
                    print!("{}> ", read_username);
                    io::stdout().flush().unwrap();
                }
                Err(e) => {
                    eprintln!("Error reading from stream: {}", e);
                }
            }
        }
    });

    let write_username = username.clone();
    let write_handle = tokio::spawn(async move {
        loop {
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            print!("\r\x1b[K\x1b[A\r\x1b[K"); // Clear the line that was just written
            let formatted_message = format!("{}: {}", write_username, input);
            if let Err(e) = writer.write_all(formatted_message.as_bytes()).await {
                eprintln!("Error writing to stream: {}", e);
                break;
            }
        }
    });

    let _ = tokio::try_join!(read_handle, write_handle)?;

    Ok(())
}
