type Fail = (message: string) => never;

export async function requireInputJournalRemoved(root: string, fail: Fail): Promise<void> {
    console.log("==> removed diagnostic input journal surface");
    try {
        await Deno.stat(`${root}/.runseal/support/host-input-replay.ts`);
        fail("guard: removed host input replay support remains");
    } catch (error) {
        if (!(error instanceof Deno.errors.NotFound)) throw error;
    }

    await requireAbsent(
        root,
        fail,
        [
            "ActiveRecording",
            "CompletedRecording",
            "InputTransaction",
            "TransactionCounters",
            "start_recording",
            "stop_recording",
            "recording_fault",
            "status_json",
            "stream_sha256",
            "held_state_sha256",
            "MAX_RECORD_(TRANSACTIONS|TRANSITIONS)",
        ].join("|"),
        ["crates/reference-host/src/input.rs", "crates/reference-host/tests/private/input.rs"],
        "input journal implementation",
    );
    await requireAbsent(
        root,
        fail,
        [
            "PostedMessage",
            "post_input\\(",
            "Input(Status|RecordStart|RecordStop|Replay|Post)",
            "input\\.(status|record\\.(start|stop)|replay|native\\.post)",
            "input-record-(start|stop)",
            "input-replay",
            "runseal :workbench input",
            "hostInputGates",
            "(^|[^A-Za-z])hostInput([^A-Za-z]|$)",
        ].join("|"),
        [
            "apps/workbench",
            "crates/reference-host/src/window.rs",
            ".runseal/wrappers",
            ".runseal/support/canonical-runtime.ts",
        ],
        "input journal control",
    );
    await requireAbsent(
        root,
        fail,
        [
            "input\\.(status|record\\.(start|stop)|replay|native\\.post)",
            "input-record-(start|stop)",
            "input-replay",
            "runseal :workbench input",
        ].join("|"),
        ["README.md", "AGENTS.md"],
        "input journal operator documentation",
    );
    await requireAbsent(
        root,
        fail,
        "state\\.input|input: HostInput",
        ["apps/workbench/src"],
        "persistent workbench input owner",
    );
}

async function requireAbsent(
    root: string,
    fail: Fail,
    pattern: string,
    paths: string[],
    label: string,
): Promise<void> {
    const output = await new Deno.Command("git", {
        args: ["grep", "--no-index", "-n", "-E", pattern, "--", ...paths],
        cwd: root,
        stdout: "piped",
        stderr: "inherit",
    }).output();
    if (output.code === 0) {
        fail(`guard: removed ${label} found\n${new TextDecoder().decode(output.stdout)}`);
    }
    if (output.code !== 1) {
        fail(`guard: removed ${label} scan failed with exit code ${output.code}`);
    }
}
