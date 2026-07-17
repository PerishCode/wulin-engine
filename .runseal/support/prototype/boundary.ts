import { fail, type Json, root } from "../canonical-runtime.ts";
import { holdPrototypeForwardKey } from "./input/actions.ts";
import { readinessLine } from "./sessions/mod.ts";

export const BOUNDARY_HOLD_MILLISECONDS = 15_000;

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
    const nativeInput = await holdPrototypeForwardKey(child.pid);
    const line = await readinessLine(reader);
    const readiness = JSON.parse(line) as Json;
    if (readiness.role !== "prototype") {
        child.kill();
        fail("prototype boundary survival emitted the wrong readiness role");
    }
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
        label: "prototype finite boundary survival",
        processId: child.pid,
        elapsedMs: performance.now() - started,
        heldMilliseconds,
        minimumHoldMilliseconds: BOUNDARY_HOLD_MILLISECONDS,
        processRemainedLive: true,
        forcedEvidenceExitCode: finalStatus.code,
        stderr: stderrText.slice(-4_096),
        trailingOutput,
        completionEmitted: false,
        nativeInput,
        readiness,
    };
}
