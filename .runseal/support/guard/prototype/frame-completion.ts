type Fail = (message: string) => never;

export async function requirePrototypeFrameCompletion(
    root: string,
    fail: Fail,
): Promise<void> {
    console.log("==> bounded frame-observable Prototype session");
    // deno-fmt-ignore
    const frameCompletion = await Deno.readTextFile(`${root}/.runseal/support/prototype/input/frame_completion.ts`);
    // deno-fmt-ignore
    const desktop = await Deno.readTextFile(`${root}/.runseal/support/prototype/input/frame_completion_desktop.ts`);
    // deno-fmt-ignore
    const contract = await Deno.readTextFile(`${root}/.runseal/support/prototype/input/frame_completion_contract.ts`);
    // deno-fmt-ignore
    const script = await Deno.readTextFile(`${root}/.runseal/support/prototype/input/frame_completion_script.ts`);
    const inputActions = await Deno.readTextFile(
        `${root}/.runseal/support/prototype/input/actions.ts`,
    );
    const focused = await Deno.readTextFile(
        `${root}/.runseal/support/prototype/object/focused-frame.ts`,
    );
    const objectGates = await Deno.readTextFile(
        `${root}/.runseal/support/prototype/object/gates.ts`,
    );
    const objectInputGates = await Deno.readTextFile(
        `${root}/.runseal/support/prototype/object/input-gates.ts`,
    );
    const wrapper = await Deno.readTextFile(
        `${root}/.runseal/wrappers/canonical-prototype.ts`,
    );
    if (
        !contract.includes("prototype-activated-frame-completion-v1") ||
        !frameCompletion.includes("PrototypeActivatedFrameObserver") ||
        !frameCompletion.includes("CountActivatedPixels") ||
        !frameCompletion.includes("DwmFlush") ||
        !frameCompletion.includes("GdiFlush") ||
        !frameCompletion.includes("GetDC(window)") ||
        !frameCompletion.includes("PrintWindow") ||
        !frameCompletion.includes("SetCaptureTopmost") ||
        !frameCompletion.includes("CaptureOwner") ||
        !script.includes("[PrototypeActivatedFrameNative]::CaptureOwner($window)") ||
        !script.includes("completionObserved = $completionObserved") ||
        !desktop.includes("export async function requireActivatedFrameDesktop") ||
        !desktop.includes("OpenInputDesktop") ||
        !desktop.includes("requires an interactive desktop") ||
        !frameCompletion.includes("observer exited before readiness;") ||
        !frameCompletion.includes("stderr=${(await stderr).trim().slice(-4_096)}") ||
        !script.includes("minimumActivatedPixelDelta") ||
        !script.includes("completionClearSampleCount") ||
        !script.includes("temporary-topmost-noactivate") ||
        !script.includes("print-window-client-full-content-v1") ||
        !script.includes('"WM_KEYDOWN:Escape"') ||
        !inputActions.includes("prepareActivatedFrameCompletion") ||
        !inputActions.includes("prototype-object-recovery-frame-completion-v1") ||
        !objectInputGates.includes("frameCompletion.completionObserved !== true") ||
        !focused.includes("focusedActivatedFrameGate") ||
        !focused.includes("activatedObjectFeedbackInvariant") ||
        !focused.includes("requireSingleOwnerInvariant") ||
        !objectGates.includes("export async function activatedObjectFeedbackInvariant") ||
        !wrapper.includes('ACTIVATED_FRAME_ARGUMENT = "--case=activated-frame"') ||
        !wrapper.includes(
            'ACTIVATED_FRAME_COLLECTION = "canonical-prototype-activated-frame"',
        ) ||
        !wrapper.includes("if (activatedFrame) await requireActivatedFrameDesktop();") ||
        !wrapper.includes("prepareCanonicalFrameSetup(") ||
        !wrapper.includes("[BASE[0] + 1, BASE[1] + 1]") ||
        !wrapper.includes("prototype = await focusedActivatedFrameGate(") ||
        !wrapper.includes("prototype = await prototypeHostGates(") ||
        inputActions.includes(
            "postObjectRecoveryExit(processId: number): Promise<Json> {\n    return await postPrototypeWindowAction",
        )
    ) {
        fail("guard: frame-observable Prototype completion diverged");
    }
}
