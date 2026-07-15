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
authority 原子发布，并由唯一的 runtime-owned source-duration presentation timeline 连续推进；
时间变化不会触发内容 I/O、GPU page copy 或 pair 重发布。独立的显式 60 Hz simulation
schedule、调用方持有的垂直 fixed-step motion 和有上修正上限的平面 terrain-body transaction
已经建立，并已固定 planar-first 的单 tick 组合顺序。`Runtime` 现持有一个带代际句柄、exact
motion 与 schema-3 presentation 的 capacity-one actor，prototype 已在 canonical publication 后
创建一个精确 grounded imported-Fox actor；
reference host 已将 bounded monotonic elapsed policy 与至多两个等价
transition 的 Win32 activation batch 组成唯一的 activation-before-sample 操作；prototype 现以
Ready-only fixed gravity command 驱动 live schedule/actor transaction，并在每个 live frame 前通过
唯一内部投影应用固定 actor-relative camera rig；capacity-one actor 也通过唯一
skeletal/surface/shadow/occlusion 路径进入 GPU；仍无多 actor store、水平速度或
locomotion controller。live simulation/actor mutation 只保留 typed advanced/render-blocked
事务：只有通过 published 但缺失于 non-prefetch pending window 的候选是无提交背压，
published-window 与其余错误仍终止；
不再暴露独立 schedule、body lifecycle 或 retained single/batch bypass。

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
Experiment 0043 将已验收的 Win32 window/message、输入 journal 与 canonical bootstrap 晋升到
`crates/reference-host`，并新增无 inspect 的 `apps/prototype`。prototype 强制配置启动，只在
canonical frame 后显示并 ready，持续运行同一 `engine-runtime`，Escape 仅负责宿主退出。587.5 秒
直接验收中，三类失败均无 readiness，首次/重启在隐藏 frame 8/9 达到同一配置与签名目标；六个
GPU 哈希、32+32 traversal、零增长资源平台和 16 次生命周期仍通过。相机、模拟、terrain contact、
actor 与 gameplay interaction 仍留在后续独立门。
Experiment 0044 在当前已提交 terrain snapshot 上加入精确只读 CPU 高度查询：输入是 signed
global region 与半开 `[-4096,4096)` 的局部 Q9，输出是分母固定为 65,536 的高度分子和三角
分类。616 秒直接验收对 5x5 窗口执行 76,800 点并与既有 grounding oracle 零差异；reorder、
alias、四类 hold、损坏回滚、32+32 traversal、资源平台、restart 与 16 次生命周期均保持精确。
查询不分配、不读源、不触碰 GPU，也尚未定义 normal、slope、material、body 或 locomotion。
Experiment 0045 删除了 canonical 收敛前遗留的 calibration scene、split-world 状态、六个
`scene/world` inspect verb 和专属 shader/draw pipeline，并将现行 depth/semantic attachment
迁到中性 frame-target owner。645.2 秒验收中，idle shell 的 921,600 个 semantic 值全部为 0，
六个旧 verb 均返回 `unknown_event`；全部 canonical/shadow/query hash 保持精确，527-handle 平台
零增长，32+32 traversal 和 16 次生命周期通过。idle shell 现在明确只做清屏，不再伪装场景。
Experiment 0046 在该精确 terrain snapshot 上接受了调用方持有的垂直 body-contact 事务：正
separation 保持悬空、零值严格 touching、负值只施加最小向上修正，不引入 actor、持久状态、
模拟时钟、重力或移动。显式验收一次解析 230,400 个 body，三类各 76,800、零 oracle mismatch；
通用 probe 仅保留 225 个确定性见证。701.5 秒最终流程中，四类 hold、失败回滚、32+32 traversal、
531-handle 零增长平台和 16 次生命周期全部通过，既有 GPU/capture hash 不变。
Experiment 0047 在 `Runtime` 中接受了与 presentation 独立的有理数 60 Hz simulation
schedule：调用方只可显式提交不超过 125 ms 的 elapsed nanoseconds，每次产生 0..=8 个 step，
所有 tick、余数和计数器以 checked transaction 原子提交，不做 clamp、drop 或 backlog。
一小时探针以 28,800 次调用得到 216,000 tick、零余数，7/8-step 各 14,400 批，重放哈希完全
一致。692.8 秒最终流程保持 32+32 traversal、531-handle 零增长平台、16 次生命周期和既有
GPU/capture/query/contact 哈希；prototype 尚未采样 wall clock，也没有 gravity 或 locomotion。
Experiment 0048 接受了第一个显式 simulation-step 空间消费者：调用方持有 body 与 Q16
逐步垂直速度，每次显式调用以半隐式积分消费恰好一个 60 Hz tick，再由已提交 terrain snapshot
和最小向上 contact 修正决定 grounded。两种一秒 schedule 分片跨进程得到同一 60-step 哈希，
首次接地为 tick 19，随后保持精确零速度；不可用 snapshot、畸形输入和三类溢出均显式失败。
33 个聚焦测试、31 秒进程门和 `runseal :guard` 通过。该变更未触碰 frame/GPU/lifecycle，因此
没有机械重复约十一分钟全量流程；新门已进入 live wrapper，留给下一次真正影响全量证据的候选。
Experiment 0049 将 query 专属的位置命名直接收敛为唯一 `TerrainPosition`，并接受 signed-region /
half-open local-Q9 的精确平移：正负、对角和多 region 跨越都以 Euclidean normalization 得到唯一
表示，任一 `i64` region 溢出则整次失败。测试专用 65,536-case sweep 覆盖完整 local 域、远距
signed region 和全宽 `i32` 位移，独立 `i128` oracle 零 mismatch，重放哈希为
`8bf1a9181426aadf6970009165f1269e9358463c58e2cca734435a5bc02ff683`。37 个聚焦测试和 11.1 秒
`guard` 通过；纯值语义未新增进程 probe 或全量 GPU 验收，也未提前接受坡度、台阶或 actor 策略。
Experiment 0050 完成强制历史清理：一次性通过的 230,400-body dense contact checkpoint 只保留在
0046 报告/ADR 中，live inspect verb、workbench/runtime/renderer 调用链、双 coverage mode 和 recurring
assertion 全部删除，不留 alias 或 hidden flag。9.33 秒 fresh-process 门确认旧 verb 在发布前后均为
`unknown_event`，225-body witness 仍三类各 75、哈希完全不变，direct contact 三类保持精确；37 个
测试和 4.6 秒 `guard` 通过。每次 canonical run 因此少做 76,800 次重复 query 和 230,400 次
contact resolution；该 CPU 诊断删除未触碰 frame/GPU/lifecycle，仍没有机械重复全量流程。
Experiment 0051 将 canonical Q9 平面位移与 exact terrain contact 组合为调用方持有的原子事务：
目的地所需向上修正不超过显式 Q16 上限时接受 resolved body 并保留垂直速度，超过时 output 与
input 完全相同；下坡 separated body 不向下吸附。真实 signed snapshot 上的 128-Q16 高差通过
等上限、宽松上限和阻挡分支，跨 region seam、零上限、下坡、错误回滚与 replay 均通过，哈希为
`391d3dde3b853590da02f45137cd554bd430be7b0004c1dc639dac2cd2d6d23a`。43 个聚焦测试、13.90 秒
进程门和 4.3 秒 `guard` 通过；该 CPU transaction 未触碰 frame/GPU/lifecycle，因此未重复全量流程。
Experiment 0052 将该平面事务固定在 vertical fixed step 之前：台阶判断只观察 tick 起点 body，
接受路径复用目的地 terrain sample，非零阻挡才按目的地/原点顺序做两次 query；下坡在同一 tick
立即得到负速度，阻挡也不抑制原点的垂直推进。真实 128-Q16 高差与两种 60-step 分组得到相同
最终状态及 SHA-256 `7463970a8748a5aa02567c2ea94b64d2b8e527968360d30b34cef2568db02142`。
49 个聚焦测试、20.52 秒进程门和 5.3 秒 `guard` 通过；仍未引入 live driver、body store 或全量
GPU/lifecycle 变化。
Experiment 0053 将首个 simulation body 的留存权交给 `Runtime`：容量刻意固定为一，spawn/read/
despawn 只接受当前非零代际句柄；占用、错误或陈旧句柄和代际耗尽都显式失败且不修改状态。
销毁后重建从 generation 1 前进到 2，旧句柄不能命中新 body；完整进程重启恢复为空。54 个聚焦
测试和 18.48 秒进程门通过，两次独立进程 replay 的 SHA-256 均为
`74f1b0e22b17fdc603d66082773e0824e0a54307364b0e57c1162f4bc1e11ced`。该边界尚不驱动
stored motion，也不提前选择多 actor 容量、ECS、input 或 presentation policy。
Experiment 0054 让该 retained body 首次执行原子 stored advance：先校验并复制 live generation，
完整复用 planar-first copied-value transaction，只有全部 query/contact/arithmetic 成功后才以同一
generation 提交 output。accepted/downhill 各一次 query，blocked 两次；snapshot 越界与速度溢出
后读回值完全不变。56 个聚焦测试和 23.36 秒双进程门通过，result/replay SHA-256 均为
`54dacac84b69c1ef1e98d127de23e646b0d18e6c9934e50d3e832abefa56f529`。操作仍不读取
simulation schedule，不采样 wall clock/input，也不绑定 actor presentation。
Experiment 0055 完成逢五强制清理：删除 `canonical.terrain.body.step/translate/advance` 的完整
inspect、protocol、workbench、`Runtime` forwarding 与 recurring support 链，不留 alias；三个纯
transaction 模块和 56 个测试仍作为 retained advance 的内部权威。三个 support 文件共删除 1,018
行，direct wrapper 从 508 降至 405 物理行并移除临时 520-line Flavor 例外。提取后的 typed setup
生成并校验全部 11 个确定性 pack/corruption artifact；10.74 秒单进程门确认十个历史 verb 均为
`unknown_event`，现有 retained 路由的失败回滚保持精确。该 control/setup-only 变更没有重复约
十分钟全量流程，稳定 guard 会阻止旧链回流。
Experiment 0056 在连接 schedule 前先证明 retained body 侧的批事务：显式 0..=8 tick 只复制
live generation 一次，在局部 motion 上逐步执行，全部成功后才替换 slot 一次；第 9 步请求在 query
前失败，零步是零 query 的精确 identity。33.79 秒三进程门中，一个 8-step batch 与 8×1 retained
advance 得到 byte-identical 状态和 8 次总 query，SHA-256 为
`110128827404dbe0dabc06fb31ccb9c5e66b5294d3728ff32a07fb986757b9f0`；受控 snapshot 边界在
第 3/8 步失败后仍完整回滚。59 个测试与 guard 通过，schedule/presentation/GPU/frame 均不变。
Experiment 0057 将 caller-supplied elapsed 与 retained batch 组成一次双提交：先复制 schedule/body，
在副本上产生并执行 0..=8 step，全部成功后才提交两者。32.59 秒三进程门中，1 ns 只提交 remainder
60 且零 query；8×125 ms 与 60-call nominal 分片都到 tick 60/remainder 0、body 完全相同且各 60 次
query，SHA-256 为 `d816aa37f7c5ad56d4bfe3c9d062dec4dda276ed9b3e51c838f9a29fa7027c8a`。
第 3/7 步 snapshot failure 与第 1 步 velocity overflow 都同时回滚 schedule/body；尚未采样 wall clock
或从 frame loop 驱动。
Experiment 0058 在 `reference-host` 中独立接受 bounded monotonic elapsed policy：首样本与 resume 后
首样本只建立 baseline；0、16,666,666、16,666,667 与 125,000,000 ns 原样 ready，125,000,001 ns
显式 stalled 且推进 baseline，下一次 1 ns 立即恢复。suspend 不累计 60 秒间隔，resume 强制 reset，
重复 transition 幂等，monotonic regression 完整回滚。14 个聚焦测试与 guard 通过，确定性 replay
SHA-256 为 `3a873571ca7a754272eeaecb0dc7fe9d5183703e88a100a1907cc9ae8bacea7d`；该 clock 尚未连接
Win32 focus、composition root 或 Runtime，因此没有运行进程/GPU/全量验收。
Experiment 0059 补齐独立 Win32 activation transport：`WM_KILLFOCUS` / `WM_SETFOCUS` 只更新
constant-state reducer，不建立事件队列；首次 drain 发布当前状态，单次变化产生一个 transition，
同轮 loss+resume 或 resume+loss 产生保留中断顺序的两个 transition。长度 1–8、两种初态的全部
focus sequence 均严格规约为 0–2 项，19 个测试与 guard 通过；replay SHA-256 为
`eed23eab9230c591d895eaede20bbe19284a0bf309302b8d692ed8c1029738f1`。该 transport 尚未驱动
HostClock、application loop 或 Runtime。
Experiment 0060 完成逢五清理：删除 `simulation.advance` / `simulation.probe` 和 retained
single/batch 四条独立控制链、四个 `Runtime` forwarding、obsolete result/live probe，以及三份
共 848 行 recurring support；canonical wrapper 同时移除 8 个进程启动点。62 个 runtime 测试保留
纯 schedule、single/batch 与 dual authority。53.3 秒 fresh setup + dual gate 在既有首进程中确认
四个旧 verb 均为 `unknown_event`，dual SHA-256 仍为
`d816aa37f7c5ad56d4bfe3c9d062dec4dda276ed9b3e51c838f9a29fa7027c8a`，没有运行全量流程。
Experiment 0061 将两项 host policy 收敛为一次 checked transition：完整 ordered activation batch
先作用于候选 `HostClock`，随后只采样一次，全部成功才提交；loss/resume 跨越 60 秒仍只产生
reset，resume/loss 则保持 suspended，独立 public `suspend` / `resume` 已删除。21 个 focused test
与 guard 通过，组合 replay SHA-256 为
`15ab39e6b25ea2a63a97378c51f7ec73242d53d87331245174b4efffef01301e`；两个 application loop
仍不采样时间，因此没有运行进程/GPU/全量验收。
Experiment 0062 让非诊断 prototype 在 canonical publication 成功后、readiness 之前，从既有
`globalCenter` 与 local Q9 `(0,0)` 查询 committed terrain 并创建一个 generation-1 retained body。
固定 half-height 为 65,536 Q16、velocity 为 0，foot 与 terrain byte-exact touching；远端测试点
terrain/center 分别为 `76288/141824`。2 个纯测试与 50.4 秒定向 lifecycle gate 通过，两次进程
body 证据完全一致且 PID 不同，invalid/missing/corrupt 均不发 readiness，最终无残留 PID；没有
运行 workbench GPU/资源/遍历全量验收。
Experiment 0063 接通第一个真实 wall-time simulation composition root：prototype 每轮按
message pump → input/exit → activation-aware sample → Ready-only dual advance → frame 排序执行。
Reset/Suspended/Stalled 都不推进；Ready 使用全零 command，因此 schedule 发出 1..=8 tick 时每
tick 一次 terrain query，而 generation-1 body input/output byte-identical。readiness 只在首次非零
commit 及其随后 frame 成功后发布。3 个 focused test、50.75 秒 fresh-cook 定向 gate、失败/重启/
Sidecar lifecycle 与 repository guard 均通过；未运行无关的 workbench GPU 全量流程。
Experiment 0064 将临时 retained-body 权威直接晋升为 capacity-one runtime actor：一个代际同时
拥有 exact `TerrainBodyMotion` 与既有 schema-3 `PresentationRecord`，prototype 固定选择 imported
Fox archetype 7 / material 63 / clip 1。非法 archetype/material/yaw/clip/phase 在 generation 分配前
拒绝；simulation 只替换 motion，handle 与 presentation byte-exact 保持。旧 body 类型、Runtime
方法、workbench verb、readiness key 与 recurring support 均直接删除。63 个 runtime test、3 个
prototype test、7 个 region-format test，以及 27.7 秒 actor lifecycle、41.8 秒 simulation-actor、
32.21 秒 prototype 定向进程门通过；CPU-only actor 尚未接入 GPU presentation，因此未运行无关
的 canonical GPU 全量流程。
Experiment 0065 完成逢五清理：唯一 canonical operator 不再把当前报告伪装成 Experiment 0060，
revision 直接改为 `canonical-runtime-v1`，cook/capture collection 直接改为
`canonical-runtime`，当前 evidence 路径为 `out/captures/canonical-runtime/`。历史 Experiment
0060/ADR 0063 文档保留为决策记录，但没有旧路径 fallback、复制器或 wrapper alias。2.1 秒静态/
operator gate、0.34 秒 init 与 14.33 秒 guard 通过；清理未启动进程或 GPU 工作，因此没有运行
canonical 全量流程。
Experiment 0066 接受第一个 actor/render 空间接缝，但仍不冒进到 GPU：renderer 只读当前 exact
generation 与 enabled published pair，把 signed `i64` region + local Q9 映射为 active ordinal、
centered semantic region 和 window-relative Q9，并原样保留 Q16 center/half-height 与 schema-3
presentation；全局坐标不转 float，越界 actor 明确失败。69 个 runtime test、workbench check、
affected clippy 与 6.61 秒真实进程 gate 通过；`2^40`、seam、alias/rollover、window edge/outside、
stale handle、rollback 和 replay 均有证据。该路径不进入 frame/GPU resource/shader/sync，因此未
运行无关的约 10 分钟 canonical 全量流程。
Experiment 0067 先清除 actor GPU 接入前的错误依赖：52-byte visible record 在 skeletal cull 一次
写入 grounded window position、height、semantic region 与既有 presentation/pose/candidate，
surface、shadow、occlusion 不再通过 `physical_index` 回读 streamed instance/ground page。两个
25,600-record buffer 与双平面 order readback 固定增加 1,638,400 bytes，surface descriptor copy
从 62 降为 10，未新增 resource/sync/lifetime。21.1 秒 fresh frame/replay 保持 color/PNG/semantic/
diagnostic 四个外部哈希和全部 shadow/occlusion 语义，只接受确定重放的新 raw shadow hash；
276.9 秒聚焦资源门与 762.4 秒完整流程通过 32+32 traversal、active/quiescent 资源平台和 16 次
生命周期。日常 GPU 与资源回归分别由 `runseal :canonical-frame` / `:canonical-resources` 承担。
Experiment 0068 将唯一 runtime actor 作为固定 candidate 25,600 接入既有 cull group，并把
visible identity 扩为两字完整 `u64` generation。一个 112-byte 双帧 upload resource 直接绑定 root
SRV，复用已完成 frame-slot fence；无 GPU copy、新 pass/barrier/fence/wait 或 streamed-page 写入。
21.539 秒 focused gate 精确接受 visible/frustum-rejected/despawn/respawn/replay/outside-window
事务：可见 actor 只增加 16 meshlets、1,014 vertices、576 triangles、4,056 skin influences，语义
ID 98,305 覆盖 3,866 pixels；失败帧不推进 frame index 或 upload write count。固定资源声明宽度
共增加 416,148 bytes 和一个 resource object。260.779 秒资源门与 739.3 秒完整流程通过 64 次
publication 平台、prototype actor 启动/重启及 16 次生命周期，Flavor 为 0 deny。
Experiment 0069 在不扩张引擎接口的前提下接纳第一个 prototype-owned simulation policy：每个
Ready due step 固定提交 `-179` Q16 重力增量，即 60 Hz 下 `-9.832763671875 m/s²`；初始 touching
actor 每步经既有 exact contact 回到原 body 与零垂直速度。新的 `runseal :canonical-prototype`
只 cook 两个所需中心，并在 32.032 秒内通过 4 个 prototype tests、21 个 reference-host tests、
三类 no-readiness 启动失败、两次直接进程及 Sidecar restart/stop。该阶段没有输入映射、水平移动、
相机策略、引擎 API、GPU resource 或同步变化，因此未运行无关的 frame/resource/全量门。
Experiment 0070 完成逢五清理：0066 为前置证明临时公开的 `Runtime::project_actor`、crate-root
projection type、`actor.project` verb 与 238 行 recurring support 已全部删除；renderer 内部 exact
projection/preflight 及私有测试保留为唯一生产路径。自闭环旧 verb 门在 7.2 秒内确认
`unknown_event` 并清空进程，21.763 秒 `canonical-actor` 保持两代 record、四类 capture hash 和
3,866 actor pixels 精确不变。清理前 live diff 净删 282 行，不新增 alias、fallback、frame/resource
或同步路径。
Experiment 0071 接受一条只写的 actor-relative camera mutation：runtime 读取代际 actor，通过
renderer-private projection 在 checked Q9 中恢复 origin-relative scene center，再让 SceneState
一次性校验并提交 caller offset camera；不重新公开 projection 数据。prototype 固定使用
`[9,4,12]` / `[0,-1,-3]` / `60°` rig，每个 live frame 前恰好 anchor 一次。35.608 秒 focused gate
中两次独立进程都得到 `[9,6.1640625,12] → [0,1.1640625,-3]`，anchor/frame 为 3/3，三类失败仍
无 readiness 且 Sidecar 最终清空。该阶段未启用水平输入、traversal 或跨窗口移动事务。
Experiment 0072 将 renderer-private actor preflight 接入既有 simulation/actor 双提交：motion 与
schedule 仍先在副本上完整准备，canonical candidate 必须同时落在 published 与非 prefetch
pending window 后才能一起提交。31.577 秒 `canonical-actor` 在同一 held diagonal publication 下
证明一次共享窗口提交与一次精确拒绝；拒绝后 actor、完整 schedule、pending token/stage 均不变，
旧 actor 仍能成功出帧。原有两代 actor record 与四类 capture hash 保持不变；未新增 projection
surface、frame/GPU resource 或同步路径。
Experiment 0073 将该预期窗口压力从 untyped error 收窄为 `RenderBlocked`：renderer 区分 active 与
pending block，runtime 只把已通过 published 的 pending block 转为 typed outcome；frame 与 published
block 仍恢复原有精确失败，配置、算术、terrain、identity 等错误不被吞掉。36.740 秒 actor gate 得到
prepared step/query 1/1、双 commit 0/0，并保持 retained frame 与既有 GPU hashes；33.457 秒
prototype gate 接受“不重试、不积压、继续旧 actor 相机/出帧”的策略，两次直接进程均为 block
count 0、anchor/frame 3/3。该阶段仍未连接水平输入或 traversal。

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
runseal :canonical-prototype
runseal :canonical-actor
runseal :canonical-frame
runseal :canonical-resources
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
sidecar start --config sidecar.prototype.toml
sidecar stop --config sidecar.prototype.toml
```

`sidecar.toml` owns the debug-layer correctness workbench and `sidecar.benchmark.toml`
owns the release measurement workbench. Sidecar starts each process tree,
waits for renderer and inspect readiness, discovers stamped processes, and closes the
entire local runtime through one manifest.

`sidecar.prototype.toml` launches the plain configured prototype without an inspect endpoint.
It becomes visible and ready only after canonical content has rendered; close the window, press
Escape, or use `sidecar stop` to end it. The bootstrap file is generated during canonical
acceptance or may be prepared with the documented cooker formats.

`runseal :canonical-prototype` is the focused real-process prototype workflow. It runs the
runtime/prototype/reference-host tests, cooks only the two required signed centers, and proves
strict bootstrap failure, grounded gravity admission, actor-relative camera/frame ordering,
typed render-block consumption with zero normal-path blocks, direct restart equality, and Sidecar
cleanup.

`runseal :canonical-frame` is the focused real-process GPU regression workflow. It cooks a fresh
minimal signed pair, checks the exact accepted canonical frame, immediately replays it, and owns
complete process cleanup. It does not replace end-to-end acceptance.

`runseal :canonical-actor` is the focused frame-safe actor GPU workflow. It proves typed
advanced/render-blocked simulation-candidate admission against published and non-prefetch pending
windows, exact rollback and retained rendering, dynamic candidate identity, alternating frame-slot
writes, cull/surface/shadow/occlusion participation, despawn/respawn clearing, frustum rejection,
outside-window rollback, and semantic capture.

`runseal :canonical-resources` is the focused same-process resource workflow. It separates the
bounded active publication plateau from post-workload recovery to the quiescent process baseline.

`runseal :canonical-runtime` is the only end-to-end engine acceptance workflow. It cooks
signed terrain and schema-3 object sources directly, validates explicit presentation,
deterministic presentation time, fixed camera-visible directional object shadows, canonical
runtime and timeline ownership, successful-frame transactions, deterministic host input/replay,
strict configured canonical readiness, shared reference-host ownership, plain prototype
startup/restart/cleanup, exact published-snapshot CPU terrain queries,
exact caller-owned vertical terrain contact and bounded transition witnesses, the explicit bounded
60 Hz simulation schedule and its frame/presentation independence, private fixed terrain-body
motion/translation/planar-first/batch contracts, one retained terrain-body generation lifecycle
with exact failure rollback and process reset, the sole explicit elapsed schedule/body dual commit
with coarse/nominal partition equality and complete rollback,
clear-only idle behavior and retired-control rejection, composition, fault rollback,
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
