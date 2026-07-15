type Fail = (message: string) => never;

export async function requireTerrainHistoryRemoved(
    root: string,
    fail: Fail,
): Promise<void> {
    console.log("==> removed caller-owned terrain transaction surface");
    for (
        const path of [
            ".runseal/support/terrain/motion.ts",
            ".runseal/support/terrain/translation.ts",
            ".runseal/support/terrain/advance.ts",
        ]
    ) {
        try {
            await Deno.stat(`${root}/${path}`);
            fail(`guard: removed terrain transaction support remains: ${path}`);
        } catch (error) {
            if (!(error instanceof Deno.errors.NotFound)) throw error;
        }
    }

    const pattern = [
        "CanonicalTerrainBody(Step|Translate|Advance)",
        "pub fn (step|translate|advance)_terrain_body",
        "canonical\\.terrain\\.body\\.(step|translate|advance)",
        "(^|[^a-z_])terrain_(motion|translation|advance)_failed",
        "terrain(Motion|Translation|Advance)Gates",
    ].join("|");
    const output = await new Deno.Command("git", {
        args: [
            "grep",
            "--no-index",
            "-n",
            "-E",
            pattern,
            "--",
            "apps",
            "crates/engine-runtime/src/runtime",
            ".runseal/wrappers",
            ".runseal/support/terrain",
        ],
        cwd: root,
        stdout: "piped",
        stderr: "inherit",
    }).output();
    if (output.code === 0) {
        fail(
            `guard: removed caller-owned terrain transaction symbol found\n${
                new TextDecoder().decode(output.stdout)
            }`,
        );
    }
    if (output.code !== 1) {
        fail(`guard: removed terrain transaction scan failed with exit code ${output.code}`);
    }
}
