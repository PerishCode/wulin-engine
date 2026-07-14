use std::env;
use std::error::Error;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use gltf::image::Source;
use gltf::texture::{MagFilter, MinFilter, WrappingMode};
use png::{BitDepth, ColorType, Decoder, Transformations};
use sha2::{Digest, Sha256};

const JSON_SHA256: &str = "6ddcabf511c0257b87dedf6ac51f1bdb6f21e570eee5fa7c4fa6162d055cb002";
const TEXTURE_SHA256: &str = "61c8b109ee7f8bf262791933380fafb1465f7b51cbe6472c2d21eff0b31f83a1";
const SOURCE_SIDE: usize = 1024;
const COOKED_SIDE: usize = 64;
const MIP_COUNT: usize = 7;

struct CookedMaterial {
    base_color: [f32; 4],
    roughness: f32,
    metallic: f32,
    mips: Vec<Vec<u8>>,
}

fn main() {
    if let Err(error) = cook() {
        panic!("failed to cook the pinned Fox material: {error}");
    }
}

fn cook() -> Result<(), Box<dyn Error>> {
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").ok_or("missing manifest")?);
    let source = manifest.join("../../assets/third-party/khronos-fox");
    let json_path = source.join("Fox.gltf");
    let texture_path = source.join("Texture.png");
    for path in [&json_path, &texture_path] {
        println!("cargo:rerun-if-changed={}", path.display());
    }

    let json = verified_read(&json_path, JSON_SHA256)?;
    let texture = verified_read(&texture_path, TEXTURE_SHA256)?;
    let factors = validate_material(&json)?;
    let source_rgba = decode_png(&texture)?;
    let cooked = CookedMaterial {
        base_color: factors.0,
        roughness: factors.1,
        metallic: factors.2,
        mips: build_mips(&source_rgba),
    };
    let payload = encode_payload(&cooked);
    let out = PathBuf::from(env::var_os("OUT_DIR").ok_or("missing OUT_DIR")?);
    fs::write(out.join("khronos-fox-material.wltm"), payload)?;
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

fn validate_material(json: &[u8]) -> Result<([f32; 4], f32, f32), Box<dyn Error>> {
    let gltf = gltf::Gltf::from_slice(json)?;
    if gltf.document.materials().count() != 1
        || gltf.document.textures().count() != 1
        || gltf.document.images().count() != 1
        || gltf.document.samplers().count() != 1
    {
        return Err("source material, texture, image, or sampler count differs".into());
    }
    let material = gltf.document.materials().next().unwrap();
    if material.name() != Some("fox_material") || material.index() != Some(0) {
        return Err("source material identity differs".into());
    }
    let pbr = material.pbr_metallic_roughness();
    let info = pbr
        .base_color_texture()
        .ok_or("source material has no base-color texture")?;
    let texture = info.texture();
    let sampler = texture.sampler();
    let image = texture.source();
    if info.tex_coord() != 0
        || texture.index() != 0
        || sampler.index() != Some(0)
        || sampler.mag_filter() != Some(MagFilter::Linear)
        || sampler.min_filter() != Some(MinFilter::LinearMipmapLinear)
        || sampler.wrap_s() != WrappingMode::Repeat
        || sampler.wrap_t() != WrappingMode::Repeat
        || image.index() != 0
    {
        return Err("source material texture/sampler join differs".into());
    }
    match image.source() {
        Source::Uri { uri, mime_type }
            if uri == "Texture.png" && mime_type == Some("image/png") => {}
        _ => return Err("source material image reference differs".into()),
    }
    let base_color = pbr.base_color_factor();
    let roughness = pbr.roughness_factor();
    let metallic = pbr.metallic_factor();
    if base_color.map(f32::to_bits) != [1.0f32.to_bits(); 4]
        || roughness.to_bits() != 0.58f32.to_bits()
        || metallic.to_bits() != 0.0f32.to_bits()
    {
        return Err("source material factors differ".into());
    }
    let primitive_materials = gltf
        .document
        .meshes()
        .flat_map(|mesh| mesh.primitives())
        .map(|primitive| primitive.material().index())
        .collect::<Vec<_>>();
    if primitive_materials != [Some(0)] {
        return Err("source primitive material join differs".into());
    }
    Ok((base_color, roughness, metallic))
}

fn decode_png(bytes: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut decoder = Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(Transformations::IDENTITY);
    let mut reader = decoder.read_info()?;
    let mut buffer = vec![
        0;
        reader
            .output_buffer_size()
            .ok_or("PNG output is too large")?
    ];
    let info = reader.next_frame(&mut buffer)?;
    if info.width != SOURCE_SIDE as u32
        || info.height != SOURCE_SIDE as u32
        || info.bit_depth != BitDepth::Eight
        || info.color_type != ColorType::Rgb
        || info.buffer_size() != SOURCE_SIDE * SOURCE_SIDE * 3
    {
        return Err("source PNG shape or format differs".into());
    }
    let mut rgba = Vec::with_capacity(SOURCE_SIDE * SOURCE_SIDE * 4);
    for rgb in buffer[..info.buffer_size()].chunks_exact(3) {
        rgba.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
    }
    Ok(rgba)
}

fn build_mips(source: &[u8]) -> Vec<Vec<u8>> {
    let mut mips = Vec::with_capacity(MIP_COUNT);
    mips.push(reduce_source(source));
    while mips.len() < MIP_COUNT {
        let previous_side = COOKED_SIDE >> (mips.len() - 1);
        mips.push(reduce_2x2(mips.last().unwrap(), previous_side));
    }
    mips
}

fn reduce_source(source: &[u8]) -> Vec<u8> {
    let ratio = SOURCE_SIDE / COOKED_SIDE;
    let area = (ratio * ratio) as u32;
    let mut output = Vec::with_capacity(COOKED_SIDE * COOKED_SIDE * 4);
    for y in 0..COOKED_SIDE {
        for x in 0..COOKED_SIDE {
            let mut sum = [0u32; 4];
            for source_y in y * ratio..(y + 1) * ratio {
                for source_x in x * ratio..(x + 1) * ratio {
                    let offset = (source_y * SOURCE_SIDE + source_x) * 4;
                    for channel in 0..4 {
                        sum[channel] += u32::from(source[offset + channel]);
                    }
                }
            }
            output.extend(sum.map(|value| ((value + area / 2) / area) as u8));
        }
    }
    output
}

fn reduce_2x2(source: &[u8], source_side: usize) -> Vec<u8> {
    let side = source_side / 2;
    let mut output = Vec::with_capacity(side * side * 4);
    for y in 0..side {
        for x in 0..side {
            let mut sum = [0u16; 4];
            for offset_y in 0..2 {
                for offset_x in 0..2 {
                    let offset = ((y * 2 + offset_y) * source_side + x * 2 + offset_x) * 4;
                    for channel in 0..4 {
                        sum[channel] += u16::from(source[offset + channel]);
                    }
                }
            }
            output.extend(sum.map(|value| ((value + 2) / 4) as u8));
        }
    }
    output
}

fn encode_payload(material: &CookedMaterial) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"WLFTX001");
    bytes.extend_from_slice(&decode_hex(JSON_SHA256));
    bytes.extend_from_slice(&decode_hex(TEXTURE_SHA256));
    for value in [SOURCE_SIDE, SOURCE_SIDE, COOKED_SIDE, MIP_COUNT] {
        bytes.extend_from_slice(&(value as u32).to_le_bytes());
    }
    for value in material
        .base_color
        .into_iter()
        .chain([material.roughness, material.metallic])
    {
        bytes.extend_from_slice(&value.to_bits().to_le_bytes());
    }
    for mip in &material.mips {
        bytes.extend_from_slice(&(mip.len() as u32).to_le_bytes());
    }
    for mip in &material.mips {
        bytes.extend_from_slice(mip);
    }
    bytes
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
    for (index, output) in bytes.iter_mut().enumerate() {
        *output = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).unwrap();
    }
    bytes
}
