import { actorRenderAdmissionGates } from "../support/actor/admission.ts";
import { actorAnimationEpochGates } from "../support/actor/animation.ts";
import { actorGpuGates } from "../support/actor/gpu.ts";
import { prepareCanonicalFrameSetup } from "../support/canonical-setup.ts";
import {
    fail,
    type Json,
    openSources,
    publish,
    root,
    startClean,
    stopCanonicalProcesses,
    target,
} from "../support/canonical-runtime.ts";

const REVISION = "canonical-actor-v5";
const COLLECTION = "canonical-actor";
const FAR = 2 ** 40;
const BASE: [number, number] = [FAR, -FAR];

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :canonical-actor");
    Deno.exit(0);
}
if (Deno.args.length !== 0) fail(`canonical-actor: unexpected argument ${Deno.args[0]}`);

const started = performance.now();
let acceptance: Json | undefined;
let report: string | undefined;
try {
    await stopCanonicalProcesses();
    const setup = await prepareCanonicalFrameSetup(COLLECTION, [
        BASE,
        [BASE[0] + 2, BASE[1] + 2],
    ]);
    report = setup.paths.report;
    const admission = await actorRenderAdmissionGates(
        setup.paths.terrain,
        setup.paths.objects,
        BASE,
        [BASE[0] + 2, BASE[1] + 2],
    );
    await startClean();
    await openSources(setup.paths.terrain, setup.paths.objects);
    const publication = await publish(target(BASE));
    const actor = await actorGpuGates(BASE, COLLECTION);
    const animationEpoch = await actorAnimationEpochGates(BASE);
    acceptance = {
        revision: REVISION,
        outcome: "pass",
        storage: setup.storage,
        publication,
        admission,
        actor,
        animationEpoch,
        elapsedMilliseconds: performance.now() - started,
    };
} finally {
    await stopCanonicalProcesses();
}

if (!acceptance || !report) fail("canonical actor workflow did not produce acceptance evidence");
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
