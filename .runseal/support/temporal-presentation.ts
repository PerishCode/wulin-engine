import {
    type Coord,
    event,
    fail,
    frame,
    type Json,
    number,
    object,
    rejectedEvent,
    same,
    target,
    waitStatus,
} from "./canonical-runtime.ts";

export function presentationInvariant(stable: Json): Json {
    const objects = object(stable, "objects");
    return {
        objects: {
            identityKeyedSha256: objects.identityKeyedSha256,
            stableKeySha256: objects.stableKeySha256,
            stableSeedSha256: objects.stableSeedSha256,
            entries: objects.entries,
        },
        grounding: stable.grounding,
        contact: stable.contact,
        terrain: stable.terrain,
    };
}

function assertClock(
    value: Json,
    expectedTick: number,
    expectedRunning: boolean,
    label: string,
): void {
    if (
        number(value, "tick") !== expectedTick || value.running !== expectedRunning ||
        number(value, "phaseCount") !== 64
    ) fail(`${label} presentation clock diverged`);
}

function assertTemporalChange(before: Json, after: Json, label: string): void {
    const beforeStable = object(before, "stable");
    const afterStable = object(after, "stable");
    same(
        presentationInvariant(afterStable),
        presentationInvariant(beforeStable),
        `${label} spatial/identity invariants`,
    );
    const beforeObjects = object(beforeStable, "objects");
    const afterObjects = object(afterStable, "objects");
    if (afterObjects.presentationKeyedSha256 !== beforeObjects.presentationKeyedSha256) {
        fail(`${label} changed authored presentation authority`);
    }
    const beforeSkeletal = object(beforeStable, "skeletal");
    const afterSkeletal = object(afterStable, "skeletal");
    if (
        number(object(afterSkeletal, "settings"), "timeTick") ===
            number(object(beforeSkeletal, "settings"), "timeTick")
    ) fail(`${label} did not change the skeletal time tick`);
    if (
        JSON.stringify(object(afterStable, "surface")) ===
            JSON.stringify(object(beforeStable, "surface"))
    ) fail(`${label} did not change GPU/CPU surface evidence`);
    const beforeCapture = object(beforeStable, "capture");
    const afterCapture = object(afterStable, "capture");
    if (afterCapture.color === beforeCapture.color && afterCapture.png === beforeCapture.png) {
        fail(`${label} did not change rendered color evidence`);
    }
}

export async function temporalGates(orderA: Json, collection: string): Promise<Json> {
    const timeZeroStatus = await event("canonical.status");
    const timeZeroClock = object(timeZeroStatus, "presentationClock");
    assertClock(timeZeroClock, 0, false, "paused tick zero");
    const timeOneClock = await event("canonical.time.step", { ticks: 1 });
    assertClock(timeOneClock, 1, false, "manual tick one");
    if (number(timeOneClock, "manualStepCount") !== 1) fail("manual tick-one count diverged");
    const timeOne = await frame("time-one", collection);
    assertTemporalChange(orderA, timeOne, "tick one");
    const timeOneStatus = await event("canonical.status");
    same(
        object(timeOneStatus, "published"),
        object(timeZeroStatus, "published"),
        "tick-one content movement",
    );

    const timeWrappedClock = await event("canonical.time.step", { ticks: 63 });
    assertClock(timeWrappedClock, 0, false, "wrapped tick zero");
    if (
        number(timeWrappedClock, "manualStepCount") !== 64 ||
        number(timeWrappedClock, "wrapCount") < 1
    ) fail("manual 64-tick wrap counters diverged");
    const timeWrapped = await frame("time-wrapped-zero", collection);
    same(timeWrapped.stable, orderA.stable, "64-tick presentation wrap");

    const invalidSet = await rejectedEvent("canonical.time.set", { tick: 64 });
    assertClock(await event("canonical.time.status"), 0, false, "invalid set rollback");
    await event("canonical.time.resume");
    const invalidStep = await rejectedEvent("canonical.time.step", { ticks: 1 });
    const invalidStepClock = await event("canonical.time.pause");
    assertClock(invalidStepClock, 0, false, "running step rollback");

    const automaticBase = number(invalidStepClock, "automaticAdvanceCount");
    await event("canonical.time.resume");
    await event("workbench.resume");
    const automaticObserved = await waitStatus(
        "automatic presentation time",
        (value) =>
            number(object(value, "presentationClock"), "automaticAdvanceCount") >=
                automaticBase + 8,
    );
    const automaticClock = await event("canonical.time.pause");
    await event("workbench.pause");
    if (number(automaticClock, "automaticAdvanceCount") < automaticBase + 8) {
        fail("automatic presentation time did not advance eight frames");
    }
    const automaticStatus = await event("canonical.status");
    same(
        object(automaticStatus, "published"),
        object(timeZeroStatus, "published"),
        "automatic time content movement",
    );
    const automaticFrame = await frame("time-automatic", collection);
    const automaticTick = number(automaticClock, "tick");
    if (
        number(
            object(object(object(automaticFrame, "stable"), "skeletal"), "settings"),
            "timeTick",
        ) !== automaticTick
    ) fail("automatic frame did not use the paused observed tick");
    await event("canonical.time.set", { tick: 0 });
    return {
        timeZeroStatus,
        timeOneClock,
        timeOne,
        timeOneStatus,
        timeWrappedClock,
        timeWrapped,
        invalidSet,
        invalidStep,
        invalidStepClock,
        automaticObserved,
        automaticClock,
        automaticStatus,
        automaticFrame,
    };
}

export async function temporalHold(
    before: Json,
    collection: string,
    base: Coord,
): Promise<Json> {
    const beforeStatus = await event("canonical.status");
    const beforePublished = object(beforeStatus, "published");
    const beforeClock = object(beforeStatus, "presentationClock");
    assertClock(beforeClock, 0, false, "temporal hold baseline");
    const automaticBase = number(beforeClock, "automaticAdvanceCount");

    const gate = "canonical.objects.copy_gate";
    await event(`${gate}.arm`);
    const scheduled = await event("canonical.schedule", target([base[0] + 6, base[1]]));
    const token = number(scheduled, "token");
    await event("canonical.time.resume");
    await event("workbench.resume");
    const animated = await waitStatus("temporal object-copy hold", (value) => {
        if (!value.pending) return false;
        const pending = object(value, "pending");
        const clock = object(value, "presentationClock");
        return pending.terrainStage === "staged" && pending.instanceStage === "in-flight" &&
            number(clock, "automaticAdvanceCount") >= automaticBase + 8;
    });
    await event("canonical.time.pause");
    await event("workbench.pause");
    let heldClock = await event("canonical.time.status");
    if (number(heldClock, "tick") === 0) {
        heldClock = await event("canonical.time.step", { ticks: 1 });
    }
    const heldStatus = await event("canonical.status");
    same(object(heldStatus, "published"), beforePublished, "temporal hold old publication");
    const heldFrame = await frame("temporal-object-copy-held", collection, true);
    assertTemporalChange(before, heldFrame, "temporal hold old frame");

    await event(`${gate}.release`);
    await event("workbench.resume");
    const completed = await waitStatus(
        "temporal object-copy release",
        (value) =>
            value.pending === null && value.published !== null &&
            number(object(value, "published"), "token") === token,
    );
    await event("workbench.pause");
    same(
        object(completed, "presentationClock"),
        heldClock,
        "temporal hold release clock ownership",
    );
    await event("canonical.time.set", { tick: 0 });
    return { beforeStatus, scheduled, animated, heldClock, heldStatus, heldFrame, completed };
}
