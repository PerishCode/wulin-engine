import { type Coord, type Json } from "../../canonical-runtime.ts";
import {
    CONFIG,
    document,
    EXECUTABLE,
    simulationDriverInvariant,
    startupDocumentExpectation,
    startupInvariant,
    writeDocument,
} from "../host.ts";
import { activatedObjectFeedbackInvariant } from "./gates.ts";
import { objectFeedbackSession } from "../sessions/mod.ts";
import { MINIMUM_COPIED_SUBTREE_BYTES, requireSingleOwnerInvariant } from "../sessions/gates.ts";
import { STATIONARY_COMMAND } from "../simulation.ts";

export async function focusedActivatedFrameGate(
    terrain: string,
    objects: string,
    base: Coord,
): Promise<Json> {
    const runtimeDocument = document(terrain, objects, base);
    const expectedStartup = await startupDocumentExpectation(runtimeDocument);
    await writeDocument(runtimeDocument);
    const launch = await objectFeedbackSession(
        EXECUTABLE,
        CONFIG,
        "prototype focused Activated-frame acceptance",
        true,
    );
    const invariant = await activatedObjectFeedbackInvariant(
        launch,
        expectedStartup,
        objects,
        base,
        startupInvariant,
        (value) => simulationDriverInvariant(value, STATIONARY_COMMAND),
    );
    requireSingleOwnerInvariant(
        launch,
        invariant,
        "prototype focused Activated-frame invariant",
    );
    return {
        configPath: CONFIG,
        launch,
        invariant,
        singleOwnerInvariantEvidence: {
            revision: "prototype-single-owner-invariant-evidence-v1",
            launchCount: 1,
            minimumCopiedSubtreeBytes: MINIMUM_COPIED_SUBTREE_BYTES,
            nontrivialCopiedSubtreeCount: 0,
        },
    };
}
