export interface ToolCall {
    tool: string;
    args: any;
}

export interface Tool {
    name: string;
    provider: string;
    risk_level: string;
    capabilities: string[];
}

export class AgentPlanner {
    async plan(userIntent: string, registry: Tool[]): Promise<ToolCall[]> {
        console.log(`[Planner] Generating plan for: "${userIntent}"`);
        const plan: ToolCall[] = [];
        const intent = userIntent.toLowerCase();

        // Simple heuristic-based planning
        if (intent.includes("take") || intent.includes("photo") || intent.includes("camera")) {
            if (registry.some(t => t.name === "camera.capture")) {
                plan.push({ tool: "camera.capture", args: {} });
            }
        }

        if (intent.includes("mom") || intent.includes("contact")) {
            if (registry.some(t => t.name === "contacts.resolve")) {
                plan.push({ tool: "contacts.resolve", args: { query: "Mom" } });
            }
        }

        if (intent.includes("send") || intent.includes("message")) {
            if (registry.some(t => t.name === "messages.send")) {
                // Heuristic: If we are sending a message after a capture/resolve,
                // we'll expect to use memory in the executor, but for now we set base args
                plan.push({ tool: "messages.send", args: { text: "Here is the photo" } });
            }
        }

        if (plan.length === 0) {
            console.warn("[Planner] No tools matched the intent, providing default plan.");
            // Fallback for demo purposes if no keywords match
            return [
                { tool: "camera.capture", args: {} },
                { tool: "contacts.resolve", args: { query: "Mom" } },
                { tool: "messages.send", args: { text: "Default message" } }
            ];
        }

        return plan;
    }
}
