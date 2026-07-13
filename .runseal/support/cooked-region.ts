export function field<T>(owner: Record<string, unknown>, name: string, type: string): T {
    const value = owner[name];
    if (typeof value !== type) throw new Error(`cooked-region: expected ${name} to be ${type}`);
    return value as T;
}

export function object(
    owner: Record<string, unknown>,
    name: string,
): Record<string, unknown> {
    const value = owner[name];
    if (typeof value !== "object" || value === null || Array.isArray(value)) {
        throw new Error(`cooked-region: expected ${name} to be an object`);
    }
    return value as Record<string, unknown>;
}

export function array(owner: Record<string, unknown>, name: string): unknown[] {
    const value = owner[name];
    if (!Array.isArray(value)) throw new Error(`cooked-region: expected ${name} to be an array`);
    return value;
}

export function config(x: number, z: number): Record<string, number> {
    return {
        world_region_side: 128,
        active_center_x: x,
        active_center_z: z,
        active_radius: 2,
    };
}

export function distribution(values: number[]): Record<string, number> {
    if (values.some((value) => !Number.isFinite(value) || value <= 0)) {
        throw new Error("cooked-region: invalid GPU timing sample");
    }
    const sorted = [...values].sort((left, right) => left - right);
    const at = (fraction: number) => sorted[Math.ceil(fraction * sorted.length) - 1];
    return {
        minimum: sorted[0],
        median: at(0.5),
        p95: at(0.95),
        p99: at(0.99),
        maximum: sorted.at(-1)!,
    };
}

function canonicalEqual(actual: unknown, expected: unknown): boolean {
    return JSON.stringify(canonical(actual)) === JSON.stringify(canonical(expected));
}

export function assertSame(actual: unknown, expected: unknown, label: string): void {
    if (!canonicalEqual(actual, expected)) {
        throw new Error(`cooked-region: ${label} mismatch`);
    }
}

export async function corruptRegion(
    source: string,
    destination: string,
    regionId: number,
): Promise<void> {
    const bytes = await Deno.readFile(source);
    const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    const regionCount = view.getUint32(16, true);
    let payloadOffset: number | undefined;
    for (let index = 0; index < regionCount; index += 1) {
        const entry = 64 + index * 56;
        if (view.getUint32(entry, true) === regionId) {
            payloadOffset = Number(view.getBigUint64(entry + 8, true));
            break;
        }
    }
    if (payloadOffset === undefined) {
        throw new Error(`cooked-region: region ${regionId} not found to corrupt`);
    }
    bytes[payloadOffset + 7] ^= 1;
    await Deno.writeFile(destination, bytes);
}

export async function fileSha256(path: string): Promise<string> {
    const digest = await crypto.subtle.digest("SHA-256", await Deno.readFile(path));
    return [...new Uint8Array(digest)].map((byte) => byte.toString(16).padStart(2, "0")).join("");
}

function canonical(value: unknown): unknown {
    if (Array.isArray(value)) return value.map(canonical);
    if (typeof value !== "object" || value === null) return value;
    const owner = value as Record<string, unknown>;
    return Object.fromEntries(
        Object.keys(owner).sort().map((key) => [key, canonical(owner[key])]),
    );
}
export type Expected = {
    retained: number;
    uploaded: number;
    evicted: number;
    resident: number;
    protectedCount: number;
    chunks: number;
    bytes: number;
    uploadSha256: string;
};
