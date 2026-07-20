import { prepareCanonicalFrameSetup, preparePrototypeSetup } from "../support/canonical-setup.ts";
import {
    fail,
    type Json,
    root,
    run,
    stopCanonicalProcesses,
} from "../support/canonical-runtime.ts";
import { prototypeHostGates } from "../support/prototype/host.ts";
import { requireActivatedFrameDesktop } from "../support/prototype/input/frame_completion_desktop.ts";
import { closeNativeTransport } from "../support/prototype/input/prepared.ts";
import { focusedActivatedFrameGate } from "../support/prototype/object/focused-frame.ts";

const FULL_REVISION = "canonical-prototype-v79";
const FULL_COLLECTION = "canonical-prototype";
const ACTIVATED_FRAME_REVISION = "canonical-prototype-activated-frame-v1";
const ACTIVATED_FRAME_COLLECTION = "canonical-prototype-activated-frame";
const ACTIVATED_FRAME_ARGUMENT = "--case=activated-frame";
const FAR = 2 ** 40;
const BASE: [number, number] = [FAR, -FAR];
const ACTIVATED_FRAME_CENTERS: [number, number][] = [
    BASE,
    [BASE[0] + 1, BASE[1] + 1],
];

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :canonical-prototype [--case=activated-frame]");
    Deno.exit(0);
}
const activatedFrame = Deno.args.length === 1 &&
    Deno.args[0] === ACTIVATED_FRAME_ARGUMENT;
if (Deno.args.length !== 0 && !activatedFrame) {
    fail(`canonical-prototype: unexpected argument ${Deno.args[0]}`);
}

const started = performance.now();
let acceptance: Json | undefined;
let report: string | undefined;
try {
    await stopCanonicalProcesses();
    if (activatedFrame) await requireActivatedFrameDesktop();
    if (!activatedFrame) {
        await run(
            "cargo",
            [
                "test",
                "--locked",
                "-p",
                "engine-runtime",
                "-p",
                "prototype",
                "-p",
                "reference-host",
            ],
            "focused runtime, prototype, and reference-host tests",
        );
    }
    await run(
        "cargo",
        ["build", "--locked", "-p", "prototype"],
        "focused prototype build",
    );
    let storage: Json;
    let prototype: Json;
    if (activatedFrame) {
        const setup = await prepareCanonicalFrameSetup(
            ACTIVATED_FRAME_COLLECTION,
            ACTIVATED_FRAME_CENTERS,
        );
        report = setup.paths.report;
        storage = setup.storage;
        prototype = await focusedActivatedFrameGate(
            setup.paths.terrain,
            setup.paths.objects,
            BASE,
        );
    } else {
        const setup = await preparePrototypeSetup(FULL_COLLECTION, BASE);
        report = setup.paths.report;
        storage = setup.storage;
        prototype = await prototypeHostGates(
            setup.paths.terrain,
            setup.paths.objects,
            setup.paths.objectsCorrupt,
            BASE,
        );
    }
    acceptance = {
        revision: activatedFrame ? ACTIVATED_FRAME_REVISION : FULL_REVISION,
        outcome: "pass",
        ...(activatedFrame ? { case: "activated-frame" } : {}),
        storage,
        prototype,
        elapsedMilliseconds: performance.now() - started,
    };
} finally {
    await stopCanonicalProcesses();
    closeNativeTransport();
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
