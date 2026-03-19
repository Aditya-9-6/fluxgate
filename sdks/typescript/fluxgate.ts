export class FluxGateClient {
    private apiKey: string;
    private baseUrl: string;

    constructor(apiKey: string, baseUrl: string = "http://localhost:8080/v1") {
        this.apiKey = apiKey;
        this.baseUrl = baseUrl;
    }

    async process(prompt: string, sessionId?: string): Promise<{ status: string, response: string }> {
        const payload: any = { prompt };
        if (sessionId) payload.session_id = sessionId;

        const res = await fetch(`${this.baseUrl}/process`, {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${this.apiKey}`,
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(payload)
        });

        if (!res.ok) {
            throw new Error(`FluxGate Error: ${res.statusText}`);
        }

        return await res.json();
    }
}
