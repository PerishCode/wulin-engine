# Experiment 0163: Focused Activated-Frame Acceptance

Status: Accepted

## Hypothesis

The existing exact Activated object-feedback process can become a maintained focused
acceptance case without weakening its evidence. Reusing the real Prototype process,
frame observer, source oracle, and single invariant owner should reduce the common
debug loop from the 156.947-second full v77 matrix to less than 30 seconds.

## Scope

- Add one exact `--case=activated-frame` mode to the existing
  `runseal :canonical-prototype` wrapper.
- Build Prototype without running the full Rust test set in focused mode.
- Cook only the base and required startup-traversal centers into a separate collection.
- Run the existing Activated recovery session and its existing product, source-oracle,
  window-composition frame-completion, native-input, and single-owner invariants.
- Write an independent generated focused report that is never consumed by full
  acceptance.
- Fail before build/cook when the interactive input desktop required by window-pixel
  observation is unavailable.
- Preserve the no-argument full workflow and advance its evidence revision to v78.

Product code, Runtime, renderer/GPU, source formats, gameplay, the complete 18-process
matrix, and the wrapper set are out of scope.

## Workload

1. Measure the current single Activated process outside a maintained operator.
2. Export a narrow Activated invariant entry point from its existing owner.
3. Add exact wrapper dispatch, a two-center cook, one real process, and a separate
   report.
4. Add actionable observer-start stderr and an interactive-desktop preflight.
5. Protect the mode, its shared invariant, exact argument, minimal cook, and unchanged
   full branch with the existing nested frame-completion guard.
6. Run the focused case, full v78, repository guard, and init validation.

## Controlled Variables

- The focused process is the same exact-PID graceful session used by the full matrix.
- The product still publishes exactly readiness and completion values and exits through
  product Escape handling.
- The frame observer retains exact visible client-pixel Activated-green rise and
  two-sample clear completion.
- The source oracle, 12-frame acknowledgement, consumption/exclusion, suppression,
  zero-render-block, and transport gates remain unchanged.
- The full no-argument operator retains its tests, six-center/corrupt cook, 18
  processes, Sidecar lifecycle, and complete behavior matrix.
- No fallback capture, fixed success dwell, retry, product telemetry, new wrapper, or
  acceptance dependency is admitted.

## Metrics

- Focused elapsed time; full elapsed time; process count/PID; cooked centers; report
  bytes; Activated/suppression/render-block counts; frame observer samples/time;
  transport outcome; copied-subtree count; tests; Flavor findings; and process cleanup.

## Acceptance Criteria

- `runseal :canonical-prototype --case=activated-frame` passes in less than 30 seconds
  on an interactive desktop and writes only its separate focused report.
- The report contains one positive PID, exact native input/readiness/completion, exact
  12 Activated frames, at least one suppression frame, one commit, no retained target or
  acknowledgement, zero render blocks, and a passing visible-frame observer.
- The existing Activated invariant is the sole behavior owner and the fixed 16-byte
  raw/invariant copy count remains zero.
- Unknown case arguments fail and no second wrapper exists.
- No-argument v78 retains all 18 processes and every v77 behavior gate.
- Guard, init, tests, Flavor, diff, and relevant-process cleanup pass.

## Results

The first isolated run spent 17.4 seconds before the observer exited without surfacing
its diagnostic. Preserving pre-readiness stderr identified screen-DC capture failure
with Win32 error 5 while the Prototype window was fully on-screen at 1280×720.
`quser` and `OpenInputDesktop` showed that Windows session 1 was disconnected.
Successive disconnected-desktop failure cost fell from 17.2 seconds with the
six-center setup, to 14.0 seconds with two centers, to 6.7 seconds with the pre-build
desktop probe.

After reconnect, screen/client DC sampling intermittently retained the initial
swap-chain image even though the exact product completion independently passed all 12
Activated frames, consumption, suppression, cleared state, and zero-render-block
checks. DWM/GDI flush and short-lived DC experiments proved this was the RDP desktop
capture surface rather than product input or rendering state. The final observer uses
`PrintWindow(PW_CLIENTONLY | PW_RENDERFULLCONTENT)` against the same exact visible
PID/window while retaining temporary no-activation topmost presentation, fixed color
thresholds, and the ten-second failure bound. On timeout it now restores z-order,
posts product Escape, and carries typed `completionObserved=false` plus the exact
capture owner through final product completion before the invariant rejects the run.

The final window-composition observer passed three consecutive maintained focused runs
in 13.956, 13.716, and 13.789 seconds. The final 27,314-byte report contains one unique
PID, exact product Escape, 12 Activated frames, 13 suppression frames, one commit,
cleared target/acknowledgement, zero render blocks, and zero copied subtrees. Its frame
observer completed in 1,808.173 ms with baseline/peak/completion `0/777/0`, six
Activated samples, two clear samples, and 20 total samples.

The representative 13.789-second focused run is 11.38 times faster than the
156.947-second v77 baseline and removes 91.21% of its elapsed feedback cost. It cooks
only the base and startup-traversal centers and writes a report that is 7.33% of the
final full report.

`canonical-prototype-v78` passed in 160.144 seconds with a 372,736-byte report. It
retained 18 unique raw launch PIDs, 18 native-input/readiness/completion triples, 17
Escape reasons, one window-close reason, and zero retired raw transport aliases or
paired flags. Its Activated process retained exact 12 Activated frames, 12 suppression
frames, one commit, cleared state, and zero render blocks; the observer completed in
1,557.354 ms at `0/724/0` with six Activated samples and two clear samples. The fixed
16-byte pairwise copy count remained zero.

All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed. The 11
acceptance support tests, static focused-owner/minimal-cook/argument guard, Flavor zero
denies/five existing warnings, init, diff checks, and process cleanup all passed. No
product, Runtime, renderer/GPU, source format, gameplay, synchronization, process count
in full acceptance, or resource ownership changed.

## Conclusion

Accepted. The common Activated debugging path is now one exact real process with an
11.38-times shorter feedback loop, deterministic window-composition evidence, fast
desktop-prerequisite failure, and no authority to replace the complete matrix.

## Reproduction

```powershell
runseal :canonical-prototype --case=activated-frame
runseal :guard
runseal :canonical-prototype
runseal :init
```
