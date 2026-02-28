import * as net from 'net';
import * as readline from 'readline';
import { AgentPlanner, Tool, ToolCall } from './planner.js';
import { MemoryStore } from './memory.js';

export class AgentOrchestrator {
    private client: net.Socket | null = null;
    private pendingRequests: Map<number | string, { resolve: (data: any) => void, reject: (err: any) => void }> = new Map();
    private buffer: string = '';
    private rl: readline.Interface;
    private confirmationQueue: ((answer: boolean) => void)[] = [];
    private planner: AgentPlanner;
    private memory: MemoryStore;

    constructor() {
        this.planner = new AgentPlanner();
        this.memory = new MemoryStore();
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

        // 1. Fetch registry
        const tools = await this.listTools();

        // 2. Plan
        const plan = await this.planner.plan(userInput, tools);

        // 3. Execute
        for (let i = 0; i < plan.length; i++) {
            await this.executeStep(plan[i], i + 1, plan.length);
        }
    }

    async listTools(): Promise<Tool[]> {
        const id = Date.now() + Math.random();
        const request = {
            jsonrpc: "2.0",
            method: "system.list_tools",
            params: {},
            id: id
        };
        const response = await this.sendRequest(request);
        if (response.error) {
            throw new Error(`Failed to list tools: ${JSON.stringify(response.error)}`);
        }
        return response.result;
    }

    async getAuditLog(): Promise<any[]> {
        const id = Date.now() + Math.random();
        const request = {
            jsonrpc: "2.0",
            method: "system.audit_log",
            params: {},
            id: id
        };
        const response = await this.sendRequest(request);
        if (response.error) {
            throw new Error(`Failed to fetch audit log: ${JSON.stringify(response.error)}`);
        }
        return response.result;
    }

    private async sendRequest(request: any): Promise<any> {
        const client = await this.getClient();
        return new Promise<any>((resolve, reject) => {
            this.pendingRequests.set(request.id, { resolve, reject });
            client.write(JSON.stringify(request) + '\n');
        });
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

    private async executeStep(step: ToolCall, stepIndex: number, totalSteps: number) {
        console.log(`[${stepIndex}/${totalSteps}] Executing: ${step.tool}`);

        // Enrich args from memory
        const enrichedArgs = { ...step.args };
        if (step.tool === "messages.send") {
            const contact = this.memory.get("last_resolved_contact");
            const photo = this.memory.get("last_captured_image");
            if (contact) {
                enrichedArgs.recipient = contact.phone;
            }
            if (photo) {
                enrichedArgs.attachment = photo.image_url;
            }
        }

        let retries = 2;
        while (retries >= 0) {
            try {
                const id = Date.now() + Math.random();
                const mcpRequest = {
                    jsonrpc: "2.0",
                    method: step.tool,
                    params: enrichedArgs,
                    id: id,
                    context: {
                        confidence: 0.5, // Lowered to trigger confirmation in tests
                        is_user_initiated: true,
                        mcpd_version: "1.0"
                    }
                };

                console.log(`Sending MCP Request to mcpd: ${JSON.stringify(mcpRequest)}`);

                const client = await this.getClient();
                const response = await this.sendRequest(mcpRequest);

                console.log(`Tool Result: ${JSON.stringify(response)}`);
                if (response.error) {
                    if (response.error.code === -32000 && response.error.data?.confirmation_token) {
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

                        const confirmResponse = await this.sendRequest(confirmRequest);

                        console.log(`Confirmation Result: ${JSON.stringify(confirmResponse)}`);
                        if (confirmResponse.error) {
                            throw new Error(`MCP Confirmation Error: ${JSON.stringify(confirmResponse.error)}`);
                        }
                        // After confirmation, we store the original response if it was actually executed
                        // In our current mcpd it returns success immediately after system.confirm
                        this.storeResultInMemory(step.tool, confirmResponse.result);
                        return;
                    }
                    throw new Error(`MCP Error: ${JSON.stringify(response.error)}`);
                }

                this.storeResultInMemory(step.tool, response.result);
                return; // Success
            } catch (err) {
                if (retries === 0) {
                    console.error(`FATAL: Tool ${step.tool} failed after retries`);
                    throw err;
                }
                retries--;
                console.warn(`Retrying ${step.tool}... (${retries + 1} left)`);
                await new Promise(r => setTimeout(r, 500));
            }
        }
    }

    private storeResultInMemory(tool: string, result: any) {
        if (tool === "camera.capture") {
            this.memory.set("last_captured_image", result);
        } else if (tool === "contacts.resolve") {
            this.memory.set("last_resolved_contact", result.contact);
        }
    }
}
