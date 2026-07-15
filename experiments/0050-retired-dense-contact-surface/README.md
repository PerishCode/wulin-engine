# Experiment 0050: Retired Dense Contact Surface

Status: Accepted

## Hypothesis

The accepted one-time 230,400-body terrain-contact proof can leave the live runtime and canonical
workflow completely while exact direct contact and the 225-body transition witness retain current
behavior and hashes, reducing executable history and recurring validation work without a
compatibility alias, fallback, new capability, or GPU/lifecycle change.

## Scope

This mandatory cleanup experiment removes the dedicated `canonical.terrain.contact.probe` inspect
verb, workbench dispatch, runtime/renderer methods, dense coverage branch, conversion helper, and
recurring support assertions. The generic composition probe becomes witness-only rather than
parameterized between current regression and historical acceptance coverage.

Experiment 0046's dense counts, timings, and hashes remain immutable decision history in its report
and ADR 0049. The public exact contact transaction, three direct classification gates, invalid-input
gates, fixed terrain motion, terrain query density, and generic 225-body witness remain live.

No position, contact, motion, time, actor, input, rendering, or gameplay capability may be added.

## Workload

1. Delete the dedicated inspect-to-renderer dense call chain and all 230,400/Dense mode branches.
   Collapse generic terrain query/contact probing to one 225-body witness contract.
2. Remove the recurring dense request/assertions/result from terrain contact acceptance support.
   Keep unavailable, malformed/invalid, overflow, direct separated/touching/penetrating, and
   canonical witness validation unchanged.
3. Add the retired verb to compatibility-removal evidence and add a stable guard against the old
   verb, Rust variants/methods, conversion helper, and dense count returning in live code.
4. Run focused tests and a short real-process gate. Require the retired verb to return
   `unknown_event`, direct exact contact to retain all three classes, and the generic probe to retain
   the accepted 225-body counts and hashes after a fresh publication.
5. Run `runseal :guard`. Do not run the long canonical GPU/lifecycle workflow: this deletion removes
   an explicit CPU diagnostic path and does not change frame, renderer, resource, or lifecycle
   execution. The compatibility and witness gates remain wired into the live wrapper for its next
   legitimately required full run.

## Controlled Variables

- Exact contact types, arithmetic, error behavior, and public `Runtime::resolve_terrain_contact`
  remain unchanged.
- Generic probe query sampling remains 76,800 points. Contact sampling remains exactly the three
  foot offsets at one controlled cell in each of 25 active regions: 225 bodies total.
- Witness revision, class/correction counts, result hash, identity hash, and mismatch behavior are
  fixed to the accepted Experiment 0046 regression evidence.
- Historical dense evidence remains documentation only. No hidden command, feature flag, alternate
  mode, alias, or fallback preserves executability.
- Retired inspect requests fail through the ordinary `unknown_event` contract before and after
  content publication.

## Metrics

- Deleted live symbol/branch/file-line counts and forbidden-symbol scan results.
- Direct contact class/correction results and failure outcomes.
- Witness body/class/correction/mismatch counts plus result and identity SHA-256 before/after.
- Short process-gate elapsed time and `runseal :guard` outcome.

## Acceptance Criteria

- No live `canonical.terrain.contact.probe`, `CanonicalTerrainContactProbe`,
  `terrain_body_contact_probe`, `BodyContactCoverage`, dense conversion helper, or 230,400-body
  branch remains in apps, crates, or current terrain support.
- The retired request returns `unknown_event`; no compatibility response or redirect exists.
- Direct exact contact retains separated/touching/penetrating semantics and zero non-CPU work.
- The generic witness remains exactly 225 bodies, 75 per class, 75 corrections, zero mismatch, and
  retains accepted result/identity hashes through a fresh process publication.
- Focused tests, the short process gate, and `runseal :guard` pass. No full canonical run is charged
  to a deletion that cannot invalidate frame/GPU/lifecycle evidence.

## Reference Environment

The experiment uses the repository-pinned Rust toolchain and reference Windows workbench. The
removed and retained contact probes are CPU-only; the short process gate uses the current committed
terrain snapshot but performs no capture or lifecycle soak.

## Evidence

The dedicated path was removed end to end: one inspect variant/verb, one workbench dispatch, one
runtime method, one renderer/composition method, the two-mode coverage enum, dense revision/count
branches, the conversion helper, and recurring TypeScript request/assertions. The existing live
files lost 98 lines. The simplified witness path and compatibility call added nine lines, while the
new isolated recurrence guard added 43, for a net reduction of 46 executable/support lines.

The focused `engine-runtime`/`workbench` suite passed all 37 tests. The short fresh-process gate
passed in 9,333.8 ms:

- `canonical.terrain.contact.probe` returned `unknown_event` both before and after publication;
- the generic witness remained exactly 225 bodies, 75 in each class, 75 corrections, and zero
  mismatch;
- witness result SHA-256 remained
  `2cd0d7110b580d58d3835f38e44a77ed3339ba028225e6cf7e2a5da590464306` and identity SHA-256
  remained `16446f145eecf59e79dda0a30f8193bf9240b5d93103b95c7c0c6c4aa7e15c9a`;
- direct penetrating/touching/separated requests retained corrections 1/0/0 and zero allocation,
  I/O, GPU, fence, or synchronization work.

The live forbidden-symbol scan covers the retired verb, Rust variant/method/mode/helper names, and
the old dense count across apps, crates, wrappers, and current terrain support. `runseal :guard`
passed in 4.6 seconds with zero Flavor denies and all repository suites green. The live canonical
wrapper now identifies Experiment 0050, retains compatibility rejection plus the bounded witness,
and no longer invokes dense contact evidence.

No full canonical workflow was repeated. The deletion removes 76,800 duplicate terrain queries and
230,400 duplicate contact resolutions per full run but changes no frame, renderer, GPU resource, or
lifecycle execution. Experiment 0047 remains the current full GPU/lifecycle evidence; the next
legitimately required full run will exercise the compacted workflow.
