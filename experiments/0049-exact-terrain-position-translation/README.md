# Experiment 0049: Exact Terrain Position Translation

Status: Accepted

## Hypothesis

One canonical signed-region/local-Q9 terrain position can translate by caller-supplied planar Q9
displacement with unique half-open normalization, exact positive and negative seam crossing,
partition and round-trip invariance, and transactional signed-region overflow, without query-
specific identity, floating point, terrain sampling, locomotion policy, actor storage, or live time.

## Scope

This experiment directly replaces `TerrainQueryPosition` with query-neutral `TerrainPosition` as
the one horizontal identity already consumed by terrain queries, body contact, and fixed vertical
motion. It adds one checked `translated_q9` operation over signed X/Z displacement. Existing JSON
shape and terrain query/contact/motion behavior remain unchanged; no alias, deprecated name, or
parallel coordinate type survives.

The local interval remains `[-4096, 4096)` at denominator 512, so a 16-meter terrain region spans
exactly 8192 Q9 units. Translation normalizes each axis with Euclidean division and applies the
resulting region deltas through the accepted checked signed-`i64` region authority.

Composing planar displacement with terrain-body contact, horizontal velocity/acceleration, step
height, slope/material response, collision, input mapping, camera follow, live schedule driving,
actor identity/storage, rendering, and gameplay tuning are out of scope.

## Workload

1. Move position ownership into a focused module and rename the type/constants directly. Update
   all query, contact, motion, workbench, and probe consumers without compatibility aliases.
2. Prove exact zero, single-unit positive/negative seam, diagonal seam, multi-region, inverse, and
   partitioned translations at origin and far signed regions.
3. Run a test-only deterministic 65,536-case sweep across the full local domain, mixed signed
   displacement, both axes, and far regions. Compare every output with an independent `i128`
   absolute-lattice oracle and record one stable result SHA-256.
4. Exercise `i64::MIN`/`i64::MAX` region boundaries and require explicit error with unchanged input
   whenever either axis cannot be represented.
5. Run focused Rust tests and `runseal :guard`. Do not add a runtime probe or repeat the canonical
   GPU/lifecycle workflow for a pure CPU value operation whose serialized boundary is unchanged.

## Controlled Variables

- `TERRAIN_POSITION_DENOMINATOR = 512`, local minimum `-4096`, exclusive maximum `4096`, and region
  side `8192` Q9 units are fixed.
- Per-call X/Z displacement is signed `i32` Q9. It is exact caller input, not velocity, acceleration,
  or an input-action mapping.
- For one axis, `local + delta` is normalized into the unique half-open interval using Euclidean
  quotient/remainder; the quotient is the signed region delta.
- Both normalized axes must be representable before a new value is returned. Failure mutates no
  runtime or caller state because positions are copied values.
- The independent oracle maps the canonical pair to an `i128` lattice coordinate, adds displacement,
  and reconstructs the unique region/local pair separately from the implementation formula.

## Metrics

- Focused test count; exact seam, multi-region, inverse, partition, and overflow assertions.
- Successful sweep count, mismatch count, signed-region coverage, local-domain coverage, result
  SHA-256, replay SHA-256, and elapsed CPU time.
- Existing query/contact/motion test results and `runseal :guard` ownership, protocol, compilation,
  test, and Flavor gates.

## Acceptance Criteria

- Exactly one public `TerrainPosition` type and neutral constant vocabulary remain; no old alias or
  conversion layer exists.
- Every successful translation returns the independent oracle's unique signed-region/local-Q9
  representation. Positive, negative, diagonal, and multi-region crossings are exact.
- Valid translation partitions and inverse round trips end at the same canonical position.
- Either-axis signed region overflow fails explicitly and returns no partial position.
- The 65,536-case sweep has zero mismatch and exact replay hash. No runtime probe, allocation, I/O,
  wall-clock sampling, terrain query, frame, or GPU work is introduced.
- Focused tests and `runseal :guard` pass. Experiment 0048 remains the current process-motion
  evidence and Experiment 0047 remains the current full frame/GPU/lifecycle evidence.

## Reference Environment

The experiment uses the repository-pinned Rust toolchain. Translation is a pure CPU value operation
and has no operating-system, source-pack, process, renderer, or GPU dependency.

## Evidence

The focused `engine-runtime`/`workbench` suite passed all 37 tests. Four position tests cover zero,
single-unit positive/negative and diagonal seams, multi-region translation, far signed regions,
valid partitions, inverse round trips, and both `i64` overflow directions.

The test-only sweep executed 65,536 successful translations twice. It covered every local X value
eight times, a full-domain permuted local Z value, five origin/far/boundary-safe region pairs, and
full-width deterministic signed `i32` displacement on both axes. Every result matched the
independent `i128` absolute-lattice oracle. Mismatch count was zero; result and replay SHA-256 were
both `8bf1a9181426aadf6970009165f1269e9358463c58e2cca734435a5bc02ff683`. The measured first sweep
took 107,882,300 ns and the two-sweep test completed in 0.21 seconds.

The old `TerrainQueryPosition` and `TERRAIN_QUERY_POSITION`/`TERRAIN_QUERY_LOCAL` symbols have no
live code references. Query, contact, fixed motion, renderer, workbench, and probe consumers all use
the one `TerrainPosition` value without an alias. `runseal :guard` passed in 11.1 seconds with zero
Flavor denies and all repository suites green.

No process or canonical GPU/lifecycle workflow was run. Translation is a copied CPU value operation,
adds no runtime diagnostic, and preserves the existing serialized terrain position shape and all
frame/runtime behavior. Experiment 0048 remains the current process-motion evidence and Experiment
0047 remains the current full frame/GPU/lifecycle evidence.
