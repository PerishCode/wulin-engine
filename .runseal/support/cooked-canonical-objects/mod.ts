import { capture, probe, stableFrame } from "../canonical-object-composition.ts";
import { type GlobalConfig, halfReports, prepare } from "../global-composition.ts";
import { event, root } from "../global-terrain.ts";
import { fail, field, object } from "../terrain.ts";

const decoder = new TextDecoder();
const HEADER_BYTES = 96;
const INDEX_ENTRY_BYTES = 64;

export async function cookObjects(
    path: string,
    centers: [number, number][],
): Promise<Record<string, unknown>> {
    const args = ["run", "--locked", "--release", "-p", "region-cooker", "--", path];
    for (const [x, z] of centers) args.push("--global-center", String(x), String(z));
    const output = await new Deno.Command("cargo", {
        args,
        cwd: root,
        stdout: "piped",
        stderr: "inherit",
    }).output();
    if (!output.success) fail("signed object cooker failed");
    return JSON.parse(decoder.decode(output.stdout).trim());
}

export async function corruptObjectPayload(
    path: string,
    region: [number, number],
): Promise<{ offset: number; original: number; corrupted: number }> {
    const bytes = await Deno.readFile(`${root}/${path}`);
    const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    const count = view.getUint32(16, true);
    for (let index = 0; index < count; index += 1) {
        const entry = HEADER_BYTES + index * INDEX_ENTRY_BYTES;
        const x = Number(view.getBigInt64(entry, true));
        const z = Number(view.getBigInt64(entry + 8, true));
        if (x !== region[0] || z !== region[1]) continue;
        const payload = Number(view.getBigUint64(entry + 16, true));
        const offset = payload + 12;
        const original = bytes[offset];
        bytes[offset] ^= 1;
        await Deno.writeFile(`${root}/${path}`, bytes);
        return { offset, original, corrupted: bytes[offset] };
    }
    fail(`signed object payload ${region[0]},${region[1]} was not found`);
}

export async function objectStatus(): Promise<Record<string, unknown>> {
    return await event("objects.status");
}

export function validateObjectTransaction(
    status: Record<string, unknown>,
    transactionId: number,
    chunks: number,
): Record<string, unknown> {
    const completed = object(status, "lastCompleted");
    if (
        completed.transactionId !== transactionId ||
        completed.revision !== "cooked-canonical-object-v1"
    ) fail("cooked object completion identity diverged");
    const io = object(completed, "io");
    if (
        io.chunkCount !== chunks || io.seekCount !== chunks ||
        io.payloadBytes !== chunks * 20_480
    ) fail("cooked object I/O bounds diverged");
    const gpu = object(completed, "gpu");
    if (
        gpu.payloadSource !== "cooked-pack" || gpu.generationMs !== 0 ||
        gpu.uploadedRegionCount !== chunks || gpu.instanceBytes !== chunks * 20_480
    ) fail("cooked object GPU transaction diverged");
    return completed;
}

export async function baseEvidence(
    terrainPack: string,
    config: GlobalConfig,
    collection: string,
    objectPack?: string,
): Promise<Record<string, unknown>> {
    const publication = await prepare(terrainPack, config, objectPack);
    const probeValue = await probe(config);
    const captureValue = await capture(
        objectPack ? "cooked-base" : "generated-base",
        collection,
        probeValue,
    );
    const halves = await halfReports(object(publication, "published"), config);
    const instance = object(halves, "instance");
    const objects = objectPack ? await objectStatus() : null;
    if (objectPack) {
        validateObjectTransaction(
            objects!,
            field<number>(instance, "transactionId", "number"),
            25,
        );
    }
    const frame = stableFrame(probeValue, captureValue);
    const canonical = object(frame, "canonicalObjects");
    delete canonical.sourceNamespace;
    delete canonical.contentSha256;
    return { publication, probe: probeValue, capture: captureValue, halves, objects, frame };
}
