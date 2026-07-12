# Architecture Decision Records

ADRs record durable architecture decisions. They do not replace experiment evidence.

## Naming

Use `NNNN-kebab-case.md`. IDs are monotonically increasing and never reused.

## Status

- `Proposed`: under active discussion and not binding.
- `Accepted`: current repository policy or architecture.
- `Rejected`: evaluated and deliberately not selected.
- `Superseded`: replaced by another ADR, which must be linked.

## Maintenance

- Do not rewrite the decision history after acceptance.
- Correct spelling or broken links in place; supersede material changes with a new ADR.
- Link the experiments or measurements that justify performance-sensitive decisions.
- Update `AGENTS.md` when an accepted ADR changes repository-wide operating rules.
