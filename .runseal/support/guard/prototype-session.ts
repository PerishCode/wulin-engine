type Fail = (message: string) => never;

export async function requireBoundedPrototypeSession(
    root: string,
    fail: Fail,
): Promise<void> {
    console.log("==> bounded non-diagnostic Prototype session");
    const main = await Deno.readTextFile(`${root}/apps/prototype/src/main.rs`);
    const session = await Deno.readTextFile(`${root}/apps/prototype/src/session.rs`);
    const acceptance = await Deno.readTextFile(
        `${root}/.runseal/support/prototype/session.ts`,
    );
    const input = await Deno.readTextFile(`${root}/.runseal/support/prototype/input.ts`);
    if (
        !main.includes("mod session;") ||
        (main.match(/session::publish_readiness/g)?.length ?? 0) !== 1 ||
        (main.match(/session::publish_completion/g)?.length ?? 0) !== 1 ||
        /(^|[^A-Za-z])println!/m.test(main) ||
        !session.includes('REVISION: &str = "live-prototype-session-completion-v1"') ||
        !session.includes('"sequence": 1') ||
        !session.includes('"sequence": 2') ||
        !session.includes('"completion": "graceful-exit-only"') ||
        !session.includes('"eventStream": false') ||
        !session.includes('"eventHistory": false') ||
        !session.includes('"live-prototype-object-rejected-feedback-v2"') ||
        (session.match(/println!/g)?.length ?? 0) !== 2 ||
        !acceptance.includes('outputLine(reader, "session completion"') ||
        !acceptance.includes("trailing session output") ||
        !acceptance.includes("completionEmitted !== false") ||
        !acceptance.includes("buffered output after") ||
        !acceptance.includes("objectNearestOracle") ||
        !acceptance.includes("capacityRejectedFrameCount: 12") ||
        !acceptance.includes("postReadinessCapacityRejection") ||
        !input.includes("postPrototypeCapacityRejection") ||
        !input.includes('{ key: "D", virtualKey: 0x44, down: false }') ||
        !input.includes('{ key: "F", virtualKey: 0x46, down: false }') ||
        !input.includes('{ key: "Enter", virtualKey: 0x0D, down: false }')
    ) fail("guard: bounded Prototype session contract diverged");

    const waitIndex = main.indexOf("runtime.wait_idle()");
    const completionIndex = main.indexOf("session::publish_completion");
    const teardownIndex = main.indexOf("window::teardown()");
    if (
        waitIndex < 0 || completionIndex < waitIndex || teardownIndex < completionIndex ||
        /event_(stream|log|history)|Vec<.*Event|inspect|replay|std::fs|File::|write_all/i.test(
            session,
        )
    ) fail("guard: Prototype completion became recurring diagnostics or changed exit ordering");
}
