import { fail, type Json } from "../../canonical-runtime.ts";
import { startPreparedWindowAction } from "./prepared.ts";

export type PrototypeKey = {
    key:
        | "A"
        | "D"
        | "E"
        | "Enter"
        | "Escape"
        | "F"
        | "OutOfRangeE"
        | "Q"
        | "S"
        | "Shift"
        | "Space"
        | "W";
    virtualKey: number;
};

export type PrototypeKeyTransition = PrototypeKey & {
    down: boolean;
};

export type PrototypeWindowAction = "close" | "input" | "resume" | "suspend";

export type PreparedPrototypeWindowAction = {
    evidence: Promise<Json>;
};

export async function preparePrototypeWindowAction(
    processId: number | null,
    keys: PrototypeKeyTransition[],
    requireVisible: boolean,
    action: PrototypeWindowAction = "input",
    delaysBeforeKeysMilliseconds: number[] = [],
    exitAfterLastMilliseconds = 0,
    atomicBatch = false,
): Promise<PreparedPrototypeWindowAction> {
    if (
        processId !== null &&
        (!Number.isSafeInteger(processId) || processId <= 0)
    ) {
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
        (exitAfterLastMilliseconds > 0 && action !== "input") ||
        (atomicBatch &&
            ((action !== "input" && action !== "suspend") ||
                keys.length < (action === "input" ? 2 : 1) ||
                keyDelays.some((delay) => delay !== 0)))
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
using System.Diagnostics;
using System.Runtime.InteropServices;

public sealed class PrototypeInputBatchResult {
    public uint ThreadId { get; set; }
    public long[] KeyTicks { get; set; }
}

public static class PrototypeInputNative {
    private const uint ThreadSuspendResume = 0x0002;

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

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern IntPtr OpenThread(uint desiredAccess, bool inheritHandle, uint threadId);

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern uint SuspendThread(IntPtr thread);

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern uint ResumeThread(IntPtr thread);

    [DllImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool CloseHandle(IntPtr handle);

    public static PrototypeInputBatchResult PostAtomicInputBatch(
        IntPtr window,
        uint[] virtualKeys,
        bool[] downs,
        bool suspendAfterInput,
        Stopwatch timer
    ) {
        if (
            virtualKeys.Length != downs.Length ||
            virtualKeys.Length < 1 ||
            (!suspendAfterInput && virtualKeys.Length < 2)
        ) {
            throw new InvalidOperationException("prototype atomic input batch shape diverged");
        }
        uint processId;
        uint threadId = GetWindowThreadProcessId(window, out processId);
        IntPtr thread = OpenThread(ThreadSuspendResume, false, threadId);
        if (thread == IntPtr.Zero) {
            throw new InvalidOperationException(
                "opening prototype window thread failed with Win32 error " +
                Marshal.GetLastWin32Error()
            );
        }
        if (SuspendThread(thread) == uint.MaxValue) {
            CloseHandle(thread);
            throw new InvalidOperationException(
                "suspending prototype window thread failed with Win32 error " +
                Marshal.GetLastWin32Error()
            );
        }
        try {
            if (!PostMessage(window, 0x0007, UIntPtr.Zero, IntPtr.Zero)) {
                throw new InvalidOperationException(
                    "posting prototype focus activation failed with Win32 error " +
                    Marshal.GetLastWin32Error()
                );
            }
            long[] ticks = new long[virtualKeys.Length];
            for (int index = 0; index < virtualKeys.Length; index++) {
                uint message = downs[index] ? 0x0100u : 0x0101u;
                if (!PostMessage(
                    window,
                    message,
                    new UIntPtr(virtualKeys[index]),
                    new IntPtr(1)
                )) {
                    throw new InvalidOperationException(
                        "posting prototype atomic key failed with Win32 error " +
                        Marshal.GetLastWin32Error()
                    );
                }
                ticks[index] = timer.ElapsedTicks;
            }
            if (suspendAfterInput && !PostMessage(
                window,
                0x0008u,
                UIntPtr.Zero,
                IntPtr.Zero
            )) {
                throw new InvalidOperationException(
                    "posting prototype atomic focus suspension failed with Win32 error " +
                    Marshal.GetLastWin32Error()
                );
            }
            return new PrototypeInputBatchResult { ThreadId = threadId, KeyTicks = ticks };
        } finally {
            uint resumeResult = ResumeThread(thread);
            bool closeResult = CloseHandle(thread);
            if (resumeResult == uint.MaxValue || !closeResult) {
                throw new InvalidOperationException(
                    "restoring prototype window thread failed with Win32 error " +
                    Marshal.GetLastWin32Error()
                );
            }
        }
    }
}
'@

$expectedProcessId = ${processId ?? 0}
$requireVisible = ${requireVisible ? "$true" : "$false"}
$action = "${action}"
$keys = @(${powershellKeys})
$keyDelays = @(${powershellDelays})
$exitAfterLastMilliseconds = ${exitAfterLastMilliseconds}
$atomicBatch = ${atomicBatch ? "$true" : "$false"}
$postedMessages = [System.Collections.Generic.List[string]]::new()
$timer = [Diagnostics.Stopwatch]::StartNew()
$previousKeyTicks = $null
$lastKeyTicks = $null
$keyPostIntervalsMilliseconds = [System.Collections.Generic.List[double]]::new()
$exitIntervalMilliseconds = $null
$batchThreadId = $null
$batchSpanMilliseconds = $null
$deadline = [DateTime]::UtcNow.AddSeconds(20)
$window = [IntPtr]::Zero
$windowProcessId = [uint32]0
[Console]::Out.WriteLine("prototype-native-helper-ready-v1")
[Console]::Out.Flush()
do {
    $candidate = [PrototypeInputNative]::FindWindow(
        "WulinEnginePrototypeWindow",
        "Wulin Engine Prototype"
    )
    if ($candidate -ne [IntPtr]::Zero) {
        [void][PrototypeInputNative]::GetWindowThreadProcessId($candidate, [ref]$windowProcessId)
        if (
            ($expectedProcessId -eq 0 -or $windowProcessId -eq $expectedProcessId) -and
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
if ($atomicBatch) {
    [uint32[]]$batchKeys = @($keys | ForEach-Object { [uint32]$_.virtualKey })
    [bool[]]$batchDowns = @($keys | ForEach-Object { [bool]$_.down })
    $batch = [PrototypeInputNative]::PostAtomicInputBatch(
        $window,
        $batchKeys,
        $batchDowns,
        $action -eq "suspend",
        $timer
    )
    $batchThreadId = [uint32]$batch.ThreadId
    $postedMessages.Add("WM_SETFOCUS")
    for ($keyIndex = 0; $keyIndex -lt $keys.Count; $keyIndex += 1) {
        $key = $keys[$keyIndex]
        $messageTicks = $batch.KeyTicks[$keyIndex]
        if ($previousKeyTicks -ne $null) {
            $keyPostIntervalsMilliseconds.Add(
                ($messageTicks - $previousKeyTicks) * 1000.0 /
                [Diagnostics.Stopwatch]::Frequency
            )
        }
        $previousKeyTicks = $messageTicks
        $lastKeyTicks = $messageTicks
        $postedMessages.Add("$($key.down ? 'WM_KEYDOWN' : 'WM_KEYUP'):$($key.key)")
    }
    if ($action -eq "suspend") {
        $postedMessages.Add("WM_KILLFOCUS")
    }
    $batchSpanMilliseconds = (
        ($batch.KeyTicks[$batch.KeyTicks.Length - 1] - $batch.KeyTicks[0]) *
        1000.0 / [Diagnostics.Stopwatch]::Frequency
    )
} else {
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
}
if ($exitAfterLastMilliseconds -gt 0) {
    $exitDeadlineTicks = $lastKeyTicks + [long][Math]::Ceiling(
        $exitAfterLastMilliseconds * [Diagnostics.Stopwatch]::Frequency / 1000.0
    )
    while ($timer.ElapsedTicks -lt $exitDeadlineTicks) {
        $remainingTicks = $exitDeadlineTicks - $timer.ElapsedTicks
        $remainingMilliseconds = [Math]::Floor(
            $remainingTicks * 1000.0 / [Diagnostics.Stopwatch]::Frequency
        )
        if ($remainingMilliseconds -gt 1) {
            Start-Sleep -Milliseconds ([int]($remainingMilliseconds - 1))
        } else {
            [Threading.Thread]::Yield() | Out-Null
        }
    }
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
if ($action -eq "suspend" -and -not $atomicBatch) {
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
    atomicBatch = $atomicBatch
    batchThreadId = $batchThreadId
    batchSpanMilliseconds = $batchSpanMilliseconds
}) -Depth 4 -Compress))
`;
    return await startPreparedWindowAction(
        script,
        {
            action,
            processId,
            nativeKeys,
            requireVisible,
            keyDelays,
            exitAfterLastMilliseconds,
            atomicBatch,
            expectedMessages,
        },
    );
}

export async function postPrototypeWindowAction(
    processId: number | null,
    keys: PrototypeKeyTransition[],
    requireVisible: boolean,
    action: PrototypeWindowAction = "input",
    delaysBeforeKeysMilliseconds: number[] = [],
    exitAfterLastMilliseconds = 0,
    atomicBatch = false,
): Promise<Json> {
    const prepared = await preparePrototypeWindowAction(
        processId,
        keys,
        requireVisible,
        action,
        delaysBeforeKeysMilliseconds,
        exitAfterLastMilliseconds,
        atomicBatch,
    );
    return await prepared.evidence;
}

export async function postPrototypeKeys(
    processId: number | null,
    keys: PrototypeKey[],
    requireVisible: boolean,
    atomicBatch = false,
): Promise<Json> {
    return await postPrototypeWindowAction(
        processId,
        keys.map((key) => ({ ...key, down: true })),
        requireVisible,
        "input",
        [],
        0,
        atomicBatch,
    );
}
