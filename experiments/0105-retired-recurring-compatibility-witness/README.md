# Experiment 0105: Retired Recurring Compatibility Witness

Status: Accepted

## Hypothesis

Ten process-level requests for long-removed inspect verbs can be deleted from every full acceptance
run because owner-specific static guards already make their absence authoritative, while the live
clear-only idle-shell proof can remain exact under a current name.

## Scope

- Remove recurring rejection of the retired calibration/world, standalone terrain-contact, and
  caller-owned terrain-body verbs.
- Remove the `compatibilityRemoval.removedVerbs` report chain and delete its mixed-purpose support
  module.
- Preserve status/image/semantic idle-shell evidence as `idleShell`.
- Preserve and strengthen static absence guards without adding a replacement runtime route.

Runtime/product behavior, source/asset formats, GPU work/resources, and historical ADR/experiment
documents are out of scope.

## Workload

1. Map every retired request to the calibration, contact, or terrain-transaction static guard that
   already rejects its route/API/symbol.
2. Delete the ten IPC calls, removed-verb accumulator, old module, and old report/function/field
   names.
3. Require the old path and names to remain absent while the current idle-shell status, capture,
   renderer health, and semantic assertions remain present.
4. Run workspace guard and the optimized full acceptance. Require every historical event key to be
   absent and all current cross-owner/resource/lifecycle gates to pass.

## Controlled Variables

- Idle clear color, extent, samples, semantic attachment, renderer health checks, and capture
  artifact remain unchanged.
- Generic unknown-event behavior and all current malformed-payload tests remain unchanged.
- Existing static guards, not a new runtime rejection table, own historical absence.
- No engine/workbench dispatcher route, Prototype path, asset, shader, resource, or synchronization
  behavior changes.

## Metrics

- Retired process request/event-key count, Sidecar invocation count, and removed report fields.
- Idle image/semantic hashes and pixel/identity counts.
- Full stage duration, artifact inventory, handles, threads, private bytes, traversal, and lifecycle.

## Acceptance Criteria

- Exactly ten retired requests and their recurring report entries are absent.
- The deleted support path and old live names cannot return without failing `runseal :guard`.
- Static owner guards continue to reject every removed route/API/symbol.
- `idleShell` preserves the exact current image and uniformly background semantic proof.
- Full acceptance, resource plateau, traversal, rollback, restart, and lifecycle remain green.

## Reference Environment

Pinned Rust/Deno toolchains, Windows D3D12 reference runtime, and the maintained optimized
`canonical-runtime` workflow.

## Evidence

The audit found ten one-shot rejected IPC requests and ten `removedVerbs` entries (2,389 bytes when
serialized independently) in `canonical-runtime-v12`. Every request reached generic unknown-event;
calibration, contact, and terrain-transaction guards already prohibited the corresponding live
surface.

`runseal :guard` passes after deletion. It checks the full Rust release workspace, clippy, Deno
format/type checks, support tests, Flavor, Sidecar plans, existing owner guards, the missing old
support file, absent old names, and the retained current idle-shell assertions.

The final-worktree `canonical-runtime-v13` passes in 263.724 seconds. `compatibilityRemoval` and all
ten retired event keys are absent. `idleShell` retains color hash `cd26eaab…76db8`, semantic hash
`0c660f2b…a5f`, 921,600 background pixels, zero differing pixels, and zero visible/unknown
semantics.

The workflow records 979 Sidecar invocations versus 988 in v12. Ten retired requests are
structurally gone; one extra state-driven `canonical.status` poll makes the observed net reduction
nine. Stage times are 10.931 seconds setup, 26.430 bootstrap, 16.878 prototype, 12.459 actor
lifecycle, 28.826 simulation actor, 97.490 canonical correctness, 13.485 reactive traversal, 13.642
prepared traversal, 26.338 resources, and 15.258 lifecycle.

Five warm/eight measured publications retain 492 handles and 21 threads; private bytes move from
412,295,168 to 412,385,280 (+90,112), within the accepted plateau. The inventory remains 24 files /
25,346,280 bytes.

## Conclusion

Accepted. Settled compatibility absence is static repository policy; current idle-shell behavior is
the only live process evidence retained from the former mixed-purpose module.

## Promotion

Promoted the `idleShell` current authority and stable guard against recurring compatibility
witnesses. Promoted no replacement route, rejection registry, alias, runtime behavior, or product
capability.

## Reproduction

```powershell
runseal :guard
runseal :canonical-runtime
```

Generated reports remain ignored under `out/captures/`.
