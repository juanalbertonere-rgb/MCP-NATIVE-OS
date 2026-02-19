# Tool Registry Specification

## Overview
The Tool Registry is the central catalog of all capabilities available to the Agent.

## Tool Definition Schema
Every tool (native or legacy) must provide a manifest:

```json
{
  "name": "namespace.tool_name",
  "description": "Clear, semantic description for the LLM planner.",
  "parameters": {
    "type": "object",
    "properties": {
      "param1": { "type": "string", "description": "..." }
    },
    "required": ["param1"]
  },
  "risk_category": "LOW | MEDIUM | HIGH",
  "capabilities_required": ["android.permission.CAMERA"],
  "reversibility": "none | undo_supported"
}
```

## Discovery Mechanism
*   **Static Registration**: System tools registered at boot.
*   **Dynamic Registration**: Apps register their tools when installed or launched via `mcpd`.
*   **Legacy Mapping**: The Legacy Bridge dynamically generates tool definitions for standard Android UI components (e.g., "Click the 'Send' button in WhatsApp").

## Tool Versioning
Tools must support semantic versioning to ensure the Planner uses compatible schemas.
