use std::net;
use std::io;
use std::net::{SocketAddr, IpAddr, Shutdown, TcpStream};
use crossbeam::thread as cb_thread;
use std::io::{Write, Read};
use std::sync::Arc;
use std::sync::mpsc::{channel, Sender, sync_channel, SyncSender, Receiver};
use crate::server::UserState::JustConnected;

pub struct Server {
    listener: net::TcpListener,
    clients: Vec<User>
}

pub struct User {
    username: String,
    outgoing: SyncSender<String>,
    incoming: Receiver<String>,
    state: UserState
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
enum UserState {
    JustConnected,
    Authentified,
}

impl User {
    fn new(name: String, outgoing: SyncSender<String>, incoming: Receiver<String>) -> Self {
        User {
            username: name,
            outgoing,
            incoming,
            state: JustConnected,
        }
    }
}

impl Server {
    pub fn new(port: u16) -> io::Result<Self> {
        let ip = IpAddr::from([0,0,0,0]);
        let addr = SocketAddr::new(ip, port);
        let listener = net::TcpListener::bind(addr)?;
        Ok(Server {
            listener,
            clients: vec![]
        })
    }

    fn sendToAll(clients: &&mut Vec<User>, message: String) {
        println!("sending: {}", message);
        for client in clients.iter() {
            client.outgoing.send(message.clone());
        }
    }

    pub fn run(mut self) {
        let listener = &self.listener;
        let clients = &mut self.clients;
        let (newClientsTx, newClientsRx): (SyncSender<User>, Receiver<User>) = sync_channel(0);
        cb_thread::scope(|scope| {
            scope.spawn(move |_scope| {
                loop {
                    let mut messagesToSend = vec![];
                    for newClient in newClientsRx.try_iter() {
                        println!("newclient: {}", newClient.username);
                        clients.push(newClient);
                    }
                    for client in clients.iter_mut() {
                        for mut message in client.incoming.try_iter() {
                            message.pop();
                            println!("Received: <{}> {}", client.username, message);
                            if client.state == UserState::JustConnected {
                                client.username = message;
                                client.state = UserState::Authentified;
                                messagesToSend.push(format!("* {} just connected.", client.username));
                                println!("{}", format!("* {} just connected.", client.username));
                            } else {
                                messagesToSend.push(format!("<{}> {}", client.username, message));
                            }
                        }
                    }
                    for message in messagesToSend {
                        Server::sendToAll(&clients, message);
                    }
                }
            });
            loop {
                let l = listener.try_clone().unwrap();
                let connection = l.accept();
                match connection {
                    Ok((mut stream, address)) => {
                        let (incomingOut, incomingIn): (SyncSender<String>, Receiver<String>) = sync_channel(0);
                        let (outgoingOut, outgoingIn): (SyncSender<String>, Receiver<String>) = sync_channel(0);

                        let mut name = String::from("Unnamed ");
                        name.push_str(&address.to_string());
                        let user = User::new(name, outgoingOut, incomingIn);
                        newClientsTx.send(user);

                        let clonedAddr1 = address.clone();
                        let clonedAddr2 = address.clone();
                        let mut clonedStream1 = stream.try_clone().unwrap();
                        let mut clonedStream2 = stream.try_clone().unwrap();
                        Server::welcome_client(&mut stream);
                        scope.spawn(move |_scope| {
                            Server::handle_client_input(clonedStream1, clonedAddr1,incomingOut);
                        });
                        scope.spawn(move |_scope| {
                            Server::handle_client_output(clonedStream2, clonedAddr2, outgoingIn);
                        });
                    }
                    Err(e) => {
                        eprintln!("Error while accepting clients: {}", e);
                        break;
                    }
                }
            }
        }).expect("Failed during client accepts");
    }

    fn welcome_client(mut connection: &TcpStream) {
        connection.write_all("Welcome! Please enter your name.".as_bytes());
    }

    fn handle_client_output(mut connection: TcpStream, address: SocketAddr, outgoing: Receiver<String>) {
        for message in outgoing.iter() {
            connection.write_all(message.as_bytes());
        }
    }

    fn handle_client_input(mut connection: TcpStream, address: SocketAddr, incoming: SyncSender<String>) {
        let mut buffer = [0; 128];

        loop {
            match connection.read(&mut buffer[..]) {
                Ok(0) => break,
                Ok(n) => {
                    let mut string = String::from_utf8_lossy(&buffer[0..n]).to_string();
                    string.pop();
                    incoming.send(string);
                }
                Err(e) => {
                    eprintln!("{}", e);
                    break;
                }
            }
        }
        println!("Connection closed");
    }
}