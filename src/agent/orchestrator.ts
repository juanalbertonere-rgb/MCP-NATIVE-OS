import * as net from 'net';
import * as readline from 'readline';

interface ToolCall {
    tool: string;
    args: any;
}

export class AgentOrchestrator {
    private client: net.Socket | null = null;
    private pendingRequests: Map<number | string, { resolve: (data: any) => void, reject: (err: any) => void }> = new Map();
    private buffer: string = '';
    private rl: readline.Interface;
    private confirmationQueue: ((answer: boolean) => void)[] = [];

    constructor() {
        this.rl = readline.createInterface({
            input: process.stdin,
            output: process.stdout,
            terminal: false
        });

        this.rl.on('line', (line) => {
            const resolve = this.confirmationQueue.shift();
            if (resolve) {
                resolve(line.trim().toLowerCase() === 'y');
            }
        });
    }

    private async getClient(retries = 3): Promise<net.Socket> {
        if (this.client && !this.client.destroyed) {
            return this.client;
        }

        for (let i = 0; i < retries; i++) {
            try {
                return await new Promise((resolve, reject) => {
                    const client = net.connect('/tmp/mcpd.sock', () => {
                        console.log('Connected to mcpd');
                        this.client = client;
                        this.setupSocketListeners(client);
                        resolve(client);
                    });
                    client.on('error', (err) => {
                        reject(err);
                    });
                });
            } catch (err) {
                console.error(`Connection attempt ${i + 1} failed: ${err}`);
                if (i === retries - 1) throw err;
                await new Promise(r => setTimeout(r, 1000));
            }
        }
        throw new Error('Failed to connect to mcpd');
    }

    private setupSocketListeners(client: net.Socket) {
        client.on('data', (data) => {
            this.buffer += data.toString();

            let boundary = this.buffer.indexOf('\n');
            while (boundary !== -1) {
                const message = this.buffer.slice(0, boundary).trim();
                this.buffer = this.buffer.slice(boundary + 1);

                if (message) {
                    try {
                        const response = JSON.parse(message);
                        const id = response.id;
                        if (id !== undefined && this.pendingRequests.has(id)) {
                            const { resolve } = this.pendingRequests.get(id)!;
                            this.pendingRequests.delete(id);
                            resolve(response);
                        }
                    } catch (err) {
                        console.error('Error parsing response:', err, message);
                    }
                }
                boundary = this.buffer.indexOf('\n');
            }
        });

        client.on('close', () => {
            console.log('Socket connection closed');
            this.client = null;
            // Reject all pending requests
            for (const [id, { reject }] of this.pendingRequests) {
                reject(new Error('Connection closed'));
            }
            this.pendingRequests.clear();
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
        this.rl.close();
    }

    private askUserConfirmation(): Promise<boolean> {
        return new Promise((resolve) => {
            process.stdout.write('Approve this action? (y/n): ');
            this.confirmationQueue.push(resolve);
        });
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

        const id = Date.now() + Math.random();
        const mcpRequest = {
            jsonrpc: "2.0",
            method: step.tool,
            params: step.args,
            id: id,
            context: {
                confidence: 0.5, // Lowered to trigger confirmation in tests
                is_user_initiated: true,
                mcpd_version: "1.0"
            }
        };

        console.log(`Sending MCP Request to mcpd: ${JSON.stringify(mcpRequest)}`);

        try {
            const client = await this.getClient();
            const response = await new Promise<any>((resolve, reject) => {
                this.pendingRequests.set(id, { resolve, reject });
                client.write(JSON.stringify(mcpRequest) + '\n');
            });

            console.log(`Tool Result: ${JSON.stringify(response)}`);
            if (response.error) {
                if (response.error.code === -32000) {
                    const token = response.error.data.confirmation_token;
                    const reason = response.error.data.reason;
                    console.log(`\n⚠️  CONFIRMATION REQUIRED: ${reason}`);

                    const confirmed = await this.askUserConfirmation();
                    if (!confirmed) {
                        throw new Error("User denied confirmation");
                    }

                    console.log(`Confirming with token: ${token}`);

                    const confirmId = Date.now() + Math.random();
                    const confirmRequest = {
                        jsonrpc: "2.0",
                        method: "system.confirm",
                        params: { confirmation_token: token },
                        id: confirmId
                    };

                    const confirmResponse = await new Promise<any>((resolve, reject) => {
                        this.pendingRequests.set(confirmId, { resolve, reject });
                        client.write(JSON.stringify(confirmRequest) + '\n');
                    });

                    console.log(`Confirmation Result: ${JSON.stringify(confirmResponse)}`);
                    if (confirmResponse.error) {
                        throw new Error(`MCP Confirmation Error: ${JSON.stringify(confirmResponse.error)}`);
                    }
                    return; // Success after confirmation
                }
                throw new Error(`MCP Error: ${JSON.stringify(response.error)}`);
            }
        } catch (err) {
            console.error(`Step failed: ${err}`);
            throw err;
        }
    }
}
