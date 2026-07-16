import {
    assertStopped,
    event,
    fail,
    lifecycle,
    number,
    object,
    rejectedEvent,
    same,
    startClean,
    status,
} from "./canonical-runtime.ts";

type Json = Record<string, unknown>;

const CONTROLLED_MESSAGES = [
    { kind: "key", key: 87, down: true },
    { kind: "key", key: 87, down: true },
    { kind: "key", key: 65, down: false },
    { kind: "key", key: 65, down: true },
    { kind: "key", key: 68, down: true },
    { kind: "focus_lost" },
    { kind: "key", key: 68, down: false },
    { kind: "key", key: 135, down: true, system: true },
    { kind: "key", key: 135, down: false, system: true },
];

function recordInvariant(value: Json): Json {
    return {
        revision: value.revision,
        transactionCount: value.transactionCount,
        rawMessageCount: value.rawMessageCount,
        transitionCount: value.transitionCount,
        invalidKeyCount: value.invalidKeyCount,
        repeatedDownCount: value.repeatedDownCount,
        unmatchedUpCount: value.unmatchedUpCount,
        focusLossCount: value.focusLossCount,
        focusReleaseCount: value.focusReleaseCount,
        initialHeldKeys: value.initialHeldKeys,
        finalHeldKeys: value.finalHeldKeys,
        initialHeldStateSha256: value.initialHeldStateSha256,
        finalHeldStateSha256: value.finalHeldStateSha256,
        streamSha256: value.streamSha256,
    };
}

function requireControlledEvidence(record: Json, label: string): void {
    if (
        record.revision !== "deterministic-host-input-v1" ||
        number(record, "transactionCount") !== 2 ||
        number(record, "rawMessageCount") !== 11 ||
        number(record, "transitionCount") !== 10 ||
        number(record, "invalidKeyCount") !== 0 ||
        number(record, "repeatedDownCount") !== 1 ||
        number(record, "unmatchedUpCount") !== 2 ||
        number(record, "focusLossCount") !== 1 ||
        number(record, "focusReleaseCount") !== 3 ||
        JSON.stringify(record.initialHeldKeys) !== "[]" ||
        JSON.stringify(record.finalHeldKeys) !== "[]" ||
        record.initialHeldStateSha256 !==
            "a539733efed454375e712ba689ac99049afb144efbbe7f9c60256c11860e2861" ||
        record.finalHeldStateSha256 !==
            "a539733efed454375e712ba689ac99049afb144efbbe7f9c60256c11860e2861" ||
        record.streamSha256 !==
            "ec86601874cb60a8c592b9caf500da94111b6a7360647d316ce1e858b55de435"
    ) fail(`${label} normalized input evidence diverged: ${JSON.stringify(record)}`);
}

async function controlledRecord(label: string): Promise<Json> {
    await event("workbench.pause");
    const paused = await status();
    const pausedFrame = number(paused, "frameIndex");

    const stopWithoutRecord = await rejectedEvent("input.record.stop");
    const replayWithoutRecord = await rejectedEvent("input.replay");
    const invalidKey = await rejectedEvent("input.native.post", {
        messages: [{ kind: "key", key: 256, down: true }],
    });
    await event("input.native.post", { messages: [{ kind: "focus_lost" }] });
    const cleanState = await event("input.status");
    if (JSON.stringify(cleanState.heldKeys) !== "[]") {
        fail(`${label} could not establish an empty initial held-key state`);
    }
    await event("input.record.start");
    const duplicateStart = await rejectedEvent("input.record.start");
    const replayWhileRecording = await rejectedEvent("input.replay");

    const firstPost = await event("input.native.post", { messages: CONTROLLED_MESSAGES });
    if (number(firstPost, "postedMessageCount") !== CONTROLLED_MESSAGES.length) {
        fail(`${label} did not post the complete first native input batch`);
    }
    const firstDrain = await event("input.status");
    const activeRecord = object(firstDrain, "recording");
    if (
        number(activeRecord, "transactionCount") !== 1 ||
        number(activeRecord, "transitionCount") !== 8
    ) fail(`${label} did not normalize the first native input batch as one transaction`);

    const secondPost = await event("input.native.post", {
        messages: [
            { kind: "key", key: 32, down: true },
            { kind: "key", key: 32, down: false },
        ],
    });
    if (number(secondPost, "postedMessageCount") !== 2) {
        fail(`${label} did not post the complete second native input batch`);
    }
    const record = await event("input.record.stop");
    requireControlledEvidence(record, label);

    const liveBeforeReplay = await event("input.status");
    const replay = await event("input.replay");
    if (replay.matchesRecord !== true || replay.liveStateUnchanged !== true) {
        fail(`${label} replay did not prove exact isolated state consumption`);
    }
    same(recordInvariant(replay), recordInvariant(record), `${label} replay invariant`);
    const liveAfterReplay = await event("input.status");
    same(
        {
            heldKeys: liveAfterReplay.heldKeys,
            heldStateSha256: liveAfterReplay.heldStateSha256,
            rawMessageCount: liveAfterReplay.rawMessageCount,
            transactionCount: liveAfterReplay.transactionCount,
            transitionCount: liveAfterReplay.transitionCount,
        },
        {
            heldKeys: liveBeforeReplay.heldKeys,
            heldStateSha256: liveBeforeReplay.heldStateSha256,
            rawMessageCount: liveBeforeReplay.rawMessageCount,
            transactionCount: liveBeforeReplay.transactionCount,
            transitionCount: liveBeforeReplay.transitionCount,
        },
        `${label} live state after replay`,
    );
    const after = await status();
    if (number(after, "frameIndex") !== pausedFrame) {
        fail(`${label} host pause allowed a frame during input record/replay`);
    }
    await event("workbench.resume");
    return {
        processId: number(after, "processId"),
        pausedFrame,
        firstDrain,
        record,
        replay,
        invalidOperations: {
            stopWithoutRecord,
            replayWithoutRecord,
            invalidKey,
            duplicateStart,
            replayWhileRecording,
        },
    };
}

export async function hostInputGates(): Promise<Json> {
    console.log("==> deterministic native host input and replay gates");
    await startClean();
    const first = await controlledRecord("first process");
    const firstProcessId = number(first, "processId");
    await lifecycle("stop");
    await assertStopped(firstProcessId);

    await lifecycle("start");
    const restarted = await controlledRecord("restarted process");
    same(
        recordInvariant(object(first, "record")),
        recordInvariant(object(restarted, "record")),
        "host input process-restart record",
    );
    return { first, restarted };
}
