type Fail = (message: string) => never;

export async function requireContactHistoryRemoved(root: string, fail: Fail): Promise<void> {
    console.log("==> removed dense contact history surface");
    const pattern = [
        "CanonicalTerrainContactProbe",
        "terrain_body_contact_probe",
        "BodyContactCoverage",
        "into_body_contact",
        "canonical\\.terrain\\.contact\\.probe",
        "230_400",
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
            "crates",
            ".runseal/wrappers/canonical-runtime.ts",
            ".runseal/wrappers/gpu-lab.ts",
            ".runseal/wrappers/init.ts",
            ".runseal/wrappers/workbench.ts",
            ".runseal/support/terrain",
        ],
        cwd: root,
        stdout: "piped",
        stderr: "inherit",
    }).output();
    if (output.code === 0) {
        fail(
            `guard: removed dense contact history symbol found\n${
                new TextDecoder().decode(output.stdout)
            }`,
        );
    }
    if (output.code !== 1) {
        fail(`guard: removed dense contact history scan failed with exit code ${output.code}`);
    }
}
