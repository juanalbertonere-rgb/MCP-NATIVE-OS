import { AgentOrchestrator } from './src/agent/orchestrator.js';

async function run() {
    const orchestrator = new AgentOrchestrator();
    try {
        // 1. Fetch registry
        const tools = await orchestrator.listTools();
        console.log(`✓ Registry loaded: ${tools.length} tools`);
        if (tools.length < 4) {
            throw new Error(`Expected at least 4 tools, got ${tools.length}`);
        }

        // 2. Execute intent
        await orchestrator.processIntent("Take a photo and send it to Mom");

        // 3. Fetch audit log
        const auditLog = await orchestrator.getAuditLog();
        console.log(`✓ Audit log fetched: ${auditLog.length} entries`);
        if (auditLog.length < 3) {
            throw new Error(`Expected at least 3 transactions in audit log, got ${auditLog.length}`);
        }

        // 4. Verify memory consistency / Audit log content
        const methods = auditLog.map(t => t.method);
        if (!methods.includes("camera.capture")) throw new Error("camera.capture not found in audit log");
        if (!methods.includes("contacts.resolve")) throw new Error("contacts.resolve not found in audit log");
        if (!methods.includes("messages.send")) throw new Error("messages.send not found in audit log");

        // Verify message.send had enriched args (recipient and attachment)
        const sendMsg = auditLog.find(t => t.method === "messages.send");
        if (!sendMsg.params.recipient || !sendMsg.params.attachment) {
            throw new Error("messages.send missing enriched parameters from memory");
        }

        console.log("✓ All validations passed");

        await orchestrator.shutdown();
        console.log("E2E Test completed successfully");
        process.exit(0);
    } catch (err) {
        console.error("E2E Test failed:", err);
        process.exit(1);
    }
}

run();
