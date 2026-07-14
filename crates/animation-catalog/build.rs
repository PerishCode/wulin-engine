use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use glam::{Mat4, Quat, Vec3};
use gltf::animation::util::ReadOutputs;
use gltf::animation::{Interpolation, Property};
use sha2::{Digest, Sha256};

const JSON_SHA256: &str = "6ddcabf511c0257b87dedf6ac51f1bdb6f21e570eee5fa7c4fa6162d055cb002";
const BIN_SHA256: &str = "c7d0d8de28a84d5b25623037f88e063e1502495a2ee6c55f182c61161ad12f80";
const BONE_COUNT: u32 = 128;
const CLIP_COUNT: u32 = 8;
const SAMPLE_COUNT: u32 = 64;
const SOURCE_JOINT_COUNT: usize = 24;
const SOURCE_CLIP_NAMES: [&str; 3] = ["Survey", "Walk", "Run"];
const SOURCE_KEY_COUNTS: [u32; 3] = [83, 18, 25];
const CLIP_ALIASES: [u32; 8] = [0, 1, 2, 0, 1, 2, 0, 1];

#[derive(Clone, Copy)]
struct Trs {
    translation: Vec3,
    rotation: Quat,
    scale: Vec3,
}

impl Trs {
    fn matrix(self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

struct Joint {
    node: usize,
    parent: u32,
    depth: u32,
}

enum TrackValues {
    Translation(Vec<Vec3>),
    Rotation(Vec<Quat>),
}

struct Track {
    node: usize,
    times: Vec<f32>,
    values: TrackValues,
}

struct SourceClip {
    duration: f32,
    key_count: u32,
    tracks: Vec<Track>,
}

fn main() {
    if let Err(error) = cook() {
        panic!("failed to cook the pinned Fox skeletal animation: {error}");
    }
}

fn cook() -> Result<(), Box<dyn Error>> {
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").ok_or("missing manifest")?);
    let source = manifest.join("../../assets/third-party/khronos-fox");
    let json_path = source.join("Fox.gltf");
    let bin_path = source.join("Fox.bin");
    for path in [&json_path, &bin_path] {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    let json = verified_read(&json_path, JSON_SHA256)?;
    let bin = verified_read(&bin_path, BIN_SHA256)?;
    let gltf = gltf::Gltf::from_slice(&json)?;
    let (joints, maximum_depth) = joints(&gltf.document)?;
    let bind = bind_transforms(&gltf.document)?;
    let inverse_bind = inverse_bind_matrices(&gltf.document, &bin)?;
    validate_bind_pose(&joints, &bind, &inverse_bind)?;
    let normalization = geometry_normalization(&gltf.document, &bin)?;
    let clips = source_clips(&gltf.document, &bin, &joints)?;
    let payload = encode_payload(
        &joints,
        maximum_depth,
        &bind,
        &inverse_bind,
        normalization,
        &clips,
    )?;
    let out = PathBuf::from(env::var_os("OUT_DIR").ok_or("missing OUT_DIR")?);
    fs::write(out.join("khronos-fox-skin.wla"), payload)?;
    Ok(())
}

fn verified_read(path: &Path, expected: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let bytes = fs::read(path)?;
    let actual = hex(Sha256::digest(&bytes));
    if actual != expected {
        return Err(format!(
            "{} SHA-256 {actual} differs from {expected}",
            path.display()
        )
        .into());
    }
    Ok(bytes)
}

fn joints(document: &gltf::Document) -> Result<(Vec<Joint>, u32), Box<dyn Error>> {
    if document.skins().count() != 1 {
        return Err("source must contain exactly one skin".into());
    }
    let nodes = document
        .skins()
        .next()
        .unwrap()
        .joints()
        .map(|node| node.index())
        .collect::<Vec<_>>();
    if nodes.len() != SOURCE_JOINT_COUNT {
        return Err("source skin joint count differs".into());
    }
    let mut parents = vec![None; document.nodes().count()];
    for node in document.nodes() {
        for child in node.children() {
            if parents[child.index()].replace(node.index()).is_some() {
                return Err("source node has multiple parents".into());
            }
        }
    }
    let ordinals = nodes
        .iter()
        .enumerate()
        .map(|(ordinal, node)| (*node, ordinal))
        .collect::<BTreeMap<_, _>>();
    let mut result = Vec::<Joint>::with_capacity(nodes.len());
    let mut root_count = 0;
    let mut maximum_depth = 0;
    for (ordinal, node) in nodes.into_iter().enumerate() {
        let parent = parents[node].and_then(|node| ordinals.get(&node).copied());
        let (parent, depth) = match parent {
            None => {
                root_count += 1;
                validate_external_ancestors(document, &parents, node)?;
                (u32::MAX, 0)
            }
            Some(parent) if parent < ordinal => (parent as u32, result[parent].depth + 1),
            Some(_) => return Err("source joints are not parent-first".into()),
        };
        maximum_depth = maximum_depth.max(depth);
        result.push(Joint {
            node,
            parent,
            depth,
        });
    }
    if root_count != 1 || maximum_depth > 7 {
        return Err("source joint root or hierarchy depth is invalid".into());
    }
    Ok((result, maximum_depth))
}

fn validate_external_ancestors(
    document: &gltf::Document,
    parents: &[Option<usize>],
    node: usize,
) -> Result<(), Box<dyn Error>> {
    let mut parent = parents[node];
    while let Some(index) = parent {
        let matrix =
            Mat4::from_cols_array_2d(&document.nodes().nth(index).unwrap().transform().matrix());
        if matrix_difference(matrix, Mat4::IDENTITY) > 0.000_001 {
            return Err("source skin has a transformed non-joint ancestor".into());
        }
        parent = parents[index];
    }
    Ok(())
}

fn bind_transforms(document: &gltf::Document) -> Result<Vec<Trs>, Box<dyn Error>> {
    document
        .nodes()
        .map(|node| {
            let (translation, rotation, scale) = node.transform().decomposed();
            let value = Trs {
                translation: Vec3::from_array(translation),
                rotation: Quat::from_array(rotation).normalize(),
                scale: Vec3::from_array(scale),
            };
            if !value.translation.is_finite()
                || !value.rotation.is_finite()
                || !value.scale.is_finite()
                || value.scale.cmple(Vec3::ZERO).any()
            {
                return Err("source node transform is invalid".into());
            }
            Ok(value)
        })
        .collect()
}

fn inverse_bind_matrices(
    document: &gltf::Document,
    bin: &[u8],
) -> Result<Vec<Mat4>, Box<dyn Error>> {
    let skin = document.skins().next().unwrap();
    let values = skin
        .reader(|buffer| (buffer.index() == 0).then_some(bin))
        .read_inverse_bind_matrices()
        .ok_or("source inverse-bind accessor is missing")?
        .map(|matrix| Mat4::from_cols_array_2d(&matrix))
        .collect::<Vec<_>>();
    if values.len() != SOURCE_JOINT_COUNT || values.iter().any(|matrix| !matrix.is_finite()) {
        return Err("source inverse-bind shape is invalid".into());
    }
    Ok(values)
}

fn validate_bind_pose(
    joints: &[Joint],
    bind: &[Trs],
    inverse_bind: &[Mat4],
) -> Result<(), Box<dyn Error>> {
    let mut globals = vec![Mat4::IDENTITY; joints.len()];
    let mut maximum_delta = 0.0f32;
    for (index, joint) in joints.iter().enumerate() {
        let local = bind[joint.node].matrix();
        let global = if joint.parent == u32::MAX {
            local
        } else {
            globals[joint.parent as usize] * local
        };
        globals[index] = global;
        maximum_delta = maximum_delta.max(matrix_difference(
            global * inverse_bind[index],
            Mat4::IDENTITY,
        ));
    }
    if maximum_delta > 0.000_1 {
        return Err(
            format!("source inverse binds differ from bind pose by {maximum_delta}").into(),
        );
    }
    Ok(())
}

fn geometry_normalization(document: &gltf::Document, bin: &[u8]) -> Result<Mat4, Box<dyn Error>> {
    let primitive = document
        .meshes()
        .next()
        .ok_or("source mesh is missing")?
        .primitives()
        .next()
        .ok_or("source primitive is missing")?;
    let positions = primitive
        .reader(|buffer| (buffer.index() == 0).then_some(bin))
        .read_positions()
        .ok_or("source positions are missing")?;
    let mut minimum = Vec3::splat(f32::INFINITY);
    let mut maximum = Vec3::splat(f32::NEG_INFINITY);
    for position in positions {
        let value = Vec3::from_array(position);
        minimum = minimum.min(value);
        maximum = maximum.max(value);
    }
    let height = maximum.y - minimum.y;
    if !minimum.is_finite() || !maximum.is_finite() || height <= f32::EPSILON {
        return Err("source geometry normalization is invalid".into());
    }
    let scale = height.recip();
    let center = Vec3::new(
        (minimum.x + maximum.x) * 0.5,
        minimum.y,
        (minimum.z + maximum.z) * 0.5,
    );
    Ok(Mat4::from_translation(-center * scale) * Mat4::from_scale(Vec3::splat(scale)))
}

fn source_clips(
    document: &gltf::Document,
    bin: &[u8],
    joints: &[Joint],
) -> Result<Vec<SourceClip>, Box<dyn Error>> {
    if document.animations().count() != SOURCE_CLIP_NAMES.len() {
        return Err("source clip count differs".into());
    }
    let joint_nodes = joints.iter().map(|joint| joint.node).collect::<Vec<_>>();
    let mut clips = Vec::new();
    for (clip_index, animation) in document.animations().enumerate() {
        if animation.name() != Some(SOURCE_CLIP_NAMES[clip_index]) {
            return Err(format!("source clip {clip_index} name differs").into());
        }
        let mut tracks = Vec::new();
        let mut targets = BTreeSet::new();
        let mut duration: Option<f32> = None;
        for channel in animation.channels() {
            if channel.sampler().interpolation() != Interpolation::Linear {
                return Err("source animation interpolation is not linear".into());
            }
            let node = channel.target().node().index();
            if !joint_nodes.contains(&node) {
                return Err("source animation targets a non-joint node".into());
            }
            let property = channel.target().property();
            let property_key = match property {
                Property::Translation => 0,
                Property::Rotation => 1,
                Property::Scale => 2,
                Property::MorphTargetWeights => 3,
            };
            if !targets.insert((node, property_key)) {
                return Err("source animation contains a duplicate target".into());
            }
            let reader = channel.reader(|buffer| (buffer.index() == 0).then_some(bin));
            let times = reader
                .read_inputs()
                .ok_or("source animation inputs are missing")?
                .collect::<Vec<_>>();
            if times.len() != SOURCE_KEY_COUNTS[clip_index] as usize
                || times.windows(2).any(|pair| pair[0] >= pair[1])
                || times.first().copied() != Some(0.0)
            {
                return Err(format!("source clip {clip_index} keys are invalid").into());
            }
            let track_duration = *times.last().unwrap();
            if let Some(expected) = duration
                && (expected - track_duration).abs() > f32::EPSILON
            {
                return Err(format!("source clip {clip_index} durations differ").into());
            }
            duration = Some(track_duration);
            let values = match (property, reader.read_outputs()) {
                (Property::Translation, Some(ReadOutputs::Translations(values))) => {
                    TrackValues::Translation(values.map(Vec3::from_array).collect())
                }
                (Property::Rotation, Some(ReadOutputs::Rotations(values))) => {
                    TrackValues::Rotation(
                        values
                            .into_f32()
                            .map(|value| Quat::from_array(value).normalize())
                            .collect(),
                    )
                }
                _ => return Err("source animation channel shape is unsupported".into()),
            };
            let value_count = match &values {
                TrackValues::Translation(values) => values.len(),
                TrackValues::Rotation(values) => values.len(),
            };
            if value_count != times.len() {
                return Err("source animation input/output counts differ".into());
            }
            tracks.push(Track {
                node,
                times,
                values,
            });
        }
        if tracks.len() != 21
            || tracks
                .iter()
                .filter(|track| matches!(track.values, TrackValues::Translation(_)))
                .count()
                != 1
            || tracks
                .iter()
                .filter(|track| matches!(track.values, TrackValues::Rotation(_)))
                .count()
                != 20
        {
            return Err(format!("source clip {clip_index} channel shape differs").into());
        }
        clips.push(SourceClip {
            duration: duration.ok_or("source clip has no duration")?,
            key_count: SOURCE_KEY_COUNTS[clip_index],
            tracks,
        });
    }
    if clips.len() != SOURCE_CLIP_NAMES.len() {
        return Err("source clip count differs".into());
    }
    Ok(clips)
}

fn encode_payload(
    joints: &[Joint],
    maximum_depth: u32,
    bind: &[Trs],
    inverse_bind: &[Mat4],
    normalization: Mat4,
    clips: &[SourceClip],
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"WLSKN001");
    bytes.extend_from_slice(&decode_hex(JSON_SHA256));
    bytes.extend_from_slice(&decode_hex(BIN_SHA256));
    for value in [
        SOURCE_JOINT_COUNT as u32,
        maximum_depth,
        clips.len() as u32,
        CLIP_COUNT,
        SAMPLE_COUNT,
        BONE_COUNT,
    ] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for clip in clips {
        bytes.extend_from_slice(&clip.duration.to_bits().to_le_bytes());
    }
    for clip in clips {
        bytes.extend_from_slice(&clip.key_count.to_le_bytes());
    }
    for alias in CLIP_ALIASES {
        bytes.extend_from_slice(&alias.to_le_bytes());
    }
    for index in 0..BONE_COUNT as usize {
        let (parent, depth) = if index < joints.len() {
            (joints[index].parent, joints[index].depth)
        } else {
            (u32::MAX, 0)
        };
        bytes.extend_from_slice(&parent.to_le_bytes());
        bytes.extend_from_slice(&depth.to_le_bytes());
        bytes.extend_from_slice(&[0; 16]);
    }
    for _ in 0..BONE_COUNT {
        push_affine(&mut bytes, Mat4::IDENTITY)?;
    }
    let inverse_normalization = normalization.inverse();
    for alias in CLIP_ALIASES {
        let clip = &clips[alias as usize];
        for phase in 0..SAMPLE_COUNT {
            let time = clip.duration * phase as f32 / SAMPLE_COUNT as f32;
            let mut locals = bind.to_vec();
            for track in &clip.tracks {
                sample_track(track, time, &mut locals[track.node]);
            }
            let mut source_globals = vec![Mat4::IDENTITY; joints.len()];
            let mut desired = vec![Mat4::IDENTITY; joints.len()];
            for (index, joint) in joints.iter().enumerate() {
                let local = locals[joint.node].matrix();
                let global = if joint.parent == u32::MAX {
                    local
                } else {
                    source_globals[joint.parent as usize] * local
                };
                source_globals[index] = global;
                desired[index] =
                    normalization * global * inverse_bind[index] * inverse_normalization;
            }
            for (index, joint) in joints.iter().enumerate() {
                let local = if joint.parent == u32::MAX {
                    desired[index]
                } else {
                    desired[joint.parent as usize].inverse() * desired[index]
                };
                push_affine(&mut bytes, local)?;
            }
            for _ in joints.len()..BONE_COUNT as usize {
                push_affine(&mut bytes, Mat4::IDENTITY)?;
            }
        }
    }
    Ok(bytes)
}

fn sample_track(track: &Track, time: f32, target: &mut Trs) {
    let upper = track.times.partition_point(|sample| *sample <= time);
    let lower = upper.saturating_sub(1).min(track.times.len() - 1);
    let upper = upper.min(track.times.len() - 1);
    let span = track.times[upper] - track.times[lower];
    let alpha = if span <= f32::EPSILON {
        0.0
    } else {
        (time - track.times[lower]) / span
    };
    match &track.values {
        TrackValues::Translation(values) => {
            target.translation = values[lower].lerp(values[upper], alpha);
        }
        TrackValues::Rotation(values) => {
            target.rotation = values[lower].slerp(values[upper], alpha).normalize();
        }
    }
}

fn push_affine(bytes: &mut Vec<u8>, matrix: Mat4) -> Result<(), Box<dyn Error>> {
    if !matrix.is_finite()
        || matrix.x_axis.w.abs() > 0.000_01
        || matrix.y_axis.w.abs() > 0.000_01
        || matrix.z_axis.w.abs() > 0.000_01
        || (matrix.w_axis.w - 1.0).abs() > 0.000_01
    {
        return Err("source animation produced a non-affine matrix".into());
    }
    for row in 0..3 {
        for column in 0..4 {
            let value = matrix.col(column)[row];
            bytes.extend_from_slice(&value.to_bits().to_le_bytes());
        }
    }
    Ok(())
}

fn matrix_difference(left: Mat4, right: Mat4) -> f32 {
    left.to_cols_array()
        .into_iter()
        .zip(right.to_cols_array())
        .map(|(left, right)| (left - right).abs())
        .fold(0.0, f32::max)
}

fn hex(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn decode_hex(value: &str) -> [u8; 32] {
    let mut bytes = [0; 32];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).unwrap();
    }
    bytes
}
