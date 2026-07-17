import { fail, type Json, root } from "../canonical-runtime.ts";

const decoder = new TextDecoder();

type PrototypeKey = {
    key: "D" | "E" | "Enter" | "Escape" | "F" | "Shift" | "Space" | "W";
    virtualKey: number;
};

type PrototypeKeyTransition = PrototypeKey & {
    down: boolean;
};

async function postPrototypeWindowAction(
    processId: number,
    keys: PrototypeKeyTransition[],
    requireVisible: boolean,
    closeWindow = false,
): Promise<Json> {
    if (!Number.isSafeInteger(processId) || processId <= 0) {
        fail(`prototype native input received invalid process id ${processId}`);
    }
    if (keys.length === 0 && !closeWindow) {
        fail("prototype native window action requires input or close");
    }
    const nativeKeys = keys;
    const powershellKeys = keys.map(({ key, virtualKey, down }) =>
        `[ordered]@{ key = "${key}"; virtualKey = ${virtualKey}; down = ${
            down ? "$true" : "$false"
        } }`
    ).join(", ");
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
$closeWindow = ${closeWindow ? "$true" : "$false"}
$keys = @(${powershellKeys})
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
    Start-Sleep -Milliseconds 10
} while ([DateTime]::UtcNow -lt $deadline)

if ($window -eq [IntPtr]::Zero) {
    throw "prototype window for process $expectedProcessId was not found"
}
$windowWasVisible = [PrototypeInputNative]::IsWindowVisible($window)
if ($keys.Count -gt 0) {
    if (-not [PrototypeInputNative]::PostMessage(
        $window,
        0x0007,
        [UIntPtr]::Zero,
        [IntPtr]::Zero
    )) {
        $code = [Runtime.InteropServices.Marshal]::GetLastWin32Error()
        throw "posting prototype focus activation failed with Win32 error $code"
    }
}
foreach ($key in $keys) {
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
}
if ($closeWindow) {
    if (-not [PrototypeInputNative]::PostMessage(
        $window,
        0x0010,
        [UIntPtr]::Zero,
        [IntPtr]::Zero
    )) {
        $code = [Runtime.InteropServices.Marshal]::GetLastWin32Error()
        throw "posting prototype window close failed with Win32 error $code"
    }
}

[Console]::Out.Write((ConvertTo-Json ([ordered]@{
    schema = "prototype-native-window-action-v1"
    action = if ($closeWindow) { "close" } else { "input" }
    processId = [int]$windowProcessId
    windowHandle = $window.ToInt64().ToString()
    activated = $keys.Count -gt 0
    closeRequested = $closeWindow
    requiredVisible = $requireVisible
    windowWasVisible = $windowWasVisible
    keys = @($keys)
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
        evidence.schema !== "prototype-native-window-action-v1" ||
        evidence.action !== (closeWindow ? "close" : "input") ||
        evidence.processId !== processId ||
        evidence.activated !== (keys.length > 0) ||
        evidence.closeRequested !== closeWindow ||
        evidence.requiredVisible !== requireVisible ||
        (requireVisible && evidence.windowWasVisible !== true) ||
        JSON.stringify(evidence.keys) !== JSON.stringify(nativeKeys)
    ) fail("prototype native window action evidence diverged");
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
    return await postPrototypeWindowAction(processId, [], true, true);
}

export function nativeWindowCloseInvariant(evidence: Json, processId: number): Json {
    if (
        evidence.schema !== "prototype-native-window-action-v1" ||
        evidence.action !== "close" ||
        evidence.processId !== processId ||
        evidence.activated !== false ||
        evidence.closeRequested !== true ||
        evidence.requiredVisible !== true ||
        evidence.windowWasVisible !== true ||
        !Array.isArray(evidence.keys) ||
        evidence.keys.length !== 0
    ) fail("prototype native window-close evidence diverged");
    return {
        exactProcessWindow: true,
        message: "WM_CLOSE",
        directDestroy: false,
    };
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
