# Legacy App Bridge (Accessibility Wrapper)

## Overview
The Legacy App Bridge allows the Agent to interact with standard Android apps that do not yet support MCP. It uses the Android Accessibility API to "see" and "touch" the UI of legacy applications.

## Components

### 1. LegacyAppAdapter (AccessibilityService)
*   **Semantic Tree Extraction**: Regularly scrapes the active app's UI tree.
*   **Node-to-Tool Mapping**: Translates UI elements into virtual MCP tools.
    *   Example: A button with text "Send" becomes the `legacy.app.click_send` tool.
*   **Action Injection**: Triggers clicks, text input, and scrolls on behalf of the Agent.

### 2. UI-to-MCP Translation Layer
Uses a lightweight heuristic or local LLM to assign semantic meaning to UI nodes.
*   *Input*: AccessibilityNodeInfo tree.
*   *Output*: MCP Tool Manifest for the current screen.

## Workflow
1.  User: "Send 'Hello' in WhatsApp."
2.  Agent: Sees no `whatsapp.send` native tool.
3.  Agent: Invokes `legacy.bridge.launch_app("WhatsApp")`.
4.  Bridge: Launches app and waits for UI to stabilize.
5.  Bridge: Reports virtual tools: `legacy.whatsapp.input_text`, `legacy.whatsapp.click_send`.
6.  Agent: Invokes virtual tools in sequence.
7.  Bridge: Translates `click_send` to `node.performAction(ACTION_CLICK)`.

## Limitations & Risks
*   **Fragility**: UI changes can break semantic mappings.
*   **Performance**: Accessibility scraping is heavier than native tool calls.
*   **Security**: Requires high-level Android permissions; must be strictly sandboxed and audited.
