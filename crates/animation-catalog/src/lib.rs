mod affine;
mod builder;

use sha2::{Digest, Sha256};

pub use affine::Affine;

pub const BONE_COUNT: u32 = 128;
pub const CLIP_COUNT: u32 = 8;
pub const SAMPLE_COUNT: u32 = 64;
pub const INFLUENCE_COUNT: u32 = 4;

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
pub struct Catalog {
    pub bones: Vec<Bone>,
    pub inverse_bind: Vec<Affine>,
    pub samples: Vec<Affine>,
    pub skin_bindings: Vec<SkinBinding>,
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
        if self.bones.len() != BONE_COUNT as usize || self.inverse_bind.len() != BONE_COUNT as usize
        {
            return Err("animation catalog bone arrays are not canonical".into());
        }
        let expected_samples = (CLIP_COUNT * SAMPLE_COUNT * BONE_COUNT) as usize;
        if self.samples.len() != expected_samples {
            return Err("animation catalog sample count is not canonical".into());
        }
        let vertex_count = meshlet_catalog::Catalog::build().vertices.len();
        if self.skin_bindings.len() != vertex_count {
            return Err("animation skin stream does not match meshlet vertices".into());
        }
        for (index, bone) in self.bones.iter().enumerate() {
            if index == 0 {
                if bone.parent != u32::MAX || bone.depth != 0 {
                    return Err("animation root bone is invalid".into());
                }
            } else if bone.parent as usize >= index
                || bone.depth != self.bones[bone.parent as usize].depth + 1
            {
                return Err(format!("animation bone {index} hierarchy is invalid"));
            }
        }
        for (index, binding) in self.skin_bindings.iter().enumerate() {
            let indices = unpack_bytes(binding.indices);
            let weights = unpack_bytes(binding.weights);
            if indices.iter().any(|value| u32::from(*value) >= BONE_COUNT)
                || weights.contains(&0)
                || weights.iter().map(|value| u32::from(*value)).sum::<u32>() != 255
            {
                return Err(format!("skin binding {index} is invalid"));
            }
        }
        Ok(())
    }

    pub fn sample(&self, clip: u32, phase: u32, bone: u32) -> Affine {
        self.samples[((clip * SAMPLE_COUNT + phase) * BONE_COUNT + bone) as usize]
    }

    pub fn evaluate_pose(
        &self,
        clip: u32,
        phase: u32,
        bone_count: u32,
        variant: u32,
    ) -> Vec<Affine> {
        assert!(clip < CLIP_COUNT && phase < SAMPLE_COUNT && bone_count <= BONE_COUNT);
        let mut globals = vec![Affine::IDENTITY; bone_count as usize];
        let mut palette = Vec::with_capacity(bone_count as usize);
        for bone_index in 0..bone_count {
            let bone = self.bones[bone_index as usize];
            let local = self
                .sample(clip, phase, bone_index)
                .with_variant(variant, bone_index);
            let global = if bone.parent == u32::MAX || bone.parent >= bone_count {
                local
            } else {
                globals[bone.parent as usize].compose(local)
            };
            globals[bone_index as usize] = global;
            palette.push(global.compose(self.inverse_bind[bone_index as usize]));
        }
        palette
    }

    pub fn encoded_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"WLANM001");
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
}

fn affine_bytes(values: &[Affine]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(values.len() * 48);
    for value in values {
        value.bytes(&mut bytes);
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
