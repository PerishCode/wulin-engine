mod affine;
mod builder;
mod imported_rig;

use sha2::{Digest, Sha256};

pub use affine::Affine;

pub const BONE_COUNT: u32 = 128;
pub const CLIP_COUNT: u32 = 8;
pub const SAMPLE_COUNT: u32 = 64;
pub const INFLUENCE_COUNT: u32 = 4;
pub const RIG_COUNT: u32 = 2;
pub const FIXTURE_RIG: u32 = 0;
pub const IMPORTED_RIG: u32 = 1;
pub const POSE_KEYS_PER_RIG: u32 = CLIP_COUNT * SAMPLE_COUNT;
pub const MAX_POSE_KEYS: u32 = RIG_COUNT * POSE_KEYS_PER_RIG;
pub const PRESENTATION_TIME_UNITS_PER_SECOND: u32 = 4_800;
pub const PRESENTATION_TIME_UNITS_PER_FRAME: u32 = 80;
pub const FIXTURE_CLIP_DURATION_UNITS: u32 = SAMPLE_COUNT * PRESENTATION_TIME_UNITS_PER_FRAME;
pub const IMPORTED_SOURCE_CLIP_DURATION_UNITS: [u32; 3] = [16_400, 3_400, 5_560];
pub const IMPORTED_CLIP_DURATION_UNITS: [u32; CLIP_COUNT as usize] =
    [16_400, 3_400, 5_560, 16_400, 3_400, 5_560, 16_400, 3_400];
pub const PRESENTATION_CLOCK_FRAME_PERIOD: u32 = 31_002_560;
pub const PRESENTATION_CLOCK_UNIT_PERIOD: u32 =
    PRESENTATION_CLOCK_FRAME_PERIOD * PRESENTATION_TIME_UNITS_PER_FRAME;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bone {
    pub parent: u32,
    pub depth: u32,
    pub local_translation: [f32; 3],
    pub reserved: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SkinBinding {
    pub indices: u32,
    pub weights: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImportedRigMetadata {
    pub revision: &'static str,
    pub source_json_sha256: String,
    pub source_bin_sha256: String,
    pub cooked_sha256: String,
    pub source_joint_count: u32,
    pub maximum_joint_depth: u32,
    pub source_clip_names: [&'static str; 3],
    pub source_clip_durations: [f32; 3],
    pub source_clip_key_counts: [u32; 3],
    pub clip_aliases: [u32; 8],
}

#[derive(Clone, Debug, PartialEq)]
pub struct Catalog {
    pub bones: Vec<Bone>,
    pub inverse_bind: Vec<Affine>,
    pub samples: Vec<Affine>,
    pub skin_bindings: Vec<SkinBinding>,
    pub imported: ImportedRigMetadata,
}

impl Catalog {
    pub fn build() -> Self {
        let catalog = builder::build();
        catalog
            .validate()
            .expect("generated animation catalog is invalid");
        catalog
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.bones.len() != (RIG_COUNT * BONE_COUNT) as usize
            || self.inverse_bind.len() != (RIG_COUNT * BONE_COUNT) as usize
        {
            return Err("animation catalog bone arrays are not canonical".into());
        }
        let expected_samples = (RIG_COUNT * CLIP_COUNT * SAMPLE_COUNT * BONE_COUNT) as usize;
        if self.samples.len() != expected_samples {
            return Err("animation catalog sample count is not canonical".into());
        }
        let vertex_count = meshlet_catalog::Catalog::build().vertices.len();
        if self.skin_bindings.len() != vertex_count {
            return Err("animation skin stream does not match meshlet vertices".into());
        }
        for rig in 0..RIG_COUNT as usize {
            let start = rig * BONE_COUNT as usize;
            let bones = &self.bones[start..start + BONE_COUNT as usize];
            for (index, bone) in bones.iter().enumerate() {
                if bone.parent == u32::MAX {
                    if bone.depth != 0 {
                        return Err(format!("animation rig {rig} root {index} is invalid"));
                    }
                } else if bone.parent as usize >= index
                    || bone.depth != bones[bone.parent as usize].depth + 1
                {
                    return Err(format!("animation rig {rig} bone {index} is invalid"));
                }
            }
        }
        for (index, binding) in self.skin_bindings.iter().enumerate() {
            let indices = unpack_bytes(binding.indices);
            let weights = unpack_bytes(binding.weights);
            if indices.iter().any(|value| u32::from(*value) >= BONE_COUNT)
                || weights.iter().map(|value| u32::from(*value)).sum::<u32>() != 255
            {
                return Err(format!("skin binding {index} is invalid"));
            }
        }
        if self.imported.revision != "cooked-gltf-skeletal-animation-v1"
            || self.imported.source_json_sha256.len() != 64
            || self.imported.source_bin_sha256.len() != 64
            || self.imported.cooked_sha256.len() != 64
            || self.imported.source_joint_count != 24
            || self.imported.maximum_joint_depth > 7
            || self.imported.source_clip_names != ["Survey", "Walk", "Run"]
            || self.imported.source_clip_key_counts != [83, 18, 25]
            || self.imported.clip_aliases != [0, 1, 2, 0, 1, 2, 0, 1]
            || self
                .imported
                .source_clip_durations
                .into_iter()
                .zip(IMPORTED_SOURCE_CLIP_DURATION_UNITS)
                .any(|(duration, units)| {
                    (duration * PRESENTATION_TIME_UNITS_PER_SECOND as f32 - units as f32).abs()
                        > 0.001
                })
        {
            return Err("imported rig metadata is invalid".into());
        }
        Ok(())
    }

    pub fn sample(&self, clip: u32, phase: u32, bone: u32) -> Affine {
        self.sample_for_rig(FIXTURE_RIG, clip, phase, bone)
    }

    pub fn sample_for_rig(&self, rig: u32, clip: u32, phase: u32, bone: u32) -> Affine {
        self.samples
            [(((rig * CLIP_COUNT + clip) * SAMPLE_COUNT + phase) * BONE_COUNT + bone) as usize]
    }

    pub fn evaluate_pose(
        &self,
        clip: u32,
        phase: u32,
        bone_count: u32,
        variant: u32,
    ) -> Vec<Affine> {
        self.evaluate_pose_for_rig(FIXTURE_RIG, clip, phase, bone_count, variant)
    }

    pub fn evaluate_pose_for_archetype(
        &self,
        archetype: u32,
        clip: u32,
        phase: u32,
        bone_count: u32,
        variant: u32,
    ) -> Vec<Affine> {
        self.evaluate_pose_for_rig(
            rig_for_archetype(archetype),
            clip,
            phase,
            bone_count,
            variant,
        )
    }

    pub fn evaluate_pose_for_rig(
        &self,
        rig: u32,
        clip: u32,
        phase: u32,
        bone_count: u32,
        variant: u32,
    ) -> Vec<Affine> {
        assert!(
            rig < RIG_COUNT
                && clip < CLIP_COUNT
                && phase < SAMPLE_COUNT
                && bone_count <= BONE_COUNT
        );
        let bone_start = (rig * BONE_COUNT) as usize;
        let mut globals = vec![Affine::IDENTITY; bone_count as usize];
        let mut palette = Vec::with_capacity(bone_count as usize);
        for bone_index in 0..bone_count {
            let bone = self.bones[bone_start + bone_index as usize];
            let mut local = self.sample_for_rig(rig, clip, phase, bone_index);
            if rig == FIXTURE_RIG {
                local = local.with_variant(variant, bone_index);
            }
            let global = if bone.parent == u32::MAX || bone.parent >= bone_count {
                local
            } else {
                globals[bone.parent as usize].compose(local)
            };
            globals[bone_index as usize] = global;
            palette.push(global.compose(self.inverse_bind[bone_start + bone_index as usize]));
        }
        palette
    }

    pub fn encoded_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"WLANM002");
        for count in [
            self.bones.len(),
            self.inverse_bind.len(),
            self.samples.len(),
            self.skin_bindings.len(),
        ] {
            bytes.extend_from_slice(&(count as u32).to_le_bytes());
        }
        bytes.extend_from_slice(&self.bone_bytes());
        bytes.extend_from_slice(&self.inverse_bind_bytes());
        bytes.extend_from_slice(&self.sample_bytes());
        bytes.extend_from_slice(&self.skin_binding_bytes());
        for value in [
            &self.imported.source_json_sha256,
            &self.imported.source_bin_sha256,
            &self.imported.cooked_sha256,
        ] {
            bytes.extend_from_slice(value.as_bytes());
        }
        for value in self.imported.source_clip_durations {
            bytes.extend_from_slice(&value.to_bits().to_le_bytes());
        }
        for value in self
            .imported
            .source_clip_key_counts
            .into_iter()
            .chain(self.imported.clip_aliases)
        {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes
    }

    pub fn sha256(&self) -> String {
        let digest = Sha256::digest(self.encoded_bytes());
        digest.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    pub fn bone_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.bones.len() * 24);
        for bone in &self.bones {
            bytes.extend_from_slice(&bone.parent.to_le_bytes());
            bytes.extend_from_slice(&bone.depth.to_le_bytes());
            for value in bone.local_translation {
                bytes.extend_from_slice(&value.to_bits().to_le_bytes());
            }
            bytes.extend_from_slice(&bone.reserved.to_bits().to_le_bytes());
        }
        bytes
    }

    pub fn inverse_bind_bytes(&self) -> Vec<u8> {
        affine_bytes(&self.inverse_bind)
    }

    pub fn sample_bytes(&self) -> Vec<u8> {
        affine_bytes(&self.samples)
    }

    pub fn skin_binding_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.skin_bindings.len() * 8);
        for binding in &self.skin_bindings {
            bytes.extend_from_slice(&binding.indices.to_le_bytes());
            bytes.extend_from_slice(&binding.weights.to_le_bytes());
        }
        bytes
    }

    pub fn gpu_bytes(&self) -> usize {
        self.bone_bytes().len()
            + self.inverse_bind_bytes().len()
            + self.sample_bytes().len()
            + self.skin_binding_bytes().len()
    }

    pub fn rig_bytes(&self, rig: u32) -> Vec<u8> {
        assert!(rig < RIG_COUNT);
        let bone_start = (rig * BONE_COUNT) as usize;
        let bone_end = bone_start + BONE_COUNT as usize;
        let sample_start = (rig * CLIP_COUNT * SAMPLE_COUNT * BONE_COUNT) as usize;
        let sample_end = sample_start + (CLIP_COUNT * SAMPLE_COUNT * BONE_COUNT) as usize;
        let mut bytes = bone_slice_bytes(&self.bones[bone_start..bone_end]);
        bytes.extend_from_slice(&affine_bytes(&self.inverse_bind[bone_start..bone_end]));
        bytes.extend_from_slice(&affine_bytes(&self.samples[sample_start..sample_end]));
        bytes
    }

    pub fn rig_sha256(&self, rig: u32) -> String {
        let digest = Sha256::digest(self.rig_bytes(rig));
        digest.iter().map(|byte| format!("{byte:02x}")).collect()
    }
}

pub const fn rig_for_archetype(archetype: u32) -> u32 {
    if archetype == meshlet_catalog::IMPORTED_ARCHETYPE {
        IMPORTED_RIG
    } else {
        FIXTURE_RIG
    }
}

pub const fn clip_duration_units(rig: u32, clip: u32) -> u32 {
    assert!(rig < RIG_COUNT && clip < CLIP_COUNT);
    if rig == FIXTURE_RIG {
        FIXTURE_CLIP_DURATION_UNITS
    } else {
        IMPORTED_CLIP_DURATION_UNITS[clip as usize]
    }
}

pub const fn phase_at_frame(rig: u32, clip: u32, frame: u32) -> u32 {
    let duration = clip_duration_units(rig, clip) as u64;
    let elapsed = (frame as u64 * PRESENTATION_TIME_UNITS_PER_FRAME as u64) % duration;
    (elapsed * SAMPLE_COUNT as u64 / duration) as u32
}

fn affine_bytes(values: &[Affine]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(values.len() * 48);
    for value in values {
        value.bytes(&mut bytes);
    }
    bytes
}

fn bone_slice_bytes(values: &[Bone]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(values.len() * 24);
    for bone in values {
        bytes.extend_from_slice(&bone.parent.to_le_bytes());
        bytes.extend_from_slice(&bone.depth.to_le_bytes());
        for value in bone.local_translation {
            bytes.extend_from_slice(&value.to_bits().to_le_bytes());
        }
        bytes.extend_from_slice(&bone.reserved.to_bits().to_le_bytes());
    }
    bytes
}

pub fn unpack_bytes(value: u32) -> [u8; 4] {
    [
        value as u8,
        (value >> 8) as u8,
        (value >> 16) as u8,
        (value >> 24) as u8,
    ]
}
