type Fail = (message: string) => never;

export async function requireTransportAliasesRemoved(
    root: string,
    fail: Fail,
): Promise<void> {
    console.log("==> removed graceful Prototype transport report aliases");
    // deno-fmt-ignore
    const acceptance = await Deno.readTextFile(`${root}/.runseal/support/prototype/sessions/mod.ts`);
    const boundary = await Deno.readTextFile(
        `${root}/.runseal/support/prototype/boundary.ts`,
    );
    if (
        !acceptance.includes('exit.kind === "timeout"') ||
        !acceptance.includes("if (!status.success)") ||
        !acceptance.includes("trailingOutput.trim()") ||
        !acceptance.includes("if (stderrText)") ||
        !acceptance.includes("completion.reason !== expectedReason") ||
        acceptance.includes("exitCode:") ||
        acceptance.includes("stderr: stderrText") ||
        acceptance.includes("\n        exitReason,") ||
        acceptance.includes("outputValueCount") ||
        acceptance.includes("\n        trailingOutput,") ||
        acceptance.includes("exactlyTwoValues") ||
        boundary.includes('"exitCode"')
    ) {
        fail("guard: retired graceful transport report alias returned");
    }
}
