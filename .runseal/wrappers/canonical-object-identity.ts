import { capability, packCenters } from "../support/canonical-origin-rollover-evidence.ts";
import { startClean, target } from "../support/canonical-origin-rollover.ts";
import {
    asyncStatus,
    authorityProbe,
    objectFailure,
    sourceEvidence,
} from "../support/authoritative-cooked-objects/mod.ts";
import {
    cookIdentity,
    corruptIdentityPlane,
    duplicateIdentity,
    identityBehaviorEvidence,
    identityEvidence,
    identityReadback,
    sourceNamespace,
    validateIdentityAuthority,
    validateIdentityTransaction,
} from "../support/canonical-object-identity/mod.ts";
import { capture, probe } from "../support/canonical-object-composition.ts";
import { setAliasCamera } from "../support/camera-relative-terrain.ts";
import {
    preparedSweep,
    unpreparedSweep,
} from "../support/canonical-traversal-prefetch-evidence.ts";
import type { PrefetchContext } from "../support/canonical-traversal-prefetch.ts";
import { publishPair, waitPair } from "../support/global-composition.ts";
import {
    assertStopped,
    event,
    lifecycle,
    root,
    run,
    sleep,
    useSidecar,
} from "../support/global-terrain.ts";
import { collectEnvironment, fail, field, object, same } from "../support/terrain.ts";

const REVISION = "canonical-object-identity-plane-v1";
const COLLECTION = "0030-canonical-object-identity-plane";
const PRIOR = "0029-authoritative-cooked-objects";
const TERRAIN = `out/terrain/${PRIOR}/terrain-a.wlt`;
const SCHEMA1 = `out/cooked/${PRIOR}/authority-a.wlr`;
const ORDER_A = `out/cooked/${COLLECTION}/identity-a.wlr`;
const ORDER_A_REPEAT = `out/cooked/${COLLECTION}/identity-a-repeat.wlr`;
const ORDER_B = `out/cooked/${COLLECTION}/identity-b.wlr`;
const MISSING = `out/cooked/${COLLECTION}/identity-missing.wlr`;
const CORRUPT_RECORD = `out/cooked/${COLLECTION}/identity-corrupt-record.wlr`;
const CORRUPT_ID = `out/cooked/${COLLECTION}/identity-corrupt-id.wlr`;
const DUPLICATE_ID = `out/cooked/${COLLECTION}/identity-duplicate-id.wlr`;
const REPORT = `out/captures/${COLLECTION}/acceptance.json`;
const FAR = 2 ** 40;
const BASE: [number, number] = [FAR, -FAR];

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :canonical-object-identity");
    Deno.exit(0);
}
if (Deno.args.length > 0) fail(`unexpected argument ${Deno.args[0]}`);

useSidecar("sidecar.toml");
await lifecycle("stop");
useSidecar("sidecar.benchmark.toml");
await lifecycle("stop");
useSidecar("sidecar.toml");
await run("runseal", [":authoritative-cooked-objects"], "Experiment 0029 compatibility workflow");
await run(
    "cargo",
    ["test", "--locked", "-p", "region-format", "-p", "region-cooker"],
    "canonical identity codec and cooker tests",
);

const centers = [
    ...packCenters([BASE]),
    [BASE[0] + 1, BASE[1] + 1] as [number, number],
    [BASE[0] - 1, BASE[1] - 1] as [number, number],
];
await Deno.mkdir(`${root}/out/cooked/${COLLECTION}`, { recursive: true });
const orderA = await cookIdentity(ORDER_A, centers, "a");
const orderARepeat = await cookIdentity(ORDER_A_REPEAT, centers, "a");
const orderB = await cookIdentity(ORDER_B, centers, "b");
const missing = await cookIdentity(MISSING, [BASE], "a");
if (
    orderA.fileSha256 !== orderARepeat.fileSha256 ||
    sourceNamespace(orderA) !== sourceNamespace(orderARepeat) ||
    sourceNamespace(orderA) === sourceNamespace(orderB) ||
    object(orderA, "metadata").stableSeedNamespaceSha256 !==
        object(orderB, "metadata").stableSeedNamespaceSha256
) fail("canonical identity cooker determinism or namespace isolation failed");

for (const path of [CORRUPT_RECORD, CORRUPT_ID, DUPLICATE_ID]) {
    await Deno.copyFile(`${root}/${ORDER_A}`, `${root}/${path}`);
}
const failureRegion: [number, number] = [BASE[0] + 73, BASE[1]];
const corruptRecord = await corruptIdentityPlane(CORRUPT_RECORD, failureRegion, "record");
const corruptId = await corruptIdentityPlane(CORRUPT_ID, failureRegion, "identity");
const duplicateId = await duplicateIdentity(DUPLICATE_ID, failureRegion);

const environment = await collectEnvironment(root);
const baseConfig = target(BASE);
const adjacentConfig = target([BASE[0] + 1, BASE[1]]);
const diagonalConfig = target([BASE[0] + 1, BASE[1] + 1]);
const aliasConfig = target(BASE, 65, 64);
const context: PrefetchContext = {
    collection: COLLECTION,
    pack: TERRAIN,
    missingPack: "out/terrain/0023-signed-terrain-storage/terrain-a.wlt",
    corruptPack: "out/terrain/0027-canonical-traversal-prefetch/terrain-corrupt.wlt",
    objectPack: ORDER_A,
    base: BASE,
};
let finalReport: Record<string, unknown> | undefined;

try {
    await startClean("sidecar.toml");
    const schema1 = await sourceEvidence(
        TERRAIN,
        baseConfig,
        COLLECTION,
        "identity-schema1",
        SCHEMA1,
    );
    const schema1Transaction = object(await asyncStatus(), "lastCompleted");
    if (
        schema1Transaction.identityCopyCount !== 0 ||
        schema1Transaction.identityCopyBytes !== 0
    ) fail("fresh schema-1 publication paid a transaction identity-copy cost");
    await identityReadback(1);

    const first = await switchAndProbe(ORDER_A, baseConfig, "identity-order-a");
    await validateIdentityTransaction(25);
    const firstEvidence = identityEvidence(first.probe, first.capture);
    same(
        identityBehaviorEvidence(
            object(schema1, "probe"),
            object(schema1, "capture"),
        ),
        firstEvidence,
        "schema-1 ordinal and schema-2 authored identity",
    );
    const second = await switchAndProbe(ORDER_B, baseConfig, "identity-order-b");
    await validateIdentityTransaction(25);
    const secondEvidence = identityEvidence(second.probe, second.capture);
    same(firstEvidence, secondEvidence, "canonical object identity reorder invariant");
    if (
        validateIdentityAuthority(first.probe).payloadSha256 ===
            validateIdentityAuthority(second.probe).payloadSha256
    ) fail("canonical identity orders did not change physical payload order");

    const revisit = await switchAndProbe(ORDER_A, baseConfig, "identity-order-a-revisit");
    await validateIdentityTransaction(0);
    same(firstEvidence, identityEvidence(revisit.probe, revisit.capture), "identity revisit");

    const heldIo = await heldTransition("objects.gate", baseConfig, adjacentConfig);
    const adjacentProbe = await authorityProbe(adjacentConfig);
    await validateIdentityTransaction(5);
    const heldCopy = await heldTransition("async.gate", adjacentConfig, diagonalConfig);
    const diagonalProbe = await authorityProbe(diagonalConfig);
    await validateIdentityTransaction(5);

    const returned = await publishPair(baseConfig);
    const returnedProbe = await authorityProbe(baseConfig);
    const returnedCapture = await capture("identity-returned", COLLECTION, returnedProbe);
    same(
        firstEvidence,
        identityEvidence(returnedProbe, returnedCapture),
        "identity movement revisit",
    );
    const alias = await publishPair(aliasConfig);
    await setAliasCamera(65);
    const aliasProbe = await authorityProbe(aliasConfig);
    const aliasCapture = await capture("identity-alias", COLLECTION, aliasProbe);
    same(firstEvidence, identityEvidence(aliasProbe, aliasCapture), "identity alias rebind");

    const missingFailure = await objectFailure("identity-missing", MISSING, context);
    const recordFailure = await objectFailure("identity-record", CORRUPT_RECORD, context);
    const idFailure = await objectFailure("identity-plane", CORRUPT_ID, context);
    const duplicateFailure = await objectFailure("identity-duplicate", DUPLICATE_ID, context);

    await startClean("sidecar.toml");
    const restarted = await sourceEvidence(
        TERRAIN,
        baseConfig,
        COLLECTION,
        "identity-restart",
        ORDER_A,
    );
    same(
        firstEvidence,
        identityEvidence(
            object(restarted, "probe"),
            object(restarted, "capture"),
        ),
        "identity restart",
    );
    const correctnessStatus = await event("workbench.status");
    const correctness = {
        processId: field<number>(correctnessStatus, "processId", "number"),
        renderer: capability(correctnessStatus, true),
    };
    await lifecycle("stop");

    const control = await unpreparedSweep(context);
    validateSweepIdentity(control, 32);
    const prepared = await preparedSweep(context);
    validateSweepIdentity(prepared, 32);
    finalReport = {
        revision: REVISION,
        outcome: "pass",
        environment,
        compatibility: { authoritativeObjects: `out/captures/${PRIOR}/acceptance.json` },
        storage: {
            orderA,
            orderARepeat,
            orderB,
            missing,
            corruptRecord,
            corruptId,
            duplicateId,
        },
        correctness: {
            ...correctness,
            schema1,
            schema1Transaction,
            first,
            second,
            revisit,
            heldIo,
            adjacentProbe,
            heldCopy,
            diagonalProbe,
            returned,
            returnedProbe,
            returnedCapture,
            alias,
            aliasProbe,
            aliasCapture,
            failures: { missingFailure, recordFailure, idFailure, duplicateFailure },
            restarted,
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
if (!finalReport) fail("canonical identity workflow did not produce a report");
await Deno.mkdir(`${root}/out/captures/${COLLECTION}`, { recursive: true });
await Deno.writeTextFile(`${root}/${REPORT}`, `${JSON.stringify(finalReport, null, 2)}\n`);
console.log(JSON.stringify(finalReport, null, 2));

async function switchAndProbe(path: string, config: ReturnType<typeof target>, label: string) {
    await event("objects.open", { path });
    const publication = await publishPair(config);
    const probeValue = await authorityProbe(config);
    const captureValue = await capture(label, COLLECTION, probeValue);
    return { publication, probe: probeValue, capture: captureValue };
}

async function heldTransition(
    gate: "objects.gate" | "async.gate",
    current: ReturnType<typeof target>,
    next: ReturnType<typeof target>,
) {
    await event(`${gate}.arm`);
    const scheduled = await event("composition.global.schedule", next);
    await event("workbench.resume");
    await sleep(50);
    const heldProbe = await authorityProbe(current);
    const pending = await event("composition.status");
    if (pending.pending === null) fail(`${gate} did not hold the composition transaction`);
    await event(`${gate}.release`);
    const published = await waitPair(field<number>(scheduled, "token", "number"));
    await event("workbench.pause");
    return { scheduled, heldProbe, pending, published };
}

function validateSweepIdentity(sweep: Record<string, unknown>, samples: number): void {
    if (sweep.sampleCount !== samples) fail("identity sweep sample count diverged");
    const readback = object(sweep, "payloadReadback");
    const identity = object(readback, "identity");
    if (identity.probeCount !== samples || identity.copyCount !== samples * 25) {
        fail("identity sweep readback accounting diverged");
    }
}
