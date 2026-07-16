type Fail = (message: string) => never;

export async function requireStandaloneContactRemoved(root: string, fail: Fail): Promise<void> {
    console.log("==> removed standalone terrain-contact surface");
    const pattern = [
        "CanonicalTerrainContactProbe",
        "CanonicalTerrainContact",
        "resolve_terrain_contact",
        "terrain_body_contact_probe",
        "terrainContactGates",
        "unavailableTerrainContactGate",
        "BodyContactCoverage",
        "into_body_contact",
        "canonical\\.terrain\\.contact",
        "exact-terrain-body-contact-v1",
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
            `guard: removed standalone terrain-contact symbol found\n${
                new TextDecoder().decode(output.stdout)
            }`,
        );
    }
    if (output.code !== 1) {
        fail(`guard: removed terrain-contact scan failed with exit code ${output.code}`);
    }

    const contract = await Deno.readTextFile(
        `${root}/crates/engine-runtime/src/terrain_query/mod.rs`,
    );
    const probe = await Deno.readTextFile(
        `${root}/crates/engine-runtime/src/rendering/composition/probe/terrain_query.rs`,
    );
    const acceptance = await Deno.readTextFile(
        `${root}/.runseal/support/compatibility-removal.ts`,
    );
    if (
        !contract.includes("pub(crate) fn resolve_body_contact(") ||
        !probe.includes("body_contact_count == 225") ||
        !acceptance.includes('"canonical.terrain.contact"') ||
        acceptance.includes('"canonical.terrain.contact.probe"')
    ) {
        fail("guard: private contact authority or sole current removal witness diverged");
    }
}
