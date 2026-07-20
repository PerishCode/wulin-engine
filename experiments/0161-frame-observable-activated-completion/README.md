# Experiment 0161: Frame-Observable Activated Completion

Status: Accepted

## Hypothesis

The post-readiness Activated recovery gate can terminate on the renderer's accepted frame semantics
instead of a fixed 250 ms wall-time proxy. An acceptance-owned observer can detect the exact
Activated-green projection and its return to the pre-action baseline after acknowledgement without
adding product telemetry, output values, renderer resources, or a new process in the maintained
Prototype matrix.

## Scope

- Start one prepared observer before the existing atomic Enter-up/F-down/Enter-down recovery input.
- Bind capture to the exact visible Prototype PID/window already owned by the session.
- Temporarily raise only that window with no activation, capture its client pixels from the desktop,
  restore z-order, and post Escape only after frame-semantic completion.
- Classify the fixed Activated mix by its bounded RGB/dominance range, require at least 64 pixels
  above the pre-action baseline, then require two samples at or below baseline plus 16 pixels.
- Keep 10 seconds as a failure deadline rather than a success dwell.
- Retain the product completion gate for exact 12 Activated frames and following suppression.
- Add exact failure-only clock context to the existing Jump continuity diagnostic.
- Advance complete Prototype acceptance from v75 to v76.

Prototype product behavior and stdout, Runtime, renderer/GPU resources, source formats,
synchronization, process count, and the other 17 graceful sessions are out of scope.

## Workload

1. Reproduce the cold-workspace failure under the existing 250 ms action-to-exit helper.
2. Distinguish product render backpressure from acceptance timing using temporary exact diagnostics.
3. Prepare a same-PID/window observer before recovery input and capture a pre-action pixel baseline.
4. Observe the fixed Activated-green class rise, its post-acknowledgement return, and only then post
   Escape.
5. Validate observer lifecycle, z-order rollback, bounded failure, final product counters, output
   framing, and all existing process/session gates.
6. Run focused tests, type/static validation, complete v76 acceptance, and initialization checks.

## Controlled Variables

- The existing missing-target, focus recovery, atomic action batch, target identity, action
  eligibility, 12-frame acknowledgement, consumption, exclusion, and suppression policy remain
  unchanged.
- Product stdout remains readiness plus completion only.
- The observer owns no Runtime call, readback resource, GPU synchronization, product field, retry,
  or success delay.
- The exact Activated RGB limits follow the retained 30% shaded color plus 70%
  `(0.12, 1.0, 0.32)` surface mix.
- All 103 engine-runtime, 48 Prototype, and 20 reference-host tests remain required.

## Metrics

- Cold failure frame/counter state; exact PID/window; pixel baseline/peak/completion counts;
  Activated/clear/total sample counts; semantic completion time and deadline; atomic input span;
  product Activated/suppression/commit/ineligible/render-block counts; output framing; report bytes;
  workflow duration; test totals; Flavor findings; pair-copy count; and process cleanup.

## Acceptance Criteria

- The observer must be ready before recovery input and bind the same exact visible PID/window.
- Capture must observe at least a 64-pixel Activated-green rise over baseline followed by two clear
  samples within 10 seconds.
- Temporary topmost state must use no activation and be restored before Escape.
- The recovery batch remains exact and contains no delayed Escape or fixed success dwell.
- Product completion retains exact 12 Activated frames, at least one suppression frame, one commit,
  cleared acknowledgement/target state, zero render blocks, exit zero, two output values, and empty
  stderr/trailing output.
- All 18 graceful launches and the fixed 16-byte raw/invariant copy gate remain unchanged.
- No product, Runtime, renderer/GPU, source, synchronization, or process-count diff is admitted.

## Results

Three unchanged cold v76 attempts with the old helper stopped at the same final Activated gate.
Temporary diagnostics found `pending=false`, zero render blocks, and live frames `2 -> 75`, but only
three projected acknowledgement frames had completed and nine remained when the 250 ms Escape
arrived. The issue was therefore the acceptance wall-time proxy, not product backpressure.

The accepted observer derives the Activated class from the fixed surface mix, captures a baseline
before input, requires a 64-pixel rise, and recognizes completion only after two baseline-return
samples. It restores the exact window's temporary no-activation topmost state before posting
Escape. The product still emits exactly two values and exposes no observer state.

Focused cold evidence completed in 1,115.6051 ms with baseline/peak/completion counts `1/66/1`,
two Activated samples, two clear samples, and 27 total samples. Isolated Jump and
observer-then-Jump diagnostics both retained exact `+1/+1/+1` suspend/resume/reset deltas, proving
the observer child and z-order state do not leak across processes.

`canonical-prototype-v76` passed in 150.059 seconds with a 375,635-byte report. Its exact Activated
PID 21060 observed baseline/peak/completion counts `1/1237/1`, six Activated samples, two clear
samples, and 37 total samples in 1,346.6549 ms against the 10-second failure deadline. The atomic
recovery batch spanned 0.0022 ms.

Product completion retained exactly 12 Activated/object-target frames, 14 suppression frames, one
commit, one earlier missing-target ineligible outcome, cleared acknowledgement and target, exact
consumed source-qualified local ID 496, zero render blocks, exit zero, readiness/completion
sequences `1/2`, and empty stderr/trailing output. All 18 graceful launches and the zero-copy pair
gate remained exact.

All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed. `runseal :guard` retained
zero Flavor denies and five existing warnings. No product Rust, Runtime, renderer/GPU, source,
resource, synchronization, or maintained process-count change was introduced.

## Conclusion

Accepted. Activated recovery now exits on the accepted rendered acknowledgement/suppression
boundary rather than a cold-fragile wall-clock guess, while failure remains bounded and product
behavior and output remain unchanged.

## Reproduction

```powershell
runseal :guard
runseal :canonical-prototype
runseal :init
```

Generated reports remain ignored under `out/captures/`.
