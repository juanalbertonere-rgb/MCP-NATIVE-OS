# Security & Permission Model

## Overview
The MCP-Native OS employs a zero-trust, capability-based security model. Traditional "app-level" permissions are replaced/augmented by "tool-level" and "intent-level" permissions.

## Risk Classification Engine (Hybrid)

Each tool call is evaluated against a two-layer risk engine:

### Layer 1: Deterministic Baseline
Tools are statically categorized in their manifests:
*   **LOW**: Data reads, non-personal queries (e.g., `clock.get_time`, `weather.get`).
    *   *Action*: Auto-allow if permission is granted once.
*   **MEDIUM**: Modifying non-critical data, sending communication (e.g., `messages.send`, `calendar.create`).
    *   *Action*: Require background notification or session-based approval.
*   **HIGH**: Financial transactions, data deletion, system modification, PII sharing.
    *   *Action*: Mandatory real-time user confirmation with Intent Verification.

### Layer 2: AI Contextual Evaluation
The Risk Engine analyzes the agent's plan:
*   **Anomaly Detection**: Is the agent trying to send 1000 messages at once?
*   **Ambiguity Detection**: Is the user intent "Send the file" but there are 5 files?
*   **Context Mismatch**: Is the agent accessing contacts while the user asked for a recipe?

## Intent Verification Flow
Before a HIGH risk tool is executed:
1.  `mcpd` pauses the request.
2.  The Risk Engine generates a "Verification Request".
3.  The Launcher displays a "Human-in-the-loop" dialog.
4.  The dialog shows:
    *   **Action**: "Send $50"
    *   **Recipient**: "Mom"
    *   **Agent Reason**: "You asked to pay Mom back for lunch."
5.  User must explicitly confirm (Biometric/Button).

## Capability Scoping
Tools do not get full access to the system. Each tool invocation is scoped to the specific resources it needs (e.g., `file.read` is limited to the specific file URI requested).

## Reversible Execution
Whenever possible, tools must implement a `rollback` or `undo` state.
*   `file.delete` moves to a system-wide "Agent Trash" first.
*   `settings.modify` creates a recovery checkpoint.
