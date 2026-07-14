# wulin-engine

`wulin-engine` 是一个面向现代 GPU 作业模型的开源游戏引擎架构实验，并计划在
引擎能力得到验证后，以大型 Wulin Mod 消费这些能力。

项目不以商业交付、通用引擎或广泛硬件兼容为目标。当前阶段只关注单一参考平台上
的架构正确性、可测量性和负载扩展曲线。

## Status

**Canonical runtime 收敛已完成**：signed terrain、固定 50 槽
GPU residency、terrain-first composition、Sidecar 生命周期和项目自有 inspect 协议已经形成
唯一、可重复的内容运行闭环。Experiment 0033 已在该闭环内接受 schema-3 authored object
presentation：空间、local-ID、archetype、material、yaw 与 animation 作为三平面 cooked
authority 原子发布，并由唯一的 64 tick renderer clock 连续推进；时间变化不会触发内容
I/O、GPU page copy 或 pair 重发布。

Experiment 0001 已通过 D3D12 Debug Layer、GPU-based Validation、全量确定性输出校验和
两档 Compute benchmark。Experiment 0002 已通过同进程重复捕获、可见状态变化和 Sidecar
restart 后的原始像素与 PNG 哈希校验。Experiment 0003 已验证 HLSL 图形管线、程序化
几何、reverse-Z 深度、相机控制、语义对象查询和跨进程确定性空间帧。
Experiment 0004 进一步验证了同 draw 的 `R32_UINT` object-ID、区域语义统计、稳定遮挡
采样以及无需图像识别的像素到对象感知闭环。
Experiment 0005 已验证 GPU 区域作业、视锥压缩与间接绘制：逻辑实例从约 100 万扩展到
1,600 万时，活动候选、可见实例和提交形状保持固定。
Experiment 0006 已将活动实例替换为有界 default-heap 区域缓存：相邻移动只上传新进入
区域，缓存回访不上传实例，远距移动通过确定性淘汰将驻留容量限制在 49 个区域。
Experiment 0007 已通过独立 copy queue、50 个独立状态槽和不可变活动快照验证非阻塞发布：
copy fence 被刻意阻塞时，direct queue 仍持续呈现旧帧，且所有 staging 与事务容量有界。
Experiment 0008 已将确定性区域写入版本化索引 pack，并通过单后台 worker 按缺失区域读取：
相邻移动只读 5 个 chunk，缓存回访零读取，损坏 chunk 在 copy 提交前回滚且旧快照不变。
Experiment 0009 已将 cooked resident snapshot 直接送入 GPU cull/LOD、可见对象压缩和一次
间接 mesh dispatch：25,600 候选执行 69,270 个真实 meshlet，计数精确匹配 CPU oracle，且
逻辑世界规模、archetype 和 LOD sweep 均不改变 CPU 提交形状。
Experiment 0010 已在同一路径加入 GPU 动画分类、pose 压缩、128 骨层级 palette 求值和四权重
meshlet skinning：18,928 个可见角色的 shared 与 fully unique pose 负载均保持固定五次提交，
最坏样本在 GPU 上求值 2,422,784 根骨骼且精确匹配 CPU oracle。
Experiment 0011 已将几何可见性与材质求值解耦：确定性 visibility payload 通过 GPU 重建
蒙皮表面属性，固定 14,400 个 compute group 解析 1280x720 全屏；材质、mip、LOD、半径、
shared/unique pose sweep 均保持固定提交并精确匹配 CPU texel 与颜色 oracle。
Experiment 0012 已将上一兼容帧的确定性 winner 构造成完整 reverse-Z 层级，并通过固定
100/1/100 组 classify/prefix/stable-scatter 在 mesh execution 前保守剔除遮挡角色；高遮挡
视角精确消除了 74.916% 源 meshlet 及对应蒙皮作业，最终 visibility、颜色和语义附件不变。
Experiment 0013 已将全局整数高度格点写入独立 4 KiB 地形 payload，经单后台 worker、受保护
50 槽缓存和独立 copy queue 发布后，以一次固定 400 组 mesh dispatch 展开 25 个区域；CPU/GPU
对 40 条共享边和 1,320 个样本零差异，I/O/copy 阻塞、损坏回滚与 restart 均保持旧帧不变。
Experiments 0014-0017 已依次验证 GPU patch LOD、精确跨 LOD 边投影、地形/角色原子组合、
任意位置精确 grounding，以及“全分辨率物理地面 + 可见 LOD 近似”的接触误差合同。
Experiment 0018 进一步让 camera 直接驱动区域中心：仅允许一个在途 pair 和一个 latest-wins
目标，held I/O、连续跨区、teleport、失败、disable/catch-up 与 restart 均不暴露半新半旧快照。
Experiment 0019 已将全局 XZ 表示为 signed 64-bit region 与半开局部坐标，并在转换为 GPU
`f32` 前完成整数 region 差值；±2^40 region anchor 与 ±4 region rebase 保持颜色、PNG、
object-ID 和诊断附件字节一致，25,600 点精确 oracle 零差异。
Experiment 0020 已让 signed 64-bit global region 成为地形缓存逻辑身份，同时通过固定局部
alias 继续消费 format V1 与现有 GPU/语义合同；±2^40 anchor 不会因局部 ID 相同而误命中，
相邻移动严格保留 20、读取/上传 5 个区域，驻留回访保留 25 且零读取。
Experiment 0021 进一步让同一个 signed window 同时拥有地形与生成对象缓存身份，并在两套
独立物理槽之上只发布一个匹配的 global/local pair；三类 I/O/copy hold 均保持完整旧帧，
远端别名重绑不会误命中，两套缓存的相邻移动都严格保留 20、上传 5 个区域。
Experiment 0022 已让 camera 在冻结的 signed origin 内自动驱动上述原子 pair：半开边界与
local traversal 完全一致，global center 通过 checked integer delta 精确生成；held/latest-wins、
缺块阻塞、disable/catch-up、溢出拒绝和 restart 均保持单在途 pair、单最新目标与完整旧帧。
Experiment 0023 已将地形源升级为 signed `i64` key 的 V2 pack，并用完整 index hash 作为不可
伪造的 source namespace；同一全局窗口从 local center 64 重绑到 96 时保留全部 25 个 GPU 槽，
I/O 与上传均为零，不同 namespace 则严格全 miss。缺块、损坏和不支持模式均在 copy 前回滚。
Experiment 0024 已将 V2 地形的 camera、位置、LOD 和语义投影到以 `(64,64)` 为中心的固定
活动窗口；同一 signed window 在 local center 2、64、96、125 下的 view matrix、颜色、
object-ID、诊断附件和 LOD oracle 均字节一致，同时保留 25 个 canonical slot 且零 I/O/upload。
Experiment 0025 已用独立 object source、signed region cache key、region-local payload 与 stable
seed 将生成对象接入同一 camera-relative projection，并恢复 V2 地形/对象原子组合；固定窗口
alias 对两套缓存均为 25/0，相邻窗口均为 20/5，terrain source 切换不会使对象误失效，三类
hold、损坏回滚、语义反查、骨骼 CPU/GPU oracle 与完整附件均通过。
Experiment 0026 已在规范 V2 traversal 上接受有界原点滚动：每轴离开 `[32,96]` 安全带时，仅在
匹配地形/对象 pair 提交边界同步更新 basis 与 camera。alias 97 到 64 的同窗口规范化保持两套
缓存 25/0 且附件字节一致；单轴边界为 20/5、双轴对角线为 16/9，hold、失败、catch-up、
restart 与 64 组 release 扫描均未暴露混合坐标帧。
Experiment 0027 已复用既有 50 槽缓存和匹配事务实现单目标遍历预取：四米边界带内先完成下一
窗口的 20/5（对角线 16/9）数据移动，但丢弃 speculative active mapping；实际跨界时两半均
为 25/0、terrain I/O 为 0 byte。三类 gate promotion、反向 stale work、缺块/损坏、rollover、
disable/restart 与 32+32 release 扫描均保持原子发布和有界 backpressure。
Experiment 0028 已引入可替换运行时生成的 signed V2 object pack：完整 header/index hash
负责缓存来源身份，独立 authored namespace 保持 stable seed 与 generated payload 逐字节一致。
相邻/对角/回访严格读取 `5/9/0` 个 chunk，terrain/object 可独立换源且不会交叉失效；object I/O
与 copy promotion、缺块/损坏回滚、disable/restart 和 32+32 release 扫描均保持既有 GPU 合同。
Experiment 0029 已让 cooked object record 成为 GPU-published authority，只对活动页做有界
readback，并以 pack-index checksum 连接磁盘来源、GPU 槽和 oracle；运行时不再从程序化 fixture
重建权威 payload。Experiment 0030 又加入与 record 原子发布的 authored local-ID plane，使
stable key、动画、材质和语义行为不受 pack 内物理记录顺序影响。
Experiment 0031 最终删除 local/schema-1/generated/standalone 与递归 wrapper 路径，只保留 idle
shell 和 canonical composition。直接 403 秒验收通过 reordered source、完整附件、四类 hold、
损坏回滚、rollover、32+32 traversal、额外 64 次同进程资源平台和 16 次完整生命周期；Runseal
现仅保留五个 wrapper 与一个 support 文件。
Experiment 0032 将 archetype、material、Q16 yaw 与 animation clip/phase/variant 从 stable key
推导改为 schema-3 cooked presentation plane。426.1 秒直接验收通过两种物理乱序、四种独立
展示变体、25/5/9 三平面复制、presentation 损坏回滚、32+32 traversal、64 次资源平台和 16
次生命周期；位置、local-ID、grounding、contact 与语义身份在展示变体间保持不变。
Experiment 0033 接管此前固定为 0 的 animation time tick，以 renderer-owned 64 tick frame
clock 在每个 canonical frame 提交后推进。449.1 秒直接验收通过 tick 0/1/64 精确步进与回环、
11 帧自动推进、事务挂起时旧 pair 持续动画、无内容重发布，以及全部既有 traversal、资源平台
和生命周期回归；pause/set/step 让验收仍可冻结到逐字节确定帧。
Experiments 0034-0036 将固定的 Khronos Fox 依次以离线 cooked geometry、material/texture 和
24-joint/three-clip rig 接入同一 canonical meshlet/surface/occlusion 路径。运行时不解析 glTF、
不合成 fallback，也不增加 imported renderer；10,538 个可见 Fox 压缩为 64 个共享 Walk pose，
CPU/GPU palette 最大差异为 `2.3283064e-10`，完整动画边界与生命周期证据通过。
Experiment 0037 以固定 4,800-unit presentation second 和每帧 80 unit 消费已 cooked 的 source
duration。Survey/Walk/Run 精确映射到 16,400/3,400/5,560 units，并与 fixture 形成
31,002,560-frame 公共周期；539.4 秒直接验收确认 Walk frame 0/42/43/85 对应 phase 0/63/0/0，
公共周期回环、全部 traversal/rollback、64 次资源平台和 16 次生命周期均通过。时钟仍由
renderer 按提交帧驱动，不引入 wall clock、clip transition 或 root motion。
Experiment 0038 复用遮挡前的 10,538 个 camera-visible animated object、既有 camera LOD、
grounding 与 pose palette，以一次间接 depth-only mesh dispatch 生成固定 1024² D32 方向硬阴影。
562 秒直接验收中，受控阴影图包含 88,557 个占用 texel，六个 receiver 的 GPU/CPU 阴影判定、
texel 与最终颜色全部精确一致；source-duration 循环、alias/revisit、32+32 traversal、64 次资源
平台和 16 次生命周期均保持确定性与有界资源。该能力不增加 CPU draw list、light-space cull、
第二套 LOD/pose authority，也尚未定义地形阴影、离屏 caster、级联或过滤。
Experiment 0039 将此前位于 workbench 私有目录的 scene/world、streaming/composition、renderer、
presentation time、shader 和 GPU lifecycle 原样晋升到 `crates/engine-runtime`，由一个 `Runtime`
facade 持有唯一 renderer 与 scene。579.4 秒直接验收中，迁移前固定的 color、PNG、object-ID、
diagnostic、light-matrix 和 shadow-depth 六个哈希逐字节不变；32+32 traversal、64 次资源平台与
16 次生命周期全部通过。workbench 现在只保留窗口/消息循环、inspect transport、capture 落盘、
perception shaping 与进程报告，不再拥有或拼装 renderer 子系统。
Experiment 0040 将 presentation timeline 的可变状态与 pause/set/step/advance 权力从 skeletal
renderer 上移到 `Runtime`。每帧先采样不可变 tick，renderer、GPU constants、capture/probe 与
CPU oracle 共同观察该 pre-commit 值；仅当 canonical frame 成功返回后 runtime 才提交一次自动
推进。598.8 秒直接验收保留六个固定哈希、Walk 的 0/63/0/0 phase、32+32 traversal、零瞬态
handle 增长和 16 次生命周期。该事务仍完全 frame-driven，尚未引入 wall-clock、delta time、
fixed-step、simulation time 或 input sampling。
Experiment 0041 在 workbench 宿主内接管 Win32 key/system-key/focus 消息，并在消息泵排空后、
inspect 与 frame 之前生成有界输入事务。533.7 秒直接验收中，两次进程运行都把 11 条原生消息
归一化为 10 个状态变化，精确复现同一 stream/held-state 哈希；重复 down、孤立 up 与失焦释放
均受控，隔离重放不修改实时状态。全部 GPU 哈希、32+32 traversal、零增长资源平台和 16 次
生命周期仍通过。该边界尚不定义相机控制、action mapping、simulation sampling 或持久 replay。
Experiment 0042 为同一 workbench 加入严格 schema-1 bootstrap 文档。无效字段、缺失 source 与
异步损坏 payload 都以非零状态退出且不发 readiness；有效启动和 restart 都在 8 个隐藏帧内完成
双源原子 pair，并仅在 canonical frame 已渲染后 ready。559.8 秒直接验收保持普通 idle-shell
启动、输入重放、六个 GPU 哈希、32+32 traversal、资源平台与 16 次生命周期。该配置尚不包含
相机、输入映射、模拟、actor、内容发现或新应用。

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
runseal :canonical-runtime
runseal :workbench start
runseal :workbench inspect
runseal :workbench input
runseal :workbench input-record-start
runseal :workbench input-record-stop
runseal :workbench input-replay
runseal :workbench terrain-open out/cooked/my-source/terrain.wlt
runseal :workbench objects-open out/cooked/my-source/objects.wlr
runseal :workbench schedule 0 0 0 0 2
runseal :workbench probe
runseal :workbench capture operator-check
runseal :workbench perception operator-perception
runseal :workbench camera
runseal :workbench stop
# With out/cooked/bootstrap/runtime.json prepared:
sidecar start --config sidecar.bootstrap.toml
sidecar stop --config sidecar.bootstrap.toml
```

`sidecar.toml` owns the debug-layer correctness workbench and `sidecar.benchmark.toml`
owns the release measurement workbench. Sidecar starts each process tree,
waits for renderer and inspect readiness, discovers stamped processes, and closes the
entire local runtime through one manifest.

`runseal :canonical-runtime` is the only end-to-end engine acceptance workflow. It cooks
signed terrain and schema-3 object sources directly, validates explicit presentation,
deterministic presentation time, fixed camera-visible directional object shadows, canonical
runtime and timeline ownership, successful-frame transactions, deterministic host input/replay,
strict configured canonical readiness,
composition, fault rollback,
traversal/prefetch/rollover, the 64-publication
resource plateau, and 16 complete lifecycle cycles without invoking an older experiment workflow.

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
