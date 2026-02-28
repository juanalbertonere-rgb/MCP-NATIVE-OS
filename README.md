# MCP-Native Mobile Operating System

An agent-first mobile operating system where AI agents are first-class citizens and all system capabilities are exposed through the Model Context Protocol (MCP).

## Project Status
**Phase**: Architecture & Scaffolding (In Progress)

## Quick Start
1. `cargo build && ./verify_e2e.sh`
2. Si pasa: Sistema funciona end-to-end
3. Ver logs en `mcpd.log`, `test_output.log`

## Architecture Now
- **mcpd**: Tool registry + RPC broker + audit log
- **Planner**: Heurístico (intent → tool sequence)
- **Executor**: Con retry + memory pass-through
- **Test**: Validación real del flujo completo

## Documentation
- [Architecture Overview](docs/ARCHITECTURE_OVERVIEW.md)
- [System Daemon Specs](docs/system/README.md)
- [Roadmap](docs/roadmap/PHASED_ROADMAP.md)
- [Implementation Status](IMPLEMENTATION_STATUS.md)

## Core Stack
*   **OS Base**: Android (AOSP)
*   **System Daemon**: Rust
*   **Application Framework**: Kotlin
*   **Protocol**: MCP (JSON-RPC 2.0 with extensions)

## Vision
Transforming mobile computing from a "tapestry of apps" into a unified "intelligent system" where intent drives execution.
