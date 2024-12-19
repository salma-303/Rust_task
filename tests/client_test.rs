use embedded_recruitment_task::{
    message::{client_message, server_message, AddRequest, EchoMessage},
    server::Server,
};
use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

mod client;

use log::info;

fn setup_server_thread(server: Arc<Server>) -> JoinHandle<()> {
    let address = server.address().to_string(); // Retrieve the dynamic address
    thread::spawn(move || {
        info!("Server running on {}", address);
        server.run().expect("Server encountered an error");
    })
}

fn create_server() -> Arc<Server> {
    // Bind to "localhost:0" for a random available port
    let server = Server::new("localhost:0").expect("Failed to start server");
    Arc::new(server)
}

#[test]
fn test_client_connection() {
    let server = create_server();
    let address = server.address(); // Get the dynamic address
    let handle = setup_server_thread(server.clone());
    thread::sleep(std::time::Duration::from_millis(100));
    // Extract host and port from the address
    let parts: Vec<&str> = address.split(':').collect();
    let host = parts[0];
    let port: u16 = parts[1].parse().unwrap();

    // Convert port from u16 to u32 for Client::new
    let mut client = client::Client::new(host, port.into(), 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    // Disconnect the client
    assert!(
        client.disconnect().is_ok(),
        "Failed to disconnect from the server"
    );

    // Stop the server
    server.stop();
    let server_shutdown_result = handle
        .join()
        .map_err(|_| "Server thread panicked or failed to join");
    assert!(
        server_shutdown_result.is_ok(),
        "Server thread did not stop cleanly"
    );
}

#[test]
fn test_client_echo_message() {
    // Set up the server in a separate thread
    let server = create_server();
    let handle = setup_server_thread(server.clone());

    // Ensure the server is ready before connecting the client
    thread::sleep(std::time::Duration::from_millis(100));
    // Create and connect the client
    let address = server.address(); // Get the server's actual address
    let parts: Vec<&str> = address.split(':').collect();
    let host = parts[0];
    let port: u16 = parts[1].parse().unwrap();

    let mut client = client::Client::new(host, port.into(), 1000);

    assert!(client.connect().is_ok(), "Failed to connect to the server");

    // Prepare the message
    let mut echo_message = EchoMessage::default();
    echo_message.content = "Hello, World!".to_string();
    let message = client_message::Message::EchoMessage(echo_message.clone());

    // Send the message to the server
    assert!(client.send(message).is_ok(), "Failed to send message");

    // Receive the echoed message
    let response = client.receive();
    assert!(
        response.is_ok(),
        "Failed to receive response for EchoMessage"
    );

    match response.unwrap().message {
        Some(server_message::Message::EchoMessage(echo)) => {
            assert_eq!(
                echo.content, echo_message.content,
                "Echoed message content does not match"
            );
        }
        _ => panic!("Expected EchoMessage, but received a different message"),
    }

    // Disconnect the client
    assert!(
        client.disconnect().is_ok(),
        "Failed to disconnect from the server"
    );

    // Stop the server and wait for thread to finish
    server.stop();
    assert!(
        handle.join().is_ok(),
        "Server thread panicked or failed to join"
    );
}

#[test]
// #[ignore = "please remove ignore and fix this test"]
fn test_multiple_echo_messages() {
    // Set up the server in a separate thread
    let server = create_server();
    let handle = setup_server_thread(server.clone());

    // Ensure the server is ready before connecting the client
    thread::sleep(std::time::Duration::from_millis(100));

    let address = server.address();
    let parts: Vec<&str> = address.split(':').collect();
    let host = parts[0];
    let port: u16 = parts[1].parse().unwrap();

    // Create and connect the client
    let mut client = client::Client::new(host, port.into(), 1000);
    log::debug!("Before client connect");
    assert!(client.connect().is_ok(), "Failed to connect to the server");
    log::debug!("After client connect");

    // Prepare multiple messages
    let messages = vec![
        "Hello, World!".to_string(),
        "How are you?".to_string(),
        "Goodbye!".to_string(),
    ];

    // Send and receive multiple messages
    for message_content in messages {
        let mut echo_message = EchoMessage::default();
        echo_message.content = message_content.clone();
        let message = client_message::Message::EchoMessage(echo_message);

        // Send the message to the server
        assert!(client.send(message).is_ok(), "Failed to send message");

        // Receive the echoed message
        let response = client.receive();
        assert!(
            response.is_ok(),
            "Failed to receive response for EchoMessage"
        );

        match response.unwrap().message {
            Some(server_message::Message::EchoMessage(echo)) => {
                assert_eq!(
                    echo.content, message_content,
                    "Echoed message content does not match"
                );
            }
            _ => panic!("Expected EchoMessage, but received a different message"),
        }
    }

    // Disconnect the client
    assert!(
        client.disconnect().is_ok(),
        "Failed to disconnect from the server"
    );

    // Stop the server and wait for thread to finish
    server.stop();
    assert!(
        handle.join().is_ok(),
        "Server thread panicked or failed to join"
    );
}

#[test]
// #[ignore = "please remove ignore and fix this test"]
fn test_multiple_clients() {
    // Set up the server in a separate thread
    let server = create_server();
    let handle = setup_server_thread(server.clone());

    // Ensure the server is ready before connecting clients
    thread::sleep(std::time::Duration::from_millis(100));

    let address = server.address();
    let parts: Vec<&str> = address.split(':').collect();
    let host = parts[0];
    let port: u16 = parts[1].parse().unwrap();

    // Create and connect multiple clients
    let mut clients = vec![
        client::Client::new(host, port.into(), 1000),
        client::Client::new(host, port.into(), 1000),
        client::Client::new(host, port.into(), 1000),
    ];

    for client in clients.iter_mut() {
        assert!(client.connect().is_ok(), "Failed to connect to the server");
    }

    // Prepare multiple messages
    let messages = vec![
        "Hello, World!".to_string(),
        "How are you?".to_string(),
        "Goodbye!".to_string(),
    ];

    // Send and receive multiple messages for each client
    for message_content in messages {
        let mut echo_message = EchoMessage::default();
        echo_message.content = message_content.clone();
        let message = client_message::Message::EchoMessage(echo_message.clone());

        for client in clients.iter_mut() {
            // Send the message to the server
            assert!(
                client.send(message.clone()).is_ok(),
                "Failed to send message"
            );

            // Receive the echoed message
            let response = client.receive();
            assert!(
                response.is_ok(),
                "Failed to receive response for EchoMessage"
            );

            match response.unwrap().message {
                Some(server_message::Message::EchoMessage(echo)) => {
                    assert_eq!(
                        echo.content, message_content,
                        "Echoed message content does not match"
                    );
                }
                _ => panic!("Expected EchoMessage, but received a different message"),
            }
        }
    }

    // Disconnect the clients
    for client in clients.iter_mut() {
        assert!(
            client.disconnect().is_ok(),
            "Failed to disconnect from the server"
        );
    }

    // Stop the server and wait for thread to finish
    server.stop();
    assert!(
        handle.join().is_ok(),
        "Server thread panicked or failed to join"
    );
}

#[test]
// #[ignore = "please remove ignore and fix this test"]
fn test_client_add_request() {
    // Set up the server and start it in a thread
    let server = create_server();
    let handle = setup_server_thread(server.clone());

    // Ensure the server is ready
    thread::sleep(std::time::Duration::from_millis(100));

    // Extract server address
    let address = server.address();
    let parts: Vec<&str> = address.split(':').collect();
    let host = parts[0];
    let port: u16 = parts[1].parse().unwrap();

    // Create and connect the client
    let mut client = client::Client::new(host, port.into(), 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    // Send AddRequest and verify AddResponse
    let mut add_request = AddRequest::default();
    add_request.a = 10;
    add_request.b = 20;
    let message = client_message::Message::AddRequest(add_request.clone());

    // Send the message to the server
    assert!(client.send(message).is_ok(), "Failed to send message");

    // Receive the response
    let response = client.receive();
    assert!(
        response.is_ok(),
        "Failed to receive response for AddRequest"
    );

    match response.unwrap().message {
        Some(server_message::Message::AddResponse(add_response)) => {
            // Log or print the received sum
            println!("Received AddResponse: result = {}", add_response.result);
            assert_eq!(
                add_response.result,
                add_request.a + add_request.b,
                "AddResponse result does not match"
            );
        }
        _ => panic!("Expected AddResponse, but received a different message"),
    }

    // Disconnect and stop the server
    assert!(
        client.disconnect().is_ok(),
        "Failed to disconnect from the server"
    );
    server.stop();
    assert!(
        handle.join().is_ok(),
        "Server thread panicked or failed to join"
    );
}

#[test]
fn test_concurrent_add_requests() {
    // Set up the server in a separate thread
    let server = create_server();
    let handle = setup_server_thread(server.clone());

    // Ensure the server is ready before connecting the clients
    thread::sleep(std::time::Duration::from_millis(100));

    let address = server.address();
    let parts: Vec<&str> = address.split(':').collect();
    let host = parts[0];
    let port: u16 = parts[1].parse().unwrap();

    // Create multiple clients and connect them
    let mut clients: Vec<client::Client> = (0..10)
        .map(|_| client::Client::new(host, port.into(), 1000))
        .collect();

    for client in clients.iter_mut() {
        assert!(client.connect().is_ok(), "Failed to connect to the server");
    }

    // Send AddRequest messages from all clients concurrently
    let handles: Vec<_> = clients
        .into_iter()
        .map(|mut client| {
            thread::spawn(move || {
                let mut add_request = AddRequest::default();
                add_request.a = 5;
                add_request.b = 15;

                let message = client_message::Message::AddRequest(add_request.clone());
                assert!(client.send(message).is_ok(), "Failed to send AddRequest");

                let response = client.receive();
                assert!(response.is_ok(), "Failed to receive AddResponse");

                if let Some(server_message::Message::AddResponse(add_response)) =
                    response.unwrap().message
                {
                    assert_eq!(add_response.result, 20, "Incorrect sum in AddResponse");
                } else {
                    panic!("Expected AddResponse, but received a different message");
                }

                client
                    .disconnect()
                    .expect("Failed to disconnect from the server");
            })
        })
        .collect();

    // Wait for all threads to finish
    for handle in handles {
        handle.join().expect("Client thread panicked");
    }

    // Stop the server and wait for it to finish
    server.stop();
    assert!(
        handle.join().is_ok(),
        "Server thread panicked or failed to join"
    );
}

#[test]
fn test_large_echo_message() {
    let server = create_server();
    let handle = setup_server_thread(server.clone());

    thread::sleep(std::time::Duration::from_millis(100));

    let address = server.address();
    let parts: Vec<&str> = address.split(':').collect();
    let host = parts[0];
    let port: u16 = parts[1].parse().unwrap();

    let mut client = client::Client::new(host, port.into(), 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    let mut echo_message = EchoMessage::default();
    echo_message.content = "s".repeat(10_000); // Large message with 10,000 characters
    let message = client_message::Message::EchoMessage(echo_message.clone());

    assert!(
        client.send(message).is_ok(),
        "Failed to send large EchoMessage"
    );

    let response = client.receive();
    assert!(
        response.is_ok(),
        "Failed to receive response for large EchoMessage"
    );

    if let Some(server_message::Message::EchoMessage(echo_response)) = response.unwrap().message {
        assert_eq!(
            echo_response.content, echo_message.content,
            "Echoed message content does not match the original"
        );
    } else {
        panic!("Expected EchoMessage, but received a different message");
    }

    client
        .disconnect()
        .expect("Failed to disconnect from the server");
    server.stop();
    assert!(
        handle.join().is_ok(),
        "Server thread panicked or failed to join"
    );
}

#[test]
fn test_rapid_connect_disconnect() {
    let server = create_server();
    let handle = setup_server_thread(server.clone());

    thread::sleep(std::time::Duration::from_millis(100));

    let address = server.address();
    let parts: Vec<&str> = address.split(':').collect();
    let host = parts[0];
    let port: u16 = parts[1].parse().unwrap();

    for _ in 0..50 {
        let mut client = client::Client::new(host, port.into(), 1000);
        assert!(client.connect().is_ok(), "Failed to connect to the server");
        assert!(
            client.disconnect().is_ok(),
            "Failed to disconnect from the server"
        );
    }

    server.stop();
    assert!(
        handle.join().is_ok(),
        "Server thread panicked or failed to join"
    );
}
