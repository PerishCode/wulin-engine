# ADR 0082: Self-Contained Prototype Operator

- Status: Accepted
- Date: 2026-07-15
- Experiment: 0079 Self-Contained Prototype Operator

## Context

The plain prototype is a real non-diagnostic application, but its documented manual workflow calls
Sidecar directly and assumes `out/cooked/bootstrap/runtime.json` came from an earlier acceptance run
or hand-written cooker sequence. That makes a product-like entry depend on validation residue.

Reusable setup and lifecycle behavior belongs behind a maintained wrapper. The operator also needs
an honest finite source boundary: the repository has deterministic offline cookers, not a production
world service or infinite asset source.

## Decision

- `runseal :prototype` is the sole documented manual prototype operator. It exposes
  `start|restart|stop|status`; no inspect or content-control verbs are added.
- Start requires stopped ownership, deterministically cooks the fixed zero-origin `[-8,8]²` center
  sandbox, writes the strict active-radius-2 bootstrap, reports concise setup metadata, then starts
  `sidecar.prototype.toml` and waits for existing readiness.
- Restart, stop, and status delegate Sidecar lifecycle only. They do not prepare or mutate content.
- Start refuses to rewrite sources/config while the prototype or broker is running.
- The fixed 289 centers expand to 441 cooked source regions. Generated sources and config remain
  ignored output; the range is documented as finite.
- Direct Sidecar/manual-bootstrap commands are removed from the live manual workflow. Sidecar and
  the cookers remain underlying maintained tools, not alternate public prototype entry points.

## Consequences

- A cold checkout can reach the visible prototype through one explicit self-contained command after
  toolchain init.
- Manual startup no longer inherits whatever source/config a prior acceptance happened to leave.
- The operator has bounded deterministic setup cost and a clear sandbox edge.
- Runtime, application loop, content formats, renderer/GPU, synchronization, traversal, and
  canonical acceptance semantics remain unchanged.

## Evidence

Experiment 0079 records two source-free/stopped starts reaching readiness in 11.8 and 11.1 seconds
with identical terrain/object/config hashes, exact 289-center/441-region shape, running-start
pre-mutation rejection, PID-replacing restart, and two complete zero-process stops. Init and guard
passed with zero Flavor denies.
