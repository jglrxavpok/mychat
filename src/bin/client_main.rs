mod client;
use std::{env, thread};
use std::process::exit;
use std::str::FromStr;
use std::net::{IpAddr, TcpStream, SocketAddr};
use std::io::{Write, Read, stdin};
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

    let mut connectionReader = connection.try_clone().unwrap();
    thread::spawn(move || {
        loop {
            let mut buffer = [0; 128];
            match connectionReader.read(&mut buffer[..]) {
                Ok(0) => {
                    println!("Connection properly closed");
                    break;
                }
                Ok(n) => {
                    let mut msg = String::from_utf8_lossy(&buffer[0..n]).to_string();
                    println!("{}", msg);
                }
                Err(e) => {
                    eprintln!("{}", e);
                    break;
                }
            }
        }
        println!("Connection closed.");
    });
    loop {
        let mut line = String::new();
        stdin().read_line(&mut line).expect("Invalid line");
        print!("\x1B[1A\x1B[K"); // clear line
        connection.write_all(line.as_bytes())?;
    }

    Ok(())
}
