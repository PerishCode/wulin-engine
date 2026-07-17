# Experiment 0115: Retired Final Unknown-Event Witnesses

Status: Accepted

## Hypothesis

The final three recurring requests for removed inspect verbs can be deleted from maintained
acceptance because the presentation, simulation, and object owner guards already make their
absence authoritative, while current malformed-payload and transaction rollback tests preserve
all live strictness.

## Scope

- Delete the recurring `simulation.status`, `canonical.time.status`, and
  `canonical.objects.query` process requests.
- Delete both `retiredStatusGate` helpers and the `retiredStatus` / `retiredVerb` report fields.
- Strengthen the existing owner guards so the removed routes, helpers, and report names cannot
  return.
- Preserve required-field, alias, unqualified-identity, invalid time operation, rollback, and
  current aggregate-status checks.

Adding a replacement rejection registry, compatibility decoder, alias, fallback field, product
behavior, or new guard owner is out of scope.

## Workload

1. Inventory every acceptance assertion that requires `unknown_event`.
2. Remove the three settled IPC requests and their private report plumbing.
3. Move permanent absence authority entirely to the existing presentation, simulation, and object
   static guards.
4. Run the actor, frame, and full-runtime workflows whose report revisions change.

## Controlled Variables

- Generic unknown-event dispatch remains unchanged for genuinely unknown external input.
- Current `canonical.status`, `canonical.time.*`, `simulation.actor.advance`,
  `canonical.objects.resolve`, and `canonical.objects.nearest` behavior remains unchanged.
- Malformed current payloads remain process-tested, including missing required velocity delta,
  rejected velocity alias, unqualified object identity, and invalid time set/step rollback.
- Runtime, product, renderer, shader, asset, resource, descriptor, copy, readback, and
  synchronization code remain unchanged.

## Metrics

- Removed process-request/helper/report-field counts; forbidden-name scan; actor/frame/runtime
  revisions and durations; full-runtime Sidecar invocation count, stage timings, resource plateau,
  lifecycle, and artifact inventory.

## Acceptance Criteria

- No maintained acceptance implementation or generated report contains the three retired verbs,
  `retiredStatus`, `retiredStatusGate`, `retiredVerb`, or `unknown_event` evidence.
- The existing owner guards fail if any deleted route/helper/report chain returns.
- Current malformed-payload and transaction rollback tests remain present and pass.
- `runseal :guard`, `runseal :canonical-actor`, `runseal :canonical-frame`, and
  `runseal :canonical-runtime` pass.

## Results

The audit found exactly three remaining `unknown_event` assertions. Each issued one request to a
generic dispatcher fallback and copied the rejection into a maintained report. The change deletes
three IPC requests, two helper functions, and three report fields, with no replacement route or
compatibility schema.

The existing owner guards now forbid the complete history:

- presentation forbids the old status verb plus `retiredStatusGate` / `retiredStatus`;
- simulation forbids the old status verb plus the same helper/report names in actor acceptance;
- object identity forbids the old query verb and `retiredVerb` in its acceptance owner.

`runseal :guard` passed with zero Flavor denies. The current malformed tests remain: invalid
presentation time set/step rollback, missing `initial_step_velocity_delta_q16`, rejected
`initial_velocity_delta_q16` alias, and missing source namespace for object resolution.

`canonical-actor-v9` passed in 75.926 seconds and `canonical-frame-v12` passed in 43.215 seconds.
`canonical-runtime-v17` passed in 253.391 seconds with 979 Sidecar invocations. Its stages were
6.333 seconds setup, 24.181 bootstrap, 16.947 prototype, 12.156 actor lifecycle, 28.168 simulation
actor, 97.234 canonical correctness, 11.239 reactive traversal, 13.576 prepared traversal, 26.300
resource checkpoint, and 15.309 lifecycle checkpoint.

Five warm and eight measured publications retained 492 handles and 21 threads; private bytes moved
from 423,276,544 to 423,620,608 (+344,064). Both lifecycle cycles passed. The representative
inventory remained 24 files / 25,346,327 bytes. All three final reports contain none of the removed
verbs, helper/report names, or `unknown_event`.

## Conclusion

Accepted. Settled route absence is now purely static owner policy; maintained process acceptance
contains no recurring historical unknown-event witness.

## Promotion

Promoted the smaller actor/frame/runtime report schemas and strengthened existing owner guards.
Promoted no alias, compatibility decoder, rejection registry, product behavior, or engine/GPU
change.

## Reproduction

```powershell
runseal :guard
runseal :canonical-actor
runseal :canonical-frame
runseal :canonical-runtime
```

Generated reports remain ignored under `out/captures/`.
