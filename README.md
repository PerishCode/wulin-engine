# wulin-engine

`wulin-engine` 是一个面向现代 GPU 作业模型的开源游戏引擎架构实验，并计划在
引擎能力得到验证后，以大型 Wulin Mod 消费这些能力。

项目不以商业交付、通用引擎或广泛硬件兼容为目标。当前阶段只关注单一参考平台上
的架构正确性、可测量性和负载扩展曲线。

## Status

当前处于 **R0 repository baseline**：仓库治理、目录所有权、ADR 和实验流程已经建立，
尚未创建引擎 Workspace 或 GPU 实验代码。

下一个阶段是 **R1 technical cold start**：使用 Rust、原生 D3D12 和 HLSL/DXC 建立可
测量的 GPU 实验台，然后执行第一项 GPU Scene 负载实验。

## Project model

- [Repository model](docs/architecture/repository-model.md)
- [Architecture decisions](docs/adr/README.md)
- [Experiment protocol](docs/experiments/README.md)
- [Agent operating rules](AGENTS.md)

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
