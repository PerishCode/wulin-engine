import {
    capture,
    expectHalfCounts,
    expectPairMovement,
    probe,
    stableFrame,
} from "./canonical-object-composition.ts";
import { retainedOverlap } from "./canonical-origin-rollover-evidence.ts";
import {
    camera,
    type Coord,
    enable,
    expectCamera,
    expectEvent,
    publication,
    rollover,
    setCamera,
    target,
    targetMatches,
    traversal,
    waitPublished,
    waitStatus,
} from "./canonical-origin-rollover.ts";
import { type GlobalConfig, globalConfig, publishPair } from "./global-composition.ts";
import { event, sleep } from "./global-terrain.ts";
import { restoreByte } from "./signed-terrain-storage.ts";
import { captureEvidence, fail, field, object, same } from "./terrain.ts";
import { traversal as baseTraversal } from "./traversal.ts";

export type ScenarioContext = {
    base: Coord;
    collection: string;
    pack: string;
    corruptPack: string;
    missingPack: string;
    corruption: { offset: number; original: number; corrupted: number };
};

export async function normalization(
    center: Coord,
    local: Coord,
    id: string,
    context: ScenarioContext,
): Promise<Record<string, unknown>> {
    const beforeConfig = target(center, local[0], local[1]);
    await setCamera(local[0], local[1]);
    const beforeProbe = await probe(beforeConfig);
    const beforeCapture = await capture(`${id}-before`, context.collection, beforeProbe);
    await enable();
    const afterConfig = target(center);
    const status = await waitPublished(afterConfig, 1);
    await event("workbench.pause");
    const report = await publication(status, afterConfig);
    expectPairMovement(report, 25, 0, 0, 0);
    const delta: Coord = [64 - local[0], 64 - local[1]];
    expectEvent(status, {
        oldOrigin: [center[0] - (local[0] - 64), center[1] - (local[1] - 64)],
        newOrigin: center,
        center,
        localCenter: [64, 64],
        delta,
        count: 1,
    });
    expectCamera(await camera(), 64, 64);
    const afterProbe = await probe(afterConfig);
    const afterCapture = await capture(`${id}-after`, context.collection, afterProbe);
    same(
        stableFrame(afterProbe, afterCapture),
        stableFrame(beforeProbe, beforeCapture),
        `${id} same-window frame`,
    );
    return { beforeConfig, beforeProbe, beforeCapture, status, report, afterProbe, afterCapture };
}

export async function boundary(
    name: string,
    origin: Coord,
    edge: Coord,
    next: Coord,
    context: ScenarioContext,
): Promise<Record<string, unknown>> {
    await event("composition.traversal.disable");
    const edgeCenter: Coord = [origin[0] + edge[0] - 64, origin[1] + edge[1] - 64];
    const edgeConfig = globalConfig(origin, edgeCenter);
    await publishPair(edgeConfig);
    await setCamera(edge[0], edge[1]);
    const enabled = await enable();
    const count = field<number>(rollover(enabled), "count", "number");
    const publications = field<number>(traversal(enabled), "automaticPublicationCount", "number");
    const beforeProbe = await probe(edgeConfig);
    const desired: Coord = [origin[0] + next[0] - 64, origin[1] + next[1] - 64];
    const crossed = [next[0] < 32 || next[0] > 96, next[1] < 32 || next[1] > 96];
    const newOrigin: Coord = [
        crossed[0] ? desired[0] : origin[0],
        crossed[1] ? desired[1] : origin[1],
    ];
    const local: Coord = [crossed[0] ? 64 : next[0], crossed[1] ? 64 : next[1]];
    const expected = globalConfig(newOrigin, desired);
    await setCamera(next[0], next[1]);
    const status = await waitPublished(expected, publications + 1);
    await event("workbench.pause");
    const report = await publication(status, expected);
    const retained = crossed[0] && crossed[1] ? 16 : 20;
    const uploaded = 25 - retained;
    expectPairMovement(report, retained, uploaded, uploaded * 4096, uploaded * 20_480);
    const delta: Coord = [origin[0] - newOrigin[0], origin[1] - newOrigin[1]];
    expectEvent(status, {
        oldOrigin: origin,
        newOrigin,
        center: desired,
        localCenter: local,
        delta,
        count: count + 1,
    });
    expectCamera(await camera(), local[0], local[1]);
    const afterProbe = await probe(expected);
    const afterCapture = await capture(name, context.collection, afterProbe);
    return {
        edgeConfig,
        enabled,
        status,
        report,
        retained: retainedOverlap(beforeProbe, afterProbe, retained),
        probe: afterProbe,
        capture: afterCapture,
    };
}

async function setupPositiveEdge(base: Coord): Promise<Record<string, unknown>> {
    await event("composition.traversal.disable");
    const config = globalConfig(base, [base[0] + 32, base[1]]);
    await publishPair(config);
    await setCamera(96);
    const status = await enable();
    return { config, status };
}

async function rawCapture(id: string, collection: string): Promise<Record<string, unknown>> {
    const raw = await event("perception.capture", {
        id,
        collection,
        samples: [{ x: 0, y: 0 }, { x: 640, y: 360 }],
    });
    return {
        ...captureEvidence(raw),
        png: field<string>(object(raw, "image"), "pngSha256", "string"),
    };
}

function expectResidentMovement(report: Record<string, unknown>, minimumRetained: number): void {
    const halves = object(report, "halves");
    const terrain = object(halves, "terrain");
    const instance = object(halves, "instance");
    const retained = field<number>(terrain, "retainedRegionCount", "number");
    const uploaded = field<number>(terrain, "uploadedRegionCount", "number");
    expectHalfCounts(report, "terrain", {});
    expectHalfCounts(report, "instance", {
        retainedRegionCount: retained,
        uploadedRegionCount: uploaded,
    });
    if (
        retained < minimumRetained || retained + uploaded !== 25 ||
        terrain.payloadBytes !== uploaded * 4096 ||
        object(terrain, "io").payloadBytes !== uploaded * 4096 ||
        instance.instanceBytes !== uploaded * 20_480
    ) fail("held rollover resident movement diverged");
}

export async function heldRollover(
    kind: "terrain-io" | "terrain-copy" | "object-copy",
    context: ScenarioContext,
): Promise<Record<string, unknown>> {
    const { base, collection } = context;
    const setup = await setupPositiveEdge(base);
    const setupStatus = object(setup, "status");
    const before = traversal(setupStatus);
    const schedules = field<number>(before, "automaticScheduleCount", "number");
    const publications = field<number>(before, "automaticPublicationCount", "number");
    const count = field<number>(rollover(setupStatus), "count", "number");
    const gate = kind === "object-copy"
        ? "async.gate"
        : kind === "terrain-io"
        ? "terrain.io_gate"
        : "terrain.copy_gate";
    await event(`${gate}.arm`);
    try {
        await setCamera(97);
        const firstTarget = target([base[0] + 33, base[1]]);
        const pending = await waitStatus(`${kind} first target`, (status) => {
            if (!status.pending || !targetMatches(status.pending, firstTarget)) return false;
            const pair = object(status, "pending");
            return kind === "object-copy"
                ? pair.terrainStage === "staged" && pair.instanceStage === "in-flight"
                : pair.instanceStage === "staged" && pair.terrainStage === "in-flight";
        });
        await setCamera(98);
        await setCamera(99);
        const queuedTarget = target([base[0] + 35, base[1]]);
        const queued = await waitStatus(
            `${kind} latest target`,
            (status) => targetMatches(traversal(status).queued, queuedTarget),
        );
        await event("workbench.pause");
        expectCamera(await camera(), 99);
        const heldProbe = await probe(setup.config as GlobalConfig, false);
        const heldCapture = await rawCapture(`${kind}-held`, collection);
        const stableCapture = await rawCapture(`${kind}-held-stable`, collection);
        same(stableCapture, heldCapture, `${kind} held attachments`);
        await event(`${gate}.release`);
        await event("workbench.resume");
        const expected = globalConfig([base[0] + 33, base[1]], [base[0] + 35, base[1]]);
        const settled = await waitPublished(expected, publications + 2);
        await event("workbench.pause");
        const report = await publication(settled, expected);
        expectResidentMovement(report, 15);
        const state = traversal(settled);
        if (
            state.automaticScheduleCount !== schedules + 2 ||
            state.automaticPublicationCount !== publications + 2 || state.queued !== null
        ) fail(`${kind} latest-wins counters diverged`);
        expectEvent(settled, {
            oldOrigin: base,
            newOrigin: [base[0] + 33, base[1]],
            center: [base[0] + 33, base[1]],
            localCenter: [64, 64],
            delta: [-33, 0],
            count: count + 1,
        });
        expectCamera(await camera(), 66);
        const settledProbe = await probe(expected);
        const settledCapture = await capture(`${kind}-settled`, collection, settledProbe);
        return {
            setup,
            pending,
            queued,
            heldProbe,
            heldCapture,
            settled,
            report,
            settledProbe,
            settledCapture,
        };
    } catch (error) {
        try {
            await event(`${gate}.release`);
            await event("workbench.resume");
        } catch {
            // The outer lifecycle owns cleanup if the inspect path is already gone.
        }
        throw error;
    }
}

async function waitBlocked(expected: GlobalConfig): Promise<Record<string, unknown>> {
    return await waitStatus(
        "rollover failure block",
        (status) => status.pending === null && targetMatches(traversal(status).blocked, expected),
    );
}

export async function failures(context: ScenarioContext): Promise<Record<string, unknown>> {
    const { base, collection, corruptPack, missingPack, pack, corruption } = context;
    const missingSetup = await setupPositiveEdge(base);
    const missingStatus = object(missingSetup, "status");
    const missingBefore = traversal(missingStatus);
    const missingCount = field<number>(rollover(missingStatus), "count", "number");
    await event("terrain.open", { path: missingPack });
    await setCamera(97);
    const failedTarget = target([base[0] + 33, base[1]]);
    const missing = await waitBlocked(failedTarget);
    expectCamera(await camera(), 97);
    await sleep(250);
    const missingStable = await event("composition.status");
    if (
        traversal(missingStable).automaticAttemptCount !==
            field<number>(traversal(missing), "automaticAttemptCount", "number") ||
        rollover(missingStable).count !== missingCount
    ) fail("missing rollover retried or changed basis");
    await event("terrain.open", { path: pack });
    await setCamera(95);
    const recoveredConfig = globalConfig(base, [base[0] + 31, base[1]]);
    const recovered = await waitPublished(
        recoveredConfig,
        field<number>(missingBefore, "automaticPublicationCount", "number") + 1,
    );

    const corruptBase: Coord = [base[0] + 37, base[1]];
    const corruptSetup = await setupPositiveEdge(corruptBase);
    const corruptCount = field<number>(
        rollover(object(corruptSetup, "status")),
        "count",
        "number",
    );
    await event("terrain.open", { path: corruptPack });
    await setCamera(97);
    const corruptTarget = target([corruptBase[0] + 33, corruptBase[1]]);
    const corrupt = await waitBlocked(corruptTarget);
    expectCamera(await camera(), 97);
    await restoreByte(corruptPack, corruption);
    await setCamera(95);
    const corruptRecovery = globalConfig(corruptBase, [corruptBase[0] + 31, corruptBase[1]]);
    await waitPublished(corruptRecovery);
    await setCamera(97);
    const retry = await waitPublished(corruptTarget);
    await event("workbench.pause");
    const retryReport = await publication(retry, corruptTarget);
    expectHalfCounts(retryReport, "terrain", {
        retainedRegionCount: 20,
        uploadedRegionCount: 5,
        payloadBytes: 20_480,
    });
    expectHalfCounts(retryReport, "instance", {
        retainedRegionCount: 25,
        uploadedRegionCount: 0,
        instanceBytes: 0,
    });
    if (field<number>(rollover(retry), "count", "number") !== corruptCount + 1) {
        fail("corrupt retry did not commit exactly one rollover");
    }
    expectCamera(await camera(), 64);
    const retryProbe = await probe(corruptTarget);
    const retryCapture = await capture("corrupt-retry", collection, retryProbe);
    return {
        missingSetup,
        missing,
        missingStable,
        recovered,
        corruptSetup,
        corrupt,
        retry,
        retryReport,
        retryProbe,
        retryCapture,
    };
}

export async function disableCatchUp(context: ScenarioContext): Promise<Record<string, unknown>> {
    const setup = await setupPositiveEdge(context.base);
    const before = traversal(object(setup, "status"));
    const publications = field<number>(before, "automaticPublicationCount", "number");
    await event("composition.traversal.disable");
    await setCamera(97);
    await event("workbench.resume");
    await sleep(200);
    const disabled = await event("composition.status");
    const disabledTraversal = baseTraversal(disabled);
    if (
        disabledTraversal.enabled !== false || "rollover" in disabledTraversal ||
        disabled.pending !== null || disabledTraversal.automaticPublicationCount !== publications
    ) fail("disabled canonical traversal crossed a boundary");
    await enable();
    const expected = target([context.base[0] + 33, context.base[1]]);
    const caughtUp = await waitPublished(expected, publications + 1);
    await event("workbench.pause");
    expectCamera(await camera(), 64);
    return { setup, disabled, caughtUp };
}
