# Experiment 0110: Retired Transient Object-Action Report

Status: Accepted

## Hypothesis

The Prototype can delete its duplicated transient object-action `attempt`/`completion` readiness
payload and `FrameCompletion` policy echo without losing acceptance authority, because exact
projected frame feedback plus committed acknowledgement/counter/consumption/exclusion/suppression
state already proves every live outcome.

## Scope

- Delete `object_interaction_driver.attempt` and `.completion` from sequence-one readiness.
- Delete the product report mapping for transient `Attempt` values.
- Change `Policy::complete_frame` to commit state and return only `Result<()>`; delete
  `FrameCompletion`.
- Derive focused acceptance proximity/facing facts from the already exact observation query and
  committed actor output instead of a copied action report.
- Keep the session completion schema, projected feedback, acknowledgement, counters, consumed
  identity, nearest exclusion, suppression, source lifetime, and every action behavior unchanged.
- Add a stable owner-specific guard that rejects every retired field/type/mapping/consumer.

Aliases, optional fallback fields, schema-version branching, compatibility decoders, copied event
history, replacement telemetry, engine changes, or product behavior changes are out of scope.

## Workload

1. Remove the duplicate product type, serialization, readiness inputs, and composition-root
   plumbing atomically.
2. Rewrite policy tests to assert committed state rather than a return echo.
3. Rewrite native front/side acceptance to use exact projected feedback, source-oracle observation,
   actor presentation, acknowledgement, and counters only.
4. Require the readiness object driver to omit both historical keys.
5. Run the complete guard and maintained focused Prototype workflow.

## Controlled Variables

- Input admission, canonical nearest/resolve work, exact proximity/facing policy, frame submission,
  renderer projection, surface color, acknowledgement duration, consumption, exclusion,
  suppression, source/window lifetime, and successful-session framing remain unchanged.
- `Attempt` remains a private same-frame policy value used to prepare feedback and commit state; it
  is no longer serialized.
- No engine, renderer, shader, resource, descriptor, copy, readback, or synchronization code
  changes.

## Metrics

- Removed field/type/mapping/consumer counts; readiness key set; exact projected feedback identity
  and kind; derived proximity/facing; acknowledgement/counters; consumed/excluded/suppressed
  identities; sustained-session results; focused duration and guard results.

## Acceptance Criteria

- No product or acceptance source contains the retired `FrameCompletion`, transient report mapper,
  or readiness action fields.
- Front Activated, side Rejected, and post-readiness capacity Rejected gates retain exact identities,
  frame kinds, counts, acknowledgement, consumption, exclusion, and suppression.
- The static guard fails if any removed surface returns.
- All focused tests, `runseal :guard`, and `runseal :canonical-prototype` pass with no behavior or
  engine/GPU/resource change.

## Reference Environment

Pinned Rust/Deno toolchains, Windows D3D12 reference runtime, schema-3 finite sandbox centered at
`(2^40, -2^40)`, and the maintained bounded Prototype session workflow.

## Evidence

Deleted two readiness fields, one duplicate policy return type, two return construction sites, one
transient mapper plus its facing view, composition-root plumbing, test echo assertions, and every
acceptance consumer. No alias, optional fallback field, version branch, or compatibility decoder
remains. The removal guard is owned by the existing Prototype session guard, so cleanup adds no new
support module.

All 45 Prototype tests, workspace Clippy, Deno checks, Flavor, and `runseal :guard` pass. The guard
requires `Policy::complete_frame -> Result<()>`, rejects every retired producer/consumer name, and
requires readiness action objects to omit both keys while focused acceptance uses exact projected
feedback and derived committed-origin/facing facts.

`canonical-prototype-v27` passes in 81.285 seconds. Front ID 496 still projects Activated and commits
consumption/exclusion with exact derived delta `(128,-32)`, yaw/direction/dot
`0/(1,0)/128`; side ID 496 still projects Rejected with exact derived delta `(160,0)` and
yaw/direction/dot `49152/(0,-1)/0`, committing no consumption. The sustained session moves from
live frame 4 to 798, retains consumed ID 496 and exclusion-oracle rejected target ID 501, reports
exactly 12 capacity-rejected frames and 783 suppression frames, and emits no event history. The
readiness interaction driver contains only current rule/status/frame-count/suppression fields; both
retired keys are absent.

No engine, renderer, shader, frame ABI, resource, descriptor, copy, readback, or synchronization
code changed, so full-runtime acceptance is not repeated.

## Conclusion

Accepted. Exact projected feedback plus durable policy state fully replaces the copied transient
action report and return echo.

## Promotion

Promoted the smaller readiness schema, state-only `complete_frame` transaction, derived focused
acceptance authority, and stable absence checks inside the existing session owner. Promoted no
alias, fallback, schema decoder, telemetry replacement, product behavior, or engine/GPU/resource
change.

## Reproduction

```powershell
cargo test --locked -p prototype
cargo clippy --locked --workspace --all-targets -- -D warnings
runseal :guard
runseal :canonical-prototype
```

Generated reports remain ignored under `out/captures/`.
