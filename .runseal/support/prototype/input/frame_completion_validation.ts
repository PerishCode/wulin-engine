import { fail, type Json } from "../../canonical-runtime.ts";
import {
    ACTIVATED_FRAME_COMPLETION_SCHEMA,
    COMPLETION_TOLERANCE_PIXELS,
    FRAME_COMPLETION_TIMEOUT_MS,
    MINIMUM_ACTIVATED_PIXEL_DELTA,
    REQUIRED_CLEAR_SAMPLES,
} from "./frame_completion_contract.ts";

export async function completeActivatedFrameCompletion(
    child: Deno.ChildProcess,
    reader: ReadableStreamDefaultReader<string>,
    stderr: Promise<string>,
    expectedProcessId: number,
): Promise<Json> {
    let stdout = "";
    while (true) {
        const chunk = await reader.read();
        if (chunk.done) break;
        stdout += chunk.value;
    }
    const status = await child.status;
    const stderrText = (await stderr).trim();
    if (!status.success) {
        fail(
            `prototype Activated frame observer failed with ${status.code}: ${
                stderrText.slice(-4_096)
            }`,
        );
    }
    if (stderrText !== "") {
        fail(`prototype Activated frame observer emitted stderr: ${stderrText.slice(-4_096)}`);
    }
    const evidence = JSON.parse(stdout.trim()) as Json;
    if (
        evidence.schema !== ACTIVATED_FRAME_COMPLETION_SCHEMA ||
        evidence.processId !== expectedProcessId ||
        typeof evidence.windowHandle !== "string" ||
        evidence.windowHandle === "" ||
        evidence.requiredVisible !== true ||
        evidence.windowWasVisible !== true ||
        evidence.captureVisibility !== "temporary-topmost-noactivate" ||
        evidence.captureMethod !== "print-window-client-full-content-v1" ||
        evidence.colorRule !== "activated-green-v1" ||
        typeof evidence.completionObserved !== "boolean" ||
        typeof evidence.baselineActivatedPixelCount !== "number" ||
        !Number.isSafeInteger(evidence.baselineActivatedPixelCount) ||
        evidence.baselineActivatedPixelCount < 0 ||
        evidence.minimumActivatedPixelDelta !== MINIMUM_ACTIVATED_PIXEL_DELTA ||
        evidence.completionTolerancePixels !== COMPLETION_TOLERANCE_PIXELS ||
        typeof evidence.activatedPixelPeak !== "number" ||
        !Number.isSafeInteger(evidence.activatedPixelPeak) ||
        typeof evidence.activatedSampleCount !== "number" ||
        !Number.isSafeInteger(evidence.activatedSampleCount) ||
        evidence.activatedSampleCount < 0 ||
        typeof evidence.completionClearSampleCount !== "number" ||
        !Number.isSafeInteger(evidence.completionClearSampleCount) ||
        evidence.completionClearSampleCount < 0 ||
        typeof evidence.sampleCount !== "number" ||
        !Number.isSafeInteger(evidence.sampleCount) ||
        evidence.sampleCount < 1 ||
        typeof evidence.elapsedMilliseconds !== "number" ||
        evidence.elapsedMilliseconds <= 0 ||
        evidence.timeoutMilliseconds !== FRAME_COMPLETION_TIMEOUT_MS ||
        JSON.stringify(evidence.messages) !== JSON.stringify(["WM_KEYDOWN:Escape"])
    ) {
        fail(`prototype Activated frame completion evidence diverged: ${JSON.stringify(evidence)}`);
    }
    if (evidence.completionObserved === true) {
        if (
            evidence.captureOwner !== null ||
            evidence.activatedPixelPeak <
                evidence.baselineActivatedPixelCount + MINIMUM_ACTIVATED_PIXEL_DELTA ||
            evidence.activatedSampleCount < 1 ||
            typeof evidence.completionPixelCount !== "number" ||
            !Number.isSafeInteger(evidence.completionPixelCount) ||
            evidence.completionPixelCount < 0 ||
            evidence.completionPixelCount >
                evidence.baselineActivatedPixelCount + COMPLETION_TOLERANCE_PIXELS ||
            evidence.completionClearSampleCount !== REQUIRED_CLEAR_SAMPLES ||
            evidence.sampleCount <
                evidence.activatedSampleCount + evidence.completionClearSampleCount ||
            evidence.elapsedMilliseconds >= FRAME_COMPLETION_TIMEOUT_MS
        ) {
            fail(
                `prototype Activated frame success evidence diverged: ${JSON.stringify(evidence)}`,
            );
        }
    } else if (
        typeof evidence.captureOwner !== "string" ||
        evidence.captureOwner === "" ||
        evidence.completionClearSampleCount >= REQUIRED_CLEAR_SAMPLES ||
        evidence.elapsedMilliseconds < FRAME_COMPLETION_TIMEOUT_MS
    ) {
        fail(
            `prototype Activated frame timeout evidence diverged: ${JSON.stringify(evidence)}`,
        );
    }
    return evidence;
}
