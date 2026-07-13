# ADR 0026: Signed Terrain Storage

- Status: Accepted
- Date: 2026-07-13
- Supersedes: None
- Superseded by: None

## Context

ADR 0025 accepted camera traversal inside one frozen signed alias window, but terrain
format V1 still selects payloads by bounded local region ID. Rebinding an origin could
therefore associate the same signed region with different cooked content. Automatic
alias rollover would preserve cache identity while silently changing the world.

Experiment 0023 tests whether cooked terrain and GPU residency can instead use exact
signed region keys and immutable source identity while preserving the bounded local
placement, renderer, and format-V1 paths.

## Decision

- Terrain pack V2 has a distinct magic and version. Its canonical sorted index contains
  signed `i64 (x,z)` keys, fixed payload sizes and offsets, flags, and payload SHA-256.
- The SHA-256 of the complete header and index is the source namespace. Because the
  index includes every key and payload hash, a caller cannot substitute source identity.
- Each V2 payload identifies its signed region in its first 16 bytes while retaining the
  accepted 4 KiB size and height/material offsets. Readers verify index-to-payload key
  binding, checksum, canonical padding, offsets, and signed shared edges.
- V2 is written only by the offline terrain cooker. Generation uses signed integer
  lattice arithmetic and remains deterministic at zero and plus/minus 2^40 regions.
- The background reader resolves V2 requests by exact signed key. It decodes a CPU tile
  projected to the current bounded local ID, but uploads the canonical payload bytes.
- V2 terrain cache identity is `{source namespace, signed global region}`. Local region
  IDs are active placement and semantic projection only; physical slots are not identity.
- Publication remaps retained canonical slots into the requested local assignment at the
  frame boundary. Cache capacity remains 50, active capacity remains 25, and I/O/copy
  capacities remain one.
- Format V1 retains its accepted alias-based source and cache behavior. V2 rejects local
  scheduling and composition before reservation because those owners do not yet carry
  canonical global terrain content.
- Missing keys, malformed metadata, payload corruption, and unsupported modes roll back
  before GPU copy or committed cache mutation.

## Consequences

The same 25 signed regions can be rebound from local center 64 to 96 while retaining the
same physical slots and canonical content with zero reads and zero uploads. Adjacent
movement reads and uploads only five 4 KiB payloads. Equal signed keys from a different
source namespace miss all 25 entries, while returning to a still-resident source hits
its own entries.

The renderer, shaders, active mapping, stable semantic IDs, and fixed submission remain
bounded and local. Exact CPU/GPU terrain oracles pass at each alias, but compensated
cross-alias color and object-ID attachments are not byte-invariant because terrain still
uses absolute local `f32` positions and local semantic IDs.

This decision makes signed terrain content and residency honest. It does not accept V2
composition, cooked global objects, automatic rebase, camera-relative terrain
transforms, global semantic indirection, authored world partitioning, prefetch,
collision, navigation, networking, or an unbounded map.

## Evidence

- [Experiment 0023](../../experiments/0023-signed-terrain-storage/README.md) records
  deterministic V2 cooking, exact signed lookup, namespace isolation, alias rebinding,
  adjacent movement, holds, rollback, restart, attachments, and release distributions.
- Experiment 0022 and its recursive compatibility chain passed unchanged.
- Across 32 release adjacent moves, each transaction retained 20 entries, read and
  uploaded 20,480 bytes, and used one fixed terrain submission. Across 32 alias
  rebinds, each transaction retained 25 entries with zero I/O and upload bytes.

## Reproduction

```powershell
runseal :signed-terrain-storage
```

The command writes the ignored report to
`out/captures/0023-signed-terrain-storage/acceptance.json`.
