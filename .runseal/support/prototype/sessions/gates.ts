import { fail, type Json, number, object, same } from "../../canonical-runtime.ts";
import { nativeWindowCloseInvariant } from "../input/actions.ts";
import {
    cameraRepeatSessionInvariant,
    invalidKeySessionInvariant,
    oppositeCameraSessionInvariant,
} from "../camera.ts";
import { counterClockwiseSessionInvariant } from "../camera_counter_clockwise.ts";
import { cameraRepressSessionInvariant } from "../camera_repress.ts";
import { jumpMidairInvariant, jumpReadmissionInvariant } from "../jump.ts";
import { sustainedCapacityInvariant } from "../object/gates.ts";
import { runReleaseSessionInvariant } from "./run_release.ts";
import { runRepressSessionInvariant } from "./run_repress.ts";
import { focusSessionInvariant } from "./focus.ts";
import { forwardReleaseSessionInvariant } from "./forward_release.ts";
import { diagonalRunSessionInvariant } from "./diagonal_run.ts";
import { diagonalWalkSessionInvariant } from "./diagonal_walk.ts";
import { locomotionOppositionSessionInvariant } from "./locomotion_opposition.ts";
import { gracefulCompletionInvariant, gracefulExit, idleCompletionInvariant } from "./mod.ts";

type LaunchInvariant = (launch: Json) => Json;

export async function sessionGates(
    executable: string,
    config: string,
    first: Json,
    sustained: Json,
    sustainedBaseline: Json,
    startupInvariant: LaunchInvariant,
    jumpInvariant: LaunchInvariant,
    source: string,
    windowCenter: [number, number],
): Promise<Json> {
    const forwardRelease = await gracefulExit(
        executable,
        config,
        "prototype native forward release",
        "forward-release",
    );
    const windowClose = await gracefulExit(
        executable,
        config,
        "prototype native window close exit",
        null,
        "window-close",
    );
    const focusDiscontinuity = await gracefulExit(
        executable,
        config,
        "prototype native focus discontinuity",
        "focus-discontinuity",
    );
    const jumpReadmission = await gracefulExit(
        executable,
        config,
        "prototype native Jump readmission",
        "jump-readmission",
    );
    const jumpMidair = await gracefulExit(
        executable,
        config,
        "prototype native midair Jump rejection",
        "jump-midair",
    );
    const cameraRepeat = await gracefulExit(
        executable,
        config,
        "prototype native held camera repeat",
        "camera-repeat",
    );
    const cameraRepress = await gracefulExit(
        executable,
        config,
        "prototype native camera re-press readmission",
        "camera-repress",
    );
    const invalidKey = await gracefulExit(
        executable,
        config,
        "prototype native out-of-range camera key",
        "invalid-camera-alias",
    );
    const oppositeCamera = await gracefulExit(
        executable,
        config,
        "prototype native opposite camera edges",
        "opposite-camera",
    );
    const counterClockwiseCamera = await gracefulExit(
        executable,
        config,
        "prototype native counter-clockwise camera wrap",
        "counter-clockwise-camera",
    );
    const runRelease = await gracefulExit(
        executable,
        config,
        "prototype native Run modifier release",
        "run-release",
    );
    const runRepress = await gracefulExit(
        executable,
        config,
        "prototype native Run modifier re-press readmission",
        "run-repress",
    );
    const locomotionOpposition = await gracefulExit(
        executable,
        config,
        "prototype native opposite locomotion release",
        "opposed-run-release",
    );
    const diagonalWalk = await gracefulExit(
        executable,
        config,
        "prototype native diagonal Walk",
        "diagonal-walk",
    );
    const diagonalRun = await gracefulExit(
        executable,
        config,
        "prototype native diagonal Run",
        "diagonal-run",
    );
    sameInitial(forwardRelease, first, "forward-release", startupInvariant, jumpInvariant);
    sameInitial(windowClose, first, "window-close", startupInvariant, jumpInvariant);
    same(
        startupInvariant(focusDiscontinuity),
        startupInvariant(first),
        "prototype focus-discontinuity configuration",
    );
    sameInitial(jumpReadmission, first, "Jump-readmission", startupInvariant, jumpInvariant);
    sameInitial(jumpMidair, first, "midair-Jump", startupInvariant, jumpInvariant);
    sameInitial(cameraRepeat, first, "held-camera-repeat", startupInvariant, jumpInvariant);
    sameInitial(cameraRepress, first, "camera-repress", startupInvariant, jumpInvariant);
    sameInitial(invalidKey, first, "invalid-key", startupInvariant, jumpInvariant);
    sameInitial(oppositeCamera, first, "opposite-camera", startupInvariant, jumpInvariant);
    sameInitial(
        counterClockwiseCamera,
        first,
        "counter-clockwise-camera",
        startupInvariant,
        jumpInvariant,
    );
    sameInitial(runRelease, first, "Run-release", startupInvariant, jumpInvariant);
    sameInitial(runRepress, first, "Run-repress", startupInvariant, jumpInvariant);
    sameInitial(
        locomotionOpposition,
        first,
        "opposite-locomotion",
        startupInvariant,
        jumpInvariant,
    );
    sameInitial(diagonalWalk, first, "diagonal-Walk", startupInvariant, jumpInvariant);
    sameInitial(diagonalRun, first, "diagonal-Run", startupInvariant, jumpInvariant);
    sameInitial(sustained, sustainedBaseline, "sustained", startupInvariant, jumpInvariant);
    return {
        forwardRelease,
        forwardReleaseInvariant: forwardReleaseSessionInvariant(
            forwardRelease,
            idleCompletionInvariant(forwardRelease),
        ),
        windowClose,
        windowCloseInvariant: {
            ...idleCompletionInvariant(windowClose, "window-close"),
            nativeWindowClose: nativeWindowCloseInvariant(
                object(windowClose, "exitInput"),
                number(windowClose, "processId"),
            ),
        },
        focusDiscontinuity,
        focusDiscontinuityInvariant: focusSessionInvariant(
            focusDiscontinuity,
            idleCompletionInvariant(focusDiscontinuity),
        ),
        jumpReadmission,
        jumpReadmissionInvariant: jumpReadmissionInvariant(
            jumpReadmission,
            idleCompletionInvariant(jumpReadmission),
        ),
        jumpMidair,
        jumpMidairInvariant: jumpMidairInvariant(
            jumpMidair,
            idleCompletionInvariant(jumpMidair),
        ),
        cameraRepeat,
        cameraRepeatInvariant: cameraRepeatSessionInvariant(
            cameraRepeat,
            idleCompletionInvariant(cameraRepeat),
        ),
        cameraRepress,
        cameraRepressInvariant: cameraRepressSessionInvariant(
            cameraRepress,
            idleCompletionInvariant(cameraRepress),
        ),
        invalidKey,
        invalidKeyInvariant: invalidKeySessionInvariant(
            invalidKey,
            idleCompletionInvariant(invalidKey),
        ),
        oppositeCamera,
        oppositeCameraInvariant: oppositeCameraSessionInvariant(
            oppositeCamera,
            idleCompletionInvariant(oppositeCamera),
        ),
        counterClockwiseCamera,
        counterClockwiseCameraInvariant: counterClockwiseSessionInvariant(
            counterClockwiseCamera,
            idleCompletionInvariant(counterClockwiseCamera),
        ),
        runRelease,
        runReleaseInvariant: runReleaseSessionInvariant(
            runRelease,
            idleCompletionInvariant(runRelease),
        ),
        runRepress,
        runRepressInvariant: runRepressSessionInvariant(
            runRepress,
            idleCompletionInvariant(runRepress),
        ),
        locomotionOpposition,
        locomotionOppositionInvariant: locomotionOppositionSessionInvariant(
            locomotionOpposition,
            idleCompletionInvariant(locomotionOpposition),
        ),
        diagonalWalk,
        diagonalWalkInvariant: diagonalWalkSessionInvariant(
            diagonalWalk,
            idleCompletionInvariant(diagonalWalk),
        ),
        diagonalRun,
        diagonalRunInvariant: diagonalRunSessionInvariant(
            diagonalRun,
            idleCompletionInvariant(diagonalRun),
        ),
        sustained,
        sustainedInvariant: await sustainedCapacityInvariant(
            sustained,
            gracefulCompletionInvariant(sustained, "escape"),
            source,
            windowCenter,
        ),
    };
}

function sameInitial(
    launch: Json,
    first: Json,
    label: string,
    startupInvariant: LaunchInvariant,
    jumpInvariant: LaunchInvariant,
): void {
    same(
        startupInvariant(launch),
        startupInvariant(first),
        `prototype ${label} configuration`,
    );
    same(
        jumpInvariant(launch),
        jumpInvariant(first),
        `prototype ${label} initial grounded policy`,
    );
}
