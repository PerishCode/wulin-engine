# Experiment 0125: Retired Actor Velocity Compatibility Probes

Status: Accepted

## Hypothesis

The recurring missing-field and invented-alias `simulation.actor.advance` requests can be deleted
from maintained actor acceptance without weakening the current transaction. The required
`initial_step_velocity_delta_q16` command shape, admitted nonzero ordering, invalid-presentation
rollback, and pending-window rollback already provide current owner evidence.

## Scope

- Delete the old-shape request that removes `initial_step_velocity_delta_q16`.
- Delete the invented `initial_velocity_delta_q16` alias request.
- Delete both rejection values from the actor acceptance report.
- Extend the existing simulation-control removal guard to reject restoration of either probe while
  requiring the current velocity and rollback evidence.
- Advance only the canonical actor report revision because no other maintained wrapper consumes
  this gate.

Product payload decoding, the required command field, generic invalid-payload behavior, Runtime,
Prototype policy, renderer/GPU resources, source lifetime, synchronization, and historical
experiment/ADR evidence are out of scope.

## Workload

1. Inventory every maintained request and report field for the two historical shapes.
2. Delete their process traffic and report plumbing.
3. Bind permanent absence and current-authority preservation to the existing simulation-control
   removal owner.
4. Run `runseal :guard`, `runseal :canonical-actor`, and `runseal :init`.

## Controlled Variables

- Every maintained live actor command continues to provide
  `initial_step_velocity_delta_q16` explicitly.
- The admitted actor gate still proves velocity `0 -> 16384`, exact center-height ordering, and
  committed presentation/epoch.
- Invalid presentation still fails before mutation and preserves actor/schedule state.
- Pending-window backpressure still preserves actor, schedule, composition, and retained frame.
- Actor lifecycle, fixed-step partition, GPU, animation, Runtime, Prototype, and product behavior
  remain unchanged.

## Metrics

- Removed process-request/assertion/report-field counts; executable line delta; forbidden-name
  scan; Flavor findings; actor revision, duration, report keys, exact velocity/rollback evidence,
  GPU hashes, and generated artifact inventory.

## Acceptance Criteria

- Maintained actor acceptance contains neither historical request nor either report field.
- The existing simulation-control removal guard fails if a removed probe returns or the current
  velocity/admission rollback evidence disappears.
- `canonical-actor-v10` passes with no `retiredPayload` or `aliasPayload`, while current admitted,
  invalid-presentation, pending-window, lifecycle, simulation, GPU, and animation evidence remains.
- `runseal :guard` and `runseal :init` pass with zero Flavor denies.

## Results

The acceptance owner deleted exactly two recurring malformed Workbench requests, their two
assertion blocks, and two report fields. The deleted request/assertion/report chain is 18 lines.
The existing simulation-control removal owner gained 24 lines of absence and current-authority
checks; with the actor revision update, the executable acceptance/guard diff is a net six-line
increase while recurring process and report work shrink.

`runseal :guard` passed all Rust/Deno formatting, lint, type, and test gates. Flavor reported zero
denies and the same five existing warnings. `runseal :init` passed with the pinned Rust 1.94.1,
Deno 2.9.1, Runseal 0.9.0, Flavor 0.3.6, and Sidecar 0.5.1 toolchain.

`canonical-actor-v10` passed in 85.111 seconds. Its root keys are exactly `revision`, `outcome`,
`storage`, `publication`, `lifecycle`, `simulation`, `admission`, `actor`, `animationEpoch`, and
`elapsedMilliseconds`. Prepublication keys are exactly `before`, `invalidPresentation`,
`response`, and `after`; neither final report nor maintained admission source contains
`retiredPayload`, `aliasPayload`, or `initial_velocity_delta_q16`.

Current transaction evidence remains exact:

- the admitted command applies delta 16,384, changes velocity `0 -> 16,384`, and changes center
  height numerator `141,824 -> 158,208`;
- invalid archetype 8 still fails through the current actor-simulation error and preserves
  actor/schedule state;
- the pending-window candidate prepares one step and one terrain query but commits zero schedule
  and actor mutations and emits no actor advance;
- lifecycle, simulation partition/rollback, frame-safe GPU admission, and animation-epoch gates all
  pass.

The generated canonical actor inventory is five files / 5,355,420 bytes. The representative first
capture retains color SHA-256 `d7fbc2e6c26cfe7f74a8b5751e1e6239f07f37f3e0204322032fe7ca4e50329e`
and object-ID SHA-256
`a9fa7a8f1bec4fc5fe62dc7a969bcd87332b308352e59bf0616c2248841bcaf6`.

## Conclusion

Accepted. Settled predecessor-shape compatibility evidence is now static deletion policy;
maintained actor acceptance spends process and report work only on current transaction behavior.
No decoder, product, Runtime, renderer/GPU, resource, or synchronization path changed.

## Reproduction

```powershell
runseal :guard
runseal :canonical-actor
runseal :init
```

Generated reports remain ignored under `out/captures/`.
