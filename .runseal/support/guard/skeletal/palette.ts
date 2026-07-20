type Fail = (message: string) => never;

export async function requireSkeletalPalette(
    root: string,
    fail: Fail,
): Promise<void> {
    console.log("==> bounded shared-pose palette capacity");
    const base = `${root}/crates/engine-runtime`;
    const resources = await Deno.readTextFile(
        `${base}/src/rendering/meshlet_scene/skeletal/resources/mod.rs`,
    );
    const renderer = await Deno.readTextFile(
        `${base}/src/rendering/meshlet_scene/skeletal/renderer.rs`,
    );
    const shader = await Deno.readTextFile(`${base}/shaders/skeletal_scene.hlsl`);
    const oracle = await Deno.readTextFile(
        `${base}/src/rendering/meshlet_scene/skeletal/oracle.rs`,
    );
    const report = await Deno.readTextFile(
        `${base}/src/rendering/meshlet_scene/skeletal/report.rs`,
    );
    const surfaceOracle = await Deno.readTextFile(
        `${base}/src/rendering/meshlet_scene/skeletal/surface/oracle.rs`,
    );
    const probe = await Deno.readTextFile(
        `${base}/src/rendering/meshlet_scene/skeletal/probe.rs`,
    );
    const acceptance = await Deno.readTextFile(
        `${root}/.runseal/support/canonical-runtime.ts`,
    );
    const capacityTest = await Deno.readTextFile(
        `${base}/tests/private/palette_capacity.rs`,
    );

    if (
        !resources.includes(
            "pub const PALETTE_ELEMENT_COUNT: u32 = MAX_SHARED_POSES * BONE_COUNT;",
        ) ||
        !resources.includes(
            "PALETTE_ELEMENT_COUNT as u64 * size_of::<Affine>() as u64",
        ) ||
        (resources.match(/PALETTE_ELEMENT_COUNT,/g)?.length ?? 0) !== 2 ||
        resources.includes(
            "SKELETAL_CANDIDATE_CAPACITY as u64 * BONE_COUNT as u64 * 48",
        )
    ) {
        fail("guard: shared-pose palette allocation or descriptors diverged");
    }

    if (
        !renderer.includes("constants[52] = 0;") ||
        !renderer.includes("constants[53] = SKELETAL_CANDIDATE_CAPACITY;") ||
        !renderer.includes("constants[54] = MAX_SHARED_POSES;") ||
        !shader.includes("pose_slot = rig * POSE_KEYS_PER_RIG") ||
        !shader.includes("pose_bitset.InterlockedOr") ||
        !shader.includes("uint key = active_pose_keys[group_id.x];") ||
        !oracle.includes("counts.active_poses = shared_poses.len() as u32;") ||
        !oracle.includes("shared_poses.insert(pose_key(presentation, settings));") ||
        !surfaceOracle.includes("input.skeletal_settings.bone_count,\n            0,") ||
        !probe.includes("pub palette_slot_capacity: u32,") ||
        !probe.includes("pub palette_storage_bytes: u64,") ||
        !probe.includes("counters[17] <= MAX_SHARED_POSES") ||
        !probe.includes("pose_slot < MAX_SHARED_POSES") ||
        !acceptance.includes("skeletal.paletteStorageBytes !== 6_291_456") ||
        !capacityTest.includes("assert_eq!(PALETTE_BYTES, 6_291_456);")
    ) {
        fail("guard: shared-pose execution or exact capacity witness diverged");
    }

    const retired = [
        renderer,
        shader,
        oracle,
        report,
        surfaceOracle,
        probe,
    ].join("\n");
    if (
        /unique_poses|uniquePoses|pose_shape\.x|apply_variant|centered_byte/.test(retired)
    ) {
        fail("guard: retired unique-pose live branch returned");
    }
}
