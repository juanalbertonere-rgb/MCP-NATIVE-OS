# MCP Protocol Extensions for Mobile OS

To meet the requirements of a high-performance mobile OS, we extend the base MCP specification with the following additive capabilities.

## 1. Low-Latency IPC (UDS Transport)
While MCP traditionally uses stdio or HTTP, `mcpd` implements a Unix Domain Socket (UDS) transport to reduce overhead and leverage file-system based permissions.

## 2. Capability Negotiation
Tools can declare specific OS capabilities they require (e.g., `android.permission.CAMERA`). The daemon ensures these match the app's manifest-declared permissions.

## 3. Streaming Binary Payloads
Support for `mcp.stream` method to handle real-time data like camera frames or microphone input without base64 encoding overhead, potentially via shared memory descriptors.

## 4. Context Propagation Metadata
Every request includes a `context` object containing:
*   `origin_app_id`: The ID of the app providing the tool.
*   `user_session_id`: To track multi-turn agent interactions.
*   `confidence_score`: Provided by the agent for the current plan.
*   `is_user_initiated`: Boolean flag indicating if this was a direct user command or an autonomous agent action.

## 5. Permission Tokens
Requests must include a short-lived `permission_token` issued by the Permission Manager.

## 6. Tool Subscription Events
Tools can emit events that the Agent or System can subscribe to (e.g., `gps.on_location_change`).

## 7. Reversibility Signals
A new metadata field `reversibility` in tool definitions:
*   `none`: Action cannot be undone (e.g., sending an SMS).
*   `undo_supported`: Tool provides an `undo` method.
*   `transactional`: Action can be rolled back automatically.
