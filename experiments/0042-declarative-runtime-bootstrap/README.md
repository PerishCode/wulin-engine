# Experiment 0042: Declarative Runtime Bootstrap

Status: Accepted

## Hypothesis

The existing workbench host can consume one strict declarative bootstrap configuration and emit
readiness only after the accepted runtime has rendered a canonical frame, while invalid or failed
bootstrap attempts exit without readiness or idle-shell fallback and unconfigured startup retains
its current behavior.

## Scope

This experiment adds one optional repository-relative JSON bootstrap argument to the existing
workbench. The document selects one signed terrain pack, one signed schema-3 object pack, and one
`GlobalRegionConfig`. Configured startup opens both sources, schedules one manual pair, advances
the existing runtime frame transaction while the window remains hidden, and becomes ready only
after canonical publication and rendering.

A new executable, shared platform-host crate, camera/input consumption, action mapping, simulation
time, actors, gameplay, persistent input replay, content discovery, multiple startup profiles,
fallback content, portability, and Wulin content are out of scope.

## Workload

1. Parse strict argument and configuration fixtures, including duplicates, unknown fields,
   oversized input, invalid repository paths, invalid schema, and invalid signed projection.
2. Launch the workbench under Sidecar with an invalid document and with a missing source. Require
   process failure before readiness and complete cleanup.
3. Launch with a structurally valid source whose requested object payload is corrupt. Require the
   asynchronous pair failure to terminate bootstrap without showing or reporting an idle shell.
4. Launch with freshly cooked valid sources and a far signed global configuration. Require the
   first readiness state to be canonical, then freeze presentation tick zero and capture/probe.
5. Restart the configured process and require the same configuration digest, published target,
   controlled frame, and probe evidence.
6. Start the ordinary unconfigured Sidecar and require the accepted idle-shell readiness before
   continuing all existing input, GPU, fault, traversal, resource, and lifecycle gates.

## Controlled Variables

- The runtime facade, renderer, presentation timeline, source formats, streaming workers,
  terrain-first pair publication, shaders, GPU resources, and frame transaction remain unchanged.
- Bootstrap schema contains only `schemaVersion = 1`, repository-relative terrain/object paths,
  signed origin/center coordinates, and active radius. Pack paths remain under `out/cooked`.
- Configured startup schedules exactly one manual pair. Traversal and prefetch remain disabled.
- The configured window stays hidden and no readiness line is emitted until a canonical frame has
  completed. Failure is terminal; there is no fallback to idle-shell readiness.
- Unconfigured Sidecar startup remains one visible idle-shell frame followed by readiness.

## Metrics

- Argument/config validation outcomes, configuration byte count and SHA-256, startup mode,
  scheduled token, frame count before readiness, elapsed host bootstrap duration, and failure
  stage/message.
- Sidecar readiness/exit outcome and remaining process count for invalid, missing, corrupt,
  successful, and restarted launches.
- Published signed target, source namespaces, controlled attachment/shadow hashes, CPU/GPU oracle
  mismatch counts, traversal, resource plateau, and lifecycle evidence.

## Acceptance Criteria

- Unknown/duplicate arguments and invalid, oversized, non-schema-1, path-escaping, or invalid
  projection configurations fail before native runtime creation. No ignored startup option or
  fallback path exists.
- Configured startup opens the exact two validated sources, schedules one manual pair, processes
  it through ordinary runtime frames while hidden, and emits readiness only after a canonical
  frame. Status identifies the bootstrap mode, exact config hash, paths, global config, schedule,
  frame count, and elapsed duration.
- Immediate and asynchronous bootstrap failures exit nonzero without a readiness line or idle-shell
  fallback. Sidecar cleanup leaves no owned workbench, cargo, inspect, or broker process.
- Successful restart reproduces the config hash, published target, tick-zero controlled frame,
  and stable probe evidence. Ordinary startup still reports idle-shell readiness.
- Focused tests, repository guard, deterministic native input/replay, all prior GPU hashes and
  fault/traversal/resource gates, and 16 complete lifecycle cycles pass without device removal or
  validation errors.

## Evidence

The direct workflow will remain:

```powershell
runseal :canonical-runtime
```

Generated evidence remains ignored under
`out/captures/0042-declarative-runtime-bootstrap/`.

## Results

The 559.8-second direct GPU workflow passed. A document with an unknown `fallback` field exited in
128.0 ms, a missing object source exited in 3,426.4 ms, and a requested corrupt object payload
failed its asynchronous pair in 3,497.4 ms. All three returned nonzero without emitting the
workbench readiness line; the corrupt pair reported its exact global region checksum mismatch and
discarded the terrain half.

The valid 332-byte schema-1 document hashed to
`15e772866c5ac43d41bd8209d3b86810426a52e282ae24d51ab9a40fad34f4dc`.
The first process rendered canonical-ready on hidden frame 8 after 102.8 ms; the restarted process
did so on frame 8 after 100.1 ms. Both reported the same document hash, exact source paths, far
signed global target, token 1, and canonical workload before readiness was observable. Freezing
the already-ready processes to presentation tick zero reproduced the complete stable frame across
restart.

Four focused bootstrap tests cover strict single-use arguments, exact schema decoding, unknown
field/schema/path/projection rejection, and document/config-path bounds. Together with the five
input tests, the workbench suite now contains nine tests. Ordinary Sidecar startup still became
ready as an idle shell, and the deterministic input record retained stream hash
`ec86601874cb60a8c592b9caf500da94111b6a7360647d316ce1e858b55de435` across restart.

The controlled color, PNG, object-ID, diagnostic, light-matrix, and shadow-depth hashes remained
exactly equal to Experiment 0041. Walk ticks 0/42/43/85 retained phases 0/63/0/0 and the maximum
CPU/GPU palette delta remained `2.3283064e-10`. All failure, hold, rollback, alias, rollover, 32
reactive, and 32 prepared gates passed. The resource workload had a 527-handle baseline, 528-handle
peak within the accepted one-handle transient allowance, and publication 64 ended at 517 handles,
412,606,464 private bytes, and 18 threads. All 16 lifecycle cycles, the repository guard, and final
cleanup of dev, benchmark, and bootstrap namespaces passed.

## Conclusion

Accepted. One strict configuration now owns configured source selection, schedule order, terminal
failure, hidden async progress, and canonical readiness in the existing host. The ordinary idle
workbench is unchanged. This establishes the contract a later thin prototype host may consume,
without conflating it with application or gameplay ownership.

## Promotion

Promoted the strict bootstrap parser/status owner under `apps/workbench`, one configured Sidecar
manifest, and direct invalid/missing/corrupt/success/restart gates. No new application or shared
platform abstraction was promoted.
