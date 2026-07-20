type Fail = (message: string) => never;

export async function requireNativeActionTransport(
    root: string,
    fail: Fail,
): Promise<void> {
    console.log("==> in-process Prototype native-action transport");
    const input = await Deno.readTextFile(
        `${root}/.runseal/support/prototype/input/mod.ts`,
    );
    const prepared = await Deno.readTextFile(
        `${root}/.runseal/support/prototype/input/prepared.ts`,
    );
    const wrapper = await Deno.readTextFile(
        `${root}/.runseal/wrappers/canonical-prototype.ts`,
    );
    const profile = await Deno.readTextFile(`${root}/runseal.toml`);
    const transport = `${input}\n${prepared}`;
    if (/Deno\.Command|Add-Type|prototype-native-helper-ready|\.spawn\(/.test(transport)) {
        fail("guard: external Prototype native-action helper returned");
    }
    const requirements: [string, boolean][] = [
        ["user32 load", prepared.includes('Deno.dlopen("user32.dll"')],
        ["kernel32 load", prepared.includes('Deno.dlopen("kernel32.dll"')],
        ["exact window search", prepared.includes("async function exactPrototypeWindow")],
        ["atomic window-thread prefix", prepared.includes("function postAtomicInputBatch")],
        ["exact process identity", prepared.includes("processId[0] === expected.processId")],
        [
            "bounded window search",
            prepared.includes("WINDOW_SEARCH_TIMEOUT_MILLISECONDS = 20_000"),
        ],
        [
            "monotonic key delay",
            prepared.includes("await waitUntil(delayBase + expected.keyDelays[index])"),
        ],
        [
            "monotonic exit delay",
            prepared.includes(
                "await waitUntil(lastKeyAt + expected.exitAfterLastMilliseconds)",
            ),
        ],
        ["prepared action dispatch", input.includes("startPreparedWindowAction")],
        ["schema validation", prepared.includes("validatePrototypeWindowAction")],
        ["atomic batch", prepared.includes("postAtomicInputBatch")],
        [
            "resolved atomic prefix",
            input.includes("const resolvedAtomicPrefixLength = atomicBatch"),
        ],
        [
            "atomic prefix evidence",
            prepared.includes("atomicPrefixLength: expected.atomicPrefixLength"),
        ],
        [
            "atomic prefix validation",
            prepared.includes("evidence.atomicPrefixLength !== expected.atomicPrefixLength"),
        ],
        ["focus suspension", prepared.includes("suspendAfterInput")],
        ["kill-focus message", prepared.includes("WM_KILLFOCUS = 0x0008")],
        ["window-close message", prepared.includes("WM_CLOSE = 0x0010")],
        ["thread suspension", prepared.includes("SuspendThread")],
        ["thread resumption", prepared.includes("ResumeThread")],
        ["thread-handle close", prepared.includes("CloseHandle")],
        ["atomic restoration", prepared.includes("finally")],
        [
            "wrapper FFI close",
            (wrapper.match(/closeNativeTransport/g)?.length ?? 0) === 2,
        ],
        ["unstable FFI activation", profile.includes('"--unstable-ffi"')],
        ["FFI permission", profile.includes('"--allow-ffi"')],
    ];
    const missing = requirements.find(([, present]) => !present);
    if (missing) fail(`guard: Prototype native-action FFI missing ${missing[0]}`);
    if (
        /prototype-native-window-action-v[23]/.test(transport) ||
        !prepared.includes("prototype-native-window-action-v4")
    ) fail("guard: Prototype native-action schema ownership diverged");
    if (transport.includes("DestroyWindow")) {
        fail("guard: direct Prototype window destruction returned");
    }
}
