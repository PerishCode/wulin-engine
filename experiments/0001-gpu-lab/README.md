# Experiment 0001: GPU Laboratory Foundation

- Status: Accepted
- Owner: PerishCode
- Created: 2026-07-12
- Related ADRs: [ADR 0001](../../docs/adr/0001-reference-platform-and-graphics-api.md)

## Hypothesis

The reference platform can execute a deterministic native D3D12 compute workload using
the pinned Agility SDK, explicit enhanced barriers, GPU timestamps, and an explicit
fence, while producing no D3D12 error messages and no incorrect output elements.

## Scope

The experiment includes adapter selection, process-level Agility SDK exports, D3D12
validation, one Direct Queue, committed buffers, a compute pipeline, enhanced barriers,
timestamp queries, explicit completion, full output validation, and JSON reporting.

It excludes windows, swap chains, graphics pipelines, ECS, assets, Render Graph,
multi-queue scheduling, custom memory allocation, GPU Scene, and engine APIs.

## Workload

A parameterized compute shader fills a GPU-resident `uint` buffer with a deterministic
hash. Dispatches are arranged in a two-dimensional grid so element count is not limited
by the maximum X group count. Warm-up and measured iteration counts are configurable.

The default correctness workload is 1,048,576 elements. Later runs may sweep element
count and iteration count without changing the pipeline.

## Controlled variables

- Windows 11 x64 reference system.
- NVIDIA GeForce RTX 4070 Ti SUPER.
- Rust 1.94.1 MSVC toolchain.
- `windows` crate 0.62.2.
- DirectX 12 Agility SDK package 1.619.4, SDK version 619.
- Windows SDK 10.0.26100.0 DXC and Shader Model 6.6.
- No presentation or VSync.
- Fixed seed supplied by the command line.
- Correctness mode enables the D3D12 debug layer and GPU-based validation.

## Metrics

- Per-dispatch GPU timestamp duration.
- Median, P95, P99, minimum, and maximum GPU duration.
- CPU command recording, submit-and-wait, and output-validation time.
- Adapter name and dedicated video memory.
- Timestamp frequency and Enhanced Barriers support.
- Output mismatch count and deterministic checksum.
- D3D12 corruption and error messages.

## Acceptance criteria

- The pinned Agility SDK is loaded successfully.
- Enhanced Barriers are reported as supported and used for the UAV-to-copy dependency.
- Every output element matches the CPU reference implementation.
- The report contains valid, positive GPU timestamp durations.
- Completion occurs through the explicit fence with no device loss.
- The D3D12 information queue contains no corruption or error messages.
- A clean checkout can reproduce bootstrap, build, correctness, and benchmark commands.

## Environment

Runtime reports record the adapter, toolchain, SDK, revision, and workload parameters.

## Reproduction

Correctness mode requires the Windows `Graphics Tools` optional capability. Check it in
an elevated PowerShell session:

```powershell
Get-WindowsCapability -Online -Name Tools.Graphics.DirectX~~~~0.0.1.0
Add-WindowsCapability -Online -Name Tools.Graphics.DirectX~~~~0.0.1.0
```

Then run from the repository root:

```powershell
./experiments/0001-gpu-lab/scripts/bootstrap.ps1
cargo run -p gpu-lab --release -- --mode correctness --report out/experiments/0001-gpu-lab/correctness.json
cargo run -p gpu-lab --release -- --mode benchmark --report out/experiments/0001-gpu-lab/benchmark.json
```

## Results

Provisional results on 2026-07-12:

| Mode | Elements | Measured dispatches | GPU median | GPU P95 | Mismatches | Result |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| Correctness | 1,048,576 | 1 | 0.005120 ms | 0.005120 ms | 0 | Pass |
| Benchmark | 1,048,576 | 100 | 0.003072 ms | 0.003072 ms | 0 | Pass |
| Benchmark | 16,777,216 | 50 | 0.132096 ms | 0.322560 ms | 0 | Pass |

All runs loaded the application-local Agility SDK Core, reported Enhanced Barriers,
produced matching deterministic checksums, and completed through an explicit fence.

Correctness mode ran with the D3D12 Debug Layer and GPU-based Validation enabled. The
D3D12 information queue reported no corruption or error messages. Benchmark reports
explicitly state that validation is disabled so their measurements are not confused with
correctness evidence.

## Conclusion

Accepted. The pinned native D3D12 foundation executes deterministic compute workloads,
uses explicit Enhanced Barriers and fence completion, records valid GPU timestamps, and
passes full output and D3D12 validation on the reference platform.

## Promotion

No promotion occurs during R1. The laboratory remains isolated until later experiments
establish production ownership boundaries.
