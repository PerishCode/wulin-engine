# Experiment 0106: Committed Prototype Object Facing

Status: Accepted

## Hypothesis

The Prototype can require an object action to lie in the actor's strict front half-plane by
combining the existing exact committed eight-way yaw with the existing exact Q9 target proximity,
without changing Runtime, canonical snapshots, rendering, resources, or capacity-one consumption.

## Scope

- Admit facing only from the committed actor output that also supplies the action origin.
- Map exactly the existing eight locomotion yaws to integer planar directions.
- After the inclusive 512-Q9 radius gate, require a positive exact dot product for every
  non-coincident target; retain exact zero-distance eligibility.
- Consume side-on and rear attempts as `OutsideFacing` without submitting Activated feedback or
  committing consumption.
- Reject malformed non-eight-way yaw without changing pending intent or policy counters.

Arbitrary steering, cones, line of sight, GPU visibility, multiple consumed identities, registry,
inventory, rewards, dispatch, respawn, persistence, networking, and Wulin semantics are out of
scope.

## Workload

1. Unit-test all eight exact yaw/direction pairs, front/side/rear/zero-distance classification,
   radius-before-facing order, malformed-yaw rollback, and unchanged consumption lifetime.
2. Run one visible native `F+Enter+D` process. Require committed yaw 0, positive-X direction,
   positive exact dot, successful projected activation, and the existing capacity-one commit.
3. Run a separate visible native `F+Enter+W` process against the same nearest identity. Require
   committed yaw 49,152, a side-on zero dot implied by exact proximity, `OutsideFacing`, Selected
   frame feedback, and zero activation/consumption.
4. Run focused Prototype acceptance and one final optimized full acceptance; preserve all existing
   cross-owner, resource, traversal, rollback, restart, and lifecycle gates.

## Controlled Variables

- Observation, typed resolution, proximity radius, target identity, frame feedback,
  acknowledgement, exclusion, suppression, and source/window lifetime remain unchanged.
- Canonical source/snapshot, Runtime and renderer APIs, visible records, root constants, shaders,
  passes, resources, descriptors, copies, readback, and synchronization remain unchanged.
- Facing is a pure Prototype policy over already committed actor and resolved target values.

## Metrics

- Exact actor yaw, integer direction, signed Q9 dot, proximity delta, distance, outcome, feedback,
  ineligible/committed count, acknowledgement, consumption, and exclusion.
- Focused and full duration, Sidecar invocations, artifacts, handles, threads, and private bytes.

## Acceptance Criteria

- All eight committed yaws admit a target strictly in front; side and rear non-coincident targets
  are ineligible; exact coincidence is eligible.
- Malformed yaw and resolution fail before policy mutation.
- The positive native process commits exact Activated feedback and consumption; the side native
  process retains only Selected feedback and commits neither.
- Focused, guard, and final full-runtime acceptance pass without structural engine/GPU/resource
  change.

## Reference Environment

Pinned Rust/Deno toolchains, Windows D3D12 reference runtime, schema-3 finite sandbox centered at
`(2^40, -2^40)`, and the maintained focused/full state-driven acceptance workflows.

## Evidence

All 96 engine-runtime tests, nine interaction-policy tests, seven observation-policy tests, the
remaining Prototype/reference-host tests, workspace Clippy, and Deno type checks pass.

`canonical-prototype-v23` passes in 75.629 seconds. Native `F+Enter+D` commits ID 496 after moving
to local X 32 with yaw 0; its exact proximity is `(128, -32)` Q9 / 17,408 Q18 and its facing
direction/dot are `(1, 0)` / 128. Activated projection applies, committed count becomes one, and 11
acknowledgement frames remain.

The independent native `F+Enter+W` process observes the same ID 496 at `(160, 0)` Q9 / 25,600 Q18
with committed yaw 49,152. It returns `outside-facing`, ineligible count one, and zero completion,
Activated frames, consumed identity, exclusion, acknowledgement, or suppression.

`runseal :guard` passes with zero Flavor denies. Its first run caught that the extra host evidence
had crossed the 500-line owner limit; the paired facing gates were moved into the existing object
gate module, returning the host to 497 lines. The resolved-object match branch was likewise
reduced to a named proximity/facing handoff.

The final-worktree `canonical-runtime-v14` passes in 254.675 seconds. Existing source replacement,
same-source departure/return, rollback, restart, 32+32 traversal, and two lifecycle cycles remain
green. Five warm and eight measured resource publications retain 492 handles and 21 threads;
private bytes move from 423,571,456 to 424,103,936 (+532,480). The report contains 24 files /
25,346,301 bytes and records 982 Sidecar invocations. Stage times are 7.541 seconds setup, 24.363
bootstrap, 17.044 Prototype, 12.349 actor lifecycle, 28.326 simulation actor, 96.624 canonical
correctness, 11.344 reactive traversal, 13.488 prepared traversal, 26.291 resources, and 15.335
lifecycle.

## Conclusion

Accepted. Prototype actions now consume exact committed facing as well as proximity, with no new
engine or retained product ownership.

## Promotion

Promoted strict front-half-plane classification, exact eight-way direction/dot evidence,
malformed-yaw rollback, and native admitted/rejected process gates. Promoted no cone, line of
sight, visibility readback, registry, reward, inventory, dispatcher, respawn, persistence,
networking, or Wulin behavior.

## Reproduction

```powershell
cargo test --locked -p engine-runtime -p prototype -p reference-host
cargo clippy --locked --workspace --all-targets -- -D warnings
runseal :canonical-prototype
runseal :guard
runseal :canonical-runtime
```

Generated reports remain ignored under `out/captures/`.
