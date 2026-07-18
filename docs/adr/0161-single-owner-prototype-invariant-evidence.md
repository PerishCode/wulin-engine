# ADR 0161: Single-Owner Prototype Invariant Evidence

- Status: Accepted
- Date: 2026-07-19
- Experiment: 0158 Single-Owner Prototype Invariant Evidence

## Context

The v72 Prototype report paired each of 19 complete graceful launch values with
an invariant summary. Although the invariant builders validated raw values
before returning, many summaries then copied the same structured values into a
second report branch.

A recursive audit found 178 exact copied object/array subtrees totaling 20,423
minified bytes: 141 in the 16 session pairs, 14 each in Activated and Rejected
feedback, and 9 in finite-boundary evidence. These included clocks, object
states, camera rigs, messages, timing arrays, identities, simulation commands,
and retained-capacity payloads. The copies were not independent observations.

## Decision

- The graceful launch report is the sole owner of complete raw execution
  evidence.
- Paired invariants may expose only facts derived after validating that raw
  evidence: booleans, counts, scalar measurements, and purpose-specific
  projections.
- Apply one recursive pairwise runtime gate to the 16 sessions, Activated,
  Rejected, and finite-boundary evidence.
- Reject any invariant object/array subtree that exactly equals one from its raw
  launch and has a minified UTF-8 size of at least 16 bytes.
- Keep the threshold fixed and publish one aggregate revision, launch count,
  threshold, and zero-copy fact.
- Require the gate and aggregate fact from the existing 500-line Prototype
  session guard; add no guard module.
- Advance complete Prototype acceptance directly to v73.
- Add no alias, decoder, fallback, dual write, optional report-version branch,
  relaxed behavior threshold, process, product behavior, Runtime/GPU/source/
  synchronization owner, or workspace resource cleanup.

## Consequences

- Full readiness, completion, native-input, camera, object, clock, and
  simulation values have one unambiguous report owner.
- Invariants remain directly useful for acceptance without masquerading as a
  second raw authority.
- Exact copied object/array payloads cannot silently return above the fixed
  threshold.
- Scalars and deliberately narrow projections remain available without treating
  common constants as ownership violations.
- Experiment 0158 performs no resource cleanup; the scheduled
  compatibility/resource boundary remains Experiment 0160.

## Evidence

`canonical-prototype-v73` passed in 167.176 seconds. Its 402,600-byte report is
39,285 bytes (8.89%) smaller than the 441,885-byte v72 baseline.

An independent recursive audit of all 19 pairs found zero exact copied
object/array subtrees at or above 16 minified UTF-8 bytes. The report publishes
revision `prototype-single-owner-invariant-evidence-v1`, launch count 19,
threshold 16, and copied count zero.

All 19 graceful launches used unique PIDs, exited with code zero, owned one
native-input value, emitted exactly two structured values, and retained empty
stderr/trailing output. Activated, Rejected, sustained-capacity, and
finite-boundary gates preserved their exact behavior evidence.

All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed. The
session transport remained at 493 lines, the central guard remained exactly 500
lines, and Flavor retained zero denies and five existing warnings.
