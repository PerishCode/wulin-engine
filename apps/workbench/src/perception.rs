use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use engine_runtime::{CapturedPixels, semantic_object};

const MAX_SAMPLES: usize = 32;

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PixelRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PixelPoint {
    pub x: u32,
    pub y: u32,
}

pub struct Request {
    pub region: PixelRegion,
    pub samples: Vec<PixelPoint>,
}

pub struct Analysis {
    pub evidence: Evidence,
    pub diagnostic_rgba: Vec<u8>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Evidence {
    pub full_frame: AreaEvidence,
    pub region: RegionEvidence,
    pub samples: Vec<SampleEvidence>,
    pub unknown_ids: Vec<u32>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionEvidence {
    pub bounds: PixelRegion,
    pub pixel_count: usize,
    pub background_pixel_count: usize,
    pub objects: Vec<ObjectPixels>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AreaEvidence {
    pub pixel_count: usize,
    pub background_pixel_count: usize,
    pub objects: Vec<ObjectPixels>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectPixels {
    pub id: u32,
    pub name: Option<String>,
    pub kind: Option<String>,
    pub pixel_count: usize,
    pub bounds: PixelRegion,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SampleEvidence {
    pub point: PixelPoint,
    pub id: u32,
    pub name: Option<String>,
    pub kind: Option<String>,
}

#[derive(Default)]
struct PixelAccumulator {
    count: usize,
    min_x: u32,
    min_y: u32,
    max_x: u32,
    max_y: u32,
}

impl Request {
    pub fn new(
        region: Option<PixelRegion>,
        samples: Vec<PixelPoint>,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        let region = region.unwrap_or(PixelRegion {
            x: 0,
            y: 0,
            width,
            height,
        });
        validate_region(region, width, height)?;
        if samples.len() > MAX_SAMPLES {
            bail!("perception capture accepts at most {MAX_SAMPLES} sample points");
        }
        for point in &samples {
            if point.x >= width || point.y >= height {
                bail!(
                    "sample point [{}, {}] is outside the frame",
                    point.x,
                    point.y
                );
            }
        }
        Ok(Self { region, samples })
    }
}

pub fn analyze(pixels: &CapturedPixels, request: &Request) -> Result<Analysis> {
    let expected_bytes = usize::try_from(pixels.width)
        .ok()
        .and_then(|width| width.checked_mul(pixels.height as usize))
        .and_then(|pixels| pixels.checked_mul(4))
        .context("object-ID frame size overflow")?;
    if pixels.bytes.len() != expected_bytes {
        bail!(
            "object-ID byte count mismatch: expected {expected_bytes}, received {}",
            pixels.bytes.len()
        );
    }
    validate_region(request.region, pixels.width, pixels.height)?;

    let ids = pixels
        .bytes
        .chunks_exact(4)
        .map(|bytes| u32::from_le_bytes(bytes.try_into().expect("four-byte chunk")))
        .collect::<Vec<_>>();
    let full_bounds = PixelRegion {
        x: 0,
        y: 0,
        width: pixels.width,
        height: pixels.height,
    };
    let full_frame = analyze_area(&ids, pixels.width, full_bounds);
    let region_area = analyze_area(&ids, pixels.width, request.region);
    let unknown_ids = full_frame
        .objects
        .iter()
        .filter(|object| object.name.is_none())
        .map(|object| object.id)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let samples = request
        .samples
        .iter()
        .map(|point| sample(&ids, pixels.width, *point))
        .collect();
    let diagnostic_rgba = ids.iter().flat_map(|id| diagnostic_color(*id)).collect();

    Ok(Analysis {
        evidence: Evidence {
            full_frame,
            region: RegionEvidence {
                bounds: request.region,
                pixel_count: region_area.pixel_count,
                background_pixel_count: region_area.background_pixel_count,
                objects: region_area.objects,
            },
            samples,
            unknown_ids,
        },
        diagnostic_rgba,
    })
}

fn validate_region(region: PixelRegion, width: u32, height: u32) -> Result<()> {
    let right = region.x.checked_add(region.width);
    let bottom = region.y.checked_add(region.height);
    if region.width == 0
        || region.height == 0
        || right.is_none_or(|right| right > width)
        || bottom.is_none_or(|bottom| bottom > height)
    {
        bail!("perception region must be a non-empty rectangle inside the frame");
    }
    Ok(())
}

fn analyze_area(ids: &[u32], frame_width: u32, region: PixelRegion) -> AreaEvidence {
    let mut background_pixel_count = 0;
    let mut objects = BTreeMap::<u32, PixelAccumulator>::new();
    for y in region.y..region.y + region.height {
        for x in region.x..region.x + region.width {
            let id = ids[(y * frame_width + x) as usize];
            if id == 0 {
                background_pixel_count += 1;
            } else {
                objects.entry(id).or_default().add(x, y);
            }
        }
    }
    AreaEvidence {
        pixel_count: region.width as usize * region.height as usize,
        background_pixel_count,
        objects: objects
            .into_iter()
            .map(|(id, pixels)| object_pixels(id, pixels))
            .collect(),
    }
}

impl PixelAccumulator {
    fn add(&mut self, x: u32, y: u32) {
        if self.count == 0 {
            self.min_x = x;
            self.max_x = x;
            self.min_y = y;
            self.max_y = y;
        } else {
            self.min_x = self.min_x.min(x);
            self.max_x = self.max_x.max(x);
            self.min_y = self.min_y.min(y);
            self.max_y = self.max_y.max(y);
        }
        self.count += 1;
    }
}

fn object_pixels(id: u32, pixels: PixelAccumulator) -> ObjectPixels {
    let object = semantic_object(id);
    ObjectPixels {
        id,
        name: object.as_ref().map(|object| object.name.clone()),
        kind: object.as_ref().map(|object| object.kind.clone()),
        pixel_count: pixels.count,
        bounds: PixelRegion {
            x: pixels.min_x,
            y: pixels.min_y,
            width: pixels.max_x - pixels.min_x + 1,
            height: pixels.max_y - pixels.min_y + 1,
        },
    }
}

fn sample(ids: &[u32], frame_width: u32, point: PixelPoint) -> SampleEvidence {
    let id = ids[(point.y * frame_width + point.x) as usize];
    let object = semantic_object(id);
    SampleEvidence {
        point,
        id,
        name: object.as_ref().map(|object| object.name.clone()),
        kind: object.as_ref().map(|object| object.kind.clone()),
    }
}

fn diagnostic_color(id: u32) -> [u8; 4] {
    if id == 0 {
        return [0, 0, 0, 255];
    }
    semantic_object(id)
        .map(|object| {
            let [red, green, blue, _] = object.color;
            [channel(red), channel(green), channel(blue), 255]
        })
        .unwrap_or([255, 0, 255, 255])
}

fn channel(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}
