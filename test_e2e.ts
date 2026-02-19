import { AgentOrchestrator } from './src/agent/orchestrator.js';

async function run() {
    const orchestrator = new AgentOrchestrator();
    try {
        await orchestrator.processIntent("Take a photo and send it to Mom");
        await orchestrator.shutdown();
        console.log("E2E Test completed successfully");
        process.exit(0);
    } catch (err) {
        console.error("E2E Test failed:", err);
        process.exit(1);
    }
}

run();
