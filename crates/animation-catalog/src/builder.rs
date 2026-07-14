use std::f32::consts::TAU;

use meshlet_catalog::Catalog as MeshletCatalog;

use crate::{Affine, BONE_COUNT, Bone, CLIP_COUNT, Catalog, SAMPLE_COUNT, SkinBinding};

pub fn build() -> Catalog {
    let bones = build_bones();
    let inverse_bind = build_inverse_bind(&bones);
    let samples = build_samples(&bones);
    let meshlets = MeshletCatalog::build();
    let imported_start = meshlets.imported.vertex_start as usize;
    let imported_end = imported_start + meshlets.imported.vertex_count as usize;
    let skin_bindings = (0..meshlets.vertices.len())
        .map(|index| {
            if (imported_start..imported_end).contains(&index) {
                rigid_binding()
            } else {
                skin_binding(index as u32)
            }
        })
        .collect();
    Catalog {
        bones,
        inverse_bind,
        samples,
        skin_bindings,
    }
}

fn build_bones() -> Vec<Bone> {
    (0..BONE_COUNT)
        .map(|index| {
            let parent = if index == 0 {
                u32::MAX
            } else {
                (index - 1) / 2
            };
            let depth = if index == 0 {
                0
            } else {
                u32::BITS - (index + 1).leading_zeros() - 1
            };
            let side = if index & 1 == 0 { -1.0 } else { 1.0 };
            let local_translation = if index == 0 {
                [0.0, 0.0, 0.0]
            } else {
                [side * (0.025 + depth as f32 * 0.004), 0.075, 0.012 * side]
            };
            Bone {
                parent,
                depth,
                local_translation,
                reserved: 0.0,
            }
        })
        .collect()
}

fn build_inverse_bind(bones: &[Bone]) -> Vec<Affine> {
    let mut globals = vec![Affine::IDENTITY; bones.len()];
    let mut inverse = Vec::with_capacity(bones.len());
    for (index, bone) in bones.iter().enumerate() {
        let local = Affine::translation(bone.local_translation);
        let global = if bone.parent == u32::MAX {
            local
        } else {
            globals[bone.parent as usize].compose(local)
        };
        globals[index] = global;
        let origin = global.transform_point([0.0, 0.0, 0.0]);
        inverse.push(Affine::translation([-origin[0], -origin[1], -origin[2]]));
    }
    inverse
}

fn build_samples(bones: &[Bone]) -> Vec<Affine> {
    let mut samples = Vec::with_capacity((CLIP_COUNT * SAMPLE_COUNT * BONE_COUNT) as usize);
    for clip in 0..CLIP_COUNT {
        for sample in 0..SAMPLE_COUNT {
            let phase = sample as f32 / SAMPLE_COUNT as f32;
            for (bone_index, bone) in bones.iter().enumerate() {
                let wave = TAU
                    * (phase
                        + clip as f32 * 0.071
                        + bone_index as f32 * (0.003 + clip as f32 * 0.0004));
                let amplitude = 0.025 + clip as f32 * 0.006 + bone.depth as f32 * 0.002;
                let angle = wave.sin() * amplitude;
                let mut translation = bone.local_translation;
                if bone_index == 0 {
                    translation[1] += (wave * 2.0).sin() * (0.015 + clip as f32 * 0.002);
                }
                samples.push(Affine::rotation_translation(
                    (bone_index as u32 + clip) % 3,
                    angle,
                    translation,
                ));
            }
        }
    }
    samples
}

fn skin_binding(vertex: u32) -> SkinBinding {
    let base = vertex.wrapping_mul(13).wrapping_add(vertex / 7) % (BONE_COUNT - 3);
    let indices = base | ((base + 1) << 8) | ((base + 2) << 16) | ((base + 3) << 24);
    let weights = 128u32 | (64u32 << 8) | (42u32 << 16) | (21u32 << 24);
    SkinBinding { indices, weights }
}

fn rigid_binding() -> SkinBinding {
    SkinBinding {
        indices: 0,
        weights: 252u32 | (1u32 << 8) | (1u32 << 16) | (1u32 << 24),
    }
}
