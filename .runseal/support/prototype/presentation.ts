import { fail, type Json, number } from "../canonical-runtime.ts";

export function presentationInvariant(
    value: Json,
    clip: number,
    yawQ16: number,
    label: string,
): Json {
    if (
        number(value, "archetype") !== 7 || number(value, "material") !== 63 ||
        number(value, "yawQ16") !== yawQ16 || number(value, "animation") !== clip
    ) fail(`${label} presentation diverged`);
    return { archetype: 7, material: 63, yawQ16, clip, phaseOffset: 0, variation: 0 };
}
