import { fail } from "../canonical-runtime.ts";

export async function readinessLine(
    reader: ReadableStreamDefaultReader<string>,
): Promise<string> {
    const deadline = performance.now() + 30_000;
    let buffered = "";
    while (performance.now() < deadline) {
        const remaining = Math.max(1, deadline - performance.now());
        const result = await Promise.race([
            reader.read(),
            new Promise<{ done: true; value: undefined }>((resolve) =>
                setTimeout(() => resolve({ done: true, value: undefined }), remaining)
            ),
        ]);
        if (result.done) {
            if (buffered.trim()) return buffered.trim();
            fail("prototype exited or timed out before readiness");
        }
        buffered += result.value;
        const newline = buffered.indexOf("\n");
        if (newline >= 0) return buffered.slice(0, newline).trim();
    }
    fail("prototype readiness timeout expired");
}
