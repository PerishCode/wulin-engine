# ADR 0163: Retired Invalid-Key Alias Witness and Resource Cleanup

- Status: Accepted
- Date: 2026-07-19
- Experiment: 0160 Retired Invalid-Key Alias Witness and Resource Cleanup

## Context

Prototype acceptance still launched `invalid-camera-alias` on every complete
run. It posted virtual key `0x145` plus W and proved that normalization did not
truncate the key to `0x45` (E). The v74 process took 4.520 seconds and its
raw/invariant report pair occupied 8,645 minified bytes.

The product already performs a checked `u8::try_from` before changing fixed
input state. Reference-host unit coverage already exercises out-of-range
rejection, and the central Prototype guard requires the checked conversion. The
full process had become a recurring historical alias witness rather than the
owner of current behavior.

The workspace also retained more than 3.7 GB of local compiler and generated
output after the latest validation sequence.

## Decision

- Delete the invalid-camera-alias process, session dispatch, native helper, raw
  report, paired invariant, and dedicated validator chain.
- Change the focused normalization test to exact `0x145` and require that
  `0x45` remains absent from held and both edge sets.
- Make the existing Prototype session guard reject every retired spelling and
  retain its `u8::try_from` source requirement.
- Reduce the single-owner aggregate directly from 19 to 18 and advance complete
  Prototype acceptance to v75.
- Add no replacement process, report flag, alias, decoder, fallback,
  compatibility branch, product delay, or relaxed input gate.
- After validation and the guarded commit, delete only resolved direct
  workspace children `target/` and `out/`. Add no cleanup wrapper and do not
  touch `.task/`, tracked state, source assets, Git metadata, or shared/global
  caches.

## Consequences

- Checked invalid-key authority lives beside the conversion it tests.
- Complete Prototype acceptance has one fewer child process and report pair.
- Valid-key product behavior and all other graceful sessions remain unchanged.
- Future builds regenerate only the local compiler/output resources they need.
- Scheduled cleanup remains an explicit operator action rather than product or
  repository command surface.

## Evidence

`canonical-prototype-v75` passed first-run in 157.466 seconds. Its 374,253-byte
report is 17,165 bytes (4.39%) smaller than the 391,418-byte v74 baseline and
contains no invalid-key branch or retired spelling.

All 18 graceful launches retained unique PIDs, exit zero, one native-input
value, exactly two outputs, and empty stderr/trailing output. The two plain
readiness captures remained distinct, the fixed 18-pair non-duplication gate
remained at zero, and every other maintained Prototype behavior gate passed.

The focused reference-host test supplied exact `0x145` and proved `0x45`
absent from held, pressed, and released state. All 103 engine-runtime, 48
Prototype, and 20 reference-host tests passed; Flavor retained zero denies and
five existing warnings.

The final guarded inventory resolved `target/` and `out/` as direct workspace
children and measured 10,082 files / 3,731,012,889 bytes combined. After the
guarded commit, native PowerShell removed only those two trees. Both were absent
afterward, `.task/` remained present, tracked state remained clean, and no
relevant process was running.
