# Experiment 0084: Normalized Host Input Edges

Status: Accepted

## Hypothesis

The host can expose exact sample-scoped keyboard press and release facts directly from its
existing normalized transition transaction, allowing one-shot application actions without
duplicating native repeat/focus state or changing continuous held-key policy, journal bytes,
simulation sampling, or engine-runtime ownership.

## Scope

Extend `reference-host::HostInput` with bounded pressed and released key sets for the most recent
`ingest` call. Expose `was_pressed` and `was_released` alongside `is_held`. Every ingest clears the
prior sets first, including an empty native drain; empty drains remain absent from the journal and
counters.

Derive edges only from accepted normalized transitions. A key may be present in both sets when it
is pressed and released within one drain, while held state reflects the final transition. Repeated
downs, unmatched ups, and invalid keys create no edge. Focus loss creates releases for previously
held keys. Replay remains isolated from all live key state.

Leave the diagnostic status, canonical v1 journal, and held-state hash domains unchanged. An
inspect round trip is not an edge consumer because a later empty host ingest may correctly expire
the sample before the request is handled. Migrate only prototype Escape from held state to the
press edge; W/A/S/D remains continuous held-state locomotion.

Do not add an action mapper, buffered event queue, frame or fixed-step binding, persistent replay,
mouse/gamepad/text transport, camera behavior, gameplay interaction, engine-runtime input API,
compatibility decoder, or new permanent operator.

## Workload

1. Sweep focused input samples across empty drains, duplicate and unmatched messages, invalid
   keys, focus loss, same-drain down/up and up/down pairs, and key zero queries.
2. Record a normalized stream, establish current live edges, replay it, and require replay to
   preserve held, pressed, and released sets exactly.
3. Through the real workbench Win32 procedure, retain the exact controlled native record/restart
   hashes and counters. Do not latch expired edge state for inspect visibility.
4. Launch the real prototype to readiness, post one native Escape key down through its window, and
   require a clean application-owned exit. Continuous locomotion still consumes `is_held`.
5. Run focused reference-host/prototype tests and the merge-checkpoint repository guard.

## Controlled Variables

- Win32 capture, message order, virtual-key range, repeat suppression, unmatched-up suppression,
  ascending focus cleanup, held-state mutation, and bounded recording capacity remain unchanged.
- A journal transaction remains the normalized result of one non-empty queue drain. Empty ingests
  clear only sample edges and do not create journal data or counters.
- Canonical stream and held-state hash byte domains remain v1, so identical recorded transactions
  retain their exact prior hashes.
- Prototype locomotion, actor transaction, presentation, clock, camera, boundary admission,
  traversal, bootstrap, frame order, and close request remain unchanged apart from Escape query
  semantics.
- Renderer, runtime, sources, shaders, GPU resources, synchronization, captures, assets, and Wulin
  paths remain unchanged.

## Metrics

- Exact pressed, released, and held key sets after each focused sample.
- Transaction, raw-message, transition, suppression, focus, and recording counters before and
  after empty ingest and replay.
- Canonical record stream and held-state hashes before/after replay and process restart.
- Focused test counts, real-process lifecycle result, Flavor denies/warnings, and final diff
  ownership.

## Acceptance Criteria

- `was_pressed` and `was_released` report only accepted normalized transitions from the most recent
  ingest. Key zero, invalid keys, repeats, and unmatched transitions never appear.
- An empty ingest expires both edge sets without changing held state, journal transactions,
  counters, completed records, or canonical hashes.
- Same-drain transitions may set both directions for one key while final held state stays exact;
  focus loss exposes releases for every previously held key.
- Replay leaves live held and edge sets byte-for-byte unchanged, and the existing controlled native
  sequence retains its exact journal hashes and process-restart equality.
- Prototype Escape consumes `was_pressed`; continuous W/A/S/D still consumes only `is_held`.
- No input-status revision, compatibility fallback, application-owned previous-key state, action
  queue, simulation/frame coupling, new operator, engine/runtime/GPU/source/asset/Wulin change, or
  generated output is added.
- Focused tests, one targeted real Win32 process gate, and `runseal :guard` pass. The long
  canonical-runtime workflow is not required because no renderer, GPU, resource, synchronization,
  lifecycle owner, or journal encoding changes.

## Reference Environment

The experiment uses the repository-pinned Windows/Rust/Deno toolchains, the existing Win32
workbench message adapter, Sidecar lifecycle ownership, and the canonical host-input support gate.

## Evidence

Focused commands:

```powershell
cargo test -p reference-host
cargo test -p prototype
runseal :guard
```

The pre-implementation focused test failed with 18 missing-field/method errors, isolating the absent
pressed/released state and queries. After implementation, 24 reference-host tests and 16 prototype
tests passed. They prove empty-ingest expiry without a journal transaction, duplicate/unmatched/
invalid suppression, same-sample dual edges, focus releases, held independence, and replay preserving
all three live key sets.

The first real-process design attempt deliberately queried edge lists through asynchronous inspect.
The controlled native transaction was recorded, but a later empty host ingest correctly expired the
edge before the inspect request arrived. The experiment therefore removed the proposed status/revision
change instead of adding a diagnostic latch with a second lifetime.

The corrected dependency-free workbench witness passed in 21,310.574 ms across process IDs 10,324
and 17,844. Both native runs retained 2 transactions, 11 messages, 10 accepted transitions, one
repeat, two unmatched ups, one focus loss, three focus releases, empty initial/final held sets, exact
held hash `a539733e...e2861`, and exact stream hash `ec866018...e435`. An intervening empty loop changed
no journal count.

`canonical-prototype-v10` passed in 75,608.032 ms with 77 engine-runtime, 16 prototype, and 24
reference-host tests. Its added real process reached canonical readiness, received activated native
Escape / virtual key 27, and exited cleanly with code 0 after 4,315.755 ms and empty stderr. Existing
native-W locomotion, 15,008.397 ms finite-edge survival, startup failures, restart, camera,
presentation, traversal, backpressure, and zero-process Sidecar cleanup also passed.

The final diff leaves input status/journal revision v1, stream bytes, runtime, renderer/GPU, source
formats, resources, synchronization, assets, and Wulin ownership unchanged. The dependency-free task
witness remains ignored task evidence; no permanent operator was added. The merge-checkpoint guard
passed with zero Flavor denies and five pre-existing warnings.

## Conclusion

Accepted. The host now exposes bounded press/release occurrence for exactly the most recent ingest,
and every later ingest expires those facts without changing continuous held state or journal history.
Prototype Escape is the first live action consumer and exits cleanly through the existing host close
path; concrete action mapping and gameplay policies remain deferred.

## Promotion

Promoted the sample-edge state and queries into `reference-host`, the existing Escape action as their
first product consumer, exact journal hashes into maintained host-input assertions, and the Escape
clean-exit case into `runseal :canonical-prototype`. No separate workflow or diagnostic surface was
promoted.
