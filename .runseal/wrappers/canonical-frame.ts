import { assertCanonicalFrameReplay } from "../support/canonical-frame.ts";
import { prepareCanonicalFrameSetup } from "../support/canonical-setup.ts";
import {
    assertObjectCopies,
    fail,
    frame,
    type Json,
    openSources,
    publish,
    root,
    startClean,
    stopCanonicalProcesses,
    target,
} from "../support/canonical-runtime.ts";

const REVISION = "canonical-frame-v1";
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
    await openSources(setup.paths.terrain, setup.paths.objects);
    const publication = await publish(target(BASE));
    assertObjectCopies(publication, 25, "canonical frame publication");
    const first = await frame("baseline", COLLECTION);
    const replay = await frame("replay", COLLECTION);
    assertCanonicalFrameReplay(first, replay);
    acceptance = {
        revision: REVISION,
        outcome: "pass",
        storage: setup.storage,
        publication,
        first,
        replay,
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
