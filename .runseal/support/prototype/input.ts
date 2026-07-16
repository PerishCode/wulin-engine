import { fail, type Json, root } from "../canonical-runtime.ts";

const decoder = new TextDecoder();

type PrototypeKey = {
    key: "E" | "Enter" | "Escape" | "F" | "Shift" | "Space" | "W";
    virtualKey: number;
};

async function postPrototypeKeys(
    processId: number,
    keys: PrototypeKey[],
    requireVisible: boolean,
): Promise<Json> {
    if (!Number.isSafeInteger(processId) || processId <= 0) {
        fail(`prototype native input received invalid process id ${processId}`);
    }
    if (keys.length === 0) fail("prototype native input requires at least one key");
    const nativeKeys = keys.map(({ key, virtualKey }) => ({ key, virtualKey, down: true }));
    const powershellKeys = keys.map(({ key, virtualKey }) =>
        `[ordered]@{ key = "${key}"; virtualKey = ${virtualKey}; down = $true }`
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
if (-not [PrototypeInputNative]::PostMessage(
    $window,
    0x0007,
    [UIntPtr]::Zero,
    [IntPtr]::Zero
)) {
    $code = [Runtime.InteropServices.Marshal]::GetLastWin32Error()
    throw "posting prototype focus activation failed with Win32 error $code"
}
foreach ($key in $keys) {
    if (-not [PrototypeInputNative]::PostMessage(
        $window,
        0x0100,
        [UIntPtr][uint32]$key.virtualKey,
        [IntPtr]1
    )) {
        $code = [Runtime.InteropServices.Marshal]::GetLastWin32Error()
        throw "posting prototype $($key.key) key down failed with Win32 error $code"
    }
}

[Console]::Out.Write((ConvertTo-Json ([ordered]@{
    schema = "prototype-native-input-v5"
    processId = [int]$windowProcessId
    windowHandle = $window.ToInt64().ToString()
    activated = $true
    requiredVisible = $requireVisible
    windowVisible = [PrototypeInputNative]::IsWindowVisible($window)
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
        evidence.schema !== "prototype-native-input-v5" ||
        evidence.processId !== processId ||
        evidence.activated !== true ||
        evidence.requiredVisible !== requireVisible ||
        (requireVisible && evidence.windowVisible !== true) ||
        JSON.stringify(evidence.keys) !== JSON.stringify(nativeKeys)
    ) fail("prototype native input evidence diverged");
    return evidence;
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

export async function postObserveActionForward(processId: number): Promise<Json> {
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
    | "observe-action-forward"
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
        case "observe-action-forward":
            return await postObserveActionForward(processId);
        case "run-forward":
            return await holdRunForwardKeys(processId);
        case undefined:
            return null;
    }
}
