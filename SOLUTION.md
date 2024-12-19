# Solution

This document outlines the identified bugs in the initial server implementation, the architectural changes made, and the solutions implemented to address these issues.

## Resolved Issues

### 1. "Address already in use" Error
*   **Error:** `Os { code: 98, kind: AddrInUse, message: "Address already in use" }`
*   **Cause:** The server was hardcoded to bind to a specific address (`localhost:8080`) during initialization. If this address was already in use by another instance of the server or another process, the binding would fail with the "Address already in use" error.
*   **Solution:** To address this issue, the server was updated to dynamically bind to an available port instead of a fixed port. Additionally, the server was enhanced to expose the actual address it was bound to, so that the client and test cases could use it dynamically.

### 2. "Failed to connect to the server" Error
*   **Error:** `Failed to connect to the server`
*   **Cause:** The issue occurred because the server and client were not properly synchronized:

- The client attempted to connect to a hardcoded address (`localhost:8080`) instead of the server's dynamically assigned address.
- There was no delay to ensure that the server was ready to accept connections before the client attempted to connect.
*   **Solution:** To resolve this, two major changes were made:

- Dynamically retrieve the server's actual address and use it to initialize the client.
- Introduce a short delay after starting the server to ensure it is ready to accept connections.

## Transition to Multithreading
### Objective
The server was updated to handle multiple clients concurrently using Rust’s multithreading libraries. Proper synchronization mechanisms were implemented to ensure thread safety.
### Changes Made
#### 1. Multithreading for Client Handling
- A new thread is spawned for each client connection using `std::thread::spawn`.
- This ensures that the server can handle multiple clients concurrently without blocking the main loop.

#### 2. Synchronization for Thread Safety
To track and manage client threads safely:

- A `Vec` of `JoinHandle<()>` is shared between threads using an `Arc<Mutex<>>`.
- The `Mutex` ensures exclusive access to the list of client threads, avoiding race conditions when multiple threads modify it concurrently.

#### 3. Graceful Shutdown
- When `server.stop()` is called, the `is_running` flag is set to `false`, signaling the main loop to exit.
- The server waits for all active client threads to finish by joining them.
- This ensures no threads are left dangling after the server stops.

#### 4. Non-Blocking Server Loop
- The `TcpListener` is set to non-blocking mode `(set_nonblocking(true))` to prevent the server from hanging while waiting for new connections.
- The main loop uses a short sleep `(thread::sleep)` when no connections are available to reduce CPU usage.

### Challenges and Solutions
#### Challenge 1: Managing Shared State
* **Problem:** Concurrent threads modifying shared state (e.g., the list of client threads) could cause data races.
* **Solution:** Use `Arc<Mutex<>>` to provide safe, synchronized access to the shared state.

#### Challenge 2: Preventing Blocking in the main loop 
* **Problem:** The main loop could block indefinitely when no clients connect.
* **Solution:** Use TcpListener::set_nonblocking(true) to make the listener non-blocking.

#### Challenge 3: Ensuring Clean Resource Cleanup
* **Problem:** Client threads could continue running even after the server stops.
* **Solution:** Track all client threads and join them during server shutdown.

## Add Handling for AddRequest in the Server
### Objective
Enhance the server to handle `AddRequest` messages from clients, ensuring data consistency, avoiding race conditions, deadlocks, and starvation during concurrent operations.

### Changes Made
#### 1. AddRequest Handling
The server now decodes incoming `AddRequest` messages, performs the addition operation, and sends back an `AddResponse` message to the client. Here's how the process is handled:

- **Message Decoding:** `AddRequest` messages are decoded using the `prost` library.

- **Computation:** The server computes the sum of two integers (`a` and `b`) in the request.

- **Response Encoding:** The result is wrapped in an `AddResponse ` message and sent back to the client.

#### 2. Avoiding Race Conditions
To prevent race conditions:

- Shared resources (e.g., the server's list of active client threads) are managed using `Arc<Mutex<>>`.
- The `Mutex` ensures that only one thread accesses the shared resource at a time.

#### 3. Handling Errors Gracefully
Errors during message decoding, computation, or communication with the client are logged and handled appropriately:

- If decoding fails, an error is logged and the server continues processing other requests.
- If a client disconnects, the server cleans up resources and waits for other clients.

#### 4. Preventing Deadlocks
Deadlocks are avoided by ensuring:

- Locks on Mutex are held for the shortest possible duration.
- All shared resources are accessed in a consistent order across threads.

#### 5. Ensuring Fairness and Avoiding Starvation
- By spawning a new thread for each client, all clients get equal opportunity to interact with the server.
- The server uses a non-blocking listener to periodically check for new connections and existing client activity.


## Test Suite Results 

```
    Running tests/client_test.rs (target/debug/deps/client_test-011d20d1cd9f753d)

    running 5 tests
    test test_client_connection ... ok
    test test_client_add_request ... ok
    test test_multiple_echo_messages ... ok
    test test_multiple_clients ... ok
    test test_client_echo_message ... ok

    successes:

    ---- test_client_connection stdout ----
    Connecting to 127.0.0.1:38587
    Connected to the server!
    Disconnected from the server!

    ---- test_client_add_request stdout ----
    Connecting to 127.0.0.1:45907
    Connected to the server!
    Sent message: AddRequest(AddRequest { a: 10, b: 20 })
    Received AddResponse: result = 30
    Disconnected from the server!

    ---- test_multiple_echo_messages stdout ----
    Connecting to 127.0.0.1:42441
    Connected to the server!
    Sent message: EchoMessage(EchoMessage { content: "Hello, World!" })
    Sent message: EchoMessage(EchoMessage { content: "How are you?" })
    Sent message: EchoMessage(EchoMessage { content: "Goodbye!" })
    Disconnected from the server!

    ---- test_multiple_clients stdout ----
    Connecting to 127.0.0.1:33541
    Connected to the server!
    Connecting to 127.0.0.1:33541
    Connected to the server!
    Connecting to 127.0.0.1:33541
    Connected to the server!
    Sent message: EchoMessage(EchoMessage { content: "Hello, World!" })
    Sent message: EchoMessage(EchoMessage { content: "Hello, World!" })
    Sent message: EchoMessage(EchoMessage { content: "Hello, World!" })
    Sent message: EchoMessage(EchoMessage { content: "How are you?" })
    Sent message: EchoMessage(EchoMessage { content: "How are you?" })
    Sent message: EchoMessage(EchoMessage { content: "How are you?" })
    Sent message: EchoMessage(EchoMessage { content: "Goodbye!" })
    Sent message: EchoMessage(EchoMessage { content: "Goodbye!" })
    Sent message: EchoMessage(EchoMessage { content: "Goodbye!" })
    Disconnected from the server!
    Disconnected from the server!
    Disconnected from the server!

    ---- test_client_echo_message stdout ----
    Connecting to 127.0.0.1:34809
    Connected to the server!
    Sent message: EchoMessage(EchoMessage { content: "Hello, World!" })
    Disconnected from the server!


    successes:
        test_client_add_request
        test_client_connection
        test_client_echo_message
        test_multiple_clients
        test_multiple_echo_messages

    test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s

```