import { type Coord, event, root } from "./global-terrain.ts";
import { array, captureEvidence, fail, field, object } from "./terrain.ts";

const decoder = new TextDecoder();
const TERRAIN_OBJECT_ID_BASE = 32_768;
const INDEX_OFFSET = 64;
const INDEX_ENTRY_BYTES = 64;
const HEIGHT_OFFSET = 16;

export async function cookSigned(
    path: string,
    centers: Coord[],
    variant = 0,
): Promise<Record<string, unknown>> {
    const args = ["run", "--locked", "--release", "-p", "terrain-cooker", "--", path];
    for (const [x, z] of centers) args.push("--global-center", String(x), String(z));
    args.push("--variant", String(variant));
    const output = await new Deno.Command("cargo", {
        args,
        cwd: root,
        stdout: "piped",
        stderr: "inherit",
    }).output();
    if (!output.success) fail("signed terrain cooker failed");
    return JSON.parse(decoder.decode(output.stdout).trim());
}

export function globalContent(probe: Record<string, unknown>): Record<string, unknown> {
    const content = object(probe, "globalContent");
    field<string>(content, "sourceNamespace", "string");
    field<string>(content, "contentSha256", "string");
    if (content.regionCount !== 25) fail("signed terrain content count mismatch");
    return content;
}

export function canonicalEvidence(probe: Record<string, unknown>): Record<string, unknown> {
    return {
        globalContent: globalContent(probe),
        cpuEdges: probe.cpuEdges,
        gpuEdges: probe.gpuEdges,
        geometry: probe.geometry,
        submission: probe.submission,
        resources: probe.resources,
    };
}

export function globalSlots(probe: Record<string, unknown>): Record<string, unknown>[] {
    return array(probe, "activeMapping").map((raw) => {
        if (!raw || typeof raw !== "object" || Array.isArray(raw)) {
            fail("signed terrain assignment is invalid");
        }
        const assignment = raw as Record<string, unknown>;
        return {
            globalRegion: object(assignment, "globalRegion"),
            slot: field<number>(assignment, "slot", "number"),
        };
    });
}

export async function captureJoined(
    id: string,
    collection: string,
    probe: Record<string, unknown>,
): Promise<Record<string, unknown>> {
    const assignments = new Map<number, Record<string, unknown>>();
    for (const raw of array(probe, "activeMapping")) {
        if (!raw || typeof raw !== "object" || Array.isArray(raw)) {
            fail("signed terrain assignment is invalid");
        }
        const assignment = raw as Record<string, unknown>;
        assignments.set(
            field<number>(assignment, "regionId", "number"),
            object(assignment, "globalRegion"),
        );
    }
    const raw = await event("perception.capture", {
        id,
        collection,
        samples: [{ x: 0, y: 0 }, { x: 640, y: 360 }],
    });
    const fullFrame = object(object(object(raw, "perception"), "evidence"), "fullFrame");
    const visibleGlobals = [];
    for (const rawObject of array(fullFrame, "objects")) {
        if (!rawObject || typeof rawObject !== "object" || Array.isArray(rawObject)) {
            fail("signed terrain semantic object is invalid");
        }
        const semantic = rawObject as Record<string, unknown>;
        if (semantic.kind !== "terrain-region") continue;
        const id = field<number>(semantic, "id", "number");
        const regionId = id - TERRAIN_OBJECT_ID_BASE - 1;
        const global = assignments.get(regionId);
        if (!global) fail(`terrain semantic ID ${id} has no signed assignment`);
        visibleGlobals.push(global);
    }
    if (visibleGlobals.length === 0) fail("signed terrain capture has no visible terrain semantic");
    const evidence = captureEvidence(raw);
    return {
        ...evidence,
        png: field<string>(object(raw, "image"), "pngSha256", "string"),
        terrainVisibleCount: visibleGlobals.length,
        visibleGlobals,
    };
}

export async function corruptSignedPayload(
    path: string,
    region: Coord,
): Promise<{ offset: number; original: number; corrupted: number }> {
    const bytes = await Deno.readFile(`${root}/${path}`);
    const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    const count = view.getUint32(16, true);
    for (let index = 0; index < count; index += 1) {
        const entry = INDEX_OFFSET + index * INDEX_ENTRY_BYTES;
        const x = Number(view.getBigInt64(entry, true));
        const z = Number(view.getBigInt64(entry + 8, true));
        if (x !== region[0] || z !== region[1]) continue;
        const payload = Number(view.getBigUint64(entry + 16, true));
        const offset = payload + HEIGHT_OFFSET;
        const original = bytes[offset];
        bytes[offset] ^= 1;
        await Deno.writeFile(`${root}/${path}`, bytes);
        return { offset, original, corrupted: bytes[offset] };
    }
    fail(`signed terrain payload ${region[0]},${region[1]} was not found`);
}

export async function restoreByte(
    path: string,
    evidence: { offset: number; original: number },
): Promise<void> {
    const bytes = await Deno.readFile(`${root}/${path}`);
    bytes[evidence.offset] = evidence.original;
    await Deno.writeFile(`${root}/${path}`, bytes);
}

export async function corruptSignedIndexOffset(source: string, output: string): Promise<void> {
    const bytes = await Deno.readFile(`${root}/${source}`);
    new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength).setBigUint64(
        INDEX_OFFSET + 16,
        0n,
        true,
    );
    await Deno.writeFile(`${root}/${output}`, bytes);
}

export async function waitFailure(): Promise<Record<string, unknown>> {
    const deadline = Date.now() + 10_000;
    while (Date.now() < deadline) {
        const status = await event("terrain.status");
        const stream = object(status, "stream");
        if (stream.pending === null && stream.lastFailure) return status;
        await new Promise((resolve) => setTimeout(resolve, 10));
    }
    fail("signed terrain failure timed out");
}
