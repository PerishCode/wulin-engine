import { prepareCanonicalFrameSetup } from "../support/canonical-setup.ts";
import {
    assertStopped,
    fail,
    type Json,
    lifecycle,
    lifecycleCycles,
    number,
    openSources,
    operationMetrics,
    publish,
    resourcePlateau,
    root,
    startClean,
    status,
    stopCanonicalProcesses,
    target,
} from "../support/canonical-runtime.ts";

const REVISION = "canonical-resources-v2";
const COLLECTION = "canonical-resources";
const FAR = 2 ** 40;
const BASE: [number, number] = [FAR, -FAR];

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :canonical-resources");
    Deno.exit(0);
}
if (Deno.args.length !== 0) fail(`canonical-resources: unexpected argument ${Deno.args[0]}`);

const started = performance.now();
let acceptance: Json | undefined;
let report: string | undefined;
try {
    await stopCanonicalProcesses();
    const setup = await prepareCanonicalFrameSetup(COLLECTION, [
        BASE,
        [BASE[0] + 40, BASE[1]],
        [BASE[0] + 41, BASE[1]],
    ]);
    report = setup.paths.report;
    await startClean();
    await openSources(setup.paths.terrain, setup.paths.objects);
    const publication = await publish(target(BASE));
    const resources = await resourcePlateau(BASE);
    const resourceProcessId = number(await status(), "processId");
    await lifecycle("stop");
    await assertStopped(resourceProcessId);
    const lifecycleEvidence = await lifecycleCycles(
        setup.paths.terrain,
        setup.paths.objects,
        target(BASE),
        16,
    );
    acceptance = {
        revision: REVISION,
        outcome: "pass",
        storage: setup.storage,
        publication,
        resources,
        lifecycle: lifecycleEvidence,
        operations: operationMetrics(),
        elapsedMilliseconds: performance.now() - started,
    };
} finally {
    await stopCanonicalProcesses();
}

if (!acceptance || !report) fail("canonical resource workflow did not produce acceptance evidence");
await Deno.writeTextFile(`${root}/${report}`, `${JSON.stringify(acceptance, null, 2)}\n`);
console.log(
    JSON.stringify(
        {
            outcome: acceptance.outcome,
            report,
            elapsedMilliseconds: acceptance.elapsedMilliseconds,
        },
        null,
        2,
    ),
);
