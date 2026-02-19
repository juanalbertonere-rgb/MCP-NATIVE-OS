# Phased Development Roadmap

## Phase 1: Prototype on Android (Current)
*   **Goal**: Demonstrate the agent-first interaction loop.
*   **Deliverables**:
    *   `mcpd` skeleton (Rust).
    *   Agent Launcher (Kotlin) with simulated orchestration.
    *   Legacy Bridge prototype (Accessibility Service).
    *   Basic tool set (Camera, Contacts, SMS).

## Phase 2: System MCP Services
*   **Goal**: Integrate MCP into the Android system core.
*   **Deliverables**:
    *   UDS-based IPC for `mcpd`.
    *   Full Tool Registry with dynamic registration.
    *   Initial Risk Classification Engine implementation.
    *   Local intent parsing (Basic).

## Phase 3: Agent Launcher & Runtime
*   **Goal**: Transition to a fully functional Agent-first launcher.
*   **Deliverables**:
    *   Advanced Planning & Memory (Vector DB).
    *   Multimodal input (Voice/Vision).
    *   Deterministic + AI safety layers.
    *   Developer SDK beta release.

## Phase 4: SDK & Developer Ecosystem
*   **Goal**: Enable 3rd party developers to build MCP-native apps.
*   **Deliverables**:
    *   Stable SDK & Documentation.
    *   Tool generation templates.
    *   App store/registry for MCP modules.
    *   AOSP compatibility patch set.

## Phase 5: Custom ROM (MCP-OS)
*   **Goal**: A standalone OS distribution.
*   **Deliverables**:
    *   Deeply integrated `mcpd` at the init/service layer.
    *   Minimalist system image (removing traditional UI bloat).
    *   Hardware-level safety verification.

## Phase 6: Production Hardware
*   **Goal**: Specialized hardware optimized for the Agent-first paradigm.
*   **Deliverables**:
    *   Dedicated AI acceleration for planning/inference.
    *   Physical "Agent Action" hardware buttons/triggers.
    *   Privacy-first hardware kill-switches.
