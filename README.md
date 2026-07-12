# wulin-engine

`wulin-engine` 是一个面向现代 GPU 作业模型的开源游戏引擎架构实验，并计划在
引擎能力得到验证后，以大型 Wulin Mod 消费这些能力。

项目不以商业交付、通用引擎或广泛硬件兼容为目标。当前阶段只关注单一参考平台上
的架构正确性、可测量性和负载扩展曲线。

## Status

**空间校准闭环已完成**：Rust Workspace、原生 Win32/D3D12 窗口、固定 Agility SDK、
Sidecar 生命周期和项目自有 inspect 协议已经形成可重复的可见控制闭环。

Experiment 0001 已通过 D3D12 Debug Layer、GPU-based Validation、全量确定性输出校验和
两档 Compute benchmark。Experiment 0002 已通过同进程重复捕获、可见状态变化和 Sidecar
restart 后的原始像素与 PNG 哈希校验。Experiment 0003 已验证 HLSL 图形管线、程序化
几何、reverse-Z 深度、相机控制、语义对象查询和跨进程确定性空间帧。
Experiment 0004 进一步验证了同 draw 的 `R32_UINT` object-ID、区域语义统计、稳定遮挡
采样以及无需图像识别的像素到对象感知闭环。

## Project model

- [Repository model](docs/architecture/repository-model.md)
- [Architecture decisions](docs/adr/README.md)
- [Experiment protocol](docs/experiments/README.md)
- [Agent operating rules](AGENTS.md)

## Developer operations

Stable-channel Flavor, Runseal, and Sidecar provide the repository iteration surface:

```powershell
runseal :init
runseal :guard
runseal :gpu-lab correctness
runseal :gpu-lab benchmark
runseal :visual-loop
runseal :spatial-scene
runseal :object-id
runseal :workbench start
runseal :workbench inspect
runseal :workbench color 0.08 0.42 0.24
runseal :workbench capture operator-check
runseal :workbench perception operator-perception
runseal :workbench camera
runseal :workbench scene
runseal :workbench stop
```

`sidecar.toml` owns the native workbench app target. Sidecar starts the process tree,
waits for renderer and inspect readiness, discovers stamped processes, and closes the
entire local runtime through one manifest.

## Scope

- 先通过实验验证能力，再向引擎核心晋升实现。
- 以扩展曲线、帧时间、数据移动、资源生命周期和同步行为评价性能。
- 引擎核心与 Wulin Mod 保持严格的所有权边界。
- 不在实验阶段承担多平台、多厂商、旧硬件或旧图形 API 兼容工作。
- 不包含原游戏的专有代码、资源、凭据或其他无明确再分发授权的内容。

## License

除非文件另有说明，本项目按以下任一许可证授权：

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

SPDX expression: `MIT OR Apache-2.0`.

项目名称和技术研究用途不代表与原游戏权利方存在隶属、授权或认可关系。
