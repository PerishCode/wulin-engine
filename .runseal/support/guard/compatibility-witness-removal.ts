type Fail = (message: string) => never;

export async function requireCompatibilityWitnessRemoved(
    root: string,
    fail: Fail,
): Promise<void> {
    console.log("==> removed recurring compatibility witness");
    try {
        await Deno.stat(`${root}/.runseal/support/compatibility-removal.ts`);
        fail("guard: recurring compatibility support returned");
    } catch (error) {
        if (!(error instanceof Deno.errors.NotFound)) throw error;
    }

    const wrapper = await Deno.readTextFile(`${root}/.runseal/wrappers/canonical-runtime.ts`);
    const idleShell = await Deno.readTextFile(`${root}/.runseal/support/idle-shell.ts`);
    const current = `${wrapper}\n${idleShell}`;
    for (
        const retired of [
            "compatibilityRemoval",
            "compatibilityRemovalGates",
            "removedVerbs",
            "requireUnknownEvent",
        ]
    ) {
        if (current.includes(retired)) {
            fail(`guard: recurring compatibility witness returned: ${retired}`);
        }
    }
    if (
        !wrapper.includes('import { idleShellGates } from "../support/idle-shell.ts"') ||
        !wrapper.includes("const idleShell = await idleShellGates(COLLECTION, idle)") ||
        !wrapper.includes("idleShell,") ||
        !idleShell.includes('object(idleStatus, "workload").mode !== "idle-shell"') ||
        !idleShell.includes('id: "idle-shell"') ||
        !idleShell.includes('number(image, "differentPixelCount") !== 0') ||
        !idleShell.includes('array(fullFrame, "objects").length !== 0')
    ) fail("guard: current idle-shell authority diverged");

    const bootstrapTests = await Deno.readTextFile(
        `${root}/crates/reference-host/tests/private/bootstrap.rs`,
    );
    if (
        bootstrapTests.includes("document_rejects_unknown_old_schema_path_and_projection") ||
        bootstrapTests.includes('\\"fallback\\": true') ||
        bootstrapTests.includes('\\"schemaVersion\\": 1')
    ) fail("guard: retired bootstrap compatibility tests returned");
    for (
        const path of [
            ".runseal/support/runtime-bootstrap.ts",
            ".runseal/support/prototype/host.ts",
        ]
    ) {
        const source = await Deno.readTextFile(`${root}/${path}`);
        if (/\.fallback\s*=/.test(source)) {
            fail(`guard: retired bootstrap fallback probe returned: ${path}`);
        }
    }
}
