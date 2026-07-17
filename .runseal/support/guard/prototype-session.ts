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
    const objectGates = await Deno.readTextFile(
        `${root}/.runseal/support/prototype/object/gates.ts`,
    );
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
        !session.includes('"live-prototype-object-rejected-feedback-v3"') ||
        (session.match(/println!/g)?.length ?? 0) !== 2 ||
        !acceptance.includes('outputLine(reader, "session completion"') ||
        !acceptance.includes("trailing session output") ||
        !acceptance.includes("completionEmitted !== false") ||
        !acceptance.includes("buffered output after") ||
        !objectGates.includes("objectNearestOracle") ||
        !objectGates.includes("capacityRejectedFrameCount: 12") ||
        !objectGates.includes("postReadinessCapacityRejection") ||
        !acceptance.includes("nativeWindowCloseInvariant") ||
        !acceptance.includes("focusSessionInvariant") ||
        !acceptance.includes("jumpReadmissionInvariant") ||
        !input.includes("postPrototypeCapacityRejection") ||
        !input.includes("requestPrototypeWindowClose") ||
        !input.includes("repressJumpAndExit") ||
        !input.includes("[Diagnostics.Stopwatch]::StartNew()") ||
        !input.includes("suspendWithForward") ||
        !input.includes("resumePrototypeFocus") ||
        !input.includes("0x0010") ||
        !input.includes("0x0008") ||
        input.includes("DestroyWindow") ||
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

    console.log("==> removed transient Prototype action report");
    const interaction = await Deno.readTextFile(
        `${root}/apps/prototype/src/object/interaction.rs`,
    );
    const tests = await Deno.readTextFile(
        `${root}/apps/prototype/tests/object_interaction_policy.rs`,
    );
    const actionAcceptance = await Deno.readTextFile(
        `${root}/.runseal/support/prototype/object/interaction.ts`,
    );
    if (
        interaction.includes("FrameCompletion") ||
        interaction.includes("pub(crate) fn attempt(") ||
        interaction.includes("Result<Option<FrameCompletion>>") ||
        session.includes("interaction_attempt") ||
        session.includes("interaction_completion") ||
        session.includes('"attempt":') ||
        session.includes('"completion": evidence.interaction') ||
        main.includes("interaction_completion") ||
        tests.includes("report::attempt") ||
        tests.includes("completion.applied") ||
        tests.includes("completion.feedback") ||
        actionAcceptance.includes("driver.attempt") ||
        actionAcceptance.includes("driver.completion") ||
        actionAcceptance.includes('object(driver, "attempt")') ||
        actionAcceptance.includes('object(driver, "completion")')
    ) fail("guard: retired transient Prototype action report returned");
    if (
        !interaction.includes(") -> Result<()>") ||
        !actionAcceptance.includes('"attempt" in driver') ||
        !actionAcceptance.includes('"completion" in driver') ||
        !actionAcceptance.includes("projectedFeedback: feedback") ||
        !actionAcceptance.includes("exactCommittedOriginProximity: true") ||
        !actionAcceptance.includes("exactCommittedFacing: true")
    ) fail("guard: current projected-feedback/state authority diverged");
}
