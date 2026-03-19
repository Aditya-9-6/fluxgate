import { FluxGateClient } from '../sdks/typescript/fluxgate';

async function researchAgentWorkflow() {
    const fg = new FluxGateClient(process.env.FLUXGATE_API_KEY!);
    const sessionId = "research-session-" + Date.now();

    console.log("Agent 1: Gathering Intelligence...");
    const step1 = await fg.process("Who is the CEO of OpenAI?", sessionId);
    console.log("Response:", step1.response);

    console.log("Agent 2: Summarizing Findings...");
    const step2 = await fg.process(`Summarize this context into exactly 3 words: ${step1.response}`, sessionId);
    console.log("Final Report:", step2.response);
}

researchAgentWorkflow().catch(console.error);
