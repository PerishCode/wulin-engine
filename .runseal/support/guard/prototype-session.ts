type Fail = (message: string) => never;

export async function requireBoundedPrototypeSession(
    root: string,
    fail: Fail,
): Promise<void> {
    console.log("==> bounded non-diagnostic Prototype session");
    const main = await Deno.readTextFile(`${root}/apps/prototype/src/main.rs`);
    const session = await Deno.readTextFile(`${root}/apps/prototype/src/session.rs`);
    const acceptance = await Deno.readTextFile(
        `${root}/.runseal/support/prototype/sessions/mod.ts`,
    );
    const sessionGates = await Deno.readTextFile(
        `${root}/.runseal/support/prototype/sessions/gates.ts`,
    );
    const focusAcceptance = await Deno.readTextFile(
        `${root}/.runseal/support/prototype/sessions/focus.ts`,
    );
    const input = await Deno.readTextFile(`${root}/.runseal/support/prototype/input/mod.ts`);
    const inputActions = await Deno.readTextFile(
        `${root}/.runseal/support/prototype/input/actions.ts`,
    );
    const inputSequences = await Deno.readTextFile(
        `${root}/.runseal/support/prototype/input/sequences.ts`,
    );
    const cameraAcceptance = await Deno.readTextFile(
        `${root}/.runseal/support/prototype/camera.ts`,
    );
    const counterClockwiseCameraAcceptance = await Deno.readTextFile(
        `${root}/.runseal/support/prototype/camera_counter_clockwise.ts`,
    );
    const cameraRepressAcceptance = await Deno.readTextFile(
        `${root}/.runseal/support/prototype/camera_repress.ts`,
    );
    const runReleaseAcceptance = await Deno.readTextFile(
        `${root}/.runseal/support/prototype/sessions/run_release.ts`,
    );
    const runRepressAcceptance = await Deno.readTextFile(
        `${root}/.runseal/support/prototype/sessions/run_repress.ts`,
    );
    const locomotionOppositionAcceptance = await Deno.readTextFile(
        `${root}/.runseal/support/prototype/sessions/locomotion_opposition.ts`,
    );
    const cameraPolicy = await Deno.readTextFile(`${root}/apps/prototype/src/camera.rs`);
    const hostInput = await Deno.readTextFile(`${root}/crates/reference-host/src/input.rs`);
    const objectGates = await Deno.readTextFile(
        `${root}/.runseal/support/prototype/object/gates.ts`,
    );
    const objectObservation = await Deno.readTextFile(
        `${root}/.runseal/support/prototype/object/observation.ts`,
    );
    const prototypeHost = await Deno.readTextFile(
        `${root}/.runseal/support/prototype/host.ts`,
    );
    const canonicalSetup = await Deno.readTextFile(
        `${root}/.runseal/support/canonical-setup.ts`,
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
        !sessionGates.includes("completionEmitted !== false") ||
        !acceptance.includes("buffered output after") ||
        !objectGates.includes("objectNearestOracle") ||
        !objectGates.includes("capacityRejectedFrameCount: 12") ||
        !objectGates.includes("postReadinessCapacityRejection") ||
        !sessionGates.includes("nativeWindowCloseInvariant") ||
        !sessionGates.includes("focusSessionInvariant") ||
        !sessionGates.includes("jumpReadmissionInvariant") ||
        !sessionGates.includes("jumpMidairInvariant") ||
        !sessionGates.includes("cameraRepeatSessionInvariant") ||
        !sessionGates.includes("cameraRepressSessionInvariant") ||
        !sessionGates.includes("invalidKeySessionInvariant") ||
        !sessionGates.includes("oppositeCameraSessionInvariant") ||
        !sessionGates.includes("counterClockwiseSessionInvariant") ||
        !sessionGates.includes("runReleaseSessionInvariant") ||
        !sessionGates.includes("runRepressSessionInvariant") ||
        !sessionGates.includes("locomotionOppositionSessionInvariant") ||
        !inputActions.includes("postPrototypeCapacityRejection") ||
        !inputActions.includes("requestPrototypeWindowClose") ||
        !inputSequences.includes("repressJumpAndExit") ||
        !inputSequences.includes("postMidairSequence") ||
        !inputSequences.includes("postCameraRepeatSequence") ||
        !inputSequences.includes("postCameraRepressSequence") ||
        !inputSequences.includes("postInvalidAliasSequence") ||
        !inputSequences.includes("postOppositeCameraSequence") ||
        !inputSequences.includes("postCounterClockwiseSequence") ||
        !inputSequences.includes("postRunReleaseSequence") ||
        !inputSequences.includes("postRunRepressSequence") ||
        !inputSequences.includes("releaseOpposedRun") ||
        !acceptance.includes("applyStartupInput(null, startupInput)") ||
        !acceptance.includes("startup input selected the wrong process") ||
        !cameraAcceptance.includes("heldRepeatSuppressed: true") ||
        !cameraAcceptance.includes("retainedOrbitIndex: 1") ||
        !cameraAcceptance.includes("checkedRangeRejected: true") ||
        !cameraAcceptance.includes('truncationWouldAlias: "E"') ||
        !cameraAcceptance.includes("oppositePressEdgesRetained: true") ||
        !cameraAcceptance.includes("cameraCandidateCancelled: true") ||
        !counterClockwiseCameraAcceptance.includes(
            "counterClockwisePressEdgeRetained: true",
        ) ||
        !counterClockwiseCameraAcceptance.includes("wrappedOrbitIndex: 3") ||
        !counterClockwiseCameraAcceptance.includes("deltaXQ9 <= 0") ||
        !cameraRepressAcceptance.includes("heldKeyReleased: true") ||
        !cameraRepressAcceptance.includes("freshPressEdgeReadmitted: true") ||
        !cameraRepressAcceptance.includes("committedOrbitIndex: 2") ||
        !cameraRepressAcceptance.includes("deltaZQ9 <= 0") ||
        !runReleaseAcceptance.includes("runModifierReleased: true") ||
        !runReleaseAcceptance.includes("retainedForwardInput: true") ||
        !runReleaseAcceptance.includes("transitionedToWalk: true") ||
        !runReleaseAcceptance.includes("runHoldIntervalMilliseconds") ||
        !runRepressAcceptance.includes("runModifierReadmitted: true") ||
        !runRepressAcceptance.includes("retainedForwardInput: true") ||
        !runRepressAcceptance.includes("transitionedToRun: true") ||
        !runRepressAcceptance.includes("walkHoldIntervalMilliseconds") ||
        !locomotionOppositionAcceptance.includes("oppositeAxisCancelled: true") ||
        !locomotionOppositionAcceptance.includes("stationarySurveyReadiness: true") ||
        !locomotionOppositionAcceptance.includes("releasedBackwardInput: true") ||
        !locomotionOppositionAcceptance.includes("retainedForwardRunReadmitted: true") ||
        !locomotionOppositionAcceptance.includes("runStepCount") ||
        !focusAcceptance.includes("atomicWindowThreadBatch") ||
        !cameraPolicy.includes("i8::from(input.was_pressed(CLOCKWISE))") ||
        !cameraPolicy.includes("i8::from(input.was_pressed(COUNTER_CLOCKWISE))") ||
        !hostInput.includes("down == key_is_set(&self.held, key)") ||
        !hostInput.includes("u8::try_from(key)") ||
        !input.includes("[Diagnostics.Stopwatch]::StartNew()") ||
        input.includes("prototype-native-window-action-v2") ||
        !input.includes("PostAtomicInputBatch") ||
        !input.includes("suspendAfterInput") ||
        !input.includes("0x0008u") ||
        !input.includes("SuspendThread") ||
        !input.includes("ResumeThread") ||
        !inputActions.includes("postInvariantObjectAction") ||
        !objectObservation.includes("maximumBatchGeometryInvariant: true") ||
        !objectObservation.includes("stepCount > 8") ||
        !prototypeHost.includes("objectActionCenter: Coord = [base[0] + 4, base[1]]") ||
        !prototypeHost.includes('"object-action"') ||
        !canonicalSetup.includes("objectActionCenter: Coord = [base[0] + 4, base[1]]") ||
        !canonicalSetup.includes(
            "objectActionTraversalCenter: Coord = [base[0] + 5, base[1] + 1]",
        ) ||
        !inputActions.includes("suspendWithForward") ||
        !inputActions.includes("resumePrototypeFocus") ||
        !input.includes("0x0010") ||
        !input.includes("0x0008") ||
        input.includes("DestroyWindow") ||
        !inputActions.includes('{ key: "D", virtualKey: 0x44, down: true }') ||
        !inputActions.includes('{ key: "D", virtualKey: 0x44, down: false }') ||
        !inputActions.includes('"prototype-capacity-rejection-input-v1"') ||
        !inputActions.includes('{ key: "F", virtualKey: 0x46, down: false }') ||
        !inputActions.includes('{ key: "Enter", virtualKey: 0x0D, down: false }')
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
