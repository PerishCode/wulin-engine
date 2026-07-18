# ADR 0162: Retired Object-Action Readiness Baseline

- Status: Accepted
- Date: 2026-07-19
- Experiment: 0159 Retired Object-Action Readiness Baseline

## Context

Prototype acceptance launched a readiness-only `objectActionBaseline` process
after writing the Rejected object-action config. The process performed no native
action, reached readiness, and was forcibly terminated.

Its 5,787-byte report and 3.540-second launch had exactly two consumers:
`feedbackSessionInvariant` compared its startup and actor projections with the
Rejected launch. The v73 projections were exactly equal. The actor projection
already validates its raw launch directly; the startup projection describes the
schema-2 document authored and written by the acceptance host.

## Decision

- Delete the object-action readiness-only baseline launch and report field.
- Remove both baseline parameters from object feedback gates.
- Serialize each schema-2 document through one host helper and derive its exact
  UTF-8 byte count and SHA-256 from those same bytes.
- Validate each object-action launch directly against the expected startup
  revision, mode, config path, source paths, signed origin/center, active radius,
  singleton playable bounds, byte count, and hash.
- Continue validating actor authority directly against the expected center.
- Keep the two first/restart readiness captures and all 19 graceful launches.
- Make the existing Prototype session guard reject the retired spelling and
  require deterministic startup expectation ownership while remaining exactly
  500 lines.
- Advance complete Prototype acceptance directly to v74.
- Add no replacement process, alias, decoder, fallback, optional report branch,
  relaxed behavior gate, product delay, product behavior, Runtime/GPU/source/
  synchronization owner, or workspace resource cleanup.

## Consequences

- Object-action startup evidence is tied directly to the bytes the host writes,
  rather than to a second process that happens to read the same document.
- One forced readiness process and one raw report branch disappear.
- Activated and Rejected retain independent raw readiness and graceful
  completion evidence.
- First/restart still prove process replacement and readiness reproducibility.
- Experiment 0159 performs no resource cleanup; the scheduled
  compatibility/resource boundary remains Experiment 0160.

## Evidence

`canonical-prototype-v74` passed first-run in 163.148 seconds. Its 391,418-byte
report is 11,182 bytes (2.78%) smaller than the 402,600-byte v73 baseline and
contains no object-action baseline field.

Independent reconstruction of the two host documents produced exact 494-byte
inputs and matched the reported SHA-256 values:

- Activated:
  `3efbe45824d7b1358588796acdee19a8b641850b5ae1ddd98ad6797ff8642f71`;
- Rejected:
  `90033862def0734932b606ffea9ef1ab58f9ceb91e4ad3704df733739d3fd7b5`.

The two first/restart readiness captures remained distinct. All 19 graceful
launches retained unique PIDs, exit zero, one native-input value, exactly two
outputs, and empty stderr/trailing output. The fixed 19-pair non-duplication
gate remained at zero, and all Activated, Rejected, sustained-capacity, and
finite-boundary behavior gates passed.

All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed. The
central guard remained exactly 500 lines, and Flavor retained zero denies and
five existing warnings.
