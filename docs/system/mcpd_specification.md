# MCP System Daemon (mcpd) Specification

## Overview
`mcpd` is the core system service responsible for implementing the Model Context Protocol (MCP) as the system's primary IPC mechanism. It acts as a broker between the Agent Runtime and various system tools/applications.

## Key Responsibilities
1.  **Tool Lifecycle**: Managing the registration, heartbeat, and availability of system tools.
2.  **Request Routing**: Routing JSON-RPC calls from the Agent to the appropriate tool provider.
3.  **Permission Enforcement**: Interfacing with the Permission Manager to validate every call.
4.  **Logging & Auditing**: Maintaining a secure, immutable log of all tool executions for user transparency.
5.  **Performance**: Leveraging Rust's safety and performance for low-latency system-level tool calls.

## Architecture
The daemon is built using a multi-threaded, asynchronous architecture using `tokio`.

### Components
*   **Server Endpoints**:
    *   `Unix Domain Socket`: For local Android service communication.
    *   `Shared Memory (optional)`: For high-bandwidth binary data (camera/vision).
*   **Tool Registry**: In-memory map of `tool_name -> provider_endpoint`.
*   **Session Manager**: Tracks active agent sessions and context propagation.
*   **Security Interceptor**: Middleware that checks permission tokens before routing requests.

## Implementation (Rust)
The implementation will use:
*   `serde_json`: For protocol parsing.
*   `tokio`: For async runtime.
*   `tower`: For middleware and service abstraction.

## Transport Layer
Primary transport on-device is **Unix Domain Sockets (UDS)** for security and speed.
Legacy compatibility is maintained via local HTTP/JSON-RPC where necessary.
