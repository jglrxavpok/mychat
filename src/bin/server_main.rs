mod server;
use server::Server;
use std::env;
use std::process::exit;
use std::str::FromStr;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <port>", args[0]);
        exit(1);
    }
    let port = match u16::from_str(&args[1]) {
        Ok(port) => {
            port
        }
        Err(e) => {
            eprintln!("Not a valid port number: {}", e);
            exit(2);
        }
    };

    println!("Opening server on port {}...", port);
    let server = Server::new(port).expect("Server could not be opened");
    server.run();
}
