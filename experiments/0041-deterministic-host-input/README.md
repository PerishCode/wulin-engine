# Experiment 0041: Deterministic Host Input

Status: Accepted

## Hypothesis

The native workbench host can convert one ordered Win32 keyboard message stream into bounded,
platform-normalized key-state transactions and replay those transactions exactly without changing the
engine runtime, renderer, presentation timeline, accepted frame output, or host pause behavior.

## Scope

This experiment adds keyboard and focus-message capture to the existing workbench window, a
host-owned normalizer and bounded in-process journal, and a synthetic key-state replay oracle.
Input transactions are applied after the native queue is drained and before inspect dispatch or
frame submission. The journal records normalized transitions rather than wall time or renderer
frames.

Camera control, action mapping, text input, mouse, gamepad, source bootstrap, elapsed or simulation
time, fixed-step sampling, gameplay state, persistent replay files, a new executable, and Wulin
content are out of scope.

## Workload

1. Post controlled key down/up messages through the live workbench window procedure while the host
   is running and while host rendering is paused.
2. Record ordered presses and releases, repeated downs, unmatched ups, a held pair, and focus loss.
3. Stop the record and replay it into a separate synthetic state consumer. Compare transaction,
   transition, stream-hash, and final-held-state evidence without mutating the live state.
4. Restart the process, repeat the same native sequence, and require exact canonical hashes and
   counters independent of process identity and workbench frame index.
5. Exercise invalid record/replay transitions and bounded overflow with focused tests.
6. Run the complete canonical GPU correctness, failure, traversal, resource, and lifecycle gates.

## Controlled Variables

- The Win32 message pump order, workbench pause/capture behavior, runtime frame transaction,
  renderer, scene, presentation timeline, sources, GPU resources, shaders, and submissions remain
  unchanged.
- Keys are unsigned Win32 virtual-key values in `1..=255`. Only state-changing down/up transitions
  are recorded; repeated downs and unmatched ups are counted and suppressed.
- Focus loss releases all currently held keys in ascending numeric order. Replay starts from the
  held-key snapshot captured at record start and never changes the live keyboard state.
- Transaction and transition capacity are fixed. Overflow invalidates recording explicitly rather than
  truncating, allocating without bound, or falling back.
- No host duration, presentation tick, or frame index enters canonical journal bytes.

## Metrics

- Raw-message, normalized-transaction, accepted-transition, repeated-down, unmatched-up, and
  focus-release counts.
- Initial and final held-key sets, canonical transaction-stream SHA-256, and final held-state
  SHA-256 before and after replay and process restart.
- Input transaction order relative to inspect dispatch, paused rendering, and runtime frame calls.
- Existing controlled attachment/shadow hashes, CPU/GPU mismatch counts, submissions, traversal,
  resource plateau, and lifecycle evidence.

## Acceptance Criteria

- The window procedure captures key down/up, system key down/up, and focus loss without placing
  native message types in `engine-runtime`. The main thread applies one ordered input transaction
  after each message-queue drain and before inspect or frame work, including while host rendering
  is paused.
- Repeated downs and unmatched ups do not create transitions. Focus loss emits one release for
  every held key in ascending order and leaves no key held.
- A completed bounded record replays into an isolated state oracle with exact transaction count,
  transition count, stream SHA-256, final held keys, and held-state SHA-256. Replay leaves live
  state byte-for-byte unchanged.
- The same controlled native-message sequence after process restart produces the same normalized
  hashes and counters. Invalid lifecycle operations and overflow fail explicitly without corrupting
  the last completed record or live state.
- Focused tests, repository guard, all prior canonical correctness/failure/traversal/resource gates,
  and 16 complete lifecycle cycles pass with unchanged accepted GPU hashes and no validation or
  device-removal error.

## Evidence

The direct workflow will remain:

```powershell
runseal :canonical-runtime
```

Generated evidence remains ignored under
`out/captures/0041-deterministic-host-input/`.

## Results

The 533.7-second direct GPU workflow passed. In both the initial and restarted processes, two
native queue drains containing 11 messages normalized to 10 state-changing transitions. The
controlled stream counted one repeated down, two unmatched ups, one focus loss, three ordered
focus releases, and zero invalid keys. Both runs began and ended with no held keys and produced
the exact stream hash
`ec86601874cb60a8c592b9caf500da94111b6a7360647d316ce1e858b55de435` and held-state hash
`a539733efed454375e712ba689ac99049afb144efbbe7f9c60256c11860e2861`.

Replay reproduced the complete record and reported exact record matching while leaving live
state and counters unchanged. Host frame index remained fixed throughout paused record/replay.
Stop without a record, replay without a record, duplicate start, replay while recording, and key
256 were rejected. Five focused tests additionally proved deterministic release ordering,
non-empty initial-state replay, equivalent-record hashing, explicit bounded overflow with prior
record retention, and invalid lifecycle rollback.

The controlled color, PNG, object-ID, diagnostic, light-matrix, and shadow-depth hashes remained
exactly equal to Experiment 0040. Walk ticks 0/42/43/85 still selected phases 0/63/0/0 with a
maximum CPU/GPU palette delta of `2.3283064e-10`. The frame retained 10,538 shadow casters, one
indirect shadow dispatch, 60 shadow root-constant DWORDs, 98 descriptors, and zero receiver
mismatch.

All failure, rollback, hold, alias, source, and rollover gates passed, as did 32 reactive and 32
prepared crossings. The resource workload established a 527-handle baseline and 527-handle peak
with zero transient growth; publication 64 ended at 516 handles, 413,294,592 private bytes, and 18
threads. All 16 complete lifecycle cycles and the repository guard passed without device removal
or validation error.

## Conclusion

Accepted. The workbench now owns one deterministic native keyboard boundary whose ordered,
bounded transactions are independent of rendering and can be replayed exactly. This proves input
transport, focus cleanup, pause independence, record lifecycle, and process-restart stability; it
does not select gameplay controls or simulation sampling.

## Promotion

Promoted the native message adapter and bounded journal into `apps/workbench`, process-local
record/status/replay controls into the inspect surface, a direct restart gate into the canonical
workflow, and stable ownership checks into the repository guard. No engine input API, camera
controller, action system, simulation clock, or persistent replay format was promoted.
