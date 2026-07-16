import {
    type Coord,
    event,
    type Json,
    lifecycle,
    number,
    object,
    probe,
    publish,
    setPosition,
    sleep,
    target,
    targetMatches,
    waitStatus,
} from "./canonical-runtime.ts";

export async function preparedRolloverGate(base: Coord): Promise<Json> {
    const basePublication = await publish(target(base, 96));
    await setPosition([512, 0]);
    await event("canonical.traversal.enable");
    await event("canonical.prefetch.enable");
    await event("workbench.resume");
    await sleep(30);
    const before = await event("canonical.status");
    const traversal = object(before, "traversal");
    const automaticBefore = number(traversal, "automaticPublicationCount");
    const rolloverCount = number(object(traversal, "rollover"), "count");
    const prefetchBefore = number(object(traversal, "prefetch"), "completionCount");
    const rolloverTarget = target([base[0] + 1, base[1]]);
    await setPosition([517, 0]);
    const prepared = await waitStatus("rollover preparation", (value) => {
        if (value.pending !== null) return false;
        const prefetch = object(object(value, "traversal"), "prefetch");
        return number(prefetch, "completionCount") === prefetchBefore + 1 &&
            targetMatches(prefetch.lastCompleted, rolloverTarget);
    });
    if (number(object(object(prepared, "traversal"), "rollover"), "count") !== rolloverCount) {
        throw new Error("prepared rollover committed before demand");
    }
    await setPosition([521, 0]);
    const published = await waitStatus("rollover publication", (value) => {
        if (value.pending !== null || !targetMatches(value.published, rolloverTarget)) return false;
        const current = object(value, "traversal");
        return number(current, "automaticPublicationCount") === automaticBefore + 1 &&
            number(object(current, "rollover"), "count") === rolloverCount + 1;
    });
    await event("workbench.pause");
    const evidence = await probe();
    await event("canonical.traversal.disable");
    await lifecycle("stop");
    return { basePublication, prepared, published, evidence };
}
