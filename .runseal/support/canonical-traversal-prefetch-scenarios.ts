import { capture, expectHalfCounts, probe, stableFrame } from "./canonical-object-composition.ts";
import {
    type Coord,
    publication,
    rollover,
    target,
    targetMatches,
    traversal,
    waitPublished,
    waitStatus,
} from "./canonical-origin-rollover.ts";
import {
    completionReports,
    expectDemandCounts,
    expectPreparedCounts,
    prefetch,
    type PrefetchContext,
    publicationToken,
    setPosition,
    setupPrefetch,
    waitPrefetchCompletion,
    waitPrefetchFailure,
    waitPrefetchPending,
} from "./canonical-traversal-prefetch.ts";
import { event, sleep } from "./global-terrain.ts";
import { fail, field, object, same } from "./terrain.ts";

export async function preparedDirection(
    name: string,
    direction: Coord,
    context: PrefetchContext,
    visual = false,
): Promise<Record<string, unknown>> {
    const baseConfig = target(context.base);
    await setupPrefetch(context.pack, baseConfig, [0, 0]);
    const before = await event("composition.status");
    const beforeToken = publicationToken(before);
    const state = traversal(before);
    const publications = field<number>(state, "automaticPublicationCount", "number");
    const expectedCenter: Coord = [
        context.base[0] + direction[0],
        context.base[1] + direction[1],
    ];
    const expected = target(expectedCenter, 64 + direction[0], 64 + direction[1]);
    await event("terrain.io_gate.arm");
    await setPosition([direction[0] * 5, direction[1] * 5]);
    const pending = await waitPrefetchPending(expected, `${name} pending`);
    await event("workbench.pause");
    if (publicationToken(pending) !== beforeToken) fail(`${name} prefetch published early`);
    const heldProbe = visual ? await probe(baseConfig) : undefined;
    const heldCapture = heldProbe
        ? await capture(`${name}-held`, context.collection, heldProbe)
        : undefined;
    await event("terrain.io_gate.release");
    await event("workbench.resume");
    const completed = await waitPrefetchCompletion(expected, 1, `${name} completion`);
    await event("workbench.pause");
    if (publicationToken(completed) !== beforeToken) fail(`${name} completion published early`);
    const preparedReports = expectPreparedCounts(
        completed,
        direction[0] !== 0 && direction[1] !== 0 ? 16 : 20,
        direction[0] !== 0 && direction[1] !== 0 ? 9 : 5,
    );
    let preparedProbe;
    let preparedCapture;
    if (visual) {
        preparedProbe = await probe(baseConfig);
        preparedCapture = await capture(`${name}-prepared`, context.collection, preparedProbe);
        same(
            stableFrame(preparedProbe, preparedCapture),
            stableFrame(heldProbe!, heldCapture!),
            `${name} speculative frame`,
        );
    }
    await setPosition([direction[0] * 9, direction[1] * 9]);
    await event("workbench.resume");
    const published = await waitPublished(expected, publications + 1);
    await event("workbench.pause");
    const report = await publication(published, expected);
    expectDemandCounts(report, 25, 0);
    return {
        pending,
        completed,
        preparedReports,
        heldProbe,
        heldCapture,
        preparedProbe,
        preparedCapture,
        published,
        report,
    };
}

export async function promotedPrefetch(
    kind: "terrain-io" | "terrain-copy" | "object-copy",
    context: PrefetchContext,
): Promise<Record<string, unknown>> {
    const baseConfig = target(context.base);
    await setupPrefetch(context.pack, baseConfig, [0, 0]);
    const before = await event("composition.status");
    const token = publicationToken(before);
    const publications = field<number>(
        traversal(before),
        "automaticPublicationCount",
        "number",
    );
    const expected = target([context.base[0] + 1, context.base[1]], 65);
    const gate = kind === "object-copy"
        ? "async.gate"
        : kind === "terrain-io"
        ? "terrain.io_gate"
        : "terrain.copy_gate";
    await event(`${gate}.arm`);
    try {
        await setPosition([5, 0]);
        const pending = await waitPrefetchPending(expected, `${kind} prefetch`);
        await setPosition([9, 0]);
        const promoted = await waitStatus(`${kind} promotion`, (status) => {
            const state = prefetch(status);
            return state.promotionCount === 1 && status.pending !== null &&
                object(status, "pending").prefetch === undefined;
        });
        if (publicationToken(promoted) !== token) fail(`${kind} promotion published early`);
        await event("workbench.pause");
        const heldProbe = await probe(baseConfig, false);
        await event(`${gate}.release`);
        await event("workbench.resume");
        const published = await waitPublished(expected, publications + 1);
        await event("workbench.pause");
        const report = await publication(published, expected);
        expectDemandCounts(report, 20, 5);
        return { pending, promoted, heldProbe, published, report };
    } catch (error) {
        try {
            await event(`${gate}.release`);
            await event("workbench.resume");
        } catch {
            // The outer lifecycle owns cleanup if the workbench has already stopped.
        }
        throw error;
    }
}

export async function staleDirection(context: PrefetchContext): Promise<Record<string, unknown>> {
    const baseConfig = target(context.base);
    await setupPrefetch(context.pack, baseConfig, [0, 0]);
    const before = await event("composition.status");
    const token = publicationToken(before);
    const publications = field<number>(
        traversal(before),
        "automaticPublicationCount",
        "number",
    );
    const staleTarget = target([context.base[0] + 1, context.base[1]], 65);
    const demandTarget = target([context.base[0] - 1, context.base[1]], 63);
    await event("terrain.io_gate.arm");
    try {
        await setPosition([5, 0]);
        const pending = await waitPrefetchPending(staleTarget, "stale positive prefetch");
        await setPosition([-9, 0]);
        const queued = await waitStatus(
            "opposite demand queued",
            (status) => targetMatches(traversal(status).queued, demandTarget),
        );
        if (publicationToken(queued) !== token) fail("stale prefetch published before release");
        await event("terrain.io_gate.release");
        const published = await waitPublished(demandTarget, publications + 1);
        await event("workbench.pause");
        const report = await publication(published, demandTarget);
        expectDemandCounts(report, 20, 5);
        const state = traversal(published);
        if (state.queued !== null || state.maxQueuedDepth !== 1) {
            fail("stale prefetch latest-demand bound diverged");
        }
        if (!targetMatches(prefetch(published).lastCompleted, staleTarget)) {
            fail("stale prefetch completion target diverged");
        }
        return { pending, queued, published, report };
    } catch (error) {
        try {
            await event("terrain.io_gate.release");
            await event("workbench.resume");
        } catch {
            // The outer lifecycle owns cleanup if the workbench has already stopped.
        }
        throw error;
    }
}

export async function failedPrefetch(context: PrefetchContext): Promise<Record<string, unknown>> {
    const start: Coord = [context.base[0] + 70, context.base[1]];
    const baseConfig = target(start);
    await setupPrefetch(context.pack, baseConfig, [0, 0]);
    const before = await event("composition.status");
    const token = publicationToken(before);
    const publications = field<number>(
        traversal(before),
        "automaticPublicationCount",
        "number",
    );
    const expected = target([start[0] + 1, start[1]], 65);
    await event("terrain.open", { path: context.missingPack });
    await setPosition([5, 0]);
    const failed = await waitPrefetchFailure(expected, 1);
    if (publicationToken(failed) !== token || traversal(failed).blocked !== null) {
        fail("prefetch failure mutated demand state");
    }
    await sleep(100);
    const stable = await event("composition.status");
    if (prefetch(stable).failureCount !== 1) fail("prefetch failure retried without motion");
    await event("terrain.open", { path: context.pack });
    await setPosition([9, 0]);
    const published = await waitPublished(expected, publications + 1);
    await event("workbench.pause");
    const report = await publication(published, expected);
    expectDemandCounts(report, 20, 5);
    return { failed, stable, published, report };
}

export async function corruptPrefetch(context: PrefetchContext): Promise<Record<string, unknown>> {
    const start: Coord = [context.base[0] + 70, context.base[1]];
    const baseConfig = target(start);
    await setupPrefetch(context.pack, baseConfig, [0, 0]);
    const before = await event("composition.status");
    const token = publicationToken(before);
    const publications = field<number>(
        traversal(before),
        "automaticPublicationCount",
        "number",
    );
    const expected = target([start[0] + 1, start[1]], 65);
    await event("terrain.open", { path: context.corruptPack });
    await setPosition([5, 0]);
    const failed = await waitPrefetchFailure(expected, 1);
    if (publicationToken(failed) !== token || traversal(failed).blocked !== null) {
        fail("corrupt prefetch failure mutated demand state");
    }
    await sleep(100);
    const stable = await event("composition.status");
    if (prefetch(stable).failureCount !== 1) fail("corrupt prefetch retried without motion");
    await event("terrain.open", { path: context.pack });
    await setPosition([9, 0]);
    const published = await waitPublished(expected, publications + 1);
    await event("workbench.pause");
    const report = await publication(published, expected);
    expectHalfCounts(report, "terrain", {
        retainedRegionCount: 20,
        uploadedRegionCount: 5,
        payloadBytes: 20_480,
    });
    expectHalfCounts(report, "instance", {
        retainedRegionCount: 25,
        uploadedRegionCount: 0,
        instanceBytes: 0,
    });
    return { failed, stable, published, report };
}

export async function preparedRollover(
    context: PrefetchContext,
): Promise<Record<string, unknown>> {
    const baseConfig = target(context.base, 96);
    await setupPrefetch(context.pack, baseConfig, [512, 0]);
    const before = await event("composition.status");
    const token = publicationToken(before);
    const rolloverCount = field<number>(rollover(before), "count", "number");
    const publications = field<number>(
        traversal(before),
        "automaticPublicationCount",
        "number",
    );
    const expected = target([context.base[0] + 1, context.base[1]]);
    await setPosition([517, 0]);
    const completed = await waitPrefetchCompletion(expected, 1, "rollover prefetch");
    expectPreparedCounts(completed, 20, 5);
    if (
        publicationToken(completed) !== token ||
        field<number>(rollover(completed), "count", "number") !== rolloverCount
    ) fail("prepared rollover changed the published basis");
    await setPosition([521, 0]);
    const published = await waitPublished(expected, publications + 1);
    await event("workbench.pause");
    const report = await publication(published, expected);
    expectDemandCounts(report, 25, 0);
    if (field<number>(rollover(published), "count", "number") !== rolloverCount + 1) {
        fail("prepared rollover did not commit exactly once");
    }
    return { completed, prepared: completionReports(completed), published, report };
}

export async function disabledPrefetch(
    context: PrefetchContext,
): Promise<Record<string, unknown>> {
    await setupPrefetch(context.pack, target(context.base), [0, 0]);
    await event("workbench.pause");
    await event("composition.prefetch.disable");
    await event("workbench.resume");
    await setPosition([5, 0]);
    await sleep(100);
    const status = await event("composition.status");
    const state = prefetch(status);
    if (state.enabled !== false || state.scheduleCount !== 0 || status.pending !== null) {
        fail("disabled prefetch scheduled work");
    }
    return status;
}
