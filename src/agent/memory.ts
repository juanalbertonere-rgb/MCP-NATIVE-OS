export class MemoryStore {
    private context: Map<string, any> = new Map();

    set(key: string, value: any) {
        console.log(`[MemoryStore] Setting ${key}`);
        this.context.set(key, value);
    }

    get(key: string): any {
        return this.context.get(key);
    }

    getAll(): Record<string, any> {
        return Object.fromEntries(this.context);
    }
}
