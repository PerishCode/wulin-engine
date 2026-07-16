import {
    type ProcessSample,
    requireActivePlateau,
    requireRecoveredBaseline,
} from "./resource-acceptance.ts";

const MIB = 1024 * 1024;
const BASELINE: ProcessSample = { handleCount: 500, privateBytes: 400 * MIB, threadCount: 18 };
const LIMITS = { handleAllowance: 1, privateByteAllowance: 16 * MIB };

Deno.test("active resource plateau accepts bounded noise", () => {
    const evidence = requireActivePlateau(
        BASELINE,
        [
            { handleCount: 501, privateBytes: 410 * MIB, threadCount: 18 },
            { handleCount: 500, privateBytes: 408 * MIB, threadCount: 18 },
        ],
        LIMITS,
    );
    if (evidence.peakHandleCount !== 501 || evidence.minimumHandleCount !== 500) {
        throw new Error("resource plateau summary diverged");
    }
});

Deno.test("active resource plateau rejects an early handle leak", () => {
    expectFailure(() =>
        requireActivePlateau(
            BASELINE,
            [
                { handleCount: 502, privateBytes: 400 * MIB, threadCount: 18 },
                { handleCount: 502, privateBytes: 400 * MIB, threadCount: 18 },
            ],
            LIMITS,
        )
    );
});

Deno.test("active resource plateau rejects retained private growth", () => {
    expectFailure(() =>
        requireActivePlateau(
            BASELINE,
            [{ handleCount: 500, privateBytes: 417 * MIB, threadCount: 18 }],
            LIMITS,
        )
    );
});

Deno.test("resource recovery rejects delayed handle retention", () => {
    expectFailure(() =>
        requireRecoveredBaseline(
            BASELINE,
            { handleCount: 501, privateBytes: 400 * MIB, threadCount: 18 },
            16 * MIB,
        )
    );
});

Deno.test("resource recovery accepts a lower settled handle count", () => {
    requireRecoveredBaseline(
        BASELINE,
        { handleCount: 490, privateBytes: 405 * MIB, threadCount: 18 },
        16 * MIB,
    );
});

function expectFailure(operation: () => void): void {
    let failed = false;
    try {
        operation();
    } catch {
        failed = true;
    }
    if (!failed) throw new Error("mutated resource evidence unexpectedly passed");
}
