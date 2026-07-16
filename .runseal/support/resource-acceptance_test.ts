import {
    type ProcessSample,
    requireActivePlateau,
    requireRecoveredBaseline,
    resourceWarmSettled,
} from "./resource-acceptance.ts";

const MIB = 1024 * 1024;
const BASELINE: ProcessSample = { handleCount: 500, privateBytes: 400 * MIB, threadCount: 18 };
const LIMITS = { handleAllowance: 1, privateByteAllowance: 16 * MIB };
const WARM_POLICY = {
    minimumSampleCount: 4,
    stableTransitionCount: 2,
    privateByteAllowance: MIB,
};

Deno.test("resource warm settles after lazy allocation drains", () => {
    const settled = resourceWarmSettled(
        [
            { handleCount: 500, privateBytes: 400 * MIB, threadCount: 18 },
            { handleCount: 500, privateBytes: 420 * MIB, threadCount: 18 },
            { handleCount: 500, privateBytes: 414 * MIB, threadCount: 18 },
            { handleCount: 500, privateBytes: 414.5 * MIB, threadCount: 18 },
        ],
        WARM_POLICY,
    );
    if (!settled) throw new Error("settled workload shape was rejected");
});

Deno.test("resource warm requires the complete bounded evidence tail", () => {
    const samples = [
        { handleCount: 500, privateBytes: 400 * MIB, threadCount: 18 },
        { handleCount: 500, privateBytes: 400 * MIB, threadCount: 18 },
        { handleCount: 500, privateBytes: 400 * MIB, threadCount: 18 },
    ];
    if (resourceWarmSettled(samples, WARM_POLICY)) {
        throw new Error("short warm evidence unexpectedly settled");
    }
});

Deno.test("resource warm rejects a late worker initialization", () => {
    const settled = resourceWarmSettled(
        [
            { handleCount: 500, privateBytes: 400 * MIB, threadCount: 18 },
            { handleCount: 500, privateBytes: 400 * MIB, threadCount: 18 },
            { handleCount: 500, privateBytes: 400 * MIB, threadCount: 18 },
            { handleCount: 504, privateBytes: 400 * MIB, threadCount: 19 },
        ],
        WARM_POLICY,
    );
    if (settled) throw new Error("late worker initialization unexpectedly settled");
});

Deno.test("resource warm rejects late private growth above its sentinel", () => {
    const settled = resourceWarmSettled(
        [
            { handleCount: 500, privateBytes: 400 * MIB, threadCount: 18 },
            { handleCount: 500, privateBytes: 400 * MIB, threadCount: 18 },
            { handleCount: 500, privateBytes: 400 * MIB, threadCount: 18 },
            { handleCount: 500, privateBytes: 402 * MIB, threadCount: 18 },
        ],
        WARM_POLICY,
    );
    if (settled) throw new Error("late private growth unexpectedly settled");
});

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
