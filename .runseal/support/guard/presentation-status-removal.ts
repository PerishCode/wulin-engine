type Fail = (message: string) => never;

export async function requirePresentationStatusRemoved(
    root: string,
    fail: Fail,
): Promise<void> {
    console.log("==> removed standalone presentation status surface");
    const pattern = [
        "CanonicalTimeStatus",
        "presentation_time_status",
        "canonical\\.time\\.status",
        "retiredStatusGate",
        "retiredStatus",
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
            "crates/engine-runtime/src",
            ".runseal/wrappers",
            ".runseal/support/actor",
            ".runseal/support/temporal-presentation.ts",
            ".runseal/support/canonical-runtime.ts",
            ".runseal/support/runtime-bootstrap.ts",
            ".runseal/support/cooked-gltf-presentation.ts",
        ],
        cwd: root,
        stdout: "piped",
        stderr: "inherit",
    }).output();
    if (output.code === 0) {
        fail(
            `guard: removed standalone presentation status symbol found\n${
                new TextDecoder().decode(output.stdout)
            }`,
        );
    }
    if (output.code !== 1) {
        fail(`guard: presentation status removal scan failed with exit code ${output.code}`);
    }
}
