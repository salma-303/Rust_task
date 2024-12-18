# Solution

## Bug Analysis and Fix Report
This document outlines the identified bugs in the initial server implementation, the architectural changes made, and the solutions implemented to address these issues.

### Resolved Issues

#### 1. "Address already in use" Error
*   **Error:** `Os { code: 98, kind: AddrInUse, message: "Address already in use" }`
*   **Cause:** The server was hardcoded to bind to a specific address (`localhost:8080`) during initialization. If this address was already in use by another instance of the server or another process, the binding would fail with the "Address already in use" error.
*   **Solution:** To address this issue, the server was updated to dynamically bind to an available port instead of a fixed port. Additionally, the server was enhanced to expose the actual address it was bound to, so that the client and test cases could use it dynamically.

#### 2. "Failed to connect to the server" Error
*   **Error:** `Failed to connect to the server`
*   **Cause:** The issue occurred because the server and client were not properly synchronized:

- The client attempted to connect to a hardcoded address (`localhost:8080`) instead of the server's dynamically assigned address.
- There was no delay to ensure that the server was ready to accept connections before the client attempted to connect.
*   **Solution:** To resolve this, two major changes were made:

- Dynamically retrieve the server's actual address and use it to initialize the client.
- Introduce a short delay after starting the server to ensure it is ready to accept connections.

### Transition to Multithreading
#### Objective
The server was updated to handle multiple clients concurrently using Rust’s multithreading libraries. Proper synchronization mechanisms were implemented to ensure thread safety.
#### Changes Made
##### 1. Multithreading for Client Handling
- A new thread is spawned for each client connection using `std::thread::spawn`.
- This ensures that the server can handle multiple clients concurrently without blocking the main loop.

##### 2. Synchronization for Thread Safety
To track and manage client threads safely:

A `Vec` of `JoinHandle<()>` is shared between threads using an `Arc<Mutex<>>`.
The `Mutex` ensures exclusive access to the list of client threads, avoiding race conditions when multiple threads modify it concurrently.

##### 3. Graceful Shutdown
- When `server.stop()` is called, the `is_running` flag is set to `false`, signaling the main loop to exit.
- The server waits for all active client threads to finish by joining them.
- This ensures no threads are left dangling after the server stops.

##### 4. Non-Blocking Server Loop
he `TcpListener` is set to non-blocking mode `(set_nonblocking(true))` to prevent the server from hanging while waiting for new connections.
The main loop uses a short sleep `(thread::sleep)` when no connections are available to reduce CPU usage.

#### Challenges and Solutions
##### Challenge 1: Managing Shared State
* **Problem:** Concurrent threads modifying shared state (e.g., the list of client threads) could cause data races.
* **Solution:** Use `Arc<Mutex<>>` to provide safe, synchronized access to the shared state.

##### Challenge 2: Preventing Blocking in the main loop 
* **Problem:** The main loop could block indefinitely when no clients connect.
* **Solution:** Use TcpListener::set_nonblocking(true) to make the listener non-blocking.

##### Challenge 3: Ensuring Clean Resource Cleanup
* **Problem:** Client threads could continue running even after the server stops.
* **Solution:** Track all client threads and join them during server shutdown.

