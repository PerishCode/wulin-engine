import { fail, type Json, number, object, same } from "../../canonical-runtime.ts";
import { nativeWindowCloseInvariant } from "../input/actions.ts";
import { cameraRepeatSessionInvariant, oppositeCameraSessionInvariant } from "../camera.ts";
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
export const MINIMUM_COPIED_SUBTREE_BYTES = 16;
const encoder = new TextEncoder();

function collectObjectSubtrees(value: unknown, values: Set<string>): void {
    if (value === null || typeof value !== "object") return;
    const serialized = JSON.stringify(value);
    if (encoder.encode(serialized).length >= MINIMUM_COPIED_SUBTREE_BYTES) {
        values.add(serialized);
    }
    if (Array.isArray(value)) {
        for (const child of value) collectObjectSubtrees(child, values);
    } else {
        for (const child of Object.values(value)) collectObjectSubtrees(child, values);
    }
}

function rejectCopiedObjectSubtree(
    value: unknown,
    rawSubtrees: Set<string>,
    label: string,
    path: string,
): void {
    if (value === null || typeof value !== "object") return;
    const serialized = JSON.stringify(value);
    if (
        encoder.encode(serialized).length >= MINIMUM_COPIED_SUBTREE_BYTES &&
        rawSubtrees.has(serialized)
    ) fail(`${label} copied raw launch evidence at ${path}`);
    if (Array.isArray(value)) {
        value.forEach((child, index) =>
            rejectCopiedObjectSubtree(child, rawSubtrees, label, `${path}[${index}]`)
        );
    } else {
        for (const [key, child] of Object.entries(value)) {
            rejectCopiedObjectSubtree(child, rawSubtrees, label, `${path}.${key}`);
        }
    }
}

export function requireSingleOwnerInvariant(
    launch: Json,
    invariant: Json,
    label: string,
): void {
    const rawSubtrees = new Set<string>();
    collectObjectSubtrees(launch, rawSubtrees);
    rejectCopiedObjectSubtree(invariant, rawSubtrees, label, "$");
}

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
    sameInitial(sustained, sustainedBaseline, "sustained", startupInvariant, jumpInvariant);
    const sustainedInvariant = await sustainedCapacityInvariant(
        sustained,
        gracefulCompletionInvariant(sustained, "escape"),
        source,
        windowCenter,
    );
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
    const evidence = {
        forwardRelease,
        forwardReleaseInvariant: forwardReleaseSessionInvariant(
            forwardRelease,
            idleCompletionInvariant(forwardRelease),
        ),
        windowClose,
        windowCloseInvariant: {
            ...idleCompletionInvariant(windowClose, "window-close"),
            nativeWindowClose: nativeWindowCloseInvariant(
                object(windowClose, "nativeInput"),
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
        sustainedInvariant,
    };
    for (
        const name of [
            "forwardRelease",
            "windowClose",
            "focusDiscontinuity",
            "jumpReadmission",
            "jumpMidair",
            "cameraRepeat",
            "cameraRepress",
            "oppositeCamera",
            "counterClockwiseCamera",
            "runRelease",
            "runRepress",
            "locomotionOpposition",
            "diagonalWalk",
            "diagonalRun",
            "sustained",
        ]
    ) {
        requireSingleOwnerInvariant(
            object(evidence, name),
            object(evidence, `${name}Invariant`),
            `prototype ${name} invariant`,
        );
    }
    return evidence;
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
