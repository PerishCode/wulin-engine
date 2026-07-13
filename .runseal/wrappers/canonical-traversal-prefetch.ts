import { capability, packCenters } from "../support/canonical-origin-rollover-evidence.ts";
import {
    corruptPrefetch,
    disabledPrefetch,
    failedPrefetch,
    preparedDirection,
    preparedRollover,
    promotedPrefetch,
    staleDirection,
} from "../support/canonical-traversal-prefetch-scenarios.ts";
import {
    preparedSweep,
    unpreparedSweep,
} from "../support/canonical-traversal-prefetch-evidence.ts";
import type { PrefetchContext } from "../support/canonical-traversal-prefetch.ts";
import {
    assertStopped,
    event,
    lifecycle,
    root,
    run,
    useSidecar,
} from "../support/global-terrain.ts";
import { cookSigned, corruptSignedPayload } from "../support/signed-terrain-storage.ts";
import { collectEnvironment, fail, field } from "../support/terrain.ts";

const REVISION = "canonical-traversal-prefetch-v1";
const COLLECTION = "0027-canonical-traversal-prefetch";
const PACK = `out/terrain/${COLLECTION}/terrain.wlt`;
const CORRUPT = `out/terrain/${COLLECTION}/terrain-corrupt.wlt`;
const REPORT = `out/captures/${COLLECTION}/acceptance.json`;
const MISSING_PACK = "out/terrain/0023-signed-terrain-storage/terrain-a.wlt";
const FAR = 2 ** 40;
const BASE: [number, number] = [FAR, -FAR];

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :canonical-traversal-prefetch");
    Deno.exit(0);
}
if (Deno.args.length > 0) fail(`unexpected argument ${Deno.args[0]}`);

useSidecar("sidecar.toml");
await lifecycle("stop");
useSidecar("sidecar.benchmark.toml");
await lifecycle("stop");
useSidecar("sidecar.toml");
await run("runseal", [":canonical-origin-rollover"], "Experiment 0026 compatibility workflow");

await Deno.mkdir(`${root}/out/terrain/${COLLECTION}`, { recursive: true });
const cooked = await cookSigned(PACK, [
    ...packCenters([BASE]),
    [BASE[0] + 1, BASE[1] + 1],
    [BASE[0] - 1, BASE[1] - 1],
]);
await Deno.copyFile(`${root}/${PACK}`, `${root}/${CORRUPT}`);
const corruption = await corruptSignedPayload(CORRUPT, [BASE[0] + 73, BASE[1]]);
const context: PrefetchContext = {
    collection: COLLECTION,
    pack: PACK,
    missingPack: MISSING_PACK,
    corruptPack: CORRUPT,
    base: BASE,
};
const environment = await collectEnvironment(root);
let finalReport: Record<string, unknown> | undefined;

try {
    const directions = {
        positiveX: await preparedDirection("positive-x", [1, 0], context, true),
        negativeX: await preparedDirection("negative-x", [-1, 0], context),
        positiveZ: await preparedDirection("positive-z", [0, 1], context),
        negativeZ: await preparedDirection("negative-z", [0, -1], context),
        positiveDiagonal: await preparedDirection("positive-diagonal", [1, 1], context),
        negativeDiagonal: await preparedDirection("negative-diagonal", [-1, -1], context),
    };
    const promotions = {
        terrainIo: await promotedPrefetch("terrain-io", context),
        terrainCopy: await promotedPrefetch("terrain-copy", context),
        objectCopy: await promotedPrefetch("object-copy", context),
    };
    const stale = await staleDirection(context);
    const failure = await failedPrefetch(context);
    const corrupt = await corruptPrefetch(context);
    const rollover = await preparedRollover(context);
    const disabled = await disabledPrefetch(context);
    const restart = await preparedDirection("restart-positive-x", [1, 0], context);

    useSidecar("sidecar.toml");
    const correctnessStatus = await event("workbench.status");
    const correctnessRenderer = capability(correctnessStatus, true);
    const correctnessProcess = field<number>(correctnessStatus, "processId", "number");
    await lifecycle("stop");

    const control = await unpreparedSweep(context);
    const prepared = await preparedSweep(context);
    finalReport = {
        revision: REVISION,
        outcome: "pass",
        environment,
        compatibility: {
            canonicalOriginRollover: "out/captures/0026-canonical-origin-rollover/acceptance.json",
        },
        cooked,
        corruption,
        correctness: {
            processId: correctnessProcess,
            renderer: correctnessRenderer,
            directions,
            promotions,
            stale,
            failure,
            corrupt,
            rollover,
            disabled,
            restart,
        },
        benchmark: { control, prepared },
    };
} finally {
    useSidecar("sidecar.toml");
    await lifecycle("stop");
    useSidecar("sidecar.benchmark.toml");
    await lifecycle("stop");
}

await assertStopped("sidecar.toml");
await assertStopped("sidecar.benchmark.toml");
if (!finalReport) fail("canonical traversal prefetch workflow did not produce a report");
await Deno.mkdir(`${root}/out/captures/${COLLECTION}`, { recursive: true });
await Deno.writeTextFile(`${root}/${REPORT}`, `${JSON.stringify(finalReport, null, 2)}\n`);
console.log(JSON.stringify(finalReport, null, 2));
