# ADR 0160: Single-Owner Prototype Native Input

- Status: Accepted
- Date: 2026-07-19
- Experiment: 0157 Single-Owner Prototype Native Input

## Context

Every graceful Prototype acceptance launch emitted both `postReadinessInput` and
`exitInput`. Seventeen of 19 launches nested the terminal native action under
the first field and serialized the exact same value again under the second.
Those duplicates alone occupied 11,217 minified bytes. The sustained capacity
launch carried two distinct actions, while window close paired a null first
field with its direct close action.

No generic completion validator consumed the pair. Eleven branch validators read
the nested action and then compared it with the second field; the other six
duplicate fields were ignored. The shape represented compatibility duplication
rather than independent evidence.

## Decision

- Replace both raw graceful-launch fields with one required `nativeInput`.
- Keep every compound phase explicitly named inside the one value.
- Store the sustained capacity launch's separately posted Escape under
  `nativeInput.terminal`.
- Store direct window-close evidence as that launch's complete `nativeInput`.
- Migrate every current camera, Jump, locomotion, boundary, focus, object, and
  window-close validator to the sole owner.
- Delete all equality-only terminal duplicate assertions.
- Make the existing Prototype session guard scan every current
  producer/validator and reject both retired spellings while remaining exactly
  500 lines.
- Advance complete Prototype acceptance directly to v72.
- Add no alias, decoder, dual write, optional field, report-version branch,
  fallback, retry, telemetry, product delay, relaxed threshold, process, product
  behavior, Runtime/GPU/source/ synchronization owner, or workspace resource
  cleanup.

## Consequences

- Each graceful launch has one unambiguous native input evidence owner.
- Simple terminal sequences appear once; compound sessions retain all phases
  without duplication.
- The only separately posted terminal action is explicit under the same owner.
- The report becomes smaller while exact messages, timing, PID/window/thread,
  behavior, and completion checks remain live.
- Current acceptance sources contain only the central guard's forbidden
  spellings.
- Experiment 0157 performs no resource cleanup; the scheduled
  compatibility/resource boundary remains Experiment 0160.

## Evidence

`canonical-prototype-v72` passed first-run in 174.608 seconds. Its 441,885-byte
report is 21,590 bytes (4.66%) smaller than the 463,475-byte v71 baseline and
contains zero retired fields.

All 19 graceful launch reports contain one `nativeInput`, use 19 unique PIDs,
exit successfully, emit exactly two output values, and retain empty
stderr/trailing output. Exactly one raw launch contains `nativeInput.terminal`,
preserving the sustained capacity session's independent Escape; window close
retains direct `WM_CLOSE` as its complete input.

Activated retained one commit, one ineligible action, and 12 feedback frames.
Rejected retained zero commits, two ineligible actions, and 12 feedback frames.
Sustained retained one commit, one post-readiness ineligible action, 12
Activated frames, and 12 capacity-Rejected frames.

All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed. The
existing session guard remained 500 lines; Flavor retained zero denies and five
existing warnings.
