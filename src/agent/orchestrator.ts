import * as net from 'net';

interface ToolCall {
    tool: string;
    args: any;
}

export class AgentOrchestrator {
    private client: net.Socket | null = null;

    private async getClient(): Promise<net.Socket> {
        if (this.client && !this.client.destroyed) {
            return this.client;
        }
        return new Promise((resolve, reject) => {
            const client = net.connect('/tmp/mcpd.sock', () => {
                this.client = client;
                resolve(client);
            });
            client.on('error', (err) => {
                console.error('Socket error:', err);
                reject(err);
            });
        });
    }

    async processIntent(userInput: string) {
        console.log(`Processing intent: ${userInput}`);

        // 1. Plan
        const plan = await this.generatePlan(userInput);

        // 2. Execute
        for (const step of plan) {
            await this.executeStep(step);
        }
    }

    async shutdown() {
        if (this.client) {
            this.client.end();
            this.client = null;
        }
    }

    private async generatePlan(input: string): Promise<ToolCall[]> {
        // Simulated LLM planning
        return [
            { tool: "camera.capture", args: {} },
            { tool: "contacts.resolve", args: { query: "Mom" } },
            { tool: "messages.send", args: { text: "Here is the photo" } }
        ];
    }

    private async executeStep(step: ToolCall) {
        console.log(`Executing tool: ${step.tool}`);

        const mcpRequest = {
            jsonrpc: "2.0",
            method: step.tool,
            params: step.args,
            id: Date.now(),
            context: {
                confidence: 0.9,
                is_user_initiated: true
            }
        };

        console.log(`Sending MCP Request to mcpd: ${JSON.stringify(mcpRequest)}`);

        return new Promise<void>(async (resolve, reject) => {
            try {
                const client = await this.getClient();
                client.once('data', (data) => {
                    console.log(`Tool Result: ${data.toString()}`);
                    resolve();
                });
                client.write(JSON.stringify(mcpRequest));
            } catch (err) {
                reject(err);
            }
        });
    }
}
