import { fail, type Json, root } from "../../canonical-runtime.ts";
import { ACTIVATED_FRAME_COMPLETION_READY } from "./frame_completion_contract.ts";
import { activatedFrameCompletionScript } from "./frame_completion_script.ts";
import { completeActivatedFrameCompletion } from "./frame_completion_validation.ts";

export type PreparedActivatedFrameCompletion = {
    cancel: () => void;
    evidence: Promise<Json>;
};

export async function prepareActivatedFrameCompletion(
    processId: number,
): Promise<PreparedActivatedFrameCompletion> {
    if (!Number.isSafeInteger(processId) || processId <= 0) {
        fail(`prototype activated frame observer received invalid process id ${processId}`);
    }
    const script = String.raw`
$ErrorActionPreference = "Stop"
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Text;

public sealed class PrototypeActivatedFrameObserver : IDisposable {
    private const uint DibRgbColors = 0;
    private const uint BiRgb = 0;
    private const uint PrintClientFullContent = 0x00000003;

    [StructLayout(LayoutKind.Sequential)]
    private struct Rect {
        public int Left;
        public int Top;
        public int Right;
        public int Bottom;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct BitmapInfoHeader {
        public uint Size;
        public int Width;
        public int Height;
        public ushort Planes;
        public ushort BitCount;
        public uint Compression;
        public uint SizeImage;
        public int XPelsPerMeter;
        public int YPelsPerMeter;
        public uint ClrUsed;
        public uint ClrImportant;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct RgbQuad {
        public byte Blue;
        public byte Green;
        public byte Red;
        public byte Reserved;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct BitmapInfo {
        public BitmapInfoHeader Header;
        public RgbQuad Color;
    }

    [DllImport("user32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool GetClientRect(IntPtr window, out Rect rect);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern IntPtr GetDC(IntPtr window);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern int ReleaseDC(IntPtr window, IntPtr deviceContext);

    [DllImport("gdi32.dll", SetLastError = true)]
    private static extern IntPtr CreateCompatibleDC(IntPtr deviceContext);

    [DllImport("gdi32.dll", SetLastError = true)]
    private static extern IntPtr CreateDIBSection(
        IntPtr deviceContext,
        ref BitmapInfo info,
        uint usage,
        out IntPtr bits,
        IntPtr section,
        uint offset
    );

    [DllImport("gdi32.dll", SetLastError = true)]
    private static extern IntPtr SelectObject(IntPtr deviceContext, IntPtr value);

    [DllImport("user32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool PrintWindow(IntPtr window, IntPtr destination, uint flags);

    [DllImport("gdi32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool GdiFlush();

    [DllImport("dwmapi.dll")]
    private static extern int DwmFlush();

    [DllImport("gdi32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool DeleteObject(IntPtr value);

    [DllImport("gdi32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool DeleteDC(IntPtr deviceContext);

    private readonly int width;
    private readonly int height;
    private readonly IntPtr window;
    private readonly int[] pixels;
    private IntPtr memory;
    private IntPtr bitmap;
    private IntPtr previous;
    private IntPtr bits;
    private bool disposed;

    public int LastNonBlackPixelCount { get; private set; }
    public int LastMaxGreen { get; private set; }
    public int LastBestRed { get; private set; }
    public int LastBestGreen { get; private set; }
    public int LastBestBlue { get; private set; }
    public int LastGreenDominance { get; private set; }

    public PrototypeActivatedFrameObserver(IntPtr window) {
        this.window = window;
        Rect rect;
        if (!GetClientRect(window, out rect)) {
            throw NativeFailure("reading prototype client rectangle");
        }
        width = rect.Right - rect.Left;
        height = rect.Bottom - rect.Top;
        if (width <= 0 || height <= 0) {
            throw new InvalidOperationException("prototype client rectangle is empty");
        }
        pixels = new int[checked(width * height)];
        IntPtr setupScreen = GetDC(window);
        if (setupScreen == IntPtr.Zero) {
            throw NativeFailure("opening prototype setup capture context");
        }
        try {
            memory = CreateCompatibleDC(setupScreen);
            if (memory == IntPtr.Zero) {
                throw NativeFailure("creating prototype capture context");
            }
            BitmapInfo info = new BitmapInfo {
                Header = new BitmapInfoHeader {
                    Size = (uint)Marshal.SizeOf<BitmapInfoHeader>(),
                    Width = width,
                    Height = -height,
                    Planes = 1,
                    BitCount = 32,
                    Compression = BiRgb,
                    SizeImage = checked((uint)(pixels.Length * 4)),
                },
            };
            bitmap = CreateDIBSection(
                setupScreen,
                ref info,
                DibRgbColors,
                out bits,
                IntPtr.Zero,
                0
            );
            if (bitmap == IntPtr.Zero || bits == IntPtr.Zero) {
                throw NativeFailure("creating prototype capture pixels");
            }
            previous = SelectObject(memory, bitmap);
            if (previous == IntPtr.Zero) {
                throw NativeFailure("selecting prototype capture pixels");
            }
        } catch {
            Dispose();
            throw;
        } finally {
            ReleaseDC(window, setupScreen);
        }
    }

    public int CountActivatedPixels() {
        if (disposed) {
            throw new ObjectDisposedException(nameof(PrototypeActivatedFrameObserver));
        }
        int compositionResult = DwmFlush();
        if (compositionResult < 0) {
            throw new InvalidOperationException(
                "synchronizing desktop composition failed with HRESULT 0x" +
                compositionResult.ToString("X8")
            );
        }
        if (!PrintWindow(window, memory, PrintClientFullContent)) {
            throw NativeFailure("rendering prototype client pixels");
        }
        if (!GdiFlush()) {
            throw NativeFailure("flushing prototype capture pixels");
        }
        Marshal.Copy(bits, pixels, 0, pixels.Length);
        int count = 0;
        int nonBlack = 0;
        int maxGreen = 0;
        int bestRed = 0;
        int bestGreen = 0;
        int bestBlue = 0;
        int bestDominance = int.MinValue;
        foreach (int pixel in pixels) {
            int blue = pixel & 0xff;
            int green = (pixel >> 8) & 0xff;
            int red = (pixel >> 16) & 0xff;
            if (red != 0 || green != 0 || blue != 0) {
                nonBlack += 1;
            }
            maxGreen = Math.Max(maxGreen, green);
            int dominance = green - Math.Max(red, blue);
            if (dominance > bestDominance) {
                bestDominance = dominance;
                bestRed = red;
                bestGreen = green;
                bestBlue = blue;
            }
            if (
                green >= 176 &&
                red <= 104 &&
                blue >= 48 &&
                blue <= 144 &&
                green - red >= 72 &&
                green - blue >= 40
            ) {
                count += 1;
            }
        }
        LastNonBlackPixelCount = nonBlack;
        LastMaxGreen = maxGreen;
        LastBestRed = bestRed;
        LastBestGreen = bestGreen;
        LastBestBlue = bestBlue;
        LastGreenDominance = bestDominance;
        return count;
    }

    public void Dispose() {
        if (disposed) {
            return;
        }
        disposed = true;
        if (memory != IntPtr.Zero && previous != IntPtr.Zero) {
            SelectObject(memory, previous);
            previous = IntPtr.Zero;
        }
        if (bitmap != IntPtr.Zero) {
            DeleteObject(bitmap);
            bitmap = IntPtr.Zero;
        }
        if (memory != IntPtr.Zero) {
            DeleteDC(memory);
            memory = IntPtr.Zero;
        }
        bits = IntPtr.Zero;
    }

    private static InvalidOperationException NativeFailure(string action) {
        return new InvalidOperationException(
            action + " failed with Win32 error " + Marshal.GetLastWin32Error()
        );
    }
}

public static class PrototypeActivatedFrameNative {
    private static readonly IntPtr NotTopmost = new IntPtr(-2);
    private static readonly IntPtr Topmost = new IntPtr(-1);
    private const uint NoActivateNoMoveNoSize = 0x0013;
    private const uint RootAncestor = 2;

    [StructLayout(LayoutKind.Sequential)]
    private struct CapturePoint {
        public int X;
        public int Y;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct CaptureRect {
        public int Left;
        public int Top;
        public int Right;
        public int Bottom;
    }

    [DllImport("user32.dll", EntryPoint = "FindWindowW", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern IntPtr FindWindow(string className, string windowName);

    [DllImport("user32.dll", EntryPoint = "GetWindowThreadProcessId", SetLastError = true)]
    public static extern uint GetWindowThreadProcessId(IntPtr window, out uint processId);

    [DllImport("user32.dll", EntryPoint = "GetClientRect", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool GetClientRect(IntPtr window, out CaptureRect rect);

    [DllImport("user32.dll", EntryPoint = "ClientToScreen", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool ClientToScreen(IntPtr window, ref CapturePoint point);

    [DllImport("user32.dll", EntryPoint = "WindowFromPoint")]
    private static extern IntPtr WindowFromPoint(CapturePoint point);

    [DllImport("user32.dll", EntryPoint = "GetAncestor")]
    private static extern IntPtr GetAncestor(IntPtr window, uint flags);

    [DllImport("user32.dll", EntryPoint = "GetClassNameW", CharSet = CharSet.Unicode)]
    private static extern int GetClassName(IntPtr window, StringBuilder value, int maximum);

    [DllImport("user32.dll", EntryPoint = "GetWindowTextW", CharSet = CharSet.Unicode)]
    private static extern int GetWindowText(IntPtr window, StringBuilder value, int maximum);

    [DllImport("user32.dll", EntryPoint = "IsWindowVisible")]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool IsWindowVisible(IntPtr window);

    [DllImport("user32.dll", EntryPoint = "PostMessageW", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool PostMessage(IntPtr window, uint message, UIntPtr wParam, IntPtr lParam);

    [DllImport("user32.dll", EntryPoint = "SetWindowPos", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool SetWindowPos(
        IntPtr window,
        IntPtr insertAfter,
        int x,
        int y,
        int width,
        int height,
        uint flags
    );

    public static bool SetCaptureTopmost(IntPtr window, bool enabled) {
        return SetWindowPos(
            window,
            enabled ? Topmost : NotTopmost,
            0,
            0,
            0,
            0,
            NoActivateNoMoveNoSize
        );
    }

    public static string CaptureOwner(IntPtr expectedWindow) {
        CaptureRect rect;
        if (!GetClientRect(expectedWindow, out rect)) {
            return "client-rect-error=" + Marshal.GetLastWin32Error();
        }
        CapturePoint center = new CapturePoint {
            X = (rect.Right - rect.Left) / 2,
            Y = (rect.Bottom - rect.Top) / 2
        };
        if (!ClientToScreen(expectedWindow, ref center)) {
            return "client-origin-error=" + Marshal.GetLastWin32Error();
        }
        IntPtr hit = WindowFromPoint(center);
        IntPtr root = GetAncestor(hit, RootAncestor);
        if (root == IntPtr.Zero) {
            root = hit;
        }
        uint processId;
        GetWindowThreadProcessId(root, out processId);
        StringBuilder className = new StringBuilder(256);
        StringBuilder title = new StringBuilder(512);
        GetClassName(root, className, className.Capacity);
        GetWindowText(root, title, title.Capacity);
        return "handle=" + root.ToInt64() + " processId=" + processId +
            " class=" + className + " title=" + title;
    }
}
'@
${activatedFrameCompletionScript(processId)}
`;
    const child = new Deno.Command("pwsh", {
        args: ["-NoProfile", "-NonInteractive", "-Command", script],
        cwd: root,
        stdout: "piped",
        stderr: "piped",
    }).spawn();
    const stderr = new Response(child.stderr).text();
    const reader = child.stdout
        .pipeThrough(new TextDecoderStream())
        .getReader();
    let readyTimeout: ReturnType<typeof setTimeout> | undefined;
    const marker = await Promise.race([
        observerReadyLine(reader).then((value) => ({ kind: "marker" as const, value })),
        new Promise<{ kind: "timeout" }>((resolve) =>
            readyTimeout = setTimeout(() => resolve({ kind: "timeout" }), 20_000)
        ),
    ]);
    if (readyTimeout !== undefined) clearTimeout(readyTimeout);
    if (marker.kind === "timeout") {
        try {
            child.kill();
        } catch {
            // The observer may have failed immediately before the timeout was observed.
        }
        await child.status;
        fail("prototype Activated frame observer did not become ready");
    }
    if (marker.value === null) {
        const status = await child.status;
        fail(
            `prototype Activated frame observer exited before readiness; ` +
                `status=${status.code} stderr=${(await stderr).trim().slice(-4_096)}`,
        );
    }
    if (marker.value !== ACTIVATED_FRAME_COMPLETION_READY) {
        try {
            child.kill();
        } catch {
            // The observer may already have exited with its diagnostic.
        }
        const status = await child.status;
        fail(
            `prototype Activated frame observer emitted invalid readiness ${marker.value}; ` +
                `status=${status.code} stderr=${(await stderr).trim().slice(-4_096)}`,
        );
    }
    return {
        cancel: () => {
            try {
                child.kill();
            } catch {
                // The observer may already have completed.
            }
        },
        evidence: completeActivatedFrameCompletion(child, reader, stderr, processId),
    };
}

async function observerReadyLine(
    reader: ReadableStreamDefaultReader<string>,
): Promise<string | null> {
    let value = "";
    while (true) {
        const chunk = await reader.read();
        if (chunk.done) return null;
        value += chunk.value;
        const newline = value.indexOf("\n");
        if (newline < 0) continue;
        if (value.slice(newline + 1).trim() !== "") {
            fail("prototype Activated frame observer emitted evidence with its readiness marker");
        }
        return value.slice(0, newline).trimEnd();
    }
}
