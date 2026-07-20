import { fail, root } from "../../canonical-runtime.ts";

const INTERACTIVE_DESKTOP_READY = "prototype-interactive-desktop-ready-v1";

export async function requireActivatedFrameDesktop(): Promise<void> {
    const script = String.raw`
$ErrorActionPreference = "Stop"
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

public static class PrototypeInteractiveDesktopNative {
    [DllImport("user32.dll", EntryPoint = "OpenInputDesktop", SetLastError = true)]
    public static extern IntPtr OpenInputDesktop(
        uint flags,
        [MarshalAs(UnmanagedType.Bool)] bool inherit,
        uint desiredAccess
    );

    [DllImport("user32.dll", EntryPoint = "CloseDesktop", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool CloseDesktop(IntPtr desktop);
}
'@
$desktop = [PrototypeInteractiveDesktopNative]::OpenInputDesktop(0, $false, 0x0100)
if ($desktop -eq [IntPtr]::Zero) {
    $code = [Runtime.InteropServices.Marshal]::GetLastWin32Error()
    throw "opening the interactive input desktop failed with Win32 error $code"
}
try {
    [Console]::Out.Write("${INTERACTIVE_DESKTOP_READY}")
} finally {
    if (-not [PrototypeInteractiveDesktopNative]::CloseDesktop($desktop)) {
        $code = [Runtime.InteropServices.Marshal]::GetLastWin32Error()
        throw "closing the interactive input desktop failed with Win32 error $code"
    }
}
`;
    const output = await new Deno.Command("pwsh", {
        args: ["-NoProfile", "-NonInteractive", "-Command", script],
        cwd: root,
        stdout: "piped",
        stderr: "piped",
    }).output();
    const stdout = new TextDecoder().decode(output.stdout).trim();
    const stderr = new TextDecoder().decode(output.stderr).trim();
    if (!output.success || stdout !== INTERACTIVE_DESKTOP_READY || stderr !== "") {
        fail(
            `prototype Activated-frame acceptance requires an interactive desktop; ` +
                `status=${output.code} stdout=${stdout.slice(-1_024)} ` +
                `stderr=${stderr.slice(-4_096)}`,
        );
    }
}
