# Solution

## Bug Analysis and Fix Report
This document outlines the identified bugs in the initial server implementation, the architectural changes made, and the solutions implemented to address these issues.

### Resolved Issues

#### 1. "Address already in use" Error
*   **Error:** `Os { code: 98, kind: AddrInUse, message: "Address already in use" }`
*   **Cause:** The server was hardcoded to bind to a specific address (`localhost:8080`) during initialization. If this address was already in use by another instance of the server or another process, the binding would fail with the "Address already in use" error.
*   **Solution:** To address this issue, the server was updated to dynamically bind to an available port instead of a fixed port. Additionally, the server was enhanced to expose the actual address it was bound to, so that the client and test cases could use it dynamically.