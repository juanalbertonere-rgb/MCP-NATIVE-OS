# MCP-Native Mobile OS: Architecture Overview

## Vision
The MCP-Native Mobile OS is an agent-first operating system designed for the era of AI orchestration. It shifts the primary computing paradigm from manual UI navigation to intent-based execution via the Model Context Protocol (MCP).

## Core Paradigm
**Human → AI Agent → MCP System Bus (mcpd) → Apps/Hardware**

The system prioritizes agent mediation while maintaining a hybrid model that allows for traditional GUI fallback and legacy app compatibility.

## System Layers

### Layer 1: Kernel & HAL
*   **Base**: AOSP (Android Open Source Project) / Linux Kernel.
*   **HAL**: Standard Hardware Abstraction Layer for camera, sensors, radio, etc.

### Layer 2: Core System Services (mcpd)
*   **mcpd (MCP Daemon)**: The central nervous system. A high-performance Rust daemon handling tool routing and discovery.
*   **Tool Registry**: Centralized database of available system and app tools.
*   **Permission Manager**: Enforces security and user consent for tool execution.
*   **Event Bus**: Asynchronous system-wide communication.

### Layer 3: Agent Runtime
*   **Orchestrator**: Plans and executes complex multi-step tasks.
*   **Memory Store**: Long-term and short-term context/user preference storage.
*   **Safety Layer**: Risk Classification Engine and Intent Verification.

### Layer 4: MCP Application Framework
*   **MCP-Native Apps**: Apps that expose capabilities primarily as MCP tools.
*   **Legacy Bridge**: Accessibility-based wrappers that translate standard Android UI to MCP tools.

### Layer 5: User Interaction Layer
*   **Agent UI**: A conversational/voice-first launcher.
*   **System Dashboard**: For monitoring agent actions and managing permissions.
*   **Minimal Fallback GUI**: Standard Android-like interaction for manual overrides.

## Key Design Principles
1.  **Safety First**: High-risk actions require explicit confirmation; reversible execution where possible.
2.  **MCP-Centric**: Every system capability is an MCP tool.
3.  **Hybrid Interaction**: Seamless transition between agent-led and user-led navigation.
4.  **Ecosystem Compatibility**: Support existing Android apps via the Legacy Bridge.

## Directory Map
*   `/docs/architecture`: Detailed design documents.
*   `/docs/system`: Core services and daemon specs.
*   `/docs/mcp`: Protocol extensions and schema definitions.
*   `/docs/security`: Permission models and safety architecture.
*   `/docs/agent`: Agent runtime and intelligence specs.
*   `/docs/sdk`: Developer tools and SDK guides.
*   `/docs/roadmap`: Phased implementation plan.
*   `/docs/examples`: Sample tools and app manifests.
