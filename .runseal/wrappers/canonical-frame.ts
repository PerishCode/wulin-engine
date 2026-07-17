import { assertCanonicalFrameReplay } from "../support/canonical-frame.ts";
import { prepareCanonicalFrameSetup } from "../support/canonical-setup.ts";
import { objectResolutionGates, unavailableObjectResolutionGate } from "../support/object/query.ts";
import { objectNearestGates, unavailableObjectNearestGate } from "../support/object/nearest.ts";
import {
    assertSuppressedFrame,
    assertTargetedFrame,
    assertUntargetedFrame,
    clearObjectSuppression,
    clearObjectTarget,
    invalidObjectFeedbackGate,
    invalidObjectSuppressionGate,
    invalidObjectTargetGate,
    setObjectSuppression,
    setObjectTarget,
    visibleObjectTarget,
} from "../support/object/feedback.ts";
import {
    assertObjectCopies,
    fail,
    frame,
    type Json,
    object,
    openSources,
    publish,
    root,
    same,
    startClean,
    stopCanonicalProcesses,
    string,
    target,
} from "../support/canonical-runtime.ts";

const REVISION = "canonical-frame-v12";
const COLLECTION = "canonical-frame";
const FAR = 2 ** 40;
const BASE: [number, number] = [FAR, -FAR];

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :canonical-frame");
    Deno.exit(0);
}
if (Deno.args.length !== 0) fail(`canonical-frame: unexpected argument ${Deno.args[0]}`);

const started = performance.now();
let acceptance: Json | undefined;
let report: string | undefined;
try {
    await stopCanonicalProcesses();
    const setup = await prepareCanonicalFrameSetup(COLLECTION, [BASE]);
    report = setup.paths.report;
    await startClean();
    const unavailableObjectResolution = await unavailableObjectResolutionGate(BASE);
    const unavailableObjectNearest = await unavailableObjectNearestGate(BASE);
    await openSources(setup.paths.terrain, setup.paths.objects);
    const publication = await publish(target(BASE));
    assertObjectCopies(publication, 25, "canonical frame publication");
    const objectResolution = await objectResolutionGates(
        setup.paths.objects,
        BASE,
        unavailableObjectResolution,
        publication,
    );
    const objectNearest = await objectNearestGates(
        setup.paths.objects,
        BASE,
        unavailableObjectNearest,
    );
    const first = await frame("baseline", COLLECTION);
    const replay = await frame("replay", COLLECTION);
    assertCanonicalFrameReplay(first, replay);
    assertUntargetedFrame(first, "canonical baseline");
    const selected = visibleObjectTarget(
        first,
        string(object(objectResolution, "snapshot"), "sourceNamespace"),
        BASE,
    );
    const identity = selected.identity;
    const invalidTarget = await invalidObjectTargetGate(identity);
    const invalidFeedback = await invalidObjectFeedbackGate(identity);
    const targetSet = await setObjectTarget(identity);
    const targeted = await frame("targeted", COLLECTION);
    const targetedReplay = await frame("targeted-replay", COLLECTION);
    const targetedPixels = assertTargetedFrame(
        targeted,
        identity,
        selected.activeIndex,
        selected.semanticRegion,
        first,
        "canonical targeted frame",
    );
    assertTargetedFrame(
        targetedReplay,
        identity,
        selected.activeIndex,
        selected.semanticRegion,
        first,
        "canonical targeted replay",
    );
    same(targetedReplay.stable, targeted.stable, "canonical targeted immediate replay");
    const activatedSet = await setObjectTarget(identity, "activated");
    const activated = await frame("target-activated", COLLECTION);
    const activatedReplay = await frame("target-activated-replay", COLLECTION);
    const activatedPixels = assertTargetedFrame(
        activated,
        identity,
        selected.activeIndex,
        selected.semanticRegion,
        first,
        "canonical activated frame",
        "activated",
    );
    assertTargetedFrame(
        activatedReplay,
        identity,
        selected.activeIndex,
        selected.semanticRegion,
        first,
        "canonical activated replay",
        "activated",
    );
    same(activatedPixels, targetedPixels, "selected/activated exact target pixel count");
    same(activatedReplay.stable, activated.stable, "canonical activated immediate replay");
    if (
        string(object(activated, "capture"), "color") ===
            string(object(targeted, "capture"), "color")
    ) fail("activated target feedback did not change the selected color");
    const rejectedSet = await setObjectTarget(identity, "rejected");
    const rejected = await frame("target-rejected", COLLECTION);
    const rejectedReplay = await frame("target-rejected-replay", COLLECTION);
    const rejectedPixels = assertTargetedFrame(
        rejected,
        identity,
        selected.activeIndex,
        selected.semanticRegion,
        first,
        "canonical rejected frame",
        "rejected",
    );
    assertTargetedFrame(
        rejectedReplay,
        identity,
        selected.activeIndex,
        selected.semanticRegion,
        first,
        "canonical rejected replay",
        "rejected",
    );
    same(rejectedPixels, targetedPixels, "selected/rejected exact target pixel count");
    same(rejectedReplay.stable, rejected.stable, "canonical rejected immediate replay");
    if (
        string(object(rejected, "capture"), "color") ===
            string(object(targeted, "capture"), "color") ||
        string(object(rejected, "capture"), "color") ===
            string(object(activated, "capture"), "color")
    ) fail("rejected target feedback did not produce one distinct color");
    const targetCleared = await clearObjectTarget();
    const cleared = await frame("target-cleared", COLLECTION);
    assertUntargetedFrame(cleared, "canonical target-cleared frame");
    same(cleared.stable, first.stable, "canonical target-cleared baseline replay");
    const invalidSuppression = await invalidObjectSuppressionGate(identity);
    const suppressionSet = await setObjectSuppression(identity);
    const suppressed = await frame("object-suppressed", COLLECTION);
    const suppressedReplay = await frame("object-suppressed-replay", COLLECTION);
    assertSuppressedFrame(
        suppressed,
        identity,
        selected.activeIndex,
        first,
        "canonical suppressed frame",
    );
    assertSuppressedFrame(
        suppressedReplay,
        identity,
        selected.activeIndex,
        first,
        "canonical suppressed replay",
    );
    same(suppressedReplay.stable, suppressed.stable, "canonical suppression immediate replay");
    const suppressionCleared = await clearObjectSuppression();
    const suppressionClearWarm = await frame("suppression-clear-warm", COLLECTION, false, false);
    const suppressionRestored = await frame("suppression-cleared", COLLECTION);
    same(suppressionRestored.stable, first.stable, "canonical suppression clear replay");
    acceptance = {
        revision: REVISION,
        outcome: "pass",
        storage: setup.storage,
        publication,
        objectResolution,
        objectNearest,
        first,
        replay,
        targetFeedback: {
            identity,
            invalidTarget,
            invalidFeedback,
            targetSet,
            targetedPixels,
            targeted,
            targetedReplay,
            activatedSet,
            activatedPixels,
            activated,
            activatedReplay,
            rejectedSet,
            rejectedPixels,
            rejected,
            rejectedReplay,
            targetCleared,
            cleared,
        },
        objectSuppression: {
            invalidSuppression,
            suppressionSet,
            suppressed,
            suppressedReplay,
            suppressionCleared,
            suppressionClearWarm,
            suppressionRestored,
        },
        elapsedMilliseconds: performance.now() - started,
    };
} finally {
    await stopCanonicalProcesses();
}

if (!acceptance || !report) fail("canonical frame workflow did not produce acceptance evidence");
await Deno.writeTextFile(`${root}/${report}`, `${JSON.stringify(acceptance, null, 2)}\n`);
console.log(JSON.stringify(
    {
        outcome: acceptance.outcome,
        report,
        elapsedMilliseconds: acceptance.elapsedMilliseconds,
    },
    null,
    2,
));
