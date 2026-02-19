# Android Launcher & Agent UI Design

## Overview
The Android layer serves as the primary interface between the human and the Agent. It replaces the traditional app grid with an agent-driven interaction model.

## Core Components

### 1. Agent Launcher (Activity)
The main entry point. It displays:
*   **Conversational Interface**: Chat history and voice input button.
*   **Agent Visualization**: Real-time feedback on what the agent is "thinking" or "doing" (planning steps).
*   **Active Tasks**: Progress bars for background tool orchestrations.
*   **Minimal Fallback Dock**: Quick access to critical manual tools (Phone, Settings).

### 2. MCP System Service (Android Service)
A persistent background service that:
*   Maintains the UDS connection to `mcpd`.
*   Acts as a bridge for Android-specific tools (e.g., Contacts, SMS).
*   Registers system-level Android tools with `mcpd`.

### 3. Intent Verification Dialogs
Custom UI components for high-risk actions.
*   "The agent wants to send $50 to Mom. Confirm?"
*   Displays the reasoning/context for the action.

### 4. Legacy Bridge Service
An `AccessibilityService` that monitors other apps and exposes their UI state to the Agent via MCP.

## Interaction Flow
1.  User provides voice/text input to the Launcher.
2.  Launcher sends intent to the Agent Runtime.
3.  Agent Runtime generates a plan and calls tools via `mcpd`.
4.  If a tool is "High Risk", `mcpd` triggers an Intent Verification Dialog in the Launcher.
5.  User confirms, and execution continues.
