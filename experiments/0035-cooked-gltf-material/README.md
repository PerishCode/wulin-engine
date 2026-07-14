# Experiment 0035: Cooked glTF Material

Status: Accepted

## Hypothesis

The pinned Fox base-color texture and PBR material can be cooked offline into one reserved
surface-catalog material/layer, producing authored orange/white/dark appearance through the
existing surface resolve without runtime glTF/PNG parsing, a new descriptor shape, or a
second shader path.

## Scope

This experiment verifies the single Fox material and 1024x1024 RGB PNG already pinned by
Experiment 0034. Build-time cooking reduces it to the existing 64x64 material texture size,
generates the complete seven-mip chain, and publishes it as material/layer 63. Materials 0
through 62 remain deterministic fixtures. The imported presentation profile explicitly
authors archetype 7 and material 63 together while retaining its existing yaw and animation.

Multiple materials or textures, alpha modes, normal/metallic-roughness/emissive maps, color
space conversion, texture transforms, sampler variants, runtime PNG/glTF loading, source
skeletons/clips, and Wulin content are out of scope.

## Workload

1. Verify the pinned glTF and PNG hashes, one material/texture/image/sampler join, UV set 0,
   default white base-color factor, metallic 0, roughness 0.58, and linear/mipmapped sampler.
2. Decode exactly one 1024x1024 8-bit RGB PNG at build time. Reject any structural, format,
   dimension, hash, or material mismatch.
3. Deterministically box-reduce the source to 64x64 RGBA8, generate six further 2x2 box mips,
   and encode a strict canonical payload with source/cooked hashes and material factors.
4. Decode only the canonical payload in `surface-catalog`, replace layer/material 63, and
   retain the accepted 64-layer, seven-mip GPU texture resource and descriptor contract.
5. Cook an imported object presentation, require archetype 7/material 63 authority, and run
   exact GPU/CPU UV, texel, color, visibility, occlusion, attachment, and capture oracles.
6. Re-run time, physical reorder, hold/failure, traversal, rollover, resource plateau,
   lifecycle, and repository guard gates through the direct workflow.

## Controlled Variables

- Geometry payload, meshlet LODs, rigid root binding, imported conservative bound, signed
  schema-3 object authority, grounding, composition, and presentation time remain unchanged.
- The existing texture array remains 64x64, 64 layers, and seven mips. No shader branch,
  descriptor, root signature, resource lifetime, dispatch, or submission count may change.
- Material selection remains explicit presentation authority. The imported cooker profile
  authors material 63 rather than deriving a material from stable identity or archetype.
- PNG and glTF parsing remain build-time only; runtime code sees canonical RGBA8 mip bytes.

## Metrics

- Source JSON/PNG and cooked payload hashes; source/cooked dimensions, formats, byte counts,
  material factors, per-mip hashes, and exact mip sizes.
- Surface catalog hash/GPU bytes, texture-array layers/mips, imported material/layer index,
  and fixture-material hash stability for layers 0 through 62.
- GPU/CPU material index, UV, texel, RGBA, surface sample delta, capture/attachment hashes,
  and fixed dispatch/resource shape.
- D3D12 device state, handle/private-byte plateau, traversal behavior, and process cleanup.

## Acceptance Criteria

- The exact source deterministically cooks to one canonical texture/material payload. Any
  source, glTF join, PNG shape, material-factor, or payload mismatch fails construction.
- Mips are exactly 64, 32, 16, 8, 4, 2, and 1 pixels per side with one RGBA8 layer each;
  repeated builds are byte-identical and every fixture layer remains unchanged.
- Imported presentation explicitly selects archetype 7 and material 63. Identity, position,
  grounding, contact, yaw, animation, publication, and fixed submission remain exact.
- GPU and CPU sample the same source-cooked texels with zero channel error. The imported
  capture changes from Experiment 0034's diagnostic material and visibly contains the Fox
  source palette rather than arbitrary per-object material colors.
- The runtime binary has no PNG/glTF parsing path or source-path dependency, and the GPU
  texture array, descriptor count, root signature, and dispatch counts remain unchanged.
- All Experiment 0034 regressions, resource bounds, lifecycle cleanup, and guard checks pass.

## Evidence

The direct workflow remains:

```powershell
runseal :canonical-runtime
```

Generated evidence remains ignored under
`out/captures/0035-cooked-gltf-material/`.

## Results

The direct workflow passed in 484.7 seconds. The pinned 1024x1024 RGB source texture cooked
to 64/32/16/8/4/2/1 RGBA8 mips of 16,384, 4,096, 1,024, 256, 64, 16, and 4 bytes. The
canonical payload hash was
`5c18b4a6c9f13f79d5b6714ece3d0ef3e4ee20c181b1a169c7eb6a8392e41f0c`; all source,
payload, and per-mip joins were exact. The surface catalog hash became
`4267365c1d71e96beaff2ece04d6a94c450fd86131ebe65c2447c7b95cb8c15d`, while its GPU
byte count remained exactly 1,500,416 and fixture layers 0 through 62 retained hash
`3f6256268867bf270268e7478145e12ab7e3216612b550cd1a412bf440357c8b`.

The imported frame observed only material-mask bit 63. All six visible surface samples
reported material 63 and exact CPU texels with zero maximum channel delta. Geometry workload
remained the Experiment 0034 shape: 10,538 visible instances, 144,605 meshlets, and 5,082,336
triangles. The authored-material color and PNG hashes were
`6b1be501bb75c2252f4a5cff1ab91d855fa92090639426d8e7a7236fcdb652f9` and
`ee7bd32c011902078c702a128d89414730fd902d7b8abd310e7fce6805152e60`, distinct from the
Experiment 0034 diagnostic-material capture.

Manual inspection confirmed aligned orange bodies, white bellies and tail tips, and dark
regions across recognizable Fox heads, ears, legs, bodies, and tails. Invalid payload/query,
hierarchy mismatch, and stable-compaction mismatch counts were zero. All time, source reorder,
hold/failure, restart, prepared rollover, 32 reactive crossings, and 32 prepared crossings
passed without changing resource or dispatch shape.

The 64-publication baseline was 531 handles and 396,681,216 private bytes. Peak handles stayed
at 531; the final sample had 516 handles and 397,053,952 private bytes. All 16 lifecycle cycles
left no process descendant or device removal.

## Conclusion

Accepted. The pinned Fox material and base-color texture now cross a verified offline cook
boundary into the unchanged fixed surface texture array. Presentation explicitly selects the
imported material, and GPU/CPU sampling plus visual evidence prove authored appearance without
runtime glTF/PNG parsing.

## Promotion

Promoted the verified single-material glTF join, deterministic PNG/mip cook, strict canonical
decode, reserved material/layer 63, and imported presentation pairing. ADR 0038 records the
durable boundary. General PBR maps/materials, color management, runtime texture streaming,
source skeletons/clips, and Wulin content remain later gates.
