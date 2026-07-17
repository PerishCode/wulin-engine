import { fail, type Json, number, object, root } from "../canonical-runtime.ts";
import { holdPrototypeBoundaryRun } from "./input/actions.ts";
import { readinessLine } from "./sessions/mod.ts";

export const BOUNDARY_HOLD_MILLISECONDS = 15_000;

export function boundaryRunInputInvariant(launch: Json): Json {
    const processId = number(launch, "processId");
    const sequence = object(launch, "nativeInput");
    const intervals = sequence.keyPostIntervalsMilliseconds;
    if (
        sequence.schema !== "prototype-native-window-action-v4" ||
        sequence.action !== "input" ||
        sequence.processId !== processId ||
        sequence.requiredVisible !== true ||
        sequence.windowWasVisible !== true ||
        JSON.stringify(sequence.keys) !==
            JSON.stringify([
                { key: "Shift", virtualKey: 0x10, down: true },
                { key: "W", virtualKey: 0x57, down: true },
            ]) ||
        JSON.stringify(sequence.messages) !==
            JSON.stringify([
                "WM_SETFOCUS",
                "WM_KEYDOWN:Shift",
                "WM_KEYDOWN:W",
            ]) ||
        JSON.stringify(sequence.delaysBeforeKeysMilliseconds) !==
            JSON.stringify([0, 0]) ||
        !Array.isArray(intervals) ||
        intervals.length !== 1 ||
        intervals.some((interval) =>
            typeof interval !== "number" || interval < 0 || interval > 50
        ) ||
        sequence.atomicBatch !== true ||
        number(sequence, "atomicPrefixLength") !== 2 ||
        typeof sequence.batchThreadId !== "number" ||
        !Number.isSafeInteger(sequence.batchThreadId) ||
        sequence.batchThreadId <= 0 ||
        typeof sequence.batchSpanMilliseconds !== "number" ||
        sequence.batchSpanMilliseconds < 0 ||
        sequence.batchSpanMilliseconds > 50 ||
        number(sequence, "exitAfterLastMilliseconds") !== 0 ||
        sequence.exitIntervalMilliseconds !== null
    ) fail("prototype native finite-boundary Run input evidence diverged");
    return {
        exactProcessWindow: true,
        atomicWindowThreadBatch: true,
        batchThreadId: sequence.batchThreadId,
        batchSpanMilliseconds: sequence.batchSpanMilliseconds,
        keyPostIntervalsMilliseconds: intervals,
        orderedMessages: sequence.messages,
        runModifierHeld: true,
        forwardHeld: true,
        actionAfterReadiness: true,
    };
}

export async function boundarySurvival(executable: string, config: string): Promise<Json> {
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
    const status = child.status;
    const line = await readinessLine(reader);
    const readiness = JSON.parse(line) as Json;
    if (readiness.role !== "prototype") {
        child.kill();
        fail("prototype boundary survival emitted the wrong readiness role");
    }
    const nativeInput = await holdPrototypeBoundaryRun(child.pid);
    const heldStarted = performance.now();
    const outcome = await Promise.race([
        status.then((value) => ({ kind: "exit" as const, status: value })),
        new Promise<{ kind: "held" }>((resolve) =>
            setTimeout(() => resolve({ kind: "held" }), BOUNDARY_HOLD_MILLISECONDS)
        ),
    ]);
    const heldMilliseconds = performance.now() - heldStarted;
    if (outcome.kind === "held") child.kill();
    const finalStatus = await status;
    let trailingOutput = "";
    while (true) {
        const remaining = await reader.read();
        if (remaining.done) break;
        trailingOutput += remaining.value;
    }
    await reader.cancel();
    const stderrText = (await stderr).trim();
    if (outcome.kind === "exit") {
        fail(
            `prototype boundary survival exited after ${heldMilliseconds.toFixed(3)} ms ` +
                `with code ${outcome.status.code}: ${stderrText.slice(-4_096)}`,
        );
    }
    if (
        heldMilliseconds < BOUNDARY_HOLD_MILLISECONDS ||
        stderrText.includes("prototype failed:") ||
        trailingOutput.trim()
    ) {
        fail("prototype boundary survival duration or stderr diverged");
    }
    return {
        label: "prototype finite-boundary Run survival",
        processId: child.pid,
        elapsedMs: performance.now() - started,
        heldMilliseconds,
        minimumHoldMilliseconds: BOUNDARY_HOLD_MILLISECONDS,
        processRemainedLive: true,
        forcedEvidenceExitCode: finalStatus.code,
        stderr: stderrText.slice(-4_096),
        trailingOutput,
        completionEmitted: false,
        actionAfterReadiness: true,
        nativeInput,
        readiness,
    };
}
