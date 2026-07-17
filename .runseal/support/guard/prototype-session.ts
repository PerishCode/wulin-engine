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
    const forwardReleaseAcceptance = await Deno.readTextFile(
        `${root}/.runseal/support/prototype/sessions/forward_release.ts`,
    );
    const boundaryAcceptance = await Deno.readTextFile(
        `${root}/.runseal/support/prototype/boundary.ts`,
    );
    const input = await Deno.readTextFile(`${root}/.runseal/support/prototype/input/mod.ts`);
    const preparedInput = await Deno.readTextFile(
        `${root}/.runseal/support/prototype/input/prepared.ts`,
    );
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
    const diagonalWalkAcceptance = await Deno.readTextFile(
        `${root}/.runseal/support/prototype/sessions/diagonal_walk.ts`,
    );
    const diagonalRunAcceptance = await Deno.readTextFile(
        `${root}/.runseal/support/prototype/sessions/diagonal_run.ts`,
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
    const prototypeSimulation = await Deno.readTextFile(
        `${root}/.runseal/support/prototype/simulation.ts`,
    );
    const canonicalSetup = await Deno.readTextFile(
        `${root}/.runseal/support/canonical-setup.ts`,
    );
    const currentNativeSessionSources = [
        cameraAcceptance,
        cameraRepressAcceptance,
        runReleaseAcceptance,
        runRepressAcceptance,
        locomotionOppositionAcceptance,
        diagonalWalkAcceptance,
        diagonalRunAcceptance,
        forwardReleaseAcceptance,
    ];
    if (currentNativeSessionSources.some((source) => source.includes("startupNativeInput"))) {
        fail("guard: retired startup native-input report branch returned");
    }
    const nativeTypeIndex = input.indexOf("Add-Type -TypeDefinition");
    const helperReadyIndex = input.indexOf(
        '[Console]::Out.WriteLine("prototype-native-helper-ready-v1")',
    );
    const windowSearchIndex = input.indexOf("do {", helperReadyIndex);
    const atomicPrefixIndex = input.indexOf(
        "if ($atomicPrefixLength -gt 0)",
        windowSearchIndex,
    );
    const remainingInputIndex = input.indexOf(
        "if ($atomicPrefixLength -lt $keys.Count)",
        atomicPrefixIndex,
    );
    const capturedReadyIndex = acceptance.indexOf("export async function capturedReady");
    const capturedReadyEndIndex = acceptance.indexOf(
        "export async function sustainedCapacitySession",
        capturedReadyIndex,
    );
    const capturedReadySource = acceptance.slice(capturedReadyIndex, capturedReadyEndIndex);
    const capturedSpawnIndex = acceptance.indexOf(
        "new Deno.Command(executable",
        capturedReadyIndex,
    );
    const gracefulExitIndex = acceptance.indexOf("export async function gracefulExit");
    const gracefulSpawnIndex = acceptance.indexOf(
        "new Deno.Command(executable",
        gracefulExitIndex,
    );
    const boundaryReadyIndex = boundaryAcceptance.indexOf("await readinessLine(reader)");
    const boundaryActionIndex = boundaryAcceptance.indexOf(
        "await holdPrototypeBoundaryRun(child.pid)",
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
        !objectGates.includes("actionAfterReadiness: true") ||
        !acceptance.includes("objectFeedbackSession") ||
        !sessionGates.includes("nativeWindowCloseInvariant") ||
        !sessionGates.includes("forwardReleaseSessionInvariant") ||
        sessionGates.includes("prototype Escape press exit") ||
        sessionGates.includes("escapeInvariant") ||
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
        !sessionGates.includes("diagonalWalkSessionInvariant") ||
        !sessionGates.includes("diagonalRunSessionInvariant") ||
        !inputActions.includes("postPrototypeCapacityRejection") ||
        !inputActions.includes("postObjectActionExit") ||
        !inputActions.includes("postConsumptionCapacity") ||
        !inputActions.includes("requestPrototypeWindowClose") ||
        !inputActions.includes("holdPrototypeBoundaryRun") ||
        inputActions.includes("holdPrototypeForwardKey") ||
        !inputSequences.includes("repressJumpAndExit") ||
        !inputSequences.includes("postMidairSequence") ||
        !inputSequences.includes("postCameraRepeatSequence") ||
        !inputSequences.includes("postCameraRepressSequence") ||
        !inputSequences.includes("postInvalidAliasSequence") ||
        !inputSequences.includes("postOppositeCameraSequence") ||
        !inputSequences.includes("postCounterClockwiseSequence") ||
        !inputSequences.includes("postRunRelease") ||
        !inputSequences.includes("postRunRepress") ||
        !inputSequences.includes("postForwardRelease") ||
        !inputSequences.includes("postOpposedRun") ||
        !inputSequences.includes("postDiagonalWalk") ||
        !inputSequences.includes("postDiagonalRun") ||
        !inputSequences.includes("releaseOpposedRun") ||
        !inputSequences.includes('{ key: "A", virtualKey: 0x41, down: true }') ||
        !inputSequences.includes("pressPrototypeCameraClockwise") ||
        inputSequences.includes("StartupInput") ||
        inputSequences.includes("prepareStartupInput") ||
        inputSequences.includes("startupInputRequest") ||
        inputSequences.includes('case "camera-clockwise"') ||
        inputSequences.includes('case "camera-forward"') ||
        inputSequences.includes('case "forward"') ||
        inputSequences.includes('case "jump"') ||
        inputSequences.includes('case "run-forward"') ||
        acceptance.includes("applyStartupInput(") ||
        acceptance.includes("prepareStartupInput") ||
        acceptance.includes("startupInput") ||
        acceptance.includes("startupNativeInput") ||
        nativeTypeIndex < 0 ||
        helperReadyIndex <= nativeTypeIndex ||
        windowSearchIndex <= helperReadyIndex ||
        atomicPrefixIndex <= windowSearchIndex ||
        remainingInputIndex <= atomicPrefixIndex ||
        capturedReadyIndex < 0 ||
        capturedReadyEndIndex <= capturedReadyIndex ||
        capturedSpawnIndex <= capturedReadyIndex ||
        capturedReadySource.includes("startupInput") ||
        capturedReadySource.includes("prepareStartupInput") ||
        capturedReadySource.includes("nativeInput") ||
        gracefulExitIndex < 0 ||
        gracefulSpawnIndex <= gracefulExitIndex ||
        boundaryReadyIndex < 0 ||
        boundaryActionIndex <= boundaryReadyIndex ||
        !boundaryAcceptance.includes("actionAfterReadiness: true") ||
        !boundaryAcceptance.includes("boundaryRunInputInvariant") ||
        !boundaryAcceptance.includes('{ key: "Shift", virtualKey: 0x10, down: true }') ||
        !boundaryAcceptance.includes('{ key: "W", virtualKey: 0x57, down: true }') ||
        !boundaryAcceptance.includes("atomicWindowThreadBatch: true") ||
        !prototypeHost.includes("boundaryRunInputInvariant(boundary)") ||
        !cameraAcceptance.includes("heldRepeatSuppressed: true") ||
        !cameraAcceptance.includes("retainedOrbitIndex: 1") ||
        !cameraAcceptance.includes("actionAfterReadiness: true") ||
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
        !cameraRepressAcceptance.includes("actionAfterReadiness: true") ||
        !cameraRepressAcceptance.includes("deltaZQ9 <= 0") ||
        !runReleaseAcceptance.includes("runModifierReleased: true") ||
        !runReleaseAcceptance.includes("retainedForwardInput: true") ||
        !runReleaseAcceptance.includes("transitionedToWalk: true") ||
        !runReleaseAcceptance.includes("runHoldIntervalMilliseconds") ||
        !runReleaseAcceptance.includes("atomicInitialPrefix: true") ||
        !runReleaseAcceptance.includes("actionAfterReadiness: true") ||
        !runRepressAcceptance.includes("runModifierReadmitted: true") ||
        !runRepressAcceptance.includes("retainedForwardInput: true") ||
        !runRepressAcceptance.includes("transitionedToRun: true") ||
        !runRepressAcceptance.includes("walkHoldIntervalMilliseconds") ||
        !runRepressAcceptance.includes("atomicInitialPrefix: true") ||
        !runRepressAcceptance.includes("actionAfterReadiness: true") ||
        !locomotionOppositionAcceptance.includes("oppositeAxisCancelled: true") ||
        !locomotionOppositionAcceptance.includes("opposedInputHeldBeforeRelease: true") ||
        !locomotionOppositionAcceptance.includes("releasedBackwardInput: true") ||
        !locomotionOppositionAcceptance.includes("retainedForwardRunReadmitted: true") ||
        !locomotionOppositionAcceptance.includes("actionAfterReadiness: true") ||
        !locomotionOppositionAcceptance.includes("runStepCount") ||
        !diagonalWalkAcceptance.includes("atomicDiagonalInput: true") ||
        !diagonalWalkAcceptance.includes("nativeLeftInput: true") ||
        !diagonalWalkAcceptance.includes("exactWalkNormalization: true") ||
        !diagonalWalkAcceptance.includes("actionAfterReadiness: true") ||
        !diagonalWalkAcceptance.includes("diagonalStepCount") ||
        !diagonalRunAcceptance.includes("atomicDiagonalRunInput: true") ||
        !diagonalRunAcceptance.includes("nativeLeftInput: true") ||
        !diagonalRunAcceptance.includes("exactRunNormalization: true") ||
        !diagonalRunAcceptance.includes("actionAfterReadiness: true") ||
        !diagonalRunAcceptance.includes("diagonalRunStepCount") ||
        !forwardReleaseAcceptance.includes("normalForwardReleased: true") ||
        !forwardReleaseAcceptance.includes("movedThenStopped: true") ||
        !forwardReleaseAcceptance.includes("transitionedToSurvey: true") ||
        !forwardReleaseAcceptance.includes("retainedForwardYaw: true") ||
        !forwardReleaseAcceptance.includes("actionAfterReadiness: true") ||
        !forwardReleaseAcceptance.includes("walkHoldIntervalMilliseconds") ||
        !forwardReleaseAcceptance.includes("stationaryHoldIntervalMilliseconds") ||
        !focusAcceptance.includes("atomicWindowThreadBatch") ||
        !focusAcceptance.includes("sameBatchJumpDidNotReachResumedSimulation: true") ||
        !focusAcceptance.includes("heldLocomotionDidNotReachSimulation: true") ||
        !focusAcceptance.includes("resumedReadyProgress: true") ||
        !focusAcceptance.includes("actionAfterReadiness: true") ||
        !focusAcceptance.includes("actionPressBeforeFocusLoss: true") ||
        !cameraPolicy.includes("i8::from(input.was_pressed(CLOCKWISE))") ||
        !cameraPolicy.includes("i8::from(input.was_pressed(COUNTER_CLOCKWISE))") ||
        !hostInput.includes("down == key_is_set(&self.held, key)") ||
        !hostInput.includes("u8::try_from(key)") ||
        !input.includes("[Diagnostics.Stopwatch]::StartNew()") ||
        !input.includes("$windowProcessId -eq $expectedProcessId") ||
        input.includes("$expectedProcessId -eq 0") ||
        !input.includes("$keyDeadlineTicks") ||
        input.includes("Start-Sleep -Milliseconds $keyDelay") ||
        !input.includes("prototype-native-window-action-v4") ||
        /prototype-native-window-action-v[23]/.test(input) ||
        !input.includes("startPreparedWindowAction") ||
        !preparedInput.includes('"prototype-native-helper-ready-v1"') ||
        !preparedInput.includes("completePrototypeWindowAction") ||
        !input.includes("PostAtomicInputBatch") ||
        !input.includes(
            "$atomicBatch = $atomicPrefixLength -eq $keys.Count -and $atomicPrefixLength -gt 0",
        ) ||
        !input.includes("atomicPrefixLength = $atomicPrefixLength") ||
        !preparedInput.includes("evidence.atomicPrefixLength !== expected.atomicPrefixLength") ||
        !input.includes("suspendAfterInput") ||
        !input.includes("0x0008u") ||
        !input.includes("SuspendThread") ||
        !input.includes("ResumeThread") ||
        inputSequences.includes('case "object-action"') ||
        !objectObservation.includes("idleObservationInvariant") ||
        !prototypeHost.includes("objectActionCenter: Coord = [base[0] + 4, base[1]]") ||
        prototypeHost.includes('"object-action"') ||
        prototypeHost.includes("prototype forward locomotion") ||
        prototypeHost.includes("prototype forward Run modifier") ||
        prototypeHost.includes("prototype camera-relative forward locomotion") ||
        prototypeHost.includes("prototype clockwise camera orbit") ||
        prototypeHost.includes("prototype committed jump") ||
        prototypeHost.includes("forwardInvariant") ||
        prototypeHost.includes("runInvariant") ||
        prototypeHost.includes("cameraForwardInvariant") ||
        prototypeHost.includes("cameraOrbitInvariant") ||
        prototypeHost.includes("jumpInvariant") ||
        prototypeSimulation.includes("RUN_FORWARD_COMMAND") ||
        prototypeSimulation.includes("CAMERA_FORWARD_COMMAND") ||
        prototypeSimulation.includes("JUMP_COMMAND") ||
        prototypeSimulation.includes("FORWARD_COMMAND") ||
        !canonicalSetup.includes("objectActionCenter: Coord = [base[0] + 4, base[1]]") ||
        !canonicalSetup.includes(
            "objectActionTraversalCenter: Coord = [base[0] + 5, base[1] + 1]",
        ) ||
        !inputActions.includes("suspendWithActionBatch") ||
        inputActions.includes("suspendWithJumpAndForward") ||
        inputActions.includes("suspendWithForward") ||
        !inputActions.includes("resumePrototypeFocus") ||
        !input.includes("0x0010") ||
        !input.includes("0x0008") ||
        input.includes("DestroyWindow") ||
        !inputActions.includes('{ key: "D", virtualKey: 0x44, down: true }') ||
        !inputActions.includes('{ key: "D", virtualKey: 0x44, down: false }') ||
        !inputActions.includes('"prototype-capacity-rejection-input-v1"') ||
        !objectGates.includes("(delayedExit ? 250 : 0)") ||
        !objectGates.includes('number(evidence, "exitIntervalMilliseconds") < 250') ||
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
        !objectGates.includes("exactCommittedOriginProximity: true") ||
        !objectGates.includes("exactCommittedFacing: true") ||
        !objectGates.includes("exactSourceIdentity")
    ) fail("guard: current projected-feedback/state authority diverged");
}
