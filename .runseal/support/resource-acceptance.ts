export type ProcessSample = {
    handleCount: number;
    privateBytes: number;
    threadCount: number;
};

export type ResourceLimits = {
    handleAllowance: number;
    privateByteAllowance: number;
};

export type ResourcePlateauEvidence = {
    baseline: ProcessSample;
    minimumHandleCount: number;
    peakHandleCount: number;
    final: ProcessSample;
    limits: ResourceLimits;
};

export function requireActivePlateau(
    baseline: ProcessSample,
    samples: ProcessSample[],
    limits: ResourceLimits,
): ResourcePlateauEvidence {
    if (samples.length === 0) throw new Error("resource plateau requires at least one sample");
    const handleCounts = samples.map((sample) => sample.handleCount);
    const peakHandleCount = Math.max(...handleCounts);
    const minimumHandleCount = Math.min(...handleCounts);
    const final = samples.at(-1) as ProcessSample;
    if (peakHandleCount > baseline.handleCount + limits.handleAllowance) {
        throw new Error(
            `active handles exceeded the baseline allowance: ${
                JSON.stringify({ baseline, limits, peakHandleCount, samples })
            }`,
        );
    }
    if (final.privateBytes > baseline.privateBytes + limits.privateByteAllowance) {
        throw new Error(
            `active private bytes exceeded the baseline allowance: ${
                JSON.stringify({ baseline, limits, final })
            }`,
        );
    }
    return { baseline, minimumHandleCount, peakHandleCount, final, limits };
}

export function requireRecoveredBaseline(
    baseline: ProcessSample,
    recovered: ProcessSample,
    privateByteAllowance: number,
): void {
    if (recovered.handleCount > baseline.handleCount) {
        throw new Error(
            `recovered handles exceeded the warmed baseline: ${
                JSON.stringify({ baseline, recovered })
            }`,
        );
    }
    if (recovered.privateBytes > baseline.privateBytes + privateByteAllowance) {
        throw new Error(
            `recovered private bytes exceeded the warmed baseline allowance: ${
                JSON.stringify({ baseline, recovered, privateByteAllowance })
            }`,
        );
    }
}
