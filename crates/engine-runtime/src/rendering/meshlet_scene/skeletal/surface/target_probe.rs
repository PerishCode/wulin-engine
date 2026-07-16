use anyhow::{Result, ensure};

pub(in crate::rendering::meshlet_scene::skeletal) fn pixel_count(
    visibility: &[u8],
    target: Option<crate::rendering::ObjectTargetFeedback>,
    local_ids: &[Vec<u32>],
) -> Result<u32> {
    let Some(target) = target else {
        return Ok(0);
    };
    let region_ids = local_ids
        .get(target.active_index as usize)
        .ok_or_else(|| anyhow::anyhow!("surface target active index has no identity page"))?;
    let mut physical_indices = region_ids
        .iter()
        .enumerate()
        .filter(|(_, local_id)| **local_id == target.authored_local_id)
        .map(|(index, _)| index);
    let physical_index = physical_indices
        .next()
        .ok_or_else(|| anyhow::anyhow!("surface target authored local ID is absent"))?;
    ensure!(
        physical_indices.next().is_none(),
        "surface target authored local ID is not unique"
    );
    let candidate = target.active_index * 1024 + physical_index as u32;
    Ok(visibility
        .chunks_exact(8)
        .filter(|pixel| {
            let word = u32::from_le_bytes(pixel[..4].try_into().unwrap());
            word != 0 && (word & 0x7fff).wrapping_sub(1) == candidate
        })
        .count() as u32)
}
