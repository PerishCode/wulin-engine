use anyhow::{Result, ensure};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::rendering::meshlet_scene::skeletal::SkeletalProbe;
use crate::rendering::resident::read_values;

use super::super::super::resources::actor::{ACTOR_VISIBLE_RECORD_BYTES, ActorVisibleCandidate};
use super::super::super::resources::{
    ACTOR_CANDIDATE_INDEX, VISIBLE_CANDIDATE_WORD, VISIBLE_IDENTITY_HIGH_WORD,
    VISIBLE_IDENTITY_LOW_WORD, VISIBLE_OBJECT_BYTES, VISIBLE_OBJECT_WORDS,
};
use super::super::probe::ProbeInput;
use super::oracle::{self, OcclusionOracle};
use super::{
    BoundProof, FILTERED_VISIBLE_BYTES, OCCLUSION_COUNTER_BYTES, OCCLUSION_GROUPS,
    OCCLUSION_MASK_BYTES,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OcclusionProbe {
    pub enabled: bool,
    pub history_queried: bool,
    pub history_reset_count: u64,
    pub bypass_reason: &'static str,
    pub source_visible: u32,
    pub survivors: u32,
    pub occluded: u32,
    pub tested: u32,
    pub bypassed: u32,
    pub invalid_queries: u32,
    pub overflow: u32,
    pub source_meshlets: u32,
    pub submitted_meshlets: u32,
    pub source_vertices: u32,
    pub submitted_vertices: u32,
    pub source_triangles: u32,
    pub submitted_triangles: u32,
    pub source_skin_influences: u32,
    pub submitted_skin_influences: u32,
    pub visible_record_bytes: u32,
    pub filtered_visible_bytes: u64,
    pub order_readback_bytes: u64,
    pub candidate_mask_sha256: String,
    pub source_order_sha256: String,
    pub filtered_order_sha256: String,
    pub stable_compaction_mismatch_count: u32,
    pub hierarchy_sha256: String,
    pub hierarchy_format: &'static str,
    pub hierarchy_mip_dimensions: Vec<[u32; 2]>,
    pub hierarchy_bytes: u64,
    pub hierarchy_mismatch_count: u32,
    pub query_dispatch_count: u32,
    pub query_groups: u32,
    pub prefix_dispatch_count: u32,
    pub prefix_groups: u32,
    pub scatter_dispatch_count: u32,
    pub scatter_groups: u32,
    pub compaction_dispatch_count: u32,
    pub hierarchy_dispatch_count: u32,
    pub gpu_query_ms: f64,
    pub cpu_oracle: OcclusionOracle,
    pub bound_proof: BoundProof,
    pub actor_candidate: Option<ActorCandidateProbe>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorCandidateProbe {
    pub candidate_index: u32,
    pub generation_decimal: String,
    pub stable_identity: [u32; 2],
    pub upload_record_bytes: u32,
    pub semantic_region: u32,
    pub source_visible: bool,
    pub source_visible_index: Option<u32>,
    pub occlusion_mask: u32,
    pub survived: bool,
    pub filtered_visible_index: Option<u32>,
    pub record_words: Option<[u32; 14]>,
    pub record_sha256: Option<String>,
    pub exact_field_mismatch_count: u32,
}

pub(in crate::rendering::meshlet_scene::skeletal::surface) unsafe fn read(
    input: &ProbeInput<'_>,
    skeletal: &SkeletalProbe,
    milliseconds: &impl Fn(usize, usize) -> f64,
) -> Result<OcclusionProbe> {
    let counters = unsafe {
        read_values::<u32>(
            &input.resources.occlusion.counter_readback,
            (OCCLUSION_COUNTER_BYTES / 4) as usize,
        )
    }?;
    let mask = unsafe {
        read_values::<u32>(
            &input.resources.occlusion.mask_readback,
            (OCCLUSION_MASK_BYTES / 4) as usize,
        )
    }?;
    let order_words = unsafe {
        read_values::<u32>(
            &input.resources.occlusion.order_readback,
            (FILTERED_VISIBLE_BYTES * 2 / 4) as usize,
        )
    }?;
    ensure!(
        counters[1] == 1 && counters[2] == 1,
        "occlusion indirect arguments are invalid"
    );
    ensure!(
        counters[3] == skeletal.gpu.visible,
        "occlusion source count differs from the accepted skeletal visible count"
    );
    ensure!(
        counters[4] + counters[5] == counters[3],
        "occlusion survivors and rejected objects do not cover the source list"
    );
    ensure!(
        counters[8] == 0,
        "occlusion query produced an invalid result"
    );
    ensure!(
        counters[9] == 0,
        "occlusion filtered-visible storage overflowed"
    );
    let gpu_history_queried = counters[18] != 0;
    ensure!(
        gpu_history_queried == input.history_queried,
        "GPU occlusion history state differs from the renderer state"
    );
    if input.history_queried {
        ensure!(
            counters[6] == counters[3] && counters[7] == 0,
            "compatible history did not query every source object"
        );
    } else {
        ensure!(
            counters[6] == 0 && counters[7] == counters[3],
            "invalid history did not bypass every source object"
        );
    }
    ensure!(
        counters[10] == skeletal.gpu.meshlets
            && counters[12] == skeletal.gpu.emitted_vertices
            && counters[14] == skeletal.gpu.emitted_triangles
            && counters[16] == skeletal.gpu.skin_influences,
        "occlusion source geometry differs from the accepted skeletal counters"
    );
    let survivors = mask.iter().filter(|value| **value == 1).count() as u32;
    let occluded = mask.iter().filter(|value| **value == 2).count() as u32;
    ensure!(
        mask.iter().all(|value| *value <= 2),
        "occlusion candidate mask contains an invalid value"
    );
    ensure!(
        survivors == counters[4] && occluded == counters[5],
        "occlusion candidate mask differs from aggregate counters"
    );
    let mask_bytes = mask
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect::<Vec<_>>();
    let winner = unsafe { input.resources.winner_readback.read() }?;
    let hierarchy = unsafe { input.resources.occlusion.hierarchy_readback.read() }?;
    let (hierarchy_sha256, hierarchy_mismatch_count, hierarchy_bytes) =
        validate_hierarchy(&winner.bytes, &hierarchy)?;
    let (cpu_oracle, cpu_mask) = oracle::evaluate(oracle::QueryInput {
        mesh: input.mesh_catalog,
        scene: input.scene,
        projection: input.projection,
        ground_numerators: input.ground_numerators,
        ground_denominator: input.ground_denominator,
        instance_records: input.instance_records,
        local_ids: input.local_ids,
        presentations: input.presentations,
        extent: [input.width, input.height],
        hierarchy: &hierarchy,
        history_queried: input.history_queried,
        actor: input.actor,
        presentation_tick: input.skeletal_settings.time_tick,
        object_suppression: input.object_suppression,
    })?;
    let first_mask_mismatch = mask
        .iter()
        .zip(&cpu_mask)
        .enumerate()
        .find(|(_, (gpu, cpu))| gpu != cpu)
        .map(|(candidate, (gpu, cpu))| (candidate, *gpu, *cpu));
    ensure!(
        first_mask_mismatch.is_none(),
        "GPU occlusion candidate mask differs from the CPU oracle at {first_mask_mismatch:?}"
    );
    let words_per_record = VISIBLE_OBJECT_WORDS;
    let capacity_words = (FILTERED_VISIBLE_BYTES / 4) as usize;
    let source_words = &order_words[..counters[3] as usize * words_per_record];
    let filtered_words =
        &order_words[capacity_words..capacity_words + counters[4] as usize * words_per_record];
    let mut expected_filtered = Vec::with_capacity(filtered_words.len());
    for record in source_words.chunks_exact(words_per_record) {
        let candidate_index = record[VISIBLE_CANDIDATE_WORD] as usize;
        ensure!(
            candidate_index < mask.len(),
            "source visible record has an invalid candidate index"
        );
        if mask[candidate_index] == 1 {
            expected_filtered.extend_from_slice(record);
        }
    }
    ensure!(
        expected_filtered == filtered_words,
        "occlusion compaction did not preserve source-visible order"
    );
    let actor_candidate = actor_candidate_probe(input, &mask, source_words, filtered_words)?;
    let gpu_oracle = OcclusionOracle {
        source_visible: counters[3],
        survivors: counters[4],
        occluded: counters[5],
        source_meshlets: counters[10],
        submitted_meshlets: counters[11],
        source_vertices: counters[12],
        submitted_vertices: counters[13],
        source_triangles: counters[14],
        submitted_triangles: counters[15],
        source_skin_influences: counters[16],
        submitted_skin_influences: counters[17],
    };
    ensure!(
        gpu_oracle == cpu_oracle,
        "GPU occlusion aggregates {gpu_oracle:?} differ from CPU oracle {cpu_oracle:?}"
    );
    Ok(OcclusionProbe {
        enabled: input.occlusion_enabled,
        history_queried: input.history_queried,
        history_reset_count: input.history_reset_count,
        bypass_reason: input.bypass_reason,
        source_visible: counters[3],
        survivors: counters[4],
        occluded: counters[5],
        tested: counters[6],
        bypassed: counters[7],
        invalid_queries: counters[8],
        overflow: counters[9],
        source_meshlets: counters[10],
        submitted_meshlets: counters[11],
        source_vertices: counters[12],
        submitted_vertices: counters[13],
        source_triangles: counters[14],
        submitted_triangles: counters[15],
        source_skin_influences: counters[16],
        submitted_skin_influences: counters[17],
        visible_record_bytes: VISIBLE_OBJECT_BYTES,
        filtered_visible_bytes: FILTERED_VISIBLE_BYTES,
        order_readback_bytes: FILTERED_VISIBLE_BYTES * 2,
        candidate_mask_sha256: format!("{:x}", Sha256::digest(&mask_bytes)),
        source_order_sha256: hash_words(source_words),
        filtered_order_sha256: hash_words(filtered_words),
        stable_compaction_mismatch_count: 0,
        hierarchy_sha256,
        hierarchy_format: "R32_UINT",
        hierarchy_mip_dimensions: hierarchy
            .iter()
            .map(|mip| [mip.width, mip.height])
            .collect(),
        hierarchy_bytes,
        hierarchy_mismatch_count,
        query_dispatch_count: 1,
        query_groups: OCCLUSION_GROUPS,
        prefix_dispatch_count: 1,
        prefix_groups: 1,
        scatter_dispatch_count: 1,
        scatter_groups: OCCLUSION_GROUPS,
        compaction_dispatch_count: 3,
        hierarchy_dispatch_count: input.resources.occlusion.mip_count,
        gpu_query_ms: milliseconds(4, 5),
        cpu_oracle,
        bound_proof: input.bound_proof,
        actor_candidate,
    })
}

fn actor_candidate_probe(
    input: &ProbeInput<'_>,
    mask: &[u32],
    source_words: &[u32],
    filtered_words: &[u32],
) -> Result<Option<ActorCandidateProbe>> {
    let source_matches = source_words
        .chunks_exact(VISIBLE_OBJECT_WORDS)
        .enumerate()
        .filter(|(_, record)| record[VISIBLE_CANDIDATE_WORD] == ACTOR_CANDIDATE_INDEX)
        .collect::<Vec<_>>();
    let filtered_matches = filtered_words
        .chunks_exact(VISIBLE_OBJECT_WORDS)
        .enumerate()
        .filter(|(_, record)| record[VISIBLE_CANDIDATE_WORD] == ACTOR_CANDIDATE_INDEX)
        .collect::<Vec<_>>();
    ensure!(
        source_matches.len() <= 1 && filtered_matches.len() <= 1,
        "actor GPU candidate appeared more than once"
    );
    let mask_value = mask[ACTOR_CANDIDATE_INDEX as usize];
    let Some(projection) = input.actor else {
        ensure!(
            source_matches.is_empty() && filtered_matches.is_empty() && mask_value == 0,
            "absent actor left GPU candidate evidence"
        );
        return Ok(None);
    };
    let expected =
        ActorVisibleCandidate::from_projection(projection, input.skeletal_settings.time_tick)?;
    let expected_words = expected.words();
    let source = source_matches.first().copied();
    let source_record = source.map(|(_, record)| record);
    let exact_field_mismatch_count = source_record.map_or(0, |record| {
        (0..VISIBLE_OBJECT_WORDS)
            .filter(|word| !matches!(*word, 6 | 9) && record[*word] != expected_words[*word])
            .count() as u32
    });
    ensure!(
        exact_field_mismatch_count == 0,
        "actor GPU visible record differs from the projected candidate"
    );
    ensure!(
        source.is_some() == (mask_value != 0),
        "actor GPU source visibility differs from its occlusion mask"
    );
    ensure!(
        filtered_matches.len() == usize::from(mask_value == 1),
        "actor GPU filtered visibility differs from its occlusion mask"
    );
    if let (Some((_, source)), Some((_, filtered))) = (source, filtered_matches.first().copied()) {
        ensure!(
            source == filtered,
            "actor GPU visible record changed during occlusion compaction"
        );
    }
    let record_words = source_record.map(|record| record.try_into().unwrap());
    let record_sha256 = source_record.map(hash_words);
    let generation = projection.actor.handle.generation();
    ensure!(
        expected_words[VISIBLE_IDENTITY_LOW_WORD] == generation as u32
            && expected_words[VISIBLE_IDENTITY_HIGH_WORD] == (generation >> 32) as u32,
        "actor GPU stable identity does not preserve generation"
    );
    Ok(Some(ActorCandidateProbe {
        candidate_index: ACTOR_CANDIDATE_INDEX,
        generation_decimal: generation.to_string(),
        stable_identity: [generation as u32, (generation >> 32) as u32],
        upload_record_bytes: ACTOR_VISIBLE_RECORD_BYTES,
        semantic_region: projection.semantic_region,
        source_visible: source.is_some(),
        source_visible_index: source.map(|(index, _)| index as u32),
        occlusion_mask: mask_value,
        survived: mask_value == 1,
        filtered_visible_index: filtered_matches.first().map(|(index, _)| *index as u32),
        record_words,
        record_sha256,
        exact_field_mismatch_count,
    }))
}

fn hash_words(words: &[u32]) -> String {
    let mut hasher = Sha256::new();
    for word in words {
        hasher.update(word.to_le_bytes());
    }
    format!("{:x}", hasher.finalize())
}

fn validate_hierarchy(
    winner: &[u8],
    hierarchy: &[super::HierarchyMip],
) -> Result<(String, u32, u64)> {
    ensure!(!hierarchy.is_empty(), "occlusion hierarchy has no mips");
    ensure!(
        winner.len() == hierarchy[0].bytes.len() * 2,
        "winner and hierarchy mip 0 dimensions differ"
    );
    let mut mismatch_count = 0u32;
    for (pixel, depth) in winner
        .chunks_exact(8)
        .zip(hierarchy[0].bytes.chunks_exact(4))
    {
        mismatch_count += u32::from(pixel[..4] != *depth);
    }
    for pair in hierarchy.windows(2) {
        let source = &pair[0];
        let destination = &pair[1];
        ensure!(
            destination.width == (source.width / 2).max(1)
                && destination.height == (source.height / 2).max(1),
            "occlusion hierarchy mip dimensions are invalid"
        );
        for y in 0..destination.height {
            for x in 0..destination.width {
                let mut expected = u32::MAX;
                for source_y in y * 2..(y * 2 + 2).min(source.height) {
                    for source_x in x * 2..(x * 2 + 2).min(source.width) {
                        expected = expected.min(read_depth(source, source_x, source_y));
                    }
                }
                mismatch_count += u32::from(read_depth(destination, x, y) != expected);
            }
        }
    }
    ensure!(
        mismatch_count == 0,
        "occlusion hierarchy contains {mismatch_count} reduction mismatches"
    );
    let bytes = hierarchy
        .iter()
        .flat_map(|mip| mip.bytes.iter().copied())
        .collect::<Vec<_>>();
    Ok((
        format!("{:x}", Sha256::digest(&bytes)),
        mismatch_count,
        bytes.len() as u64,
    ))
}

fn read_depth(mip: &super::HierarchyMip, x: u32, y: u32) -> u32 {
    let offset = ((y * mip.width + x) * 4) as usize;
    u32::from_le_bytes(mip.bytes[offset..offset + 4].try_into().unwrap())
}
