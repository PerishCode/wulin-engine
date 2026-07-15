# Experiment 0055: Mandatory Terrain-Transaction Cleanup

Status: Accepted

## Hypothesis

After retained read-compute-commit is accepted, the three caller-owned body step/translate/advance
inspect surfaces and recurring support gates can be deleted as one complete history chain while
their pure modules/tests remain the retained transaction's implementation authority. Canonical
setup can move to one typed support owner so the temporary 520-line wrapper exception is removed,
without changing runtime behavior or adding compatibility aliases.

## Scope

Delete `canonical.terrain.body.step`, `.translate`, and `.advance`; their protocol variants,
payload decoders, workbench branches, public `Runtime` forwarders, three Runseal support files, and
live-wrapper calls/evidence. Retain `terrain_query::{motion, translation, advance}` and all focused
tests because the current retained advance directly composes them.

Move canonical test/build startup, deterministic source cooking, source-identity assertions, and
controlled corruption setup verbatim from the wrapper into one typed `canonical-setup` support
function. The wrapper remains the only entry point and keeps correctness, failure, temporal,
movement, rollover, traversal, resource, and lifecycle gate order visible.

No simulation, time, input, body-capacity, physics, rendering, content, or mod feature is added.

## Workload

1. Delete all three inspect/parser/dispatch/runtime-forwarding/support chains without aliases,
   deprecated names, fallback routing, or hidden modes.
2. Add an explicit forbidden-symbol/file guard and require all three verbs to return
   `unknown_event` through the live workbench.
3. Extract canonical setup into one typed support owner and require its returned paths/storage
   evidence to include every current source, variant, corruption target, and report path.
4. Remove the wrapper 520-line Flavor override and pass the default source-length gate while the
   wrapper remains direct and non-recursive.
5. Run the extracted setup directly, a short idle-shell retired-verb/current-retained-surface gate,
   focused runtime/workbench tests, `runseal :init`, and `runseal :guard`.
6. Do not execute the long GPU/lifecycle workflow: production frame/GPU/resource code is unchanged,
   and setup plus live control removal have narrower direct gates.

## Controlled Variables

- Pure terrain motion/translation/advance modules, ordering, fixed-point contracts, public result
  types, and focused tests remain unchanged.
- Retained spawn/read/despawn and retained advance verbs remain the only live body mutation surface.
- Extracted setup preserves package lists, build command, center list/order, all cooker profiles,
  source-identity comparisons, corruption regions, paths, and storage evidence keys.
- The wrapper invokes no historical wrapper and retains one linear acceptance order.
- Historical experiment/ADR evidence keeps old verb names as decision history; live code/support and
  forbidden runtime surfaces do not.

## Metrics

- Deleted files, variants, parser/dispatch branches, Runtime forwarders, and wrapper gate/evidence
  entries.
- Wrapper line count under the default Flavor limit and absence of its temporary override.
- Direct setup outputs/source identity/corruption evidence and elapsed time.
- Live `unknown_event` responses for all retired verbs plus successful retained lifecycle/route
  checks.
- Focused/all-repository test counts, Flavor denies, and `runseal :guard` result.

## Acceptance Criteria

- No retired verb, variant, forwarding method, error prefix, gate function, or support file remains
  in live apps/runtime/wrappers/support. A stable guard prevents restoration.
- Pure transaction modules/tests still pass and retained advance remains the sole current consumer.
- Extracted setup returns the same complete path and storage shape and passes source-identity and
  controlled-corruption checks when invoked directly.
- The temporary 520-line override is absent; the direct wrapper passes default Flavor rules.
- Short process gates, `runseal :init`, and `runseal :guard` pass. No long canonical run is charged
  to this control/setup-only cleanup.

## Reference Environment

The experiment uses the repository-pinned Rust/Deno toolchains and reference Windows workbench.
Generated setup artifacts remain ignored under `out/`.

## Evidence

The complete caller-owned diagnostic chain was removed without aliases: three inspect verbs,
protocol variants/payloads, workbench dispatch branches, public `Runtime` forwarding methods,
wrapper gates/evidence fields, and three dedicated support files. The support deletion alone
removed 1,018 physical lines. Pure motion, translation, and planar-first modules remain private to
the retained transaction and all 56 focused engine-runtime tests pass.

`prepareCanonicalSetup` completed the selected tests/build and produced all eleven deterministic
pack/corruption artifacts. Its existing object source-identity and controlled corruption assertions
passed. The direct wrapper shrank from 508 to 405 physical lines (482 to 382 measured source lines),
so the temporary 520-line Flavor override was deleted and the default rule passes.

The stable guard rejects the removed files, variants, verbs, Runtime forwarders, error prefixes,
and gate names. A single fresh-process control gate completed in 10.74 seconds: all ten retired
verbs returned `unknown_event`; retained spawn/read/despawn remained exact; stale retained advance
and a negative step-up limit failed before terrain lookup and preserved byte-identical stored state.
Focused Rust/Deno/Flavor checks, `runseal :init`, and `runseal :guard` passed.

The approximately ten-minute canonical workflow was not run. No production frame, renderer, GPU
resource, synchronization, or lifecycle owner changed, and the modified setup/control boundaries
have direct short gates. Their checks remain in the live wrapper for the next candidate that can
legitimately invalidate full canonical evidence.
