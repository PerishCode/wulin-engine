# Canonical acceptance cost and ownership

`runseal :canonical-runtime` is the direct end-to-end integration proof. It is not the sole owner of
every soak multiplier. Long evidence belongs to the narrowest maintained workflow that can fail for
the relevant ownership change.

## Evidence classes

| Evidence                                                       | Full runtime                                                               | Focused owner                                                     |
| -------------------------------------------------------------- | -------------------------------------------------------------------------- | ----------------------------------------------------------------- |
| Correctness, failure rollback, presentation, restart, rollover | Complete                                                                   | Frame, actor, or prototype workflow during iteration              |
| Prototype process behavior                                     | Invalid/corrupt startup, stationary restart, and cleanup checkpoint        | `canonical-prototype`: native input and complete product behavior |
| Reactive and prepared traversal                                | 32 + 32 crossings                                                          | Full runtime                                                      |
| Same-process resources                                         | 4..8 state-driven warm + 8-publication active checkpoint                   | `canonical-resources`: 32 warm + 64 measured + 60-second recovery |
| Process lifecycle                                              | 2 complete checkpoint cycles                                               | `canonical-resources`: 16 complete cycles                         |
| Rendered pixels and semantic attachments                       | Raw color/ID hashes for every assertion; representative persisted captures | Focused workflows persist their owned representative captures     |

The resource checkpoint is not described as a plateau or recovery soak. It first alternates the same
two measured windows for 4..8 publications and takes its baseline after two consecutive samples keep
handle/thread counts equal and add at most 1 MiB of private bytes. This excludes one-time workload
initialization without a fixed wait. It then rejects more than one measured transient handle or 16
MiB of final private-byte growth across eight publications. The deep owner uses 32 fixed warm
publications before the same active policy, then requires six consecutive equal handle samples over
60 seconds and recovery no higher than the warmed handle baseline.

## Cost invariants

- `perception.capture` remains the explicit persistent evidence operation. It performs synchronized
  readback, semantic analysis, PNG/diagnostic encoding, and artifact writes.
- `perception.observe` performs the same synchronized color/object-ID readback, raw hashing, and
  semantic analysis without diagnostic image materialization, encoding, or filesystem output.
- Repeated equality/change assertions use raw color and object-ID hashes. PNG and diagnostic PNG
  hashes are deterministic encodings of those bytes and are retained only in representative capture
  gates.
- A frame assertion performs one canonical probe. First-frame and immediate replay equality in the
  focused frame workflow is the warm/stability witness; an unobserved first probe is not repeated.
- Capture collections are cleared before a workflow so artifact counts and bytes cannot include
  stale evidence.

The full report records total and per-stage wall time, Sidecar/event/process operation counts, and
artifact file/byte totals. Cost growth is therefore reviewable as an acceptance regression rather
than inferred from a static window.

## Gate selection

- Use `canonical-frame` for renderer, committed query, and exact attachment changes.
- Use `canonical-actor` for retained actor, simulation admission, or actor GPU changes.
- Use `canonical-prototype` for host/input/application-loop behavior.
- Add `canonical-resources` when GPU/source/cache ownership, resource lifetime, process teardown, or
  long-soak behavior changes.
- Use `canonical-runtime` for integration boundaries, stage seals, and explicit end-to-end proof.

No wrapper invokes another wrapper. A focused result is not silently cached or reused across a
different worktree; segmentation changes evidence ownership, not freshness semantics.

## Measured maintenance result

The 2026-07-16 accepted baseline took 807.8 seconds and persisted 38 captures / 175,174,275 bytes.
The optimized `canonical-runtime-v2` completed in 262.161 seconds while retaining 32 reactive and 32
prepared crossings, all correctness/failure/restart/rollover gates, 113 probes, 32 raw observations,
an 8-publication/four-sample resource checkpoint, and two lifecycle checkpoints. It persisted six
representative captures / 24 files / 25,346,280 bytes. That is a 67.5% wall-time and 85.5%
artifact-byte reduction.

The separated `canonical-resources-v2` deep owner completed in 387.364 seconds. After 32 warm and 64
measured publications, handles remained within the 531 + 1 active allowance and recovered to 516;
private bytes recovered from the 413,630,464-byte warm baseline to 412,565,504. Sixteen of sixteen
lifecycle cycles left all Sidecar namespaces empty. The deep duration is intentionally excluded from
routine full acceptance and selected only for resource/lifecycle ownership or an explicit soak.

The 0096 integration first exercised the routine checkpoint from a process that had not yet used its
alternating measured windows. Sampling before that lazy workload initialization produced mutually
inconsistent handle/thread and private-byte failures. The v5 checkpoint now uses the bounded
state-driven warm policy above without widening either measured allowance. The final worktree passed
after five warm publications and eight measured publications in a 251.987-second full run:
handles/threads remained 492/21 and final private bytes were 503,808 below baseline. Nine pure
injected tests maintain the warm,
active, and recovery decisions in the repository guard.
