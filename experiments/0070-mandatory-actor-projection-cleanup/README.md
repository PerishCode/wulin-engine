# Experiment 0070: Mandatory Actor Projection Cleanup

Status: Accepted

## Hypothesis

After frame-safe actor GPU admission, the standalone actor projection API, inspect verb, and
recurring process gate can be deleted while retaining one renderer-internal exact projection and
all current actor frame behavior, eliminating a temporary experiment surface without adding a
replacement alias, fallback, or compatibility layer.

## Scope

Remove `Runtime::project_actor`, the crate-root `ActorRenderProjection` export, `actor.project`
protocol/parser/dispatch behavior, and `.runseal/support/actor/projection.ts`. Remove the standalone
projection result from the canonical runtime workflow and required-file index.

Keep `Renderer::project_actor`, frame preflight, `ActorRenderProjection` inside the private
rendering boundary, private arithmetic tests, frame actor upload, actor lifecycle/simulation APIs,
and `runseal :canonical-actor`. This cleanup changes no projection formula, candidate layout,
shader, GPU resource, frame order, synchronization, prototype behavior, or content format.

## Workload

1. Inventory live and historical references to the projection API, verb, support path, revision,
   report field, and public type export.
2. Delete the Runtime forwarding and workbench control chain; make the projection re-export
   crate-private and remove the recurring standalone gate from canonical acceptance.
3. Extend the existing simulation/actor history-removal guard to reject the deleted support path,
   inspect verb, Runtime forwarding, and crate-root export.
4. Run focused compile/tests and a dependency-free temporary live script that requires
   `actor.project` to return `unknown_event` without importing `.runseal/support`.
5. Run `runseal :canonical-actor` to prove exact projection/preflight and GPU behavior through the
   sole current frame consumer, then run `runseal :guard`.

## Controlled Variables

- The renderer's signed global-to-window Q9/Q16 projection algorithm and private tests are
  unchanged.
- Actor lifecycle, simulation transaction, full generation identity, presentation, fixed candidate
  address, and two-frame upload remain unchanged.
- The removed verb has no alias and always reaches the protocol's existing `unknown_event` result.
- Historical Experiment 0066 and ADR 0069 remain immutable decision history; their names do not
  become live compatibility surfaces.
- No source cooking format, camera, terrain query, input, gravity, or prototype behavior changes.

## Metrics

- Removed public methods/types, protocol variants/parsers/dispatchers, support files, imports,
  canonical report fields, and lines of recurring gate code.
- Exact old-verb rejection code before and after publication-independent workbench startup.
- Private actor projection/runtime test count and canonical actor workload/hash equality.
- Engine API, frame, shader, GPU resource, copy, barrier, fence, signal, wait, and allocation
  deltas.

## Acceptance Criteria

- `Runtime` exposes no projection forwarding and the crate root exports no actor projection type.
  The type remains accessible only within the private rendering implementation.
- `actor.project` returns `unknown_event`; its protocol variant, parser, dispatcher, support file,
  canonical import/call/result, and required-file entry are absent with no alias or fallback.
- Renderer frame preflight/projection and private exact tests remain, and `runseal :canonical-actor`
  reproduces accepted actor visibility, rejection, rollback, identity, and capture evidence.
- The recurring cleanup guard rejects restoration of every deleted live surface.
- Focused checks and `runseal :guard` pass with no frame/resource/synchronization change.

## Reference Environment

The cleanup uses the repository-pinned Rust/Deno toolchains, reference Windows workbench, the
current capacity-one actor renderer, and fresh sources cooked by `runseal :canonical-actor`.

## Evidence

The cleanup removed one public Runtime method, the crate-root projection type export, one workbench
protocol variant, parser route, dispatcher arm, and 26-line response owner. It deleted the 238-line
standalone process support plus its canonical import, call, report field, stable file entries, and
type-check input. Before documentation, the live diff was 293 deletions and 11 additions; additions
are crate-private visibility and recurring forbidden-surface patterns only.

`ActorRenderProjection`, `Renderer::project_actor`, pending-window preflight, frame snapshot
projection, and four private exact arithmetic tests remain. `cargo check` passed for
`engine-runtime` and workbench. All 71 engine-runtime tests plus the semantic actor-ID test passed,
including far/alias, signed seam, window edge/outside, generation encoding, and actor resource
cases. Canonical-runtime, init, and guard TypeScript entry points passed Deno checking.

The dependency-free `.task/resources/0070-actor-projection-removal-gate.ts` imports no repository
support. Its first iteration exposed two script defects rather than product failures: piped
Sidecar lifecycle output held the background broker pipe open until the outer 120-second timeout,
and `Deno.exit` bypassed `finally` on a later assertion. Both residual process sets were explicitly
detected and stopped. The corrected script inherits lifecycle streams, throws through `finally`,
and in 7.2 seconds proved a running D3D12 workbench followed by exact
`unknown_event: unsupported event \"actor.project\"`; final stop removed the workbench and broker.

`runseal :canonical-actor` then passed in 21,763.199 ms through the sole renderer-internal
projection/preflight path. Generation-one and generation-two records retained SHA-256
`2f14c5bbe4268821f0cdd3cbc7a8fbce6d92c08c944476a91f82f353f505ac3a` and
`ab33d261aebcc263cf32a7849fb3637a2a82e37c130a021a1962327a8198dc78`.
Color, PNG, object-ID, and diagnostic captures remained
`d7fbc2e6c26cfe7f74a8b5751e1e6239f07f37f3e0204322032fe7ca4e50329e`,
`7e9d40d716870dd7a00dd54cebfd333ee9344de086093566b15b74b8e635962c`,
`a9fa7a8f1bec4fc5fe62dc7a969bcd87332b308352e59bf0616c2248841bcaf6`, and
`eba4b91018819328e51b9a503ba00503887b3a4aa88bf3dd4426abc6d888b3bb`;
the runtime actor still covered 3,866 pixels.

No frame, shader, GPU allocation, copy, barrier, fence, signal, wait, source format, or prototype
behavior changed. Resource and full canonical workflows were therefore not rerun; the focused
actor workflow exercises the affected private frame consumer directly.

`runseal :init` passed with the deleted/new stable file set. `runseal :guard` passed in 12.3 seconds
with all workspace release tests, affected Clippy under `-D warnings`, Deno formatting/type checks,
the new recurring removal patterns, and zero Flavor denies; the five pre-existing warnings remain.

## Conclusion

The hypothesis is accepted. Actor projection has one live owner inside renderer frame admission;
the temporary public/diagnostic Experiment 0066 surface and its recurring workload are gone, with
no compatibility alias or fallback.
