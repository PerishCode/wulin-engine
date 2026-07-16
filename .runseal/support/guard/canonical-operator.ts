type Fail = (message: string) => never;

export async function requireCanonicalOperatorIdentity(
    root: string,
    fail: Fail,
): Promise<void> {
    console.log("==> neutral canonical operator identity");
    const wrapper = await Deno.readTextFile(`${root}/.runseal/wrappers/canonical-runtime.ts`);
    requireExactConstant(wrapper, "REVISION", "canonical-runtime-v6", fail);
    requireExactConstant(wrapper, "COLLECTION", "canonical-runtime", fail);
    if (wrapper.includes("0060-mandatory-simulation-control-cleanup")) {
        fail("guard: canonical wrapper retains the historical Experiment 0060 collection");
    }

    const agents = await Deno.readTextFile(`${root}/AGENTS.md`);
    if (!agents.includes("`out/captures/canonical-runtime/` and remains ignored.")) {
        fail("guard: canonical evidence documentation diverged from the neutral collection");
    }
}

function requireExactConstant(
    source: string,
    name: string,
    expected: string,
    fail: Fail,
): void {
    const pattern = new RegExp(`^const ${name} = \"([^\"]+)\";$`, "gm");
    const values = [...source.matchAll(pattern)].map((match) => match[1]);
    if (values.length !== 1 || values[0] !== expected) {
        fail(`guard: canonical ${name.toLowerCase()} diverged: ${JSON.stringify(values)}`);
    }
}
