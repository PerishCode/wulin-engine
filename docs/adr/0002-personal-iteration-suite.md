# ADR 0002: Personal Iteration Suite

- Status: Accepted
- Date: 2026-07-12
- Supersedes: None
- Superseded by: None

## Context

PerishCode projects share three product-neutral tools for recurring repository concerns:
Flavor for check-only code-shape policy, Runseal for explicit local resources and
operator wrappers, and Sidecar for local multi-process lifecycle identity and control.

Reimplementing those concerns inside Wulin Engine would create parallel scripts,
environment conventions, checks, and process state that drift from the owner's other
projects.

## Decision

- Consume installed stable-channel `flavor`, `runseal`, and `sidecar` CLIs. Do not add
  their implementation crates as engine dependencies or vendor their source.
- Use `runseal.toml` and Deno `.ts` wrappers as the canonical repository operator surface.
- Use `runseal :init` for prerequisite validation and hook installation.
- Use `runseal :guard` for the canonical local validation path.
- Use `flavor.toml` to keep code-shape policy consumer-owned and check-only.
- Keep `sidecar.toml` target-free until a real long-lived client, server, asset daemon,
  or inspection process exists. The native workbench now satisfies that gate. Do not
  model one-shot experiments as lifecycle targets.
- Add Sidecar targets only when their binaries accept and ignore the canonical
  `--sidecar-stamp` argument.
- Keep generated state under ignored `.local/`, `out/`, and tool-owned data homes.

## Consequences

Repository operations have one discoverable entrypoint and can reuse the owner's stable
tool contracts. Engine code remains unaware of repository tooling and local process
management.

The stable channels, rather than sibling source checkouts, define the executable
consumer contract. Sibling repositories remain implementation references and independent
projects. Sidecar initially contributed only project identity and validation. ADR 0003
promotes the native workbench into its first real runtime target.

## Evidence

- Stable metadata on 2026-07-12 resolves Runseal 0.9.0, Flavor 0.3.6, and Sidecar 0.5.1.
- Flavor 0.3.6 and Sidecar 0.5.1 are installed locally; Runseal 0.9.0 was already current.
- The three sibling repositories were inspected at their current `main` revisions before
  defining the consumer boundary.
