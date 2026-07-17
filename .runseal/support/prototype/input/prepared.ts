import { fail, type Json, root } from "../../canonical-runtime.ts";
import type {
    PreparedPrototypeWindowAction,
    PrototypeKeyTransition,
    PrototypeWindowAction,
} from "./mod.ts";

type ExpectedWindowAction = {
    action: PrototypeWindowAction;
    processId: number | null;
    nativeKeys: PrototypeKeyTransition[];
    requireVisible: boolean;
    keyDelays: number[];
    exitAfterLastMilliseconds: number;
    atomicBatch: boolean;
    expectedMessages: string[];
};

export async function startPreparedWindowAction(
    script: string,
    expected: ExpectedWindowAction,
): Promise<PreparedPrototypeWindowAction> {
    const child = new Deno.Command("pwsh", {
        args: ["-NoProfile", "-NonInteractive", "-Command", script],
        cwd: root,
        stdout: "piped",
        stderr: "piped",
    }).spawn();
    const stderr = new Response(child.stderr).text();
    const reader = child.stdout
        .pipeThrough(new TextDecoderStream())
        .getReader();
    let readyTimeout: ReturnType<typeof setTimeout> | undefined;
    const marker = await Promise.race([
        helperReadyLine(reader).then((value) => ({ kind: "marker" as const, value })),
        new Promise<{ kind: "timeout" }>((resolve) =>
            readyTimeout = setTimeout(() => resolve({ kind: "timeout" }), 20_000)
        ),
    ]);
    if (readyTimeout !== undefined) clearTimeout(readyTimeout);
    if (marker.kind === "timeout") {
        try {
            child.kill();
        } catch {
            // The helper may have failed immediately before the timeout was observed.
        }
        await child.status;
        fail("prototype native input helper did not become ready");
    }
    if (marker.value !== "prototype-native-helper-ready-v1") {
        try {
            child.kill();
        } catch {
            // The helper may already have exited with its diagnostic.
        }
        const status = await child.status;
        fail(
            `prototype native input helper emitted invalid readiness ${marker.value}; ` +
                `status=${status.code} stderr=${(await stderr).trim().slice(-4_096)}`,
        );
    }
    return {
        evidence: completePrototypeWindowAction(child, reader, stderr, expected),
    };
}

async function helperReadyLine(
    reader: ReadableStreamDefaultReader<string>,
): Promise<string> {
    let value = "";
    while (true) {
        const chunk = await reader.read();
        if (chunk.done) fail("prototype native input helper exited before readiness");
        value += chunk.value;
        const newline = value.indexOf("\n");
        if (newline < 0) continue;
        if (value.slice(newline + 1).trim() !== "") {
            fail("prototype native input helper emitted evidence before child launch");
        }
        return value.slice(0, newline).trimEnd();
    }
}

async function completePrototypeWindowAction(
    child: Deno.ChildProcess,
    reader: ReadableStreamDefaultReader<string>,
    stderr: Promise<string>,
    expected: ExpectedWindowAction,
): Promise<Json> {
    let stdout = "";
    while (true) {
        const chunk = await reader.read();
        if (chunk.done) break;
        stdout += chunk.value;
    }
    const status = await child.status;
    const stderrText = (await stderr).trim();
    if (!status.success) {
        fail(`prototype native input failed with ${status.code}: ${stderrText.slice(-4_096)}`);
    }
    if (stderrText !== "") {
        fail(`prototype native input emitted stderr: ${stderrText.slice(-4_096)}`);
    }
    const evidence = JSON.parse(stdout.trim()) as Json;
    if (
        evidence.schema !== "prototype-native-window-action-v3" ||
        evidence.action !== expected.action ||
        typeof evidence.processId !== "number" ||
        !Number.isSafeInteger(evidence.processId) ||
        evidence.processId <= 0 ||
        (expected.processId !== null && evidence.processId !== expected.processId) ||
        evidence.activated !== (expected.action !== "close") ||
        evidence.closeRequested !== (expected.action === "close") ||
        evidence.requiredVisible !== expected.requireVisible ||
        (expected.requireVisible && evidence.windowWasVisible !== true) ||
        JSON.stringify(evidence.keys) !== JSON.stringify(expected.nativeKeys) ||
        JSON.stringify(evidence.messages) !== JSON.stringify(expected.expectedMessages) ||
        JSON.stringify(evidence.delaysBeforeKeysMilliseconds) !==
            JSON.stringify(expected.keyDelays) ||
        !Array.isArray(evidence.keyPostIntervalsMilliseconds) ||
        evidence.keyPostIntervalsMilliseconds.length !==
            Math.max(0, expected.nativeKeys.length - 1) ||
        evidence.keyPostIntervalsMilliseconds.some((interval, index) =>
            typeof interval !== "number" ||
            interval < expected.keyDelays[index + 1]
        ) ||
        evidence.exitAfterLastMilliseconds !== expected.exitAfterLastMilliseconds ||
        evidence.atomicBatch !== expected.atomicBatch ||
        (expected.atomicBatch
            ? typeof evidence.batchThreadId !== "number" ||
                !Number.isSafeInteger(evidence.batchThreadId) ||
                evidence.batchThreadId <= 0 ||
                typeof evidence.batchSpanMilliseconds !== "number" ||
                evidence.batchSpanMilliseconds < 0 ||
                evidence.batchSpanMilliseconds > 50
            : evidence.batchThreadId !== null || evidence.batchSpanMilliseconds !== null) ||
        (expected.exitAfterLastMilliseconds === 0
            ? evidence.exitIntervalMilliseconds !== null
            : typeof evidence.exitIntervalMilliseconds !== "number" ||
                evidence.exitIntervalMilliseconds < expected.exitAfterLastMilliseconds)
    ) {
        fail(
            `prototype native window action evidence diverged: ${
                JSON.stringify({ expected, evidence })
            }`,
        );
    }
    return evidence;
}
