export type TraversalObservationInput = {
    automaticPublicationCount: number;
    lastPublishedToken: number | null;
    publicationToken: number;
    targetToken: number;
};

export type TraversalObservationOrder = {
    observedAfterTraversalPublication: boolean;
    revalidatedAfterPublication: boolean;
};

export function traversalObservationOrder(
    input: TraversalObservationInput,
): TraversalObservationOrder {
    const {
        automaticPublicationCount,
        lastPublishedToken,
        publicationToken,
        targetToken,
    } = input;
    let observedAfterTraversalPublication = false;
    if (automaticPublicationCount === 0) {
        if (lastPublishedToken !== null || targetToken !== publicationToken) {
            throw new Error("prototype target changed without a completed traversal publication");
        }
    } else if (automaticPublicationCount === 1) {
        if (publicationToken > targetToken || lastPublishedToken !== targetToken) {
            throw new Error("prototype target snapshot diverged from traversal publication");
        }
        observedAfterTraversalPublication = targetToken === publicationToken;
    } else {
        throw new Error("prototype target observed an excessive traversal publication count");
    }
    return {
        observedAfterTraversalPublication,
        revalidatedAfterPublication: targetToken !== publicationToken,
    };
}
