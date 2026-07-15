import {
    array,
    assertStopped,
    event,
    fail,
    type Json,
    lifecycle,
    number,
    object,
    rejectedEvent,
    same,
    sleep,
    startClean,
    status,
} from "./canonical-runtime.ts";

const REVISION = "deterministic-fixed-simulation-schedule-v1";
const MAX_ELAPSED = 125_000_000;

function invariant(value: Json): Json {
    return {
        revision: value.revision,
        tick: value.tick,
        remainderNumerator: value.remainderNumerator,
        remainderDenominator: value.remainderDenominator,
        stepsPerSecond: value.stepsPerSecond,
        maximumElapsedNanoseconds: value.maximumElapsedNanoseconds,
        maximumStepsPerAdvance: value.maximumStepsPerAdvance,
        successfulAdvanceCount: value.successfulAdvanceCount,
        emittedStepCount: value.emittedStepCount,
    };
}

function requireStatus(
    value: Json,
    tick: number,
    remainder: number,
    advances: number,
    emitted: number,
    label: string,
): void {
    if (
        value.revision !== REVISION || number(value, "tick") !== tick ||
        number(value, "remainderNumerator") !== remainder ||
        number(value, "remainderDenominator") !== 1_000_000_000 ||
        number(value, "stepsPerSecond") !== 60 ||
        number(value, "maximumElapsedNanoseconds") !== MAX_ELAPSED ||
        number(value, "maximumStepsPerAdvance") !== 8 ||
        number(value, "successfulAdvanceCount") !== advances ||
        number(value, "emittedStepCount") !== emitted
    ) fail(`${label} simulation status diverged: ${JSON.stringify(value)}`);
}

function requireZeroWork(value: Json, label: string): Json {
    if (
        value.revision !== REVISION || value.perAdvanceAllocationBytes !== 0 ||
        value.sourceReadCount !== 0 || value.gpuCopyCount !== 0 ||
        value.gpuReadbackCount !== 0 || value.fenceWaitCount !== 0 ||
        value.synchronizationCount !== 0
    ) fail(`${label} performed work outside fixed integer schedule mutation`);
    return object(value, "advance");
}

async function advanceSequence(intervals: number[], label: string): Promise<Json> {
    const batches: Json[] = [];
    for (const elapsed_nanoseconds of intervals) {
        const response = await event("simulation.advance", { elapsed_nanoseconds });
        const advance = requireZeroWork(response, label);
        if (
            number(advance, "elapsedNanoseconds") !== elapsed_nanoseconds ||
            number(advance, "stepCount") < 0 || number(advance, "stepCount") > 8 ||
            number(advance, "endTick") !==
                number(advance, "startTick") + number(advance, "stepCount")
        ) fail(`${label} returned an invalid fixed-step batch`);
        batches.push(advance);
    }
    return { intervals, batches, batchSha256: await sha256(batches) };
}

async function sha256(value: unknown): Promise<string> {
    const bytes = new TextEncoder().encode(JSON.stringify(value));
    const digest = await crypto.subtle.digest("SHA-256", bytes);
    return Array.from(new Uint8Array(digest), (byte) => byte.toString(16).padStart(2, "0")).join(
        "",
    );
}

export async function simulationScheduleGates(): Promise<Json> {
    console.log("==> deterministic fixed simulation schedule gates");
    await startClean();
    await event("workbench.pause");
    const initial = await event("simulation.status");
    const initialPresentation = await event("canonical.time.status");
    requireStatus(initial, 0, 0, 0, 0, "initial");
    const invalid = await rejectedEvent("simulation.advance", {
        elapsed_nanoseconds: MAX_ELAPSED + 1,
    });
    if (
        typeof invalid.error !== "string" ||
        !invalid.error.startsWith("simulation_advance_failed: ")
    ) fail("oversized simulation elapsed returned the wrong rejection");
    const malformed = await rejectedEvent("simulation.advance", { elapsed_nanoseconds: -1 });
    if (typeof malformed.error !== "string" || !malformed.error.startsWith("invalid_payload: ")) {
        fail("malformed simulation elapsed returned the wrong rejection");
    }
    same(invariant(await event("simulation.status")), invariant(initial), "invalid rollback");

    const longDuration = await event("simulation.probe");
    const histogram = array(longDuration, "batchHistogram");
    if (
        longDuration.revision !== REVISION ||
        longDuration.elapsedInputNanoseconds !== 3_600_000_000_000 ||
        longDuration.advanceCount !== 28_800 || longDuration.emittedStepCount !== 216_000 ||
        longDuration.finalTick !== 216_000 || longDuration.remainderNumerator !== 0 ||
        longDuration.remainderDenominator !== 1_000_000_000 || histogram.length !== 9 ||
        histogram.slice(0, 7).some((count) => count !== 0) || histogram[7] !== 14_400 ||
        histogram[8] !== 14_400 || longDuration.resultSha256 !== longDuration.replaySha256 ||
        typeof longDuration.resultSha256 !== "string" || longDuration.resultSha256.length !== 64 ||
        typeof longDuration.elapsedCpuNanoseconds !== "number" ||
        longDuration.perAdvanceAllocationBytes !== 0 || longDuration.sourceReadCount !== 0 ||
        longDuration.gpuCopyCount !== 0 || longDuration.gpuReadbackCount !== 0 ||
        longDuration.fenceWaitCount !== 0 || longDuration.synchronizationCount !== 0
    ) fail(`long-duration simulation probe diverged: ${JSON.stringify(longDuration)}`);
    same(invariant(await event("simulation.status")), invariant(initial), "probe isolation");

    const coarse = await advanceSequence(Array(8).fill(MAX_ELAPSED), "coarse sequence");
    const coarseStatus = await event("simulation.status");
    requireStatus(coarseStatus, 60, 0, 8, 60, "coarse one-second");
    same(
        await event("canonical.time.status"),
        initialPresentation,
        "simulation advance presentation independence",
    );
    const coarseSteps = (coarse.batches as Json[]).map((value) => number(value, "stepCount"));
    if (JSON.stringify(coarseSteps) !== "[7,8,7,8,7,8,7,8]") {
        fail(`coarse simulation batches diverged: ${JSON.stringify(coarseSteps)}`);
    }
    const beforeFrames = invariant(coarseStatus);
    await event("workbench.resume");
    await sleep(250);
    await event("workbench.pause");
    same(
        invariant(await event("simulation.status")),
        beforeFrames,
        "idle render frame independence",
    );

    const coarseProcess = number(await status(), "processId");
    await lifecycle("stop");
    await assertStopped(coarseProcess);
    await lifecycle("start");
    await event("workbench.pause");
    const restarted = await event("simulation.status");
    requireStatus(restarted, 0, 0, 0, 0, "process restart");
    const nominalIntervals = [
        ...Array(20).fill(16_666_666),
        ...Array(40).fill(16_666_667),
    ];
    const nominal = await advanceSequence(nominalIntervals, "nominal sequence");
    const nominalStatus = await event("simulation.status");
    requireStatus(nominalStatus, 60, 0, 60, 60, "nominal one-second");
    same(
        {
            tick: nominalStatus.tick,
            remainderNumerator: nominalStatus.remainderNumerator,
            emittedStepCount: nominalStatus.emittedStepCount,
        },
        {
            tick: coarseStatus.tick,
            remainderNumerator: coarseStatus.remainderNumerator,
            emittedStepCount: coarseStatus.emittedStepCount,
        },
        "one-second partition invariant",
    );

    const nominalProcess = number(await status(), "processId");
    await lifecycle("stop");
    await assertStopped(nominalProcess);
    await lifecycle("start");
    const clean = await event("simulation.status");
    requireStatus(clean, 0, 0, 0, 0, "clean canonical process");
    return {
        coarseProcess,
        nominalProcess,
        initial,
        initialPresentation,
        invalid,
        malformed,
        longDuration,
        coarse,
        coarseStatus,
        nominal,
        nominalStatus,
        clean,
    };
}
