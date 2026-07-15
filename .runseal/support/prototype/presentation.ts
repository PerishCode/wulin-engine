import { fail, type Json, number } from "../canonical-runtime.ts";

export function presentationInvariant(value: Json, clip: number, label: string): Json {
    if (
        number(value, "archetype") !== 7 || number(value, "material") !== 63 ||
        number(value, "yawQ16") !== 0 || number(value, "animation") !== clip
    ) fail(`${label} presentation diverged`);
    return { archetype: 7, material: 63, yawQ16: 0, clip, phaseOffset: 0, variation: 0 };
}
