import { traversalObservationOrder } from "./observation_order.ts";

function order(count: number, last: number | null, observed: number, target: number) {
    return traversalObservationOrder({
        automaticPublicationCount: count,
        lastPublishedToken: last,
        publicationToken: observed,
        targetToken: target,
    });
}

function expectFailure(run: () => unknown): void {
    let failed = false;
    try {
        run();
    } catch {
        failed = true;
    }
    if (!failed) throw new Error("expected traversal observation ordering to fail");
}

Deno.test("observation accepts all valid traversal orders", () => {
    const beforePublication = order(0, null, 1, 1);
    if (
        beforePublication.observedAfterTraversalPublication ||
        beforePublication.revalidatedAfterPublication
    ) {
        throw new Error("pre-publication observation flags diverged");
    }

    const revalidated = order(1, 2, 1, 2);
    if (revalidated.observedAfterTraversalPublication || !revalidated.revalidatedAfterPublication) {
        throw new Error("post-observation traversal flags diverged");
    }

    const afterPublication = order(1, 2, 2, 2);
    if (
        !afterPublication.observedAfterTraversalPublication ||
        afterPublication.revalidatedAfterPublication
    ) {
        throw new Error("post-publication observation flags diverged");
    }
});

Deno.test("observation rejects impossible traversal orders", () => {
    expectFailure(() => order(0, null, 1, 2));
    expectFailure(() => order(1, 2, 1, 1));
    expectFailure(() => order(1, 2, 3, 2));
    expectFailure(() => order(2, 2, 2, 2));
});
