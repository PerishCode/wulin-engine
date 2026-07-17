import { fail, type Json, root } from "../canonical-runtime.ts";

const decoder = new TextDecoder();

type PrototypeKey = {
    key:
        | "D"
        | "E"
        | "Enter"
        | "Escape"
        | "F"
        | "OutOfRangeE"
        | "Shift"
        | "Space"
        | "W";
    virtualKey: number;
};

type PrototypeKeyTransition = PrototypeKey & {
    down: boolean;
};

type PrototypeWindowAction = "close" | "input" | "resume" | "suspend";

async function postPrototypeWindowAction(
    processId: number,
    keys: PrototypeKeyTransition[],
    requireVisible: boolean,
    action: PrototypeWindowAction = "input",
    delaysBeforeKeysMilliseconds: number[] = [],
    exitAfterLastMilliseconds = 0,
): Promise<Json> {
    if (!Number.isSafeInteger(processId) || processId <= 0) {
        fail(`prototype native input received invalid process id ${processId}`);
    }
    const requiresKeys = action === "input" || action === "suspend";
    if (requiresKeys === (keys.length === 0)) {
        fail(`prototype native ${action} action key shape diverged`);
    }
    const keyDelays = delaysBeforeKeysMilliseconds.length === 0
        ? keys.map(() => 0)
        : delaysBeforeKeysMilliseconds;
    if (
        keyDelays.length !== keys.length ||
        keyDelays.some((delay) =>
            !Number.isSafeInteger(delay) ||
            delay < 0 ||
            delay > 1_000
        ) ||
        !Number.isSafeInteger(exitAfterLastMilliseconds) ||
        exitAfterLastMilliseconds < 0 ||
        exitAfterLastMilliseconds > 1_000 ||
        (exitAfterLastMilliseconds > 0 && action !== "input")
    ) fail(`prototype native ${action} action delay diverged`);
    const nativeKeys = keys;
    const expectedMessages = requiresKeys
        ? [
            "WM_SETFOCUS",
            ...keys.map(({ key, down }) => `${down ? "WM_KEYDOWN" : "WM_KEYUP"}:${key}`),
            ...(action === "suspend" ? ["WM_KILLFOCUS"] : []),
            ...(exitAfterLastMilliseconds > 0 ? ["WM_KEYDOWN:Escape"] : []),
        ]
        : [action === "resume" ? "WM_SETFOCUS" : "WM_CLOSE"];
    const powershellKeys = keys.map(({ key, virtualKey, down }) =>
        `[ordered]@{ key = "${key}"; virtualKey = ${virtualKey}; down = ${
            down ? "$true" : "$false"
        } }`
    ).join(", ");
    const powershellDelays = keyDelays.join(", ");
    const script = String.raw`
$ErrorActionPreference = "Stop"
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

public static class PrototypeInputNative {
    [DllImport("user32.dll", EntryPoint = "FindWindowW", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern IntPtr FindWindow(string className, string windowName);

    [DllImport("user32.dll", EntryPoint = "GetWindowThreadProcessId", SetLastError = true)]
    public static extern uint GetWindowThreadProcessId(IntPtr window, out uint processId);

    [DllImport("user32.dll", EntryPoint = "IsWindowVisible")]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool IsWindowVisible(IntPtr window);

    [DllImport("user32.dll", EntryPoint = "PostMessageW", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool PostMessage(IntPtr window, uint message, UIntPtr wParam, IntPtr lParam);
}
'@

$expectedProcessId = ${processId}
$requireVisible = ${requireVisible ? "$true" : "$false"}
$action = "${action}"
$keys = @(${powershellKeys})
$keyDelays = @(${powershellDelays})
$exitAfterLastMilliseconds = ${exitAfterLastMilliseconds}
$postedMessages = [System.Collections.Generic.List[string]]::new()
$timer = [Diagnostics.Stopwatch]::StartNew()
$previousKeyTicks = $null
$lastKeyTicks = $null
$keyPostIntervalsMilliseconds = [System.Collections.Generic.List[double]]::new()
$exitIntervalMilliseconds = $null
$deadline = [DateTime]::UtcNow.AddSeconds(20)
$window = [IntPtr]::Zero
$windowProcessId = [uint32]0
do {
    $candidate = [PrototypeInputNative]::FindWindow(
        "WulinEnginePrototypeWindow",
        "Wulin Engine Prototype"
    )
    if ($candidate -ne [IntPtr]::Zero) {
        [void][PrototypeInputNative]::GetWindowThreadProcessId($candidate, [ref]$windowProcessId)
        if (
            $windowProcessId -eq $expectedProcessId -and
            (-not $requireVisible -or [PrototypeInputNative]::IsWindowVisible($candidate))
        ) {
            $window = $candidate
            break
        }
    }
    Start-Sleep -Milliseconds 1
} while ([DateTime]::UtcNow -lt $deadline)

if ($window -eq [IntPtr]::Zero) {
    throw "prototype window for process $expectedProcessId was not found"
}
$windowWasVisible = [PrototypeInputNative]::IsWindowVisible($window)
if ($action -eq "input" -or $action -eq "suspend") {
    if (-not [PrototypeInputNative]::PostMessage(
        $window,
        0x0007,
        [UIntPtr]::Zero,
        [IntPtr]::Zero
    )) {
        $code = [Runtime.InteropServices.Marshal]::GetLastWin32Error()
        throw "posting prototype focus activation failed with Win32 error $code"
    }
    $postedMessages.Add("WM_SETFOCUS")
}
$keyIndex = 0
foreach ($key in $keys) {
    $keyDelay = $keyDelays[$keyIndex]
    if ($keyDelay -gt 0) {
        Start-Sleep -Milliseconds $keyDelay
    }
    $message = if ($key.down) { 0x0100 } else { 0x0101 }
    if (-not [PrototypeInputNative]::PostMessage(
        $window,
        $message,
        [UIntPtr][uint32]$key.virtualKey,
        [IntPtr]1
    )) {
        $code = [Runtime.InteropServices.Marshal]::GetLastWin32Error()
        throw "posting prototype $($key.key) key down failed with Win32 error $code"
    }
    $messageTicks = $timer.ElapsedTicks
    if ($previousKeyTicks -ne $null) {
        $keyPostIntervalsMilliseconds.Add(
            ($messageTicks - $previousKeyTicks) * 1000.0 /
            [Diagnostics.Stopwatch]::Frequency
        )
    }
    $previousKeyTicks = $messageTicks
    $lastKeyTicks = $messageTicks
    $postedMessages.Add("$($message -eq 0x0100 ? 'WM_KEYDOWN' : 'WM_KEYUP'):$($key.key)")
    $keyIndex += 1
}
if ($exitAfterLastMilliseconds -gt 0) {
    Start-Sleep -Milliseconds $exitAfterLastMilliseconds
    if (-not [PrototypeInputNative]::PostMessage(
        $window,
        0x0100,
        [UIntPtr][uint32]0x1B,
        [IntPtr]1
    )) {
        $code = [Runtime.InteropServices.Marshal]::GetLastWin32Error()
        throw "posting prototype delayed Escape key down failed with Win32 error $code"
    }
    $exitIntervalMilliseconds = (
        ($timer.ElapsedTicks - $lastKeyTicks) * 1000.0 /
        [Diagnostics.Stopwatch]::Frequency
    )
    $postedMessages.Add("WM_KEYDOWN:Escape")
}
if ($action -eq "suspend") {
    if (-not [PrototypeInputNative]::PostMessage(
        $window,
        0x0008,
        [UIntPtr]::Zero,
        [IntPtr]::Zero
    )) {
        $code = [Runtime.InteropServices.Marshal]::GetLastWin32Error()
        throw "posting prototype focus suspension failed with Win32 error $code"
    }
    $postedMessages.Add("WM_KILLFOCUS")
} elseif ($action -eq "resume") {
    if (-not [PrototypeInputNative]::PostMessage(
        $window,
        0x0007,
        [UIntPtr]::Zero,
        [IntPtr]::Zero
    )) {
        $code = [Runtime.InteropServices.Marshal]::GetLastWin32Error()
        throw "posting prototype focus resume failed with Win32 error $code"
    }
    $postedMessages.Add("WM_SETFOCUS")
} elseif ($action -eq "close") {
    if (-not [PrototypeInputNative]::PostMessage(
        $window,
        0x0010,
        [UIntPtr]::Zero,
        [IntPtr]::Zero
    )) {
        $code = [Runtime.InteropServices.Marshal]::GetLastWin32Error()
        throw "posting prototype window close failed with Win32 error $code"
    }
    $postedMessages.Add("WM_CLOSE")
}

[Console]::Out.Write((ConvertTo-Json ([ordered]@{
    schema = "prototype-native-window-action-v3"
    action = $action
    processId = [int]$windowProcessId
    windowHandle = $window.ToInt64().ToString()
    activated = $action -ne "close"
    closeRequested = $action -eq "close"
    requiredVisible = $requireVisible
    windowWasVisible = $windowWasVisible
    keys = @($keys)
    messages = @($postedMessages)
    delaysBeforeKeysMilliseconds = @($keyDelays)
    keyPostIntervalsMilliseconds = @($keyPostIntervalsMilliseconds)
    exitAfterLastMilliseconds = $exitAfterLastMilliseconds
    exitIntervalMilliseconds = $exitIntervalMilliseconds
}) -Depth 4 -Compress))
`;
    const output = await new Deno.Command("pwsh", {
        args: ["-NoProfile", "-NonInteractive", "-Command", script],
        cwd: root,
        stdout: "piped",
        stderr: "piped",
    }).output();
    const stdout = decoder.decode(output.stdout).trim();
    const stderr = decoder.decode(output.stderr).trim();
    if (!output.success) {
        fail(`prototype native input failed with ${output.code}: ${stderr.slice(-4_096)}`);
    }
    const evidence = JSON.parse(stdout) as Json;
    if (
        evidence.schema !== "prototype-native-window-action-v3" ||
        evidence.action !== action ||
        evidence.processId !== processId ||
        evidence.activated !== (action !== "close") ||
        evidence.closeRequested !== (action === "close") ||
        evidence.requiredVisible !== requireVisible ||
        (requireVisible && evidence.windowWasVisible !== true) ||
        JSON.stringify(evidence.keys) !== JSON.stringify(nativeKeys) ||
        JSON.stringify(evidence.messages) !== JSON.stringify(expectedMessages) ||
        JSON.stringify(evidence.delaysBeforeKeysMilliseconds) !== JSON.stringify(keyDelays) ||
        !Array.isArray(evidence.keyPostIntervalsMilliseconds) ||
        evidence.keyPostIntervalsMilliseconds.length !== Math.max(0, keys.length - 1) ||
        evidence.keyPostIntervalsMilliseconds.some((interval, index) =>
            typeof interval !== "number" ||
            interval < keyDelays[index + 1]
        ) ||
        evidence.exitAfterLastMilliseconds !== exitAfterLastMilliseconds ||
        (exitAfterLastMilliseconds === 0
            ? evidence.exitIntervalMilliseconds !== null
            : typeof evidence.exitIntervalMilliseconds !== "number" ||
                evidence.exitIntervalMilliseconds < exitAfterLastMilliseconds)
    ) {
        fail(
            `prototype native window action evidence diverged: ${
                JSON.stringify({
                    action,
                    nativeKeys,
                    keyDelays,
                    exitAfterLastMilliseconds,
                    expectedMessages,
                    evidence,
                })
            }`,
        );
    }
    return evidence;
}

async function postPrototypeKeys(
    processId: number,
    keys: PrototypeKey[],
    requireVisible: boolean,
): Promise<Json> {
    return await postPrototypeWindowAction(
        processId,
        keys.map((key) => ({ ...key, down: true })),
        requireVisible,
    );
}

export async function holdPrototypeForwardKey(processId: number): Promise<Json> {
    return await postPrototypeKeys(processId, [{ key: "W", virtualKey: 0x57 }], false);
}

export async function holdRunForwardKeys(processId: number): Promise<Json> {
    return await postPrototypeKeys(
        processId,
        [{ key: "Shift", virtualKey: 0x10 }, { key: "W", virtualKey: 0x57 }],
        true,
    );
}

export async function holdOrbitForwardKeys(processId: number): Promise<Json> {
    return await postPrototypeKeys(
        processId,
        [{ key: "E", virtualKey: 0x45 }, { key: "W", virtualKey: 0x57 }],
        true,
    );
}

export async function postObserveActionFacing(processId: number): Promise<Json> {
    return await postPrototypeKeys(
        processId,
        [
            { key: "F", virtualKey: 0x46 },
            { key: "Enter", virtualKey: 0x0D },
            { key: "D", virtualKey: 0x44 },
        ],
        true,
    );
}

export async function postObserveActionSide(processId: number): Promise<Json> {
    return await postPrototypeKeys(
        processId,
        [
            { key: "F", virtualKey: 0x46 },
            { key: "Enter", virtualKey: 0x0D },
            { key: "W", virtualKey: 0x57 },
        ],
        true,
    );
}

export async function pressPrototypeEscape(processId: number): Promise<Json> {
    return await postPrototypeKeys(processId, [{ key: "Escape", virtualKey: 0x1B }], false);
}

export async function requestPrototypeWindowClose(processId: number): Promise<Json> {
    return await postPrototypeWindowAction(processId, [], true, "close");
}

export function nativeWindowCloseInvariant(evidence: Json, processId: number): Json {
    if (
        evidence.schema !== "prototype-native-window-action-v3" ||
        evidence.action !== "close" ||
        evidence.processId !== processId ||
        evidence.activated !== false ||
        evidence.closeRequested !== true ||
        evidence.requiredVisible !== true ||
        evidence.windowWasVisible !== true ||
        !Array.isArray(evidence.keys) ||
        evidence.keys.length !== 0 ||
        JSON.stringify(evidence.messages) !== JSON.stringify(["WM_CLOSE"])
    ) fail("prototype native window-close evidence diverged");
    return {
        exactProcessWindow: true,
        message: "WM_CLOSE",
        directDestroy: false,
    };
}

export async function suspendWithForward(processId: number): Promise<Json> {
    return await postPrototypeWindowAction(
        processId,
        [{ key: "W", virtualKey: 0x57, down: true }],
        true,
        "suspend",
    );
}

export async function resumePrototypeFocus(processId: number): Promise<Json> {
    return await postPrototypeWindowAction(processId, [], true, "resume");
}

export async function postPrototypeCapacityRejection(processId: number): Promise<Json> {
    return await postPrototypeWindowAction(
        processId,
        [
            { key: "D", virtualKey: 0x44, down: false },
            { key: "F", virtualKey: 0x46, down: false },
            { key: "F", virtualKey: 0x46, down: true },
            { key: "Enter", virtualKey: 0x0D, down: false },
            { key: "Enter", virtualKey: 0x0D, down: true },
        ],
        true,
    );
}

export async function pressPrototypeCameraClockwise(processId: number): Promise<Json> {
    return await postPrototypeKeys(processId, [{ key: "E", virtualKey: 0x45 }], true);
}

export async function pressPrototypeJump(processId: number): Promise<Json> {
    return await postPrototypeKeys(processId, [{ key: "Space", virtualKey: 0x20 }], true);
}

export async function repressJumpAndExit(processId: number): Promise<Json> {
    return await postPrototypeWindowAction(
        processId,
        [
            { key: "Space", virtualKey: 0x20, down: false },
            { key: "Space", virtualKey: 0x20, down: true },
            { key: "Escape", virtualKey: 0x1B, down: true },
        ],
        true,
        "input",
        [0, 0, 100],
    );
}

export async function postMidairSequence(processId: number): Promise<Json> {
    return await postPrototypeWindowAction(
        processId,
        [
            { key: "Space", virtualKey: 0x20, down: true },
            { key: "Space", virtualKey: 0x20, down: false },
            { key: "Space", virtualKey: 0x20, down: true },
            { key: "W", virtualKey: 0x57, down: true },
        ],
        true,
        "input",
        [0, 0, 200, 0],
        200,
    );
}

export async function postCameraRepeatSequence(processId: number): Promise<Json> {
    return await postPrototypeWindowAction(
        processId,
        [
            { key: "E", virtualKey: 0x45, down: true },
            { key: "W", virtualKey: 0x57, down: true },
        ],
        true,
        "input",
        [0, 0],
        200,
    );
}

export async function postInvalidAliasSequence(processId: number): Promise<Json> {
    return await postPrototypeWindowAction(
        processId,
        [
            { key: "OutOfRangeE", virtualKey: 0x145, down: true },
            { key: "W", virtualKey: 0x57, down: true },
        ],
        true,
        "input",
        [0, 0],
        200,
    );
}

export type StartupInput =
    | "camera-clockwise"
    | "camera-forward"
    | "forward"
    | "jump"
    | "observe-action-facing"
    | "observe-action-side"
    | "run-forward";

export async function applyStartupInput(
    processId: number,
    input?: StartupInput,
): Promise<Json | null> {
    switch (input) {
        case "camera-clockwise":
            return await pressPrototypeCameraClockwise(processId);
        case "camera-forward":
            return await holdOrbitForwardKeys(processId);
        case "forward":
            return await holdPrototypeForwardKey(processId);
        case "jump":
            return await pressPrototypeJump(processId);
        case "observe-action-facing":
            return await postObserveActionFacing(processId);
        case "observe-action-side":
            return await postObserveActionSide(processId);
        case "run-forward":
            return await holdRunForwardKeys(processId);
        case undefined:
            return null;
    }
}
