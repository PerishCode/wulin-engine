export function fail(message: string): never {
    throw new Error(`skeletal-crowds: ${message}`);
}

export type Settings = {
    animated_percent: number;
    bone_count: number;
    phase_count: number;
    time_tick: number;
    unique_poses: boolean;
    forced_lod: number | null;
};

export function settings(overrides: Partial<Settings> = {}): Settings {
    return {
        animated_percent: 100,
        bone_count: 64,
        phase_count: 64,
        time_tick: 0,
        unique_poses: false,
        forced_lod: null,
        ...overrides,
    };
}

export function loadConfig(x = 64, z = 64): Record<string, number> {
    return {
        world_region_side: 128,
        active_center_x: x,
        active_center_z: z,
        active_radius: 2,
    };
}

async function command(root: string, tool: string, args: string[]): Promise<string> {
    const output = await new Deno.Command(tool, {
        args,
        cwd: root,
        stdout: "piped",
        stderr: "piped",
    }).output();
    if (!output.success) fail(`${tool} environment query failed`);
    return new TextDecoder().decode(output.stdout).trim();
}

export async function collectEnvironment(root: string): Promise<Record<string, unknown>> {
    const platformScript = [
        "$os=Get-CimInstance Win32_OperatingSystem",
        "$gpu=Get-CimInstance Win32_VideoController | Where-Object Name -Like '*NVIDIA*' | Select-Object -First 1",
        "$dxc=(Get-Item 'C:\\Program Files (x86)\\Windows Kits\\10\\bin\\10.0.26100.0\\x64\\dxc.exe').VersionInfo",
        "[pscustomobject]@{windowsBuild=$os.BuildNumber;driver=$gpu.DriverVersion;dxcProductVersion=$dxc.ProductVersion}|ConvertTo-Json -Compress",
    ].join(";");
    const [revision, dirty, cargo, platform] = await Promise.all([
        command(root, "git", ["rev-parse", "HEAD"]),
        command(root, "git", ["status", "--porcelain"]),
        command(root, "cargo", ["-vV"]),
        command(root, "pwsh", ["-NoProfile", "-Command", platformScript]),
    ]);
    return {
        revision,
        dirty: dirty.length > 0,
        rustToolchain: cargo,
        agilitySdk: "1.619.4",
        dxcPath: "C:\\Program Files (x86)\\Windows Kits\\10\\bin\\10.0.26100.0\\x64\\dxc.exe",
        ...JSON.parse(platform),
    };
}

export function field<T>(
    owner: Record<string, unknown>,
    name: string,
    type: string,
): T {
    const value = owner[name];
    if (typeof value !== type) fail(`expected ${name} to be ${type}`);
    return value as T;
}

export function object(
    owner: Record<string, unknown>,
    name: string,
): Record<string, unknown> {
    const value = owner[name];
    if (typeof value !== "object" || value === null || Array.isArray(value)) {
        fail(`expected ${name} to be an object`);
    }
    return value as Record<string, unknown>;
}

export function array(owner: Record<string, unknown>, name: string): unknown[] {
    const value = owner[name];
    if (!Array.isArray(value)) fail(`expected ${name} to be an array`);
    return value;
}

function canonical(value: unknown): unknown {
    if (Array.isArray(value)) return value.map(canonical);
    if (typeof value !== "object" || value === null) return value;
    const owner = value as Record<string, unknown>;
    return Object.fromEntries(
        Object.keys(owner).sort().map((key) => [key, canonical(owner[key])]),
    );
}

export function same(actual: unknown, expected: unknown, label: string): void {
    if (JSON.stringify(canonical(actual)) !== JSON.stringify(canonical(expected))) {
        fail(
            `${label} mismatch: actual=${JSON.stringify(actual)} expected=${
                JSON.stringify(expected)
            }`,
        );
    }
}

export function distribution(
    values: number[],
    label = "GPU timing",
    allowZero = false,
): Record<string, number> {
    if (
        values.some((value) => !Number.isFinite(value) || value < 0 || (!allowZero && value === 0))
    ) {
        fail(`invalid ${label} sample: ${JSON.stringify(values)}`);
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

export const REVISION = "gpu-skeletal-crowds-v1";
export const MESH_SHA256 = "9553748209f9de17e9b524b1c21080404f32df57be62959714b58db1121f0a4e";
export const ANIMATION_SHA256 = "cc075037175990f29083ad1fc63823c1a77002d7aeccfbc429eee4f54de22a6e";

export function validateProbe(probe: Record<string, unknown>): void {
    if (
        field<string>(probe, "revision", "string") !== REVISION ||
        field<string>(probe, "meshletCatalogSha256", "string") !== MESH_SHA256 ||
        field<string>(probe, "animationCatalogSha256", "string") !== ANIMATION_SHA256
    ) fail("probe revision or catalog hash mismatch");
    for (
        const name of [
            "resetDispatchCount",
            "cullDispatchCount",
            "poseCompactDispatchCount",
            "indirectPoseDispatchCount",
            "indirectMeshDispatchCount",
        ]
    ) {
        if (field<number>(probe, name, "number") !== 1) fail(`${name} is not fixed at one`);
    }
    const gpu = object(probe, "gpu");
    same(gpu, object(probe, "cpuOracle"), "GPU/CPU oracle");
    const visible = field<number>(gpu, "visible", "number");
    const animated = field<number>(gpu, "animated", "number");
    const active = field<number>(gpu, "activePoses", "number");
    const probeSettings = object(probe, "settings");
    const boneCount = field<number>(probeSettings, "boneCount", "number");
    if (
        visible + field<number>(gpu, "rejected", "number") !==
            field<number>(probe, "candidateInstanceCount", "number") ||
        animated + field<number>(gpu, "staticCount", "number") !== visible ||
        (array(gpu, "lodCounts") as number[]).reduce((sum, value) => sum + value, 0) !== visible ||
        field<number>(gpu, "evaluatedBones", "number") !== active * boneCount ||
        field<number>(probe, "poseDispatchGroups", "number") !== active ||
        field<number>(probe, "paletteWriteBytes", "number") !== active * boneCount * 48 ||
        active > 25_600
    ) fail("probe aggregate or capacity invariant failed");
    const unique = field<boolean>(probeSettings, "uniquePoses", "boolean");
    const phases = field<number>(probeSettings, "phaseCount", "number");
    if ((unique && active !== animated) || (!unique && active > 8 * phases)) {
        fail("pose diversity invariant failed");
    }
    if (!unique && animated - active >= 2 && field<number>(gpu, "reusedPoses", "number") <= 0) {
        fail("shared pose reuse was not reported");
    }
    const sample = probe.paletteSample;
    if (active === 0 && sample !== null) fail("zero-pose workload returned a palette sample");
    if (active > 0) {
        if (typeof sample !== "object" || sample === null || Array.isArray(sample)) {
            fail("active workload omitted its palette sample");
        }
        const palette = sample as Record<string, unknown>;
        if (
            field<number>(palette, "maximumAbsoluteDelta", "number") >
                field<number>(palette, "tolerance", "number")
        ) fail("palette sample exceeded tolerance");
    }
}
