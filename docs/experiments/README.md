# Experiment Protocol

Experiments are isolated proofs of falsifiable architecture hypotheses. They are not
production modules and must not become dependencies of engine code.

## Identity and location

- Experiment definitions and implementations live in `experiments/NNNN-kebab-case/`.
- Use the next unused numeric ID. IDs remain permanent regardless of outcome.
- Use the template in this directory before creating implementation files.

## Required evidence

Every experiment defines its hypothesis, controlled variables, workload sweep, metrics,
acceptance criteria, environment, and reproduction command before conclusions are drawn.

Performance experiments report distributions and scaling curves. Average FPS alone is
not evidence. Relevant measurements include CPU and GPU timings, upload volume,
allocation behavior, memory use, visible and submitted work, and synchronization.

## Output ownership

- Commit the experiment definition, reproducible configurations, implementation, and
  concise result summary.
- Write bulky raw CSV, captures, logs, crash dumps, and generated binaries under
  `out/experiments/NNNN-kebab-case/`.
- Never treat ignored output as the only source for an accepted conclusion.

## Outcomes

- `Accepted`: the hypothesis passed its predefined criteria.
- `Revise`: evidence was inconclusive or exposed a correctable design problem.
- `Rejected`: the hypothesis failed or the mechanism is unsuitable.

Accepted experiments may inform an ADR. Production promotion requires a deliberate
rewrite into the owning module and a stable regression benchmark.
