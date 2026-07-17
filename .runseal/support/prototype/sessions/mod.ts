import { fail, type Json, number, object, root, same, string } from "../../canonical-runtime.ts";
import {
    postBoundaryRunStart,
    postConsumptionCapacity,
    postFocusLocomotionReadmission,
    postObjectActionExit,
    postPrototypeCapacityRejection,
    pressPrototypeEscape,
    requestPrototypeWindowClose,
    resumePrototypeFocus,
    suspendHeldPrototypeJump,
    suspendWithActionBatch,
    suspendWithObjectBatch,
} from "../input/actions.ts";
import {
    postCameraRepeatSequence,
    postCameraRepressSequence,
    postCounterClockwiseSequence,
    postDiagonalRun,
    postDiagonalWalk,
    postForwardRelease,
    postInvalidAliasSequence,
    postMidairSequence,
    postOpposedRun,
    postOppositeCameraSequence,
    postRunRelease,
    postRunRepress,
    pressPrototypeCameraClockwise,
    pressPrototypeJump,
    releaseOpposedRun,
    repressJumpAndExit,
} from "../input/sequences.ts";

const REVISION = "live-prototype-session-completion-v2";
export const BOUNDARY_RUN_HOLD_MILLISECONDS = 15_000;

export async function outputLine(
    reader: ReadableStreamDefaultReader<string>,
    label: string,
    timeoutMilliseconds = 30_000,
): Promise<string> {
    const deadline = performance.now() + timeoutMilliseconds;
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
            fail(`prototype exited or timed out before ${label}`);
        }
        buffered += result.value;
        const newline = buffered.indexOf("\n");
        if (newline >= 0) {
            if (buffered.slice(newline + 1).trim()) {
                fail(`prototype emitted buffered output after ${label}`);
            }
            return buffered.slice(0, newline).trim();
        }
    }
    fail(`prototype ${label} timeout expired`);
}

export async function readinessLine(reader: ReadableStreamDefaultReader<string>): Promise<string> {
    return await outputLine(reader, "readiness");
}

export async function capturedReady(
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
    let value: Json;
    let trailingOutput = "";
    try {
        value = JSON.parse(await readinessLine(reader)) as Json;
        if (value.role !== "prototype") fail(`${label} emitted the wrong readiness role`);
        const startup = object(value, "startup");
        if (
            startup.mode !== "canonical-bootstrap" ||
            number(startup, "readyFrameIndex") < 1 ||
            number(startup, "elapsedMs") <= 0
        ) fail(`${label} emitted incomplete canonical readiness`);
    } finally {
        child.kill();
    }
    const status = await child.status;
    while (true) {
        const remaining = await reader.read();
        if (remaining.done) break;
        trailingOutput += remaining.value;
    }
    await reader.cancel();
    if (trailingOutput.trim()) {
        fail(`${label} emitted session completion after forced evidence termination`);
    }
    return {
        label,
        processId: Number(string(value, "instance_id")),
        elapsedMs: performance.now() - started,
        forcedEvidenceExitCode: status.code,
        stderr: (await stderr).trim().slice(-4_096),
        trailingOutput,
        completionEmitted: false,
        readiness: value,
    };
}

export async function sustainedCapacitySession(executable: string, config: string): Promise<Json> {
    return await gracefulExit(
        executable,
        config,
        "prototype sustained capacity-one session",
        "consumption-capacity-rejection",
    );
}

export async function objectFeedbackSession(
    executable: string,
    config: string,
    label: string,
    focusRecovery: boolean,
): Promise<Json> {
    return await gracefulExit(
        executable,
        config,
        label,
        focusRecovery ? "object-feedback-focus" : "object-feedback",
    );
}

export async function gracefulExit(
    executable: string,
    config: string,
    label: string,
    postReadiness:
        | "boundary-run"
        | "capacity-rejection"
        | "camera-repeat"
        | "camera-repress"
        | "consumption-capacity-rejection"
        | "counter-clockwise-camera"
        | "diagonal-run"
        | "diagonal-walk"
        | "focus-discontinuity"
        | "forward-release"
        | "invalid-camera-alias"
        | "jump-midair"
        | "jump-readmission"
        | "object-feedback"
        | "object-feedback-focus"
        | "opposed-run-release"
        | "opposite-camera"
        | "run-release"
        | "run-repress"
        | null = null,
    exitReason: "escape" | "window-close" = "escape",
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
    let completion: Json;
    let postReadinessInput: Json | null = null;
    let exitInput: Json | null = null;
    let status: Deno.CommandStatus;
    let trailingOutput = "";
    try {
        readiness = JSON.parse(await readinessLine(reader)) as Json;
        if (readiness.role !== "prototype") fail(`${label} emitted the wrong readiness role`);
        if (postReadiness === "boundary-run") {
            const sequence = await postBoundaryRunStart(child.pid);
            const heldStartedAt = performance.now();
            await new Promise((resolve) => setTimeout(resolve, BOUNDARY_RUN_HOLD_MILLISECONDS));
            const heldMilliseconds = performance.now() - heldStartedAt;
            const exit = await pressPrototypeEscape(child.pid);
            postReadinessInput = {
                sequence,
                exit,
                heldMilliseconds,
                minimumHoldMilliseconds: BOUNDARY_RUN_HOLD_MILLISECONDS,
            };
            exitInput = exit;
        } else if (postReadiness === "capacity-rejection") {
            await new Promise((resolve) => setTimeout(resolve, 250));
            postReadinessInput = await postPrototypeCapacityRejection(child.pid);
            await new Promise((resolve) => setTimeout(resolve, 250));
        } else if (postReadiness === "consumption-capacity-rejection") {
            postReadinessInput = await postConsumptionCapacity(child.pid);
            await new Promise((resolve) => setTimeout(resolve, 250));
        } else if (postReadiness === "object-feedback") {
            const sequence = await postObjectActionExit(child.pid);
            postReadinessInput = { sequence };
            exitInput = sequence;
        } else if (postReadiness === "object-feedback-focus") {
            const suspended = await suspendWithObjectBatch(child.pid);
            await new Promise((resolve) => setTimeout(resolve, 250));
            const resumed = await resumePrototypeFocus(child.pid);
            await new Promise((resolve) => setTimeout(resolve, 250));
            const sequence = await postObjectActionExit(child.pid);
            postReadinessInput = { suspended, resumed, sequence };
            exitInput = sequence;
        } else if (postReadiness === "camera-repeat") {
            const initialPress = await pressPrototypeCameraClockwise(child.pid);
            const initialPressedAt = performance.now();
            await new Promise((resolve) => setTimeout(resolve, 250));
            const initialHoldMilliseconds = performance.now() - initialPressedAt;
            const sequence = await postCameraRepeatSequence(child.pid);
            postReadinessInput = {
                initialPress,
                sequence,
                requestedInitialHoldMilliseconds: 250,
                initialHoldMilliseconds,
            };
            exitInput = sequence;
        } else if (postReadiness === "camera-repress") {
            const initialPress = await pressPrototypeCameraClockwise(child.pid);
            const initialPressedAt = performance.now();
            await new Promise((resolve) => setTimeout(resolve, 250));
            const initialHoldMilliseconds = performance.now() - initialPressedAt;
            const sequence = await postCameraRepressSequence(child.pid);
            postReadinessInput = {
                initialPress,
                sequence,
                requestedInitialHoldMilliseconds: 250,
                initialHoldMilliseconds,
            };
            exitInput = sequence;
        } else if (postReadiness === "counter-clockwise-camera") {
            const sequence = await postCounterClockwiseSequence(child.pid);
            postReadinessInput = { sequence };
            exitInput = sequence;
        } else if (postReadiness === "diagonal-run") {
            const sequence = await postDiagonalRun(child.pid);
            postReadinessInput = { sequence };
            exitInput = sequence;
        } else if (postReadiness === "diagonal-walk") {
            const sequence = await postDiagonalWalk(child.pid);
            postReadinessInput = { sequence };
            exitInput = sequence;
        } else if (postReadiness === "focus-discontinuity") {
            const suspended = await suspendWithActionBatch(child.pid);
            await new Promise((resolve) => setTimeout(resolve, 250));
            const resumed = await resumePrototypeFocus(child.pid);
            await new Promise((resolve) => setTimeout(resolve, 250));
            const readmission = await postFocusLocomotionReadmission(child.pid);
            postReadinessInput = { suspended, resumed, readmission };
            exitInput = readmission;
        } else if (postReadiness === "forward-release") {
            const sequence = await postForwardRelease(child.pid);
            postReadinessInput = { sequence };
            exitInput = sequence;
        } else if (postReadiness === "invalid-camera-alias") {
            const sequence = await postInvalidAliasSequence(child.pid);
            postReadinessInput = { sequence };
            exitInput = sequence;
        } else if (postReadiness === "opposite-camera") {
            const sequence = await postOppositeCameraSequence(child.pid);
            postReadinessInput = { sequence };
            exitInput = sequence;
        } else if (postReadiness === "opposed-run-release") {
            const opposedInput = await postOpposedRun(child.pid);
            const opposedStartedAt = performance.now();
            await new Promise((resolve) => setTimeout(resolve, 250));
            const opposedHoldMilliseconds = performance.now() - opposedStartedAt;
            const sequence = await releaseOpposedRun(child.pid);
            postReadinessInput = {
                opposedInput,
                sequence,
                requestedOpposedHoldMilliseconds: 250,
                opposedHoldMilliseconds,
            };
            exitInput = sequence;
        } else if (postReadiness === "jump-readmission") {
            const firstJump = await pressPrototypeJump(child.pid);
            const firstJumpPostedAt = performance.now();
            await new Promise((resolve) => setTimeout(resolve, 1_250));
            const suspended = await suspendHeldPrototypeJump(child.pid);
            await new Promise((resolve) => setTimeout(resolve, 250));
            const resumed = await resumePrototypeFocus(child.pid);
            await new Promise((resolve) => setTimeout(resolve, 250));
            const readmitStartedAt = performance.now();
            const secondJump = await repressJumpAndExit(child.pid);
            postReadinessInput = {
                firstJump,
                suspended,
                resumed,
                secondJump,
                firstToSecondPostingLowerBoundMs: readmitStartedAt - firstJumpPostedAt,
            };
            exitInput = secondJump;
        } else if (postReadiness === "jump-midair") {
            const sequence = await postMidairSequence(child.pid);
            postReadinessInput = { sequence };
            exitInput = sequence;
        } else if (postReadiness === "run-release") {
            const sequence = await postRunRelease(child.pid);
            postReadinessInput = { sequence };
            exitInput = sequence;
        } else if (postReadiness === "run-repress") {
            const sequence = await postRunRepress(child.pid);
            postReadinessInput = { sequence };
            exitInput = sequence;
        }
        if (exitInput === null) {
            exitInput = exitReason === "escape"
                ? await pressPrototypeEscape(child.pid)
                : await requestPrototypeWindowClose(child.pid);
        }
        completion = JSON.parse(await outputLine(reader, "session completion", 10_000)) as Json;
        const exit = await Promise.race([
            child.status.then((value) => ({ kind: "status" as const, value })),
            new Promise<{ kind: "timeout" }>((resolve) =>
                setTimeout(() => resolve({ kind: "timeout" }), 10_000)
            ),
        ]);
        if (exit.kind === "timeout") fail(`${label} did not exit after ${exitReason}`);
        status = exit.value;
        if (!status.success) fail(`${label} exited with code ${status.code}`);
        while (true) {
            const remaining = await reader.read();
            if (remaining.done) break;
            trailingOutput += remaining.value;
        }
        if (trailingOutput.trim()) fail(`${label} emitted trailing session output`);
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
        postReadinessInput,
        exitInput,
        exitReason,
        readiness,
        completion,
        outputValueCount: 2,
        trailingOutput,
    };
}

export function gracefulCompletionInvariant(
    launch: Json,
    expectedReason: "escape" | "window-close",
): Json {
    const readiness = object(launch, "readiness");
    const completion = object(launch, "completion");
    const contract = object(readiness, "session_contract");
    if (
        number(readiness, "sequence") !== 1 ||
        contract.revision !== REVISION ||
        number(contract, "readinessSequence") !== 1 ||
        number(contract, "completionSequence") !== 2 ||
        contract.completion !== "graceful-exit-only" ||
        completion.role !== "prototype-session-completion" ||
        completion.revision !== REVISION ||
        number(completion, "sequence") !== 2 ||
        completion.outcome !== "completed" ||
        completion.reason !== expectedReason ||
        number(launch, "outputValueCount") !== 2 ||
        launch.trailingOutput !== ""
    ) fail("prototype bounded session contract diverged");
    if (
        string(completion, "instance_id") !== string(readiness, "instance_id") ||
        number(launch, "processId") !== Number(string(completion, "instance_id"))
    ) fail("prototype session completion changed process identity");

    const readyActor = object(object(readiness, "actor"), "state");
    const finalActor = object(object(completion, "actor"), "state");
    same(
        object(finalActor, "handle"),
        object(readyActor, "handle"),
        "prototype final actor handle",
    );
    const frames = object(completion, "frames");
    const bootstrapFrames = number(frames, "bootstrapFrameCount");
    const liveFrames = number(frames, "liveFrameCount");
    if (
        number(object(completion, "actor"), "capacity") !== 1 ||
        number(object(completion, "actor"), "liveCount") !== 1 ||
        bootstrapFrames !==
            number(object(readiness, "simulation_driver"), "bootstrapFrameCount") ||
        liveFrames < number(object(readiness, "simulation_driver"), "liveFrameCount") ||
        number(frames, "totalFrameCount") !== bootstrapFrames + liveFrames ||
        number(frames, "cameraAnchorCount") !== liveFrames
    ) fail("prototype session final frame authority diverged");

    const clock = object(completion, "clock");
    if (
        number(clock, "sampleCount") <
            number(object(object(readiness, "simulation_driver"), "clock"), "sampleCount")
    ) fail("prototype session final clock regressed");
    const observation = object(completion, "object_observation");
    const interaction = object(completion, "object_interaction");
    if (number(interaction, "capacity") !== 1) {
        fail("prototype session completion interaction capacity diverged");
    }

    return {
        revision: REVISION,
        readinessSequence: 1,
        completionSequence: 2,
        reason: expectedReason,
        processIdentityStable: true,
        finalActorHandleStable: true,
        bootstrapFrameCount: bootstrapFrames,
        readyLiveFrameCount: number(object(readiness, "simulation_driver"), "liveFrameCount"),
        finalLiveFrameCount: liveFrames,
        finalClock: clock,
        finalObservation: observation,
        finalInteraction: interaction,
        exactlyTwoValues: true,
    };
}

export function idleCompletionInvariant(
    launch: Json,
    expectedReason: "escape" | "window-close" = "escape",
): Json {
    const session = gracefulCompletionInvariant(launch, expectedReason);
    const interaction = object(object(launch, "completion"), "object_interaction");
    const observation = object(object(launch, "completion"), "object_observation");
    if (
        interaction.pending !== false ||
        interaction.acknowledgement !== null ||
        number(interaction, "committedCount") !== 0 ||
        number(interaction, "ineligibleCount") !== 0 ||
        interaction.consumed !== null ||
        interaction.nearestExclusion !== null ||
        observation.pending !== false ||
        observation.target !== null
    ) fail("prototype idle session completion changed object policy state");
    return {
        ...session,
        idleFinalObjectState: true,
    };
}
