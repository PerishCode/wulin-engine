import {
    ACTIVATED_FRAME_COMPLETION_READY,
    ACTIVATED_FRAME_COMPLETION_SCHEMA,
    COMPLETION_TOLERANCE_PIXELS,
    FRAME_COMPLETION_TIMEOUT_MS,
    MINIMUM_ACTIVATED_PIXEL_DELTA,
    REQUIRED_CLEAR_SAMPLES,
} from "./frame_completion_contract.ts";

export function activatedFrameCompletionScript(processId: number): string {
    return String.raw`
$expectedProcessId = ${processId}
$timeoutMilliseconds = ${FRAME_COMPLETION_TIMEOUT_MS}
$minimumActivatedPixelDelta = ${MINIMUM_ACTIVATED_PIXEL_DELTA}
$completionTolerancePixels = ${COMPLETION_TOLERANCE_PIXELS}
$requiredClearSamples = ${REQUIRED_CLEAR_SAMPLES}
$deadline = [DateTime]::UtcNow.AddSeconds(20)
$window = [IntPtr]::Zero
$windowProcessId = [uint32]0
do {
    $candidate = [PrototypeActivatedFrameNative]::FindWindow(
        "WulinEnginePrototypeWindow",
        "Wulin Engine Prototype"
    )
    if ($candidate -ne [IntPtr]::Zero) {
        [void][PrototypeActivatedFrameNative]::GetWindowThreadProcessId(
            $candidate,
            [ref]$windowProcessId
        )
        if (
            $windowProcessId -eq $expectedProcessId -and
            [PrototypeActivatedFrameNative]::IsWindowVisible($candidate)
        ) {
            $window = $candidate
            break
        }
    }
    Start-Sleep -Milliseconds 1
} while ([DateTime]::UtcNow -lt $deadline)

if ($window -eq [IntPtr]::Zero) {
    throw "visible prototype window for process $expectedProcessId was not found"
}
$observer = $null
$captureTopmost = $false
try {
    if (-not [PrototypeActivatedFrameNative]::SetCaptureTopmost($window, $true)) {
        $code = [Runtime.InteropServices.Marshal]::GetLastWin32Error()
        throw "temporarily raising prototype capture window failed with Win32 error $code"
    }
    $captureTopmost = $true
    $observer = [PrototypeActivatedFrameObserver]::new($window)
    $baselineActivatedPixelCount = $observer.CountActivatedPixels()
    [Console]::Out.WriteLine("${ACTIVATED_FRAME_COMPLETION_READY}")
    [Console]::Out.Flush()
    $timer = [Diagnostics.Stopwatch]::StartNew()
    $activatedObserved = $false
    $activatedPixelPeak = $baselineActivatedPixelCount
    $activatedSampleCount = 0
    $completionPixelCount = $null
    $completionClearSampleCount = 0
    $sampleCount = 0
    while ($timer.ElapsedMilliseconds -lt $timeoutMilliseconds) {
        $activatedPixels = $observer.CountActivatedPixels()
        $sampleCount += 1
        if (
            $activatedPixels -ge
                $baselineActivatedPixelCount + $minimumActivatedPixelDelta
        ) {
            $activatedObserved = $true
            $activatedSampleCount += 1
            $completionClearSampleCount = 0
            if ($activatedPixels -gt $activatedPixelPeak) {
                $activatedPixelPeak = $activatedPixels
            }
        } elseif (
            $activatedObserved -and
            $activatedPixels -le
                $baselineActivatedPixelCount + $completionTolerancePixels -and
            $activatedPixelPeak - $activatedPixels -ge $minimumActivatedPixelDelta
        ) {
            $completionClearSampleCount += 1
            $completionPixelCount = $activatedPixels
            if ($completionClearSampleCount -ge $requiredClearSamples) {
                break
            }
        } else {
            $completionClearSampleCount = 0
        }
        Start-Sleep -Milliseconds 5
    }
    $completionObserved = $completionClearSampleCount -ge $requiredClearSamples
    $captureOwner = $null
    if (-not $completionObserved) {
        $captureOwner = [PrototypeActivatedFrameNative]::CaptureOwner($window)
    }
    if (
        $captureTopmost -and
        -not [PrototypeActivatedFrameNative]::SetCaptureTopmost($window, $false)
    ) {
        $code = [Runtime.InteropServices.Marshal]::GetLastWin32Error()
        throw "restoring prototype capture z-order failed with Win32 error $code"
    }
    $captureTopmost = $false
    if (-not [PrototypeActivatedFrameNative]::PostMessage(
        $window,
        0x0100,
        [UIntPtr][uint32]0x1B,
        [IntPtr]1
    )) {
        $code = [Runtime.InteropServices.Marshal]::GetLastWin32Error()
        throw "posting frame-complete prototype Escape failed with Win32 error $code"
    }
    [Console]::Out.Write((ConvertTo-Json ([ordered]@{
        schema = "${ACTIVATED_FRAME_COMPLETION_SCHEMA}"
        processId = [int]$windowProcessId
        windowHandle = $window.ToInt64().ToString()
        requiredVisible = $true
        windowWasVisible = [PrototypeActivatedFrameNative]::IsWindowVisible($window)
        captureVisibility = "temporary-topmost-noactivate"
        captureMethod = "print-window-client-full-content-v1"
        colorRule = "activated-green-v1"
        completionObserved = $completionObserved
        captureOwner = $captureOwner
        baselineActivatedPixelCount = $baselineActivatedPixelCount
        minimumActivatedPixelDelta = $minimumActivatedPixelDelta
        completionTolerancePixels = $completionTolerancePixels
        activatedPixelPeak = $activatedPixelPeak
        activatedSampleCount = $activatedSampleCount
        completionPixelCount = $completionPixelCount
        completionClearSampleCount = $completionClearSampleCount
        sampleCount = $sampleCount
        elapsedMilliseconds = $timer.Elapsed.TotalMilliseconds
        timeoutMilliseconds = $timeoutMilliseconds
        messages = @("WM_KEYDOWN:Escape")
    }) -Compress))
} finally {
    if ($observer -ne $null) {
        $observer.Dispose()
    }
    if (
        $captureTopmost
    ) {
        [void][PrototypeActivatedFrameNative]::SetCaptureTopmost($window, $false)
    }
}
`;
}
