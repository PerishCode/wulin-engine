import { canonicalObjects } from "../canonical-object-composition.ts";
import { event, root } from "../global-terrain.ts";
import { fail, field, object } from "../terrain.ts";

const decoder = new TextDecoder();
const HEADER_BYTES = 96;
const INDEX_ENTRY_BYTES = 64;
const RECORD_BYTES = 20_480;
const IDENTITY_BYTES = 4_096;

export async function cookIdentity(
    path: string,
    centers: [number, number][],
    order: "a" | "b",
): Promise<Record<string, unknown>> {
    const args = [
        "run",
        "--locked",
        "--release",
        "-p",
        "region-cooker",
        "--",
        path,
        "--authority",
        "--identity-order",
        order,
    ];
    for (const [x, z] of centers) args.push("--global-center", String(x), String(z));
    const output = await new Deno.Command("cargo", {
        args,
        cwd: root,
        stdout: "piped",
        stderr: "inherit",
    }).output();
    if (!output.success) fail("canonical identity cooker failed");
    const report = JSON.parse(decoder.decode(output.stdout).trim());
    if (
        report.identityOrder !== order || object(report, "metadata").payloadSchema !== 2
    ) fail("canonical identity cooker emitted the wrong schema or order");
    return report;
}

export async function corruptIdentityPlane(
    path: string,
    region: [number, number],
    plane: "record" | "identity",
): Promise<Record<string, number>> {
    const { bytes, entry, payload } = await packRegion(path, region);
    const offset = payload + (plane === "record" ? 12 : RECORD_BYTES + 4);
    const original = bytes[offset];
    bytes[offset] ^= 1;
    await Deno.writeFile(`${root}/${path}`, bytes);
    return { entry, offset, original, corrupted: bytes[offset] };
}

export async function duplicateIdentity(
    path: string,
    region: [number, number],
): Promise<Record<string, number>> {
    const { bytes, view, entry, payload } = await packRegion(path, region);
    const identity = payload + RECORD_BYTES;
    const original = view.getUint32(identity + 4, true);
    const duplicate = view.getUint32(identity, true);
    view.setUint32(identity + 4, duplicate, true);
    const payloadBytes = view.getUint32(entry + 24, true);
    const digest = new Uint8Array(
        await crypto.subtle.digest("SHA-256", bytes.subarray(payload, payload + payloadBytes)),
    );
    bytes.set(digest, entry + 32);
    await Deno.writeFile(`${root}/${path}`, bytes);
    return { entry, offset: identity + 4, original, duplicate };
}

async function packRegion(path: string, region: [number, number]) {
    const bytes = await Deno.readFile(`${root}/${path}`);
    const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    const count = view.getUint32(16, true);
    if (view.getUint32(56, true) !== 2) fail("identity mutation requires payload schema 2");
    for (let index = 0; index < count; index += 1) {
        const entry = HEADER_BYTES + index * INDEX_ENTRY_BYTES;
        const x = Number(view.getBigInt64(entry, true));
        const z = Number(view.getBigInt64(entry + 8, true));
        if (x === region[0] && z === region[1]) {
            return { bytes, view, entry, payload: Number(view.getBigUint64(entry + 16, true)) };
        }
    }
    fail(`identity region ${region[0]},${region[1]} was not found`);
}

export function validateIdentityAuthority(probe: Record<string, unknown>) {
    const canonical = canonicalObjects(probe);
    const authority = object(canonical, "payloadAuthority");
    if (
        authority.revision !== "cooked-object-payload-authority-v2" ||
        authority.payloadSchema !== 2 || authority.regionCount !== 25 ||
        authority.recordCount !== 25_600 || authority.copyCount !== 50 ||
        authority.readbackBytes !== 614_400 || authority.chunkMismatchCount !== 0 ||
        authority.expectedIndexSha256 !== authority.observedIndexSha256 ||
        canonical.localIdCount !== 25_600 || canonical.localIdDuplicateCount !== 0
    ) fail("canonical identity payload authority failed");
    return authority;
}

export function identityEvidence(
    probe: Record<string, unknown>,
    capture: Record<string, unknown>,
): Record<string, unknown> {
    validateIdentityAuthority(probe);
    return identityBehaviorEvidence(probe, capture);
}

export function identityBehaviorEvidence(
    probe: Record<string, unknown>,
    capture: Record<string, unknown>,
): Record<string, unknown> {
    const canonical = canonicalObjects(probe);
    const grounding = object(probe, "grounding");
    const contact = object(probe, "contact");
    const skeletal = object(probe, "skeletal");
    return {
        identityKeyedSha256: canonical.identityKeyedSha256,
        stableKeySha256: canonical.stableKeySha256,
        stableSeedSha256: canonical.stableSeedSha256,
        positionSha256: grounding.identityKeyedPositionSha256,
        groundSha256: grounding.identityKeyedGroundSha256,
        boundaries: grounding.boundaries,
        selectedSurfaceSha256: contact.identityKeyedSelectedSurfaceSha256,
        residualSha256: contact.identityKeyedResidualSha256,
        contactCounts: {
            negative: contact.negativeCount,
            zero: contact.zeroCount,
            positive: contact.positiveCount,
            lod: contact.ownerPatchLodCounts,
        },
        skeletal: {
            settings: skeletal.settings,
            gpu: skeletal.gpu,
            cpuOracle: skeletal.cpuOracle,
        },
        capture,
    };
}

export async function validateIdentityTransaction(
    chunks: number,
): Promise<Record<string, unknown>> {
    const objects = await event("objects.status");
    const completed = object(objects, "lastCompleted");
    const io = object(completed, "io");
    const gpu = object(completed, "gpu");
    if (
        io.chunkCount !== chunks || io.payloadBytes !== chunks * (RECORD_BYTES + IDENTITY_BYTES) ||
        io.recordBytes !== chunks * RECORD_BYTES || io.identityBytes !== chunks * IDENTITY_BYTES ||
        gpu.identityCopyCount !== chunks || gpu.identityCopyBytes !== chunks * IDENTITY_BYTES
    ) fail("canonical identity transaction accounting diverged");
    return completed;
}

export async function identityReadback(probeCount: number): Promise<Record<string, unknown>> {
    const readback = object(await event("async.status"), "payloadReadback");
    const identity = object(readback, "identity");
    if (
        identity.resourceCount !== 1 || identity.capacityPages !== 25 ||
        identity.capacityBytes !== 102_400 || identity.allocationBytes !== 131_072 ||
        identity.probeCount !== probeCount || identity.copyCount !== probeCount * 25
    ) fail("canonical identity readback accounting diverged");
    return identity;
}

export function sourceNamespace(report: Record<string, unknown>): string {
    return field<string>(object(report, "metadata"), "sourceNamespaceSha256", "string");
}
