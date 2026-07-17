# ADR 0123: Retired Bootstrap Probes and Resource Cleanup

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0120 Retired Bootstrap Probes and Resource Cleanup

## Context

Three maintained acceptance paths still launched processes with a historical unknown
`fallback=true` bootstrap field and retained `invalidDocument` report values. A mixed unit test
also repeated fallback and schema-1 rejection beside current path/projection failures. These
negative probes no longer informed the live schema-2 boundary and accumulated process/report
weight. The workspace also retained more than 4.9 GB of compiler and generated acceptance output
since the previous scheduled cleanup.

## Decision

- Delete every live fallback mutation, the three resulting invalid-document launches, their report
  fields, and the fallback/schema-1 unit assertions.
- Retain current schema-2 decoding, exact path/projection/bounds tests, missing-source and
  corrupt-payload no-readiness gates, readiness/restart, resource, and lifecycle evidence.
- Extend the existing recurring compatibility-removal guard as the sole authority preventing the
  retired probes from returning.
- Advance focused Prototype and full runtime report revisions to v36 and v18 for the smaller
  shapes.
- After all validation and the guarded commit, delete only the resolved workspace-local `target/`
  and `out/` trees. Do not add a recurring cleanup wrapper or touch global caches.

## Consequences

- Full acceptance performs three fewer historical process launches and emits no
  `invalidDocument` field.
- Current terminal source/payload failures remain live and strict; `Document` continues to deny
  unknown fields and accept only schema 2.
- The general guard dispatcher stays under its source-size deny threshold; bootstrap removal
  ownership lives with the existing compatibility-removal guard.
- Future builds regenerate only the resources they need. Cleanup is an explicit scheduled
  operation, not product behavior or a repository command surface.

## Evidence

`runseal :guard` and all 20 reference-host tests passed. `canonical-prototype-v36` passed in
128.894 seconds with current missing/corrupt failure evidence and every prior Prototype gate.
`canonical-runtime-v18` passed in 286.415 seconds with 1,108 Sidecar invocations, 4 warm/8 measured
publications, stable 527 handles/24 threads, private bytes
`412,213,248 -> 411,742,208`, 2/2 lifecycle cycles, and 24 artifacts / 25,346,219 bytes. Neither
report contains `invalidDocument`.

The final post-hook resource operation removed resolved repository-local `target/` and `out/`,
reclaiming 12,239 files and 4,910,745,280 bytes. `.task/`, global caches, source assets, and tracked
state were not touched.
