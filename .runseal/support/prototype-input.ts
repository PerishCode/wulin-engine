import { fail, type Json, root } from "./canonical-runtime.ts";

const decoder = new TextDecoder();

export async function holdPrototypeForwardKey(processId: number): Promise<Json> {
    if (!Number.isSafeInteger(processId) || processId <= 0) {
        fail(`prototype native input received invalid process id ${processId}`);
    }
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

    [DllImport("user32.dll", EntryPoint = "PostMessageW", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool PostMessage(IntPtr window, uint message, UIntPtr wParam, IntPtr lParam);
}
'@

$expectedProcessId = ${processId}
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
        if ($windowProcessId -eq $expectedProcessId) {
            $window = $candidate
            break
        }
    }
    Start-Sleep -Milliseconds 10
} while ([DateTime]::UtcNow -lt $deadline)

if ($window -eq [IntPtr]::Zero) {
    throw "prototype window for process $expectedProcessId was not found"
}
if (-not [PrototypeInputNative]::PostMessage($window, 0x0100, [UIntPtr]0x57, [IntPtr]1)) {
    $code = [Runtime.InteropServices.Marshal]::GetLastWin32Error()
    throw "posting prototype W key down failed with Win32 error $code"
}

[Console]::Out.Write((ConvertTo-Json ([ordered]@{
    schema = "prototype-native-key-v1"
    processId = [int]$windowProcessId
    windowHandle = $window.ToInt64().ToString()
    key = "W"
    virtualKey = 0x57
    down = $true
}) -Compress))
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
        evidence.schema !== "prototype-native-key-v1" ||
        evidence.processId !== processId ||
        evidence.key !== "W" ||
        evidence.virtualKey !== 0x57 ||
        evidence.down !== true
    ) fail("prototype native input evidence diverged");
    return evidence;
}
