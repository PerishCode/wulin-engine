import { capability, packCenters } from "../support/canonical-origin-rollover-evidence.ts";
import { startClean, target } from "../support/canonical-origin-rollover.ts";
import {
    asyncStatus,
    authorityProbe,
    cookAuthority,
    expectReadback,
    expectSweepReadback,
    objectFailure,
    payloadAuthority,
    requireNoPayloadAuthority,
    sourceEvidence,
} from "../support/authoritative-cooked-objects/mod.ts";
import {
    canonicalObjects,
    capture,
    expectHalfCounts,
    expectPairMovement,
    probe,
    stableFrame,
} from "../support/canonical-object-composition.ts";
import {
    preparedDirection,
    preparedRollover,
    promotedPrefetch,
} from "../support/canonical-traversal-prefetch-scenarios.ts";
import {
    preparedSweep,
    unpreparedSweep,
} from "../support/canonical-traversal-prefetch-evidence.ts";
import type { PrefetchContext } from "../support/canonical-traversal-prefetch.ts";
import {
    cookObjects,
    corruptObjectPayload,
    objectStatus,
    validateObjectTransaction,
} from "../support/cooked-canonical-objects/mod.ts";
import { prepare, publishPair } from "../support/global-composition.ts";
import {
    assertStopped,
    event,
    lifecycle,
    root,
    run,
    useSidecar,
} from "../support/global-terrain.ts";
import { cookSigned } from "../support/signed-terrain-storage.ts";
import { collectEnvironment, fail, field, object, same } from "../support/terrain.ts";

const REVISION = "authoritative-cooked-object-v1";
const COLLECTION = "0029-authoritative-cooked-objects";
const TERRAIN = `out/terrain/${COLLECTION}/terrain-a.wlt`;
const COMPATIBILITY = `out/cooked/${COLLECTION}/compatibility-a.wlr`;
const AUTHORITY = `out/cooked/${COLLECTION}/authority-a.wlr`;
const AUTHORITY_REPEAT = `out/cooked/${COLLECTION}/authority-a-repeat.wlr`;
const AUTHORITY_MISSING = `out/cooked/${COLLECTION}/authority-missing.wlr`;
const AUTHORITY_CORRUPT = `out/cooked/${COLLECTION}/authority-corrupt.wlr`;
const REPORT = `out/captures/${COLLECTION}/acceptance.json`;
const FAR = 2 ** 40;
const BASE: [number, number] = [FAR, -FAR];

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :authoritative-cooked-objects");
    Deno.exit(0);
}
if (Deno.args.length > 0) fail(`unexpected argument ${Deno.args[0]}`);

useSidecar("sidecar.toml");
await lifecycle("stop");
useSidecar("sidecar.benchmark.toml");
await lifecycle("stop");
useSidecar("sidecar.toml");
await run(
    "runseal",
    [":cooked-canonical-objects"],
    "Experiment 0028 compatibility workflow",
);
await run(
    "cargo",
    ["test", "--locked", "-p", "region-format", "-p", "canonical-object-fixture"],
    "signed object authority codec tests",
);

const centers = [
    ...packCenters([BASE]),
    [BASE[0] + 1, BASE[1] + 1] as [number, number],
    [BASE[0] - 1, BASE[1] - 1] as [number, number],
];
await Deno.mkdir(`${root}/out/terrain/${COLLECTION}`, { recursive: true });
await Deno.mkdir(`${root}/out/cooked/${COLLECTION}`, { recursive: true });
const terrain = await cookSigned(TERRAIN, centers);
const compatibility = await cookObjects(COMPATIBILITY, centers);
const authority = await cookAuthority(AUTHORITY, centers);
const authorityRepeat = await cookAuthority(AUTHORITY_REPEAT, centers);
const authorityMissing = await cookAuthority(AUTHORITY_MISSING, [BASE]);
const compatibilityMetadata = object(compatibility, "metadata");
const authorityMetadata = object(authority, "metadata");
if (
    authority.fileSha256 !== authorityRepeat.fileSha256 ||
    authorityMetadata.sourceNamespaceSha256 !==
        object(authorityRepeat, "metadata").sourceNamespaceSha256
) fail("authority cooker is not deterministic");
if (
    compatibilityMetadata.stableSeedNamespaceSha256 !==
        authorityMetadata.stableSeedNamespaceSha256 ||
    compatibilityMetadata.sourceNamespaceSha256 === authorityMetadata.sourceNamespaceSha256
) fail("authority source identity does not preserve only stable seeds");
await Deno.copyFile(`${root}/${AUTHORITY}`, `${root}/${AUTHORITY_CORRUPT}`);
const corruption = await corruptObjectPayload(
    AUTHORITY_CORRUPT,
    [BASE[0] + 73, BASE[1]],
);

const environment = await collectEnvironment(root);
const baseConfig = target(BASE);
const context: PrefetchContext = {
    collection: COLLECTION,
    pack: TERRAIN,
    missingPack: "out/terrain/0023-signed-terrain-storage/terrain-a.wlt",
    corruptPack: "out/terrain/0027-canonical-traversal-prefetch/terrain-corrupt.wlt",
    objectPack: AUTHORITY,
    base: BASE,
};
let finalReport: Record<string, unknown> | undefined;

try {
    await startClean("sidecar.toml");
    const generatedBase = await sourceEvidence(
        TERRAIN,
        baseConfig,
        COLLECTION,
        "generated-base",
    );
    await lifecycle("stop");

    await startClean("sidecar.toml");
    const compatibilityBase = await sourceEvidence(
        TERRAIN,
        baseConfig,
        COLLECTION,
        "compatibility-base",
        COMPATIBILITY,
    );
    const compatibilityProbe = object(compatibilityBase, "probe");
    const compatibilityObjects = canonicalObjects(compatibilityProbe);
    const compatibilityPayload = payloadAuthority(compatibilityProbe);
    await event("objects.open", { path: AUTHORITY });
    const sourceSwitch = await publishPair(baseConfig);
    expectHalfCounts(sourceSwitch, "terrain", {
        retainedRegionCount: 25,
        uploadedRegionCount: 0,
        payloadBytes: 0,
    });
    expectHalfCounts(sourceSwitch, "instance", {
        retainedRegionCount: 0,
        uploadedRegionCount: 25,
        instanceBytes: 25 * 20_480,
    });
    expectReadback(await asyncStatus(), 1);
    const authoritySwitchProbe = await authorityProbe(baseConfig);
    expectReadback(await asyncStatus(), 2);
    const authoritySwitchCapture = await capture(
        "authority-switch",
        COLLECTION,
        authoritySwitchProbe,
    );
    expectReadback(await asyncStatus(), 2);
    const authorityObjects = canonicalObjects(authoritySwitchProbe);
    const authorityPayload = payloadAuthority(authoritySwitchProbe);
    if (
        compatibilityObjects.stableSeedSha256 !== authorityObjects.stableSeedSha256 ||
        compatibilityObjects.sourceNamespace === authorityObjects.sourceNamespace ||
        compatibilityObjects.contentSha256 === authorityObjects.contentSha256 ||
        compatibilityPayload.payloadSha256 === authorityPayload.payloadSha256 ||
        object(compatibilityProbe, "grounding").positionSha256 ===
            object(authoritySwitchProbe, "grounding").positionSha256 ||
        field<string>(object(compatibilityBase, "capture"), "png", "string") ===
            field<string>(authoritySwitchCapture, "png", "string")
    ) fail("authority switch did not isolate authored content from stable identity");
    validateObjectTransaction(
        await objectStatus(),
        field<number>(
            object(object(sourceSwitch, "halves"), "instance"),
            "transactionId",
            "number",
        ),
        25,
    );

    const adjacentConfig = target([BASE[0] + 1, BASE[1]], 65);
    const adjacent = await publishPair(adjacentConfig);
    expectPairMovement(adjacent, 20, 5, 5 * 4_096, 5 * 20_480);
    const adjacentProbe = await authorityProbe(adjacentConfig);
    const revisit = await publishPair(baseConfig);
    expectPairMovement(revisit, 25, 0, 0, 0);
    const revisitProbe = await authorityProbe(baseConfig);
    const aliasConfig = target(BASE, 63);
    const alias = await publishPair(aliasConfig);
    expectPairMovement(alias, 25, 0, 0, 0);
    const aliasProbe = await authorityProbe(aliasConfig);
    await lifecycle("stop");

    await startClean("sidecar.toml");
    await prepare(TERRAIN, baseConfig, AUTHORITY);
    const diagonalConfig = target([BASE[0] + 1, BASE[1] + 1], 65, 65);
    const diagonal = await publishPair(diagonalConfig);
    expectPairMovement(diagonal, 16, 9, 9 * 4_096, 9 * 20_480);
    const diagonalProbe = await authorityProbe(diagonalConfig);
    await lifecycle("stop");

    const prepared = await preparedDirection("authority-positive-x", [1, 0], context, true);
    const preparedProbe = await authorityProbe(adjacentConfig);
    await lifecycle("stop");
    const objectIo = await promotedPrefetch("object-io", context);
    const objectIoProbe = await authorityProbe(adjacentConfig);
    await lifecycle("stop");
    const objectCopy = await promotedPrefetch("object-copy", context);
    const objectCopyProbe = await authorityProbe(adjacentConfig);
    await lifecycle("stop");
    const rollover = await preparedRollover(context);
    const rolloverProbe = await authorityProbe(target([BASE[0] + 1, BASE[1]]));
    await lifecycle("stop");
    const missing = await objectFailure("missing", AUTHORITY_MISSING, context);
    await lifecycle("stop");
    const corrupt = await objectFailure("corrupt", AUTHORITY_CORRUPT, context);
    await lifecycle("stop");

    await startClean("sidecar.toml");
    await prepare(TERRAIN, baseConfig, AUTHORITY);
    const beforeDisable = await authorityProbe(baseConfig);
    const disabled = await event("objects.disable");
    const generatedAfterDisable = await publishPair(baseConfig);
    expectHalfCounts(generatedAfterDisable, "terrain", {
        retainedRegionCount: 25,
        uploadedRegionCount: 0,
    });
    expectHalfCounts(generatedAfterDisable, "instance", {
        retainedRegionCount: 0,
        uploadedRegionCount: 25,
    });
    const disabledProbe = await probe(baseConfig);
    requireNoPayloadAuthority(disabledProbe);
    if (disabled.disabled !== true) fail("authority disable did not restore generated sourcing");
    await lifecycle("stop");

    await startClean("sidecar.toml");
    const restart = await sourceEvidence(
        TERRAIN,
        baseConfig,
        COLLECTION,
        "authority-restart",
        AUTHORITY,
    );
    same(
        stableFrame(authoritySwitchProbe, authoritySwitchCapture),
        object(restart, "frame"),
        "authority restart frame",
    );
    const correctnessStatus = await event("workbench.status");
    const correctnessRenderer = capability(correctnessStatus, true);
    const correctnessProcess = field<number>(correctnessStatus, "processId", "number");
    await lifecycle("stop");

    const control = await unpreparedSweep(context);
    const controlReadback = expectSweepReadback(control, 32);
    const preparedRelease = await preparedSweep(context);
    const preparedReadback = expectSweepReadback(preparedRelease, 32);
    finalReport = {
        revision: REVISION,
        outcome: "pass",
        environment,
        compatibility: {
            cookedCanonicalObjects: "out/captures/0028-cooked-canonical-objects/acceptance.json",
        },
        storage: {
            terrain,
            compatibility,
            authority,
            authorityRepeat,
            authorityMissing,
            corruption,
        },
        correctness: {
            processId: correctnessProcess,
            renderer: correctnessRenderer,
            generatedBase,
            compatibilityBase,
            sourceSwitch,
            authoritySwitchProbe,
            authoritySwitchCapture,
            adjacent,
            adjacentProbe,
            revisit,
            revisitProbe,
            alias,
            aliasProbe,
            diagonal,
            diagonalProbe,
            prepared,
            preparedProbe,
            promotions: { objectIo, objectIoProbe, objectCopy, objectCopyProbe },
            rollover,
            rolloverProbe,
            missing,
            corrupt,
            beforeDisable,
            disabled,
            generatedAfterDisable,
            disabledProbe,
            restart,
        },
        benchmark: {
            control: { ...control, payloadReadback: controlReadback },
            prepared: { ...preparedRelease, payloadReadback: preparedReadback },
        },
    };
} finally {
    useSidecar("sidecar.toml");
    await lifecycle("stop");
    useSidecar("sidecar.benchmark.toml");
    await lifecycle("stop");
}

await assertStopped("sidecar.toml");
await assertStopped("sidecar.benchmark.toml");
if (!finalReport) fail("authority workflow did not produce a report");
await Deno.mkdir(`${root}/out/captures/${COLLECTION}`, { recursive: true });
await Deno.writeTextFile(`${root}/${REPORT}`, `${JSON.stringify(finalReport, null, 2)}\n`);
console.log(JSON.stringify(finalReport, null, 2));
