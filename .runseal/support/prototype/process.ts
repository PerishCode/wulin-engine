import { fail, type Json, root, string } from "../canonical-runtime.ts";
import { pressPrototypeEscape } from "./input.ts";

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

export async function escapeExit(
    executable: string,
    config: string,
    label: string,
): Promise<Json> {
    const started = performance.now();
    const child = new Deno.Command(executable, {
        args: [`--bootstrap=${config}`],
        cwd: root,
        stdout: "piped",
        stderr: "piped",
    }).spawn();
    const stderr = new Response(child.stderr).text();
    const reader = child.stdout
        .pipeThrough(new TextDecoderStream())
        .getReader();
    let readiness: Json;
    let nativeInput: Json;
    let status: Deno.CommandStatus;
    try {
        readiness = JSON.parse(await readinessLine(reader)) as Json;
        if (readiness.role !== "prototype") fail(`${label} emitted the wrong readiness role`);
        nativeInput = await pressPrototypeEscape(child.pid);
        const exit = await Promise.race([
            child.status.then((value) => ({ kind: "status" as const, value })),
            new Promise<{ kind: "timeout" }>((resolve) =>
                setTimeout(() => resolve({ kind: "timeout" }), 10_000)
            ),
        ]);
        if (exit.kind === "timeout") fail(`${label} did not exit after the Escape press edge`);
        status = exit.value;
        if (!status.success) fail(`${label} exited with code ${status.code}`);
    } catch (error) {
        try {
            child.kill();
        } catch {
            // The process may already have exited while the failure was being reported.
        }
        try {
            await child.status;
        } catch {
            // Preserve the original acceptance failure after best-effort process cleanup.
        }
        throw error;
    } finally {
        try {
            await reader.cancel();
        } catch {
            // The process may close its stream before the evidence reader is cancelled.
        }
    }
    const stderrText = (await stderr).trim().slice(-4_096);
    if (stderrText) fail(`${label} emitted stderr: ${stderrText}`);
    return {
        label,
        processId: Number(string(readiness, "instance_id")),
        elapsedMs: performance.now() - started,
        exitCode: status.code,
        stderr: stderrText,
        nativeInput,
        readiness,
    };
}
