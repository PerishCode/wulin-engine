# Experiment 0062: Prototype Body Bootstrap

Status: Accepted

## Hypothesis

The non-diagnostic prototype can create one deterministic retained terrain body only after strict
canonical publication, deriving exact grounded state from the configured global center and committed
terrain without changing the shared bootstrap schema, workbench authority, simulation schedule,
host time, input, frames, rendering, or presentation.

## Scope

After `bootstrap::drive` succeeds, derive `TerrainPosition(globalCenter, 0, 0)`, query the committed
terrain, construct one prototype-owned 65,536-Q16 half-height body whose foot exactly touches that
terrain, give it zero step velocity, and spawn it into Runtime's existing capacity-one slot before
readiness is emitted. Include the exact terrain sample and retained body in prototype readiness.

Do not change schema-1 bootstrap, spawn a body in workbench, sample HostClock, advance Runtime,
introduce gravity/locomotion/input policy, move the camera, create actor storage, or alter rendering.

## Workload

1. Prove exact body derivation at signed far global centers and committed local Q9 zero.
2. Require center height to equal checked terrain height plus fixed half-height, with zero velocity.
3. Reject controlled center-height overflow before constructing motion.
4. Start a real prototype over fresh cooked sources and require canonical readiness plus generation
   one retained identity only after terrain publication.
5. Restart the process and require byte-identical terrain/motion/readiness invariants with a new PID.
6. Preserve invalid config, missing source, corrupt payload, Sidecar start/restart/stop, and terminal
   no-readiness behavior in the existing prototype host gate.
7. Run focused prototype tests, the targeted prototype process gate, `runseal :init`, and
   `runseal :guard`. Do not run the full canonical workflow because workbench GPU/resource evidence
   is unchanged and the targeted gate covers the modified prototype lifecycle.

## Controlled Variables

- Existing schema-1 source paths, signed global origin/center, and active radius remain authoritative.
- Initial local coordinates are exactly Q9 zero; half-height is 65,536 Q16 and velocity is zero.
- Runtime retained capacity/generation, exact terrain query, and canonical publication are unchanged.
- No simulation elapsed, schedule mutation, terrain-body advance, frame, or renderer work is added.

## Metrics

- Configured and retained signed region/local position.
- Terrain, center, and half-height numerators plus denominators and step velocity.
- Retained generation/live count and deterministic restart equality.
- Readiness ordering, failure exit/readiness status, process identity, and lifecycle cleanup.
- Focused test count, targeted gate result, Flavor denies, and guard result.

## Acceptance Criteria

- Prototype readiness always contains one generation-one retained body at exact configured center.
- The body's foot equals the committed terrain height byte-exactly and velocity is zero.
- Arithmetic/query/spawn failure emits no readiness and retains no partial externally visible state.
- Identical restarts preserve exact body evidence while replacing process identity.
- Workbench/bootstrap schema, clock, schedule, input, frames, renderer, and presentation are unchanged.
- Focused tests, targeted process gate, `runseal :init`, and `runseal :guard` pass without the full
  canonical workflow.

## Reference Environment

The experiment uses the pinned Windows reference platform, fresh canonical cooked fixture sources,
the existing prototype Sidecar lifecycle, and Runtime's accepted exact terrain/retained-body owners.

## Evidence

Two focused prototype tests prove exact signed-center/touching derivation and reject both mismatched
height authority and controlled center overflow. The unchanged workbench compiles, and the complete
workspace guard passes.

A 50.4-second fresh-cook targeted prototype gate covered invalid configuration, missing source,
corrupt payload, two direct process starts, and Sidecar start/restart/stop. At signed region
`(1099511627776, -1099511627776)` and local Q9 `(0, 0)`, both direct processes published generation
one, terrain height `76288/65536`, center `141824`, half-height `65536`, and step velocity zero.
Their complete simulation-body evidence was byte-identical while PIDs differed. Every failure
emitted no readiness, and final Sidecar status retained no PID.

`runseal :init` and `runseal :guard` passed with zero Flavor denies. The full canonical workflow was
not run because workbench frames, GPU resources, synchronization, traversal, rendering, and lifecycle
behavior are unchanged; the targeted gate directly covers the modified prototype startup lifecycle.
