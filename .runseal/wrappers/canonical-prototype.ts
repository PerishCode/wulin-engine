import { preparePrototypeSetup } from "../support/canonical-setup.ts";
import {
    fail,
    type Json,
    root,
    run,
    stopCanonicalProcesses,
} from "../support/canonical-runtime.ts";
import { prototypeHostGates } from "../support/prototype-host.ts";

const REVISION = "canonical-prototype-v1";
const COLLECTION = "canonical-prototype";
const FAR = 2 ** 40;
const BASE: [number, number] = [FAR, -FAR];

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :canonical-prototype");
    Deno.exit(0);
}
if (Deno.args.length !== 0) fail(`canonical-prototype: unexpected argument ${Deno.args[0]}`);

const started = performance.now();
let acceptance: Json | undefined;
let report: string | undefined;
try {
    await stopCanonicalProcesses();
    await run(
        "cargo",
        ["test", "--locked", "-p", "prototype", "-p", "reference-host"],
        "focused prototype and reference-host tests",
    );
    await run(
        "cargo",
        ["build", "--locked", "-p", "prototype"],
        "focused prototype build",
    );
    const setup = await preparePrototypeSetup(COLLECTION, BASE);
    report = setup.paths.report;
    const prototype = await prototypeHostGates(
        setup.paths.terrain,
        setup.paths.objects,
        setup.paths.objectsCorrupt,
        BASE,
    );
    acceptance = {
        revision: REVISION,
        outcome: "pass",
        storage: setup.storage,
        prototype,
        elapsedMilliseconds: performance.now() - started,
    };
} finally {
    await stopCanonicalProcesses();
}

if (!acceptance || !report) {
    fail("canonical prototype workflow did not produce acceptance evidence");
}
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
