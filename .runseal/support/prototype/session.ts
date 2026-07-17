import { fail, type Json, number, object, root, same, string } from "../canonical-runtime.ts";
import {
    applyStartupInput,
    nativeFocusDiscontinuityInvariant,
    nativeWindowCloseInvariant,
    postPrototypeCapacityRejection,
    pressPrototypeEscape,
    pressPrototypeJump,
    repressJumpAndExit,
    requestPrototypeWindowClose,
    resumePrototypeFocus,
    type StartupInput,
    suspendWithForward,
} from "./input.ts";
import { jumpReadmissionInvariant } from "./jump.ts";
import { sustainedCapacityInvariant } from "./object/gates.ts";

const REVISION = "live-prototype-session-completion-v1";

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

export async function readinessLine(
    reader: ReadableStreamDefaultReader<string>,
): Promise<string> {
    return await outputLine(reader, "readiness");
}

export async function capturedReady(
    executable: string,
    config: string,
    label: string,
    startupInput?: StartupInput,
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
    let nativeInput: Json | null;
    let trailingOutput = "";
    try {
        nativeInput = await applyStartupInput(child.pid, startupInput);
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
        nativeInput,
        readiness: value,
    };
}

async function gracefulExit(
    executable: string,
    config: string,
    label: string,
    startupInput?: StartupInput,
    postReadiness:
        | "capacity-rejection"
        | "focus-discontinuity"
        | "jump-readmission"
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
    let startupNativeInput: Json | null;
    let postReadinessInput: Json | null = null;
    let exitInput: Json | null = null;
    let status: Deno.CommandStatus;
    let trailingOutput = "";
    try {
        startupNativeInput = await applyStartupInput(child.pid, startupInput);
        readiness = JSON.parse(await readinessLine(reader)) as Json;
        if (readiness.role !== "prototype") fail(`${label} emitted the wrong readiness role`);
        if (postReadiness === "capacity-rejection") {
            await new Promise((resolve) => setTimeout(resolve, 250));
            postReadinessInput = await postPrototypeCapacityRejection(child.pid);
            await new Promise((resolve) => setTimeout(resolve, 250));
        } else if (postReadiness === "focus-discontinuity") {
            const suspended = await suspendWithForward(child.pid);
            await new Promise((resolve) => setTimeout(resolve, 250));
            const resumed = await resumePrototypeFocus(child.pid);
            await new Promise((resolve) => setTimeout(resolve, 250));
            postReadinessInput = { suspended, resumed };
        } else if (postReadiness === "jump-readmission") {
            const firstJump = await pressPrototypeJump(child.pid);
            const firstJumpPostedAt = performance.now();
            await new Promise((resolve) => setTimeout(resolve, 1_250));
            const readmitStartedAt = performance.now();
            const secondJump = await repressJumpAndExit(child.pid);
            postReadinessInput = {
                firstJump,
                secondJump,
                firstToSecondPostingLowerBoundMs: readmitStartedAt - firstJumpPostedAt,
            };
            exitInput = secondJump;
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
        startupNativeInput,
        postReadinessInput,
        exitInput,
        exitReason,
        readiness,
        completion,
        outputValueCount: 2,
        trailingOutput,
    };
}

export async function sessionGates(
    executable: string,
    config: string,
    first: Json,
    startupInvariant: (launch: Json) => Json,
    jumpInvariant: (launch: Json) => Json,
    source: string,
    windowCenter: [number, number],
): Promise<Json> {
    if (first.completionEmitted !== false || first.trailingOutput !== "") {
        fail("prototype forced readiness process emitted session completion");
    }
    const escape = await gracefulExit(executable, config, "prototype Escape press exit");
    const windowClose = await gracefulExit(
        executable,
        config,
        "prototype native window close exit",
        undefined,
        null,
        "window-close",
    );
    const focusDiscontinuity = await gracefulExit(
        executable,
        config,
        "prototype native focus discontinuity",
        undefined,
        "focus-discontinuity",
    );
    const jumpReadmission = await gracefulExit(
        executable,
        config,
        "prototype native Jump readmission",
        undefined,
        "jump-readmission",
    );
    const sustained = await gracefulExit(
        executable,
        config,
        "prototype sustained capacity-one session",
        "observe-action-facing",
        "capacity-rejection",
    );
    same(startupInvariant(escape), startupInvariant(first), "prototype Escape configuration");
    same(
        jumpInvariant(escape),
        jumpInvariant(first),
        "prototype Escape jump policy",
    );
    same(
        startupInvariant(windowClose),
        startupInvariant(first),
        "prototype window-close configuration",
    );
    same(
        jumpInvariant(windowClose),
        jumpInvariant(first),
        "prototype window-close jump policy",
    );
    same(
        startupInvariant(focusDiscontinuity),
        startupInvariant(first),
        "prototype focus-discontinuity configuration",
    );
    same(
        startupInvariant(jumpReadmission),
        startupInvariant(first),
        "prototype Jump-readmission configuration",
    );
    same(
        jumpInvariant(jumpReadmission),
        jumpInvariant(first),
        "prototype Jump-readmission initial grounded policy",
    );
    same(
        startupInvariant(sustained),
        startupInvariant(first),
        "prototype sustained session configuration",
    );
    return {
        escape,
        escapeInvariant: idleCompletionInvariant(escape),
        windowClose,
        windowCloseInvariant: {
            ...idleCompletionInvariant(windowClose, "window-close"),
            nativeWindowClose: nativeWindowCloseInvariant(
                object(windowClose, "exitInput"),
                number(windowClose, "processId"),
            ),
        },
        focusDiscontinuity,
        focusDiscontinuityInvariant: focusSessionInvariant(focusDiscontinuity),
        jumpReadmission,
        jumpReadmissionInvariant: jumpReadmissionInvariant(
            jumpReadmission,
            idleCompletionInvariant(jumpReadmission),
        ),
        sustained,
        sustainedInvariant: await sustainedCapacityInvariant(
            sustained,
            gracefulCompletionInvariant(sustained, "escape"),
            source,
            windowCenter,
        ),
        forcedReadinessCompletionEmitted: false,
    };
}

function focusSessionInvariant(launch: Json): Json {
    const session = idleCompletionInvariant(launch);
    const readiness = object(launch, "readiness");
    const completion = object(launch, "completion");
    const readyActor = object(object(readiness, "actor"), "state");
    const finalActor = object(object(completion, "actor"), "state");
    same(finalActor, readyActor, "prototype focus-discontinuity actor state");

    const readyClock = object(object(readiness, "simulation_driver"), "clock");
    const finalClock = object(completion, "clock");
    if (
        finalClock.suspended !== false ||
        finalClock.hasBaseline !== true ||
        number(finalClock, "suspendCount") !== number(readyClock, "suspendCount") + 1 ||
        number(finalClock, "resumeCount") !== number(readyClock, "resumeCount") + 1 ||
        number(finalClock, "suspendedSampleCount") <=
            number(readyClock, "suspendedSampleCount") ||
        number(finalClock, "resetCount") !== number(readyClock, "resetCount") + 1 ||
        number(finalClock, "readyCount") <= number(readyClock, "readyCount") ||
        number(finalClock, "sampleCount") <= number(readyClock, "sampleCount") ||
        number(finalClock, "stallCount") !== number(readyClock, "stallCount") ||
        number(object(completion, "frames"), "renderBlockCount") !== 0
    ) fail("prototype focus-discontinuity clock recovery diverged");

    const postReadiness = object(launch, "postReadinessInput");
    return {
        ...session,
        actorStateUnchanged: true,
        clock: {
            ready: readyClock,
            final: finalClock,
            exactSuspendResumeCount: 1,
            postResumeResetCount: 1,
            elapsedBacklog: false,
        },
        nativeFocus: nativeFocusDiscontinuityInvariant(
            object(postReadiness, "suspended"),
            object(postReadiness, "resumed"),
            number(launch, "processId"),
        ),
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
        contract.eventStream !== false ||
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
    if (
        observation.copiedObjectState !== false ||
        interaction.eventHistory !== false ||
        interaction.copiedObjectState !== false ||
        number(interaction, "capacity") !== 1
    ) fail("prototype session completion retained diagnostic or copied object state");

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
        eventStream: false,
        copiedObjectState: false,
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
