mod client;
use std::env;
use std::process::exit;
use std::str::FromStr;
use std::net::{IpAddr, TcpStream, SocketAddr};
use std::io::{Write, Read};
use std::thread::sleep;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <server ip> <port>", args[0]);
        exit(1);
    }
    let ip = match IpAddr::from_str(&args[1]) {
        Ok(ip) => { ip }
        Err(e) => {
            eprintln!("Not a valid ip address: {}", e);
            exit(3);
        }
    };
    let port = match u16::from_str(&args[2]) {
        Ok(port) => {
            port
        }
        Err(e) => {
            eprintln!("Not a valid port number: {}", e);
            exit(2);
        }
    };

    let mut connection = TcpStream::connect(SocketAddr::new(ip, port))?;
    connection.write_all("Hello from client".as_bytes())?;
    connection.flush()?;
    sleep(Duration::from_millis(100));
    connection.write_all("Hello from client2".as_bytes())?;
    let mut buffer = [0; 128];
    loop {
        match connection.read(&mut buffer[..]) {
            Ok(0) => {
                println!("Connection properly closed");
                break;
            }
            Ok(n) => {
                println!("{}", String::from_utf8_lossy(&buffer[0..n]));
            }
            Err(e) => {
                eprintln!("{}", e);
                break;
            }
        }
    }
    println!("Connection closed.");
    Ok(())
}
