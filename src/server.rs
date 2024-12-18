use crate::message::EchoMessage;
use log::{error, info, warn};
use prost::Message;
use std::{
    io::{self, ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        Client { stream }
    }

    pub fn handle(&mut self) -> io::Result<()> {
        let mut buffer = [0; 512];
        // Read data from the client
        loop {
            match self.stream.read(&mut buffer) {
                Ok(0) => {
                    // Client disconnected
                    info!("Client disconnected.");
                    break;
                }
                Ok(bytes_read) => {
                    if let Ok(message) = EchoMessage::decode(&buffer[..bytes_read]) {
                        info!("Received: {}", message.content);
                        // Echo back the message
                        let payload = message.encode_to_vec();
                        self.stream.write_all(&payload)?;
                    } else {
                        error!("Failed to decode message");
                    }
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    // No data available; wait briefly to avoid busy-waiting
                    thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    error!("Error reading from client: {}", e);
                    break;
                }
            }
        }
        Ok(())
    }
}

pub struct Server {
    listener: TcpListener,
    is_running: Arc<AtomicBool>,
    address: String, // Store the address the server is bound to
    clients: Arc<Mutex<Vec<thread::JoinHandle<()>>>>,
}

impl Server {
    /// Creates a new server instance
    pub fn new(addr: &str) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        let local_addr = listener.local_addr()?; // Retrieve the actual address the server is bound to
        listener.set_nonblocking(true)?;
        Ok(Server {
            listener,
            is_running: Arc::new(AtomicBool::new(false)),
            address: local_addr.to_string(),
            clients: Arc::new(Mutex::new(Vec::new())),
        })
    }
    /// Returns the server's address
    pub fn address(&self) -> &str {
        &self.address
    }

    /// Runs the server, accepting connections and handling them concurrently
    pub fn run(&self) -> io::Result<()> {
        self.is_running.store(true, Ordering::SeqCst); // Set the server as running
        info!("Server is running on {}", self.listener.local_addr()?);

        while self.is_running.load(Ordering::SeqCst) {
            match self.listener.accept() {
                Ok((stream, addr)) => {
                    info!("New client connected: {}", addr);

                    let mut client = Client::new(stream);
                    let handle = thread::spawn(move || {
                        client
                            .handle()
                            .unwrap_or_else(|e| error!("Client error: {}", e));
                    });

                    self.clients.lock().unwrap().push(handle);
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    // No incoming connections, sleep briefly to reduce CPU usage
                    thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
        }

        info!("Server stopping. Waiting for all client threads to finish...");

        // Wait for all client threads to finish
        let mut clients = self.clients.lock().unwrap();
        while let Some(handle) = clients.pop() {
            handle
                .join()
                .unwrap_or_else(|_| warn!("A client thread failed to join."));
        }

        info!("Server stopped.");
        Ok(())
    }

    /// Stops the server by setting the `is_running` flag to `false`
    pub fn stop(&self) {
        if self.is_running.load(Ordering::SeqCst) {
            self.is_running.store(false, Ordering::SeqCst);
            info!("Shutdown signal sent.");
        } else {
            warn!("Server was already stopped or not running.");
        }
    }
}
