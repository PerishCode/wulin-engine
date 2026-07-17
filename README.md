# wulin-engine

`wulin-engine` 是一个面向现代 GPU 作业模型的开源游戏引擎架构实验，并计划在
引擎能力得到验证后，以大型 Wulin Mod 消费这些能力。

项目不以商业交付、通用引擎或广泛硬件兼容为目标。当前阶段只关注单一参考平台上
的架构正确性、可测量性和负载扩展曲线。

## Status

**Plain Prototype v0 已完成阶段封版**：`runseal :prototype start` 可从无生成 source/config 的状态
确定性准备有限 sandbox，并启动唯一 canonical renderer 下的原生窗口。当前边界包含一个 grounded
imported-Fox actor、Ready-only 60 Hz gravity/W/A/S/D transaction、Survey/Walk 与精确八向 facing、
held Shift 的固定 Run displacement/clip、随当前 Q/E quarter-orbit candidate 精确旋转的 camera-relative
locomotion、actor-local clip phase、四态 Q/E actor-relative camera
orbit、capacity-one grounded Space Jump intent
和启用一次的 engine traversal。它明确不是无限世界、
持续 product traversal、gameplay interaction、multi-actor、networking 或 Wulin content 承诺。
首个 post-v0 依赖现由 Experiment 0083 接受：strict bootstrap schema 2 显式声明包含式 signed
playable-region rectangle，prototype 在既有 runtime transaction 前按最多八 step 独立裁减可能越界的
轴。维护型 operator 在 cooked `[-8,8]²` center 内声明 `[-6,6]²` 可玩范围；runtime 对缺失 source、
published window 和 terrain query 的严格失败语义不变。

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
Ready-only fixed gravity 加固定 W/A/S/D 整数水平 command 驱动 live schedule/actor transaction；
held Shift 将最终非零准入位移从 Walk 32/23 Q9 切为 Run 64/45 Q9，当前纯 Q/E camera candidate
再用整数符号/轴置换将 local input 旋为 world XZ；同一 typed command/commit
以 Survey 表示静止、Walk/Run 表示对应非零位移，fractional 与 render-block 均不
提前改变 presentation；nonzero command 使用 Fox +X forward 的精确八方向 Q16 yaw，静止时保留最后
一次 nonzero committed facing，并在每个 live frame 前通过唯一内部投影应用已提交的四态 Q/E
actor-relative camera rig；capacity-one actor 也通过唯一 skeletal/surface/shadow/occlusion 路径进入 GPU；prototype
在 spawn 后只启用一次 composition traversal、保持 prefetch 关闭，并由首个 camera frame 调度精确
对角目标；仍无多 actor
store、水平速度/加速度或转向动画。live simulation/actor mutation 只保留 typed advanced/render-blocked
事务：只有通过 published 但缺失于 non-prefetch pending window 的候选是无提交背压，
published-window 与其余错误仍终止；
readiness 的顶层 actor authority 只发布 readiness-producing transaction 的 committed output，
不再保留 spawn-time actor/terrain 快照；也不再暴露独立 schedule、body lifecycle 或 retained
single/batch bypass。

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
Experiment 0074 接受第一个有界 prototype locomotion policy：W/S 映射负/正 Z、A/D 映射负/正 X，
cardinal 每个 60 Hz step 为 32 Q9、diagonal 固定取 23 Q9 分量、对向输入逐轴抵消，step-up 固定
32,768 Q16，重力仍为 -179 Q16。38.742 秒 focused gate 通过维护型 Win32 helper 向真实且 PID
匹配的 prototype window 投递 W；第三个进程精确得到 Z `0 -> -32`、query/step 1/1、block 0，
相机 Z 同移 0.0625m，普通两进程仍零位移且 restart-identical。prototype 没有新增 inspect/test mode，
composition traversal 继续关闭，engine API、renderer/GPU resource 与同步均未变化。
Experiment 0075 完成逢五清理：locomotion 使旧 top-level `actor.state` 暴露的 spawn 快照失真，
同层 terrain witness 也只描述旧位置；两者已从 live readiness 删除。现在顶层 current state 与
`advance.actor.output` 字节等价，`advance.actor.input` 独占 transaction 初态证明，camera 绑定同一
current actor。36.921 秒 `canonical-prototype-v4` 中普通进程 input=output=current，native W 进程为
input Z 0、output/current Z -32、step/query 1/1、block 0、anchor/frame 4/4；没有 v3 alias、fallback
字段、Runtime readback、inspect route 或运行行为变化。
Experiment 0076 接受 prototype-owned 一次性 traversal activation：canonical bootstrap 与 actor spawn
完成后启用既有 camera-driven traversal，prefetch 保持关闭。固定相机 X/Z `9/12` 在首个真实 frame
精确调度 token 2、本地 `(65,65)` / 全局 `base+(1,1)`；36.119 秒
`canonical-prototype-v5` 中三次独立进程均为 session/desired/attempt/schedule `1/1/1/1`、publication
0，且无 queue/block/failure/prefetch/rollover。该阶段只观察既有 composition status，不新增 inspect、
Runtime API、engine traversal 算法、renderer/GPU resource、同步或格式路径。
Experiment 0077 将 motion 与 desired presentation 收进唯一 typed actor-simulation command：只有
nonzero fixed step 才构造完整 actor candidate，并在既有 published/pending preflight 后与 schedule
一起提交；fractional、invalid、fatal 与 typed block 都保留完整旧 actor。30.932 秒
`canonical-actor-v3` 证明 Survey→Walk 与 X `0 -> 1` 同时提交、随后 Walk→Survey block 的
step/query `1/1` 与 schedule/actor/presentation mutation `0/0/0`；38.080 秒
`canonical-prototype-v6` 中两个静止进程保持 clip 0，native-W 进程在同一 transaction 得到 clip 1
与 Z `0 -> -32`。没有旧 scalar signature、motion-only commit type、schema-2 fallback、第二条
presentation mutation、blend/yaw policy 或 renderer/GPU/synchronization 变化。
Experiment 0078 接受 prototype-owned committed facing policy：D/S+A 等八个 normalized 方向精确
映射到 Q16 yaw `0..57344`，zero/opposed input 保留最后一次 nonzero admitted output，而 fractional
advance 与 typed block 不推进 policy。38.726 秒 `canonical-prototype-v7` 中两个静止进程保持
yaw/clip `0/0`，native-W 进程在同一 transaction 将 yaw/clip 变为 `49152/1` 并提交 Z
`0 -> -32`、step/query `1/1`；camera/frame、traversal、restart、failure 与 cleanup 证据不变。
没有 Runtime/inspect API、actor readback、第二条 presentation mutation、renderer/GPU/resource、
synchronization、format、traversal 或 camera 变化。
Experiment 0079 将手动 prototype 从“先依赖 acceptance 遗留 bootstrap，再直接调用 Sidecar”收敛为
唯一 `runseal :prototype` wrapper。`start` 在 stopped 状态确定性 cook 零原点 `[-8,8]²` 的 289 个
center/441 个 source region，写 strict bootstrap 后等待既有 Sidecar readiness；running-start 在任何
写入前拒绝，restart/stop/status 不 cook。两次 source-free cold start 分别 11.8/11.1 秒，terrain/
objects/config hashes 完全一致，restart 更换 PID，最终均为零残留进程。没有 inspect/test mode、
Runtime/application/renderer/GPU/synchronization/format/traversal 或 canonical acceptance 变化。
Experiment 0080 完成逢五清理：删除 `repository-model.md` 中滞留在 0066/0069 的 85 行重复 runtime
状态账本，以 8 行稳定规则将变化中的 live capability 唯一指向 `AGENTS.md` section 4；repository
model 只保留 dependency、directory、naming、promotion 与 data ownership。新门禁同时禁止 live
文档重新暴露 direct prototype Sidecar command；一次故意回归在 0.7 秒、Rust build/test 前被拒绝。
`init` 与单次 merge-checkpoint `guard` 通过，Flavor 为 0 deny / 5 个既有 warning；没有 runtime、
application、GPU、format、Sidecar lifecycle、generated output 或 canonical acceptance 变化。
Experiment 0081 为 runtime actor 增加 bounded animation epoch，但不增加第二套 clock：spawn 与
committed clip/rig transition 记录当前 presentation tick，renderer 每帧把 elapsed-since-epoch phase
解析到既有 56-byte actor record。37.433 秒 `canonical-actor-v4` 中 Survey global/epoch 42/42 为
phase 0、tick 46 为 phase 1；Walk 在 epoch 46 重启 phase 0、tick 47 为 phase 1，same-clip yaw 保留
epoch，fractional/block/failure 全回滚。35.284 秒 `canonical-prototype-v8` 中静止/重启均保持 epoch
1，native-W 在 live frame 3 原子提交 epoch `1 -> 3`、clip/yaw/Z `0/0/0 -> 1/49152/-32`。shader、
streamed object time、GPU ABI/resource/copy/sync、format、camera 与 traversal 均未变化。
Experiment 0082 不增加 feature，而是封存 plain Prototype v0 的诚实阶段边界。source-free
`runseal :prototype start` 在 13.3 秒完成 289-center / 441-region sandbox 与 canonical readiness，
restart 替换全部 product PID 且三份生成物哈希不变，stop 达到零进程；36.748 秒
`canonical-prototype-v8` 和阶段唯一一次 744.8 秒 `canonical-runtime-v1` 均通过。后者覆盖 32+32
traversal、64-publication resource plateau 与 16 lifecycle cycles。该实验只改 stage 文档及其
registration/index，不增加 product code、operator behavior、diagnostic、format、GPU 或 Wulin 面。
Experiment 0083 将该有限 sandbox 的 edge 从隐含 source failure 提升为显式 product policy。
bootstrap schema 2 唯一声明 inclusive signed region rectangle；prototype 读取当前 actor，并在原
transaction 前按最大八 step 独立将不安全轴归零。`canonical-prototype-v9` 在 71.271 秒内通过 77
runtime、16 prototype、23 reference-host tests；显式激活并 held W 的真实进程持续 15,002.745 ms
仍存活且 stderr 为空。`prototype-operator-v2` 生成 `[-6,6]²` bounds、完成 live start/zero-process
stop。没有 schema-1 fallback、engine boundary mode、source-index inference、product telemetry、
renderer/GPU/source-format/asset 或 Wulin 变化。
Experiment 0084 将 host 已归一化却只用于 journal 的 transition 提升为最近一次 ingest 的
`was_pressed` / `was_released` facts；每次 ingest（包括空 drain）先让旧 edge 失效，连续 W/A/S/D
仍只读取 `is_held`。journal/status 继续保持 v1，原生两进程记录仍为 2 transactions、11 messages、
10 transitions 和 `ec8660…435` stream hash。`canonical-prototype-v10` 在 75.608 秒内通过；ready
后的真实 Escape/VK 27 进程在 4.316 秒内 exit 0 且 stderr 为空，held-W finite-edge 仍持续
15.008 秒。没有 latched inspect edge、action queue、frame/simulation binding、engine input API、
renderer/GPU/source/asset 或 Wulin 变化。
Experiment 0085 按逢 5 规则清理 Experiment 0041 已完成使命的 diagnostic input journal。live
`HostInput` 从 414 行收敛为 94 行、固定 96 bytes 且无需 drop，只保留 held/pressed/released、
repeat/unmatched/invalid suppression、focus cleanup 与 empty-ingest expiry；record/status/hash/
replay、`PostedMessage`/native-post、5 个 inspect verbs、4 个 workbench commands 和 long report
field 全部直接删除。真实 workbench 的 9 个旧入口均明确拒绝；`canonical-prototype-v10` 在
75.045 秒内保留 pre-ready W 的 `deltaZQ9=-32 / clip=1`、Escape exit 0 与 held-W 15.008 秒
survival。没有 alias、fallback、replacement journal、feature、engine/runtime/GPU/source/asset
或 Wulin 变化。
Experiment 0086 接受首个离散 camera action：prototype 私有四态 quarter-orbit policy 以 Q/E press
edge 生成完整 rig candidate，只在既有 `set_actor_relative_camera` 成功后提交 index；held/empty ingest
不重复，同 sample Q/E 抵消。`canonical-prototype-v11` 在 77.671 秒内通过 77 runtime、18 prototype、
20 host tests；可见窗口 E/VK 69 进程精确得到 orbit 1、rig `[12,4,-9]/[-3,-1,0]`、anchored camera
`[12,6.1640625,-9]/[-3,1.1640625,0]`，并驱动 `[+1,-1]` traversal desire，保持 Survey、零 render
block/failure/prefetch/rollover。没有 engine input/camera controller、inspect、第二 projection、renderer/
GPU/resource/sync/format/asset 或 Wulin 变化。
Experiment 0087 为唯一 actor-simulation command 增加 required batch-entry vertical velocity delta，
只在 nonzero fixed-step batch 的第一步前 checked-add 一次；zero-step 不消费，overflow 与 typed
pending block 都保持 actor/schedule 双回滚。最终 typed-command 版 56.174 秒
`canonical-actor-v5` 拒绝旧 shape 与 alias，精确提交 velocity `0 -> 16384`、center
`141824 -> 158208`，并对 delta 8192 的 blocked candidate
报告 step/query `1/1`、commit `0/0`。81.960 秒 `canonical-prototype-v12` 中 81 runtime、18 prototype、
20 host tests 及全部真实进程显式传零并保持既有行为。没有 jump verb/intent queue/default、独立 actor
mutation、renderer/GPU/resource/sync/source/format/asset 或 Wulin 变化。
Experiment 0088 将既有 planar-first step 已计算的 grounded 结果提升为 committed actor transition
witness：zero-step 为 null，nonzero 只报告最后一步，failure/pending block 不泄漏 candidate 状态。
38.654 秒 `canonical-actor-v6` 精确证明 fractional null、向上 impulse false、grounded animation
true 与 blocked 无 transition；67.343 秒 `canonical-prototype-v13` 中 83 runtime、18 prototype、
20 host tests 及所有真实进程直接消费 committed true，不再从零 velocity 推断
`groundedAfterBatch`。没有 RuntimeActor contact cache、额外 query、Space/jump policy、renderer/GPU/
resource/sync/source/format/asset 或 Wulin 变化。
Experiment 0089 将 Space 接为首个有界 actor action intent：prototype 只在最后一次 exact committed
grounded witness 为 true 时接收一个 pending intent，fractional work、Stalled 与 typed pending-window
block 只保留这个 bit 而不保留 elapsed backlog；Reset/Suspended 先清旧 intent，再接收当前 native
message batch 的新 edge。下一次 nonzero actor commit 通过 Experiment 0087 的既有字段只施加一次
delta `4369`，并以 Experiment 0088 的最终 grounded witness 更新 eligibility。67.470 秒
`canonical-prototype-v14` 通过 83 runtime、22 prototype、20 host tests；可见窗口 Space/VK 32 进程
在一步 gravity `-179` 下精确得到 velocity `0 -> 4190`、center `141824 -> 146014`、grounded false、
pending false，同时保持 XZ、Survey、actor identity、camera/traversal 与零 render block。没有 engine
action/input state、queue、额外 query、jump animation、config/compatibility、renderer/GPU/resource/
sync/source/format/asset 或 Wulin 变化。
Experiment 0090 完成逢五清理：删除重复的 `Runtime::simulation_status`、`simulation.status`
protocol/dispatch 全链，以及每次 actor gate 仍执行的 8 个历史 retired-control 请求和 report 字段。
16 个维护型断言直接消费既有 `canonical.status.simulationSchedule`；actor transaction 自身继续携带
exact `SimulationAdvance`，frame/probe 与私有 schedule 状态不变。扩展后的 79.793 秒
`canonical-actor-v7` 直接覆盖 lifecycle/restart、fractional、coarse/nominal partition、query/arithmetic
rollback、pending block、GPU/epoch；旧 verb 返回 generic `unknown_event`，两种一秒分区均收敛到
tick/remainder `60/0`，blocked 前后 canonical/probe 状态保持 `1/20`。guard 的故意 method 回填在
编译前被拒绝；最终 init/guard 为零 deny。没有 alias、replacement status、product、renderer/GPU/
resource/sync/source/format/asset 或 Wulin 变化。
Experiment 0091 直接消费已经 cook/验收的 Fox Run clip：prototype 仅在 held Shift 与最终准入的
非零 W/A/S/D 同时成立时选择固定 64/45-Q9 Run command，Walk 保持 32/23，Shift-only、对消或边界
全裁减仍为 Survey；private `running` 事实不进入 host/runtime storage。72.651 秒
`canonical-prototype-v15` 通过 83 runtime、25 prototype、20 host tests；可见窗口按序收到
Shift/VK 16 与 W/VK 87，3 step 精确提交 Z `0 -> -192`、Survey/clip 0 -> Run/clip 2、yaw `49152`、
epoch `1 -> 3`、3 queries、grounded true、vertical velocity 0 与零 render block，并保持 camera/
traversal/actor identity。没有 acceleration、horizontal velocity、stamina/toggle/config、root motion、
blend、host/engine action state、renderer/GPU/resource/sync/source/format/asset 或 Wulin 变化。
Experiment 0092 将 W/A/S/D 与四态 camera orbit 闭合：同 sample 的 pure camera candidate 在
playable-boundary admission 前按 orbit 0 `(x,z)`、1 `(z,-x)`、2 `(-x,-z)`、3 `(-z,x)` 精确旋转
Walk/Run，camera index 仍只在既有 runtime mutation 成功后提交。72.161 秒
`canonical-prototype-v16` 通过 83 runtime、26 prototype、20 host tests；可见窗口按序收到 E/VK 69
与 W/VK 87，2 step 将 local X `0 -> -64`、Z 保持 0，并原子提交 Walk/clip 1、yaw `32768`、epoch
`1 -> 3`、2 queries、grounded true、orbit 1、精确 camera anchor、`[+1,-1]` traversal 与零 block。
没有 arbitrary steering、cross-subsystem transaction、host/engine state、renderer/GPU/resource/sync/
source/format/asset 或 Wulin 变化。
Experiment 0093 将 object worker 已验证的 schema-3 spatial/identity/presentation triple 直接移入
既有 source-addressed 50-slot cache 的有界 CPU 页，并让 committed snapshot 与 GPU mapping 共享同一
reservation/copy completion/stage/pair publication/rollback 生命周期。新的只读
`Runtime::query_canonical_object` 按 signed region + authored local ID 精确返回 raw triple；成功路径零
allocation/source I/O/GPU copy/readback/fence/sync。16.131 秒 `canonical-frame-v2` 以独立 `.wlr` 字节
oracle 验证 0/511/1023 与严格失败并保持全部 GPU hashes；807.8 秒 `canonical-runtime-v1` 进一步通过
A/B physical order、adjacent old/new window、object/terrain failure retention、restart、32+32 traversal、
64-publication plateau 和 16 lifecycle。50 页 payload 固定为 2,048,000 bytes；资源从 531 handles /
413,949,952 private bytes 收敛到 516 / 412,336,128，最终 60 秒稳定。没有 spatial selection、fixed-point
conversion、interaction policy、persistent gameplay ID、second scene、format/asset 或 Wulin 变化。
Experiment 0094 将已查询 `CanonicalObject` 的 authored X/Z 严格接入唯一 `TerrainPosition`：只接受
closed `[-4096,4096]` Q9 lattice，`+4096` 独立归一化为相邻 signed region 的 `-4096`，所有非有限、
非 lattice、越界和 signed overflow 均失败且不 round/clamp。17.289 秒 `canonical-frame-v3` 以独立
source-byte oracle 精确证明 0/31/511/992/1023 的 same/X/Z/diagonal seam 与零 query-side work；273.472
秒 `canonical-runtime-v3` 保持 A/B、revisit、adjacent、两类 rollback、restart、32+32 traversal、8
publication resource checkpoint 和 2 lifecycle。owner region + local ID 身份不随 spatial seam 改写；
没有 enumeration、selection、interaction、persistent ID、format/asset、networking 或 Wulin 变化。
Experiment 0095 完成逢五清理：删除重复的 `Runtime::presentation_time_status`、
`canonical.time.status` protocol/dispatch 全链，11 个维护型读取统一消费既有
`canonical.status.presentationClock`；pause/resume/set/step 保持唯一 mutation 与原响应形状，直接从
私有 timeline 返回。80.180 秒 `canonical-actor-v8` 保持 lifecycle/restart、fractional、60-step
coarse/nominal、rollback、pending admission、GPU/epoch；268.804 秒 `canonical-runtime-v4` 中旧 verb
仅一次并返回 generic `unknown_event`，manual/wrap/invalid/automatic/held-publication 时间证据、32+32
traversal、8 publication resource checkpoint 与 2 lifecycle 全部通过。故意回填旧 method 名在一秒内
被 guard 拒绝；最终零 Flavor deny。没有 alias、replacement status、product、renderer/GPU/resource/
source/format/asset、networking 或 Wulin 变化。
Experiment 0096 在既有 committed 25-page CPU object snapshot 上加入唯一有界 nearest query：输入
active-window 内的精确 `TerrainPosition` 与 inclusive `u32` Q9 radius，一次验证并扫描最多 25,600
triples，只返回一个 optional raw object、归一化位置、signed delta 与 squared Q18 distance；等距严格按
owner X/Z/local ID 决定，不受 physical order、spatial seam 或 GPU visibility 影响。13.639 秒
`canonical-frame-v4` 以独立 `.wlr` 全页 oracle 证明 zero-radius seam、inclusive/no-result、center 和
`u32::MAX` radius，全部 query-side work 为零且 GPU hashes 不变；251.987 秒
`canonical-runtime-v5` 的 28 次成功/3 次严格失败进一步保持 A/B、revisit、adjacent、两类 rollback、
restart、32+32 traversal、资源 checkpoint 与 2 lifecycle。资源基线改为同 workload 4..8 次
state-driven warm，正式容差未放宽；最终工作树在第 5 次收敛后保持 492 handles/21 threads，最终
private bytes 比基线低 503,808。没有 enumeration/index、selection/interaction、persistent ID、renderer/GPU
resource、format/asset、networking 或 Wulin 变化。
Experiment 0097 把 nearest 接入第一个非诊断消费方：原型的 F press 只建立一个 capacity-one 观察意图，
仅在下一次成功的 nonzero actor transaction 后，以 committed output position 和固定 inclusive 512-Q9
半径执行一次查询；fractional/stall/render block 保留，Reset/Suspended 取消，成功后不保留 target 或
selection。71.900 秒最终 `canonical-prototype-v17` 中，可见窗口同时收到 F/VK 70 与 W/VK 87，actor
先提交到 local Z `-32`，观察再以该位置为原点扫描 25,600 triples；独立 `.wlr` oracle 精确复算出
local ID 496、delta `(160,0)`、distance squared 25,600。没有自动逐帧 scan、选择/交互状态、engine/host
input policy、新资源、兼容入口、networking 或 Wulin 变化。
Experiment 0098 用 committed `.wlr` source namespace 补齐 object address lifetime：唯一 identity 为
`(source namespace, owner region, authored local ID)`，exact lookup 必须携带完整 identity，旧无 namespace
API/payload 直接移除；nearest 返回同一 identity，但同源 tie 仍只按 distance/region/local ID。13.731 秒
`canonical-frame-v5` 中独立 header+index SHA-256 oracle、strict mismatch/旧 payload、零 query work 与 GPU
hashes 全部通过；75.899 秒 `canonical-prototype-v18` 保持 F+W committed observation。238.700 秒
`canonical-runtime-v6` 证明 A/B namespace 不同且双方 stale identity 都失败，A revisit 精确恢复；两类
rollback、restart、32+32 traversal、资源 checkpoint 和 2 lifecycle 全通过。最终保持 492 handles/21
threads，private bytes 仅比基线高 299,008。它仍是 source/snapshot qualification，不是 gameplay persistent
ID、retained selection 或 interaction。
Experiment 0099 将 qualified address 的普通失效从文本错误中分离：唯一
`Runtime::resolve_canonical_object`/`canonical.objects.resolve` 返回 `resolved`、
`source-replaced` 或 `outside-published-window`；未发布、非法 local ID、快照/页损坏、缺失或重复
authored ID 仍严格失败。旧 Runtime method 与 inspect verb 均直接移除，无 alias。18.164 秒
`canonical-frame-v6` 精确通过三种 outcome、独立 source oracle、旧 verb `unknown_event`、零 resolver-side
work 与不变的 color/object-ID replay hashes。249.862 秒 `canonical-runtime-v7` 中 A/B 双向 stale 均为
`source-replaced`，A revisit 恢复 `resolved`，同源 adjacent departure 为
`outside-published-window`；两类 rollback、restart、32+32 traversal、资源 checkpoint 与 2 lifecycle
全部通过。资源保持 492 handles/21 threads，private bytes 从 424,329,216 到 424,501,248（+172,032），
报告 24 files/25,346,200 bytes。prototype 尚不 retain target，也没有逐帧 resolution、registry、
interaction、networking 或 Wulin 语义。
Experiment 0100 完成逢五/逢二十双重清理：删除仅由 workbench/full acceptance 消费的
`Runtime::resolve_terrain_contact`、`CanonicalTerrainContact`、payload/route/dispatch 与独立 support
全链，无 alias 或替代入口；private `resolve_body_contact`、motion/translation 消费与通用 probe 的
225-body witness 保持唯一权威。旧 dense-probe rejection 不再累计，唯一当前
`canonical.terrain.contact` witness 返回 generic `unknown_event`。247.249 秒
`canonical-runtime-v8` 保持 75/75/75 classification、75 corrections 与既有 contact hashes，并通过
A/B、rollback、restart、32+32 traversal、5 warm/8 measured resource checkpoint 和 2 lifecycle；资源
保持 492 handles/21 threads，private +376,832 bytes。最终在全部验证与 commit hook 后移除
`target/` 35,590 files/10,908,689,993 bytes 和 `out/` 5,394 files/9,356,065,266 bytes，合计回收
40,984 files/20,264,755,259 bytes；不影响 committed source/evidence、asset、networking 或 Wulin 行为。
Experiment 0101 新增 live-Runtime-scoped `CanonicalObjectSnapshot(publication token + source
namespace)`，prototype 的 F observation 现在原子保留一个只含 qualified identity、last-validated
snapshot 和 availability 的 target。无 target 不读 stamp；stamp 不变不调用 resolver；同源 window
departure 保留 unavailable target，后续同源 publication 可恢复，source replacement 清空，explicit
empty scan 也清空。27.122 秒 `canonical-frame-v7`、71.515 秒 `canonical-prototype-v19` 与 253.412 秒
`canonical-runtime-v9` 全通过；native F+W 在 token 1 获取 ID 496，并在同帧 traversal publication 后
精确重验证到 token 2，target 不复制 object/position/presentation。全量 A/B token/source、adjacent
10→11、两类失败保持 token 21、GPU replay、4 warm/8 measured resource checkpoint 与 2 lifecycle
通过；资源保持 492 handles/21 threads，private +1,187,840 bytes，报告 24 files/25,346,346 bytes。
没有逐帧 nearest、unchanged-frame resolver、engine target、persistent gameplay ID、highlight、
interaction、asset/format、networking 或 Wulin 语义。
Experiment 0102 将 retained qualified identity 作为可选 immutable `FrameRequest` 输入，并只在当帧
pending publication 提交后投影 source/window；现有 56-byte visible record 的高 identity word 承载
streamed authored local ID，sole surface resolve 对 exact semantic/local-ID match 施加静态 amber mix。
没有 marker/outline、新 pass/resource/descriptor/copy/readback/synchronization 或 renderer-owned target。
24.280 秒 `canonical-frame-v8` 对可见 ID 987 精确得到 3,472 target pixels、确定性 replay、清除后
回到 baseline 且 object-ID attachment 不变；73.149 秒 `canonical-prototype-v20` 在 native F+W 后用
一个 product frame 转发完整 identity，无 copied object state。282.045 秒 `canonical-runtime-v10`
证明 source replacement/window departure 当帧禁用反馈，source revisit/return 恢复相同 3,472 pixels，
并通过 rollback、restart、32+32 traversal、5 warm/8 measured resource checkpoint 与 2 lifecycle；
active baseline/final 保持 527 handles/24 threads，private +589,824 bytes 且处于 accepted plateau，
报告 24 files/25,346,225 bytes。当前只证明
presentation feedback，不引入 interaction eligibility/effect、persistent gameplay ID、networking 或
Wulin 语义。
Experiment 0103 将 canonical object 的 exact signed-region/Q9 proximity 提升为唯一纯计算权威，既被
nearest scan 复用，也在 actor 非零提交后对 retained identity 进行固定 512-Q9 动作资格检查；frame
feedback 直接替换为 Selected/Activated 两类 immutable 输入，并只在同一 identity 被成功帧实际投影
后返回。Prototype 以 Enter 持有 capacity-one intent，成功 candidate 作为 12 帧绿色确认的第一帧，
余下帧只随成功 projected frame 递减；没有 object mutation、engine action state、新 pass/resource/
descriptor/copy/readback/synchronization。32.302 秒 `canonical-frame-v9` 对 ID 987 的两类反馈都精确
覆盖 3,472 pixels，颜色 hash 不同、object-ID attachment 不变；72.589 秒
`canonical-prototype-v21` 中 native F+Enter+W 在同一 committed step 获取 ID 496，以 `(160,0)` Q9 /
25,600 Q18 通过资格，frame 返回 exact Activated identity，committed count 为 1、remaining frames
为 11，且没有 copied canonical object state。245.223 秒 `canonical-runtime-v11` 保持 source
replacement/departure 禁用与 revisit/return 精确恢复 3,472 pixels，并通过 rollback、restart、32+32
traversal、4 warm/8 measured resource checkpoint 与 2 lifecycle；active baseline/final 均为 492
handles/21 threads，private +487,424 bytes，报告 24 files/25,346,259 bytes。
Experiment 0104 将一次成功 Activated action 提升为 prototype 进程期 capacity-one consumption：
完整 qualified identity 立即成为 nearest exclusion，既有 12-frame 绿色确认结束后才作为 immutable
frame suppression；唯一 skeletal cull 在完成 grounding 写入后、frustum/visibility 前淘汰 exact
active-index/local-ID，因此后续 animation/shadow/occlusion/surface 共用同一减一结果。40.920 秒
`canonical-frame-v10` 对 ID 987 得到 visible 10,538→10,537、rejected 15,062→15,063、shadow 与
occlusion source 同步减一，CPU/GPU、replay、clear 全精确；75.750 秒 `canonical-prototype-v22`
中 native F+Enter+W 把 ID 496 同帧提交为 consumed/exclusion，并将 suppression 正确延迟到确认结束。
265.079 秒 `canonical-runtime-v12` 证明 A source 抑制、B replacement 失效、A revisit 恢复、同源离窗
失效与返回恢复；5 warm/8 measured 保持 492 handles/21 threads，private 427,048,960→426,463,232，
报告 24 files/25,346,262 bytes。没有 canonical source mutation、registry、inventory、dispatcher、
respawn、persistence、networking 或 Wulin 语义。
Experiment 0105 完成逢五兼容清理：删除每次 full acceptance 都经 generic unknown-event 重放的
10 个历史 `scene/world/contact/terrain-body` inspect 请求、`removedVerbs` 报告链和混合职责的
`compatibility-removal.ts`；这些缺席事实继续由 calibration/contact/terrain-transaction 静态守卫
负责。当前 clear-only 行为独立为 `idleShell`，仍保持 color `cd26eaab…76db8`、semantic
`0c660f2b…a5f`、921,600 background pixels、0 different/visible/unknown。263.724 秒
`canonical-runtime-v13` 全通过，十个历史 event key 与旧字段均不存在；本轮因状态驱动多一次当前
`canonical.status` poll，总 Sidecar invocation 从 988 净降到 979。5 warm/8 measured 保持 492
handles/21 threads，private +90,112 bytes，inventory 24 files/25,346,280 bytes。没有替代 rejection
registry、alias、runtime/product/GPU/source/asset/networking 或 Wulin 行为。
Experiment 0106 收紧 Prototype object action 的最后一个纯应用层前置条件：复用同一 committed actor
output 的八向 yaw 与既有 exact Q9 proximity，非重合目标只在整数 dot 为正时准入，重合仍可交互，
非法 yaw 在 policy mutation 前失败。75.629 秒 `canonical-prototype-v23` 中 native F+Enter+D 对 ID 496
得到 delta `(128,-32)`、yaw/direction/dot `0/(1,0)/128` 并精确提交 Activated/consumption；独立
native F+Enter+W 对同一 ID 得到 delta `(160,0)`、yaw `49152`，返回 `outside-facing`，completion、
Activated frame、consumption 与 exclusion 均为零。没有 engine/renderer/GPU/resource/source 状态、
registry、inventory、reward、dispatch、respawn、persistence、networking 或 Wulin 变化。
最终 `canonical-runtime-v14` 在 254.675 秒内通过 source/window、rollback、restart、32+32 traversal、
5 warm/8 measured resource checkpoint 与 2 lifecycle；资源保持 492 handles/21 threads，private
423,571,456→424,103,936，报告 24 files/25,346,301 bytes。
Experiment 0107 把已解析、半径内但侧/后向的 `OutsideFacing` 提升为第三种 immutable frame
feedback：沿用同一 identity/frame transaction 与 12 个成功投影帧的 acknowledgement slot，在既有
surface resolve 产生固定红色 `Rejected`，completion 始终 `applied=false`，且不产生 consumption、
exclusion 或 suppression。45.636 秒 `canonical-frame-v11` 中 Selected/Activated/Rejected 对 ID 987
均精确覆盖 3,472 pixels，三种 color hash 不同，object-ID attachment、replay 与 clear 不变；77.910 秒
`canonical-prototype-v24` 中 native 侧向动作对 ID 496 得到 `(160,0)` Q9、yaw/direction/dot
`49152/(0,-1)/0`，精确提交并投影 Rejected，remaining frames 11、ineligible/rejected 各一，applied、
consumption 与 exclusion 均为空。没有第二计时器、队列、registry、action state、新 pass/resource/
descriptor/copy/readback/synchronization、mutation、inventory、reward、networking 或 Wulin 行为。
最终 `canonical-runtime-v15` 在 257.299 秒内通过 source/window、rollback、restart、32+32 traversal、
5 warm/8 measured resource checkpoint 与 2 lifecycle；980 次 Sidecar invocation，资源保持 492
handles/21 threads，private 419,368,960→419,319,808，报告 24 files/25,346,275 bytes。
Experiment 0108 将一次性 readiness 扩展为仍然有界的成功会话契约：启动只输出 sequence-one
canonical readiness，只有优雅 Escape/window close 在 Runtime idle 后输出一次 sequence-two immutable
completion；失败、强制证据终止与 15 秒边界进程不输出 completion，也不存在事件流、inspect、journal、
replay 或产品文件写入。81.332 秒 `canonical-prototype-v25` 的同一真实进程先以 `F+Enter+D` 消费
qualified ID 496，在 live frame 5 读取 readiness，随后接收新的 Enter release/press，并在 Escape
completion 的 live frame 970 精确保持 committed/consumed 为一、只将 ineligible 推进到一、清空
acknowledgement/target 且累计 954 个 suppression frames；stdout 恰为两值、无 trailing output/event
history/copied object state。最终 `canonical-runtime-v16` 在 252.427 秒内通过原有 source/window、
rollback、restart、32+32 traversal、5 warm/8 measured resource 与 2 lifecycle；979 次 Sidecar
invocation，资源保持 495 handles/21 threads，private +1,257,472，报告 24 files/25,346,292 bytes。
Experiment 0109 让 capacity-one 消费耗尽成为有界可见结果：只有另一个当前 resolved target 才从其
qualified identity 直接生成既有红色 `Rejected`，不做 canonical object resolve/proximity/facing，
`completion.applied=false`，并复用同一 12-frame acknowledgement；该第二 identity 的反馈期间，第一个
consumed identity 的 suppression 不得中断。80.596 秒 `canonical-prototype-v26` 中，同一进程在
readiness live frame 5 消费 ID 496，继续移动后接收精确 `D up`、`F up/down`、`Enter up/down`，独立
exclusion-aware source oracle 从最终 stationary actor position 选中 ID 501；completion live frame
792 精确保持 consumed/exclusion ID 496、resolved target ID 501、committed/ineligible 1/1、12 个
Rejected frames、776 个 suppression frames，且 acknowledgement 为空、无 copied object state/event
history。engine/renderer/shader/frame ABI/resource/sync 结构未变，因此本实验由聚焦 Prototype 工作流
收口，未重复全量运行时验收。
Experiment 0110 按兼容清理节奏删除已被精确 projected feedback 与 durable policy state 取代的瞬态
object-action report：sequence-one readiness 不再包含 `attempt`/`completion`，删除
`FrameCompletion`、Attempt/facing 序列化、composition plumbing、测试回声断言和所有 acceptance
consumer；`Policy::complete_frame` 只提交状态并返回 `Result<()>`，无 alias/fallback/schema branch。
81.285 秒 `canonical-prototype-v27` 中，front/side ID 496 仍以独立 observation query 与 committed
actor 精确推导 delta/yaw/direction/dot `(128,-32)/0/(1,0)/128` 和
`(160,0)/49152/(0,-1)/0`，分别保持 Activated consumption 与零消费 Rejected；持续会话从 live
frame 4 到 798，保持 consumed ID 496、exclusion-oracle target ID 501、12 个 capacity-rejected
frames 和 783 个 suppression frames。现有 session guard 稳定禁止所有被删 surface，未新增 guard
模块；engine/GPU/resource 结构未变，因此未重复全量运行时验收。
Experiment 0111 补齐既有优雅退出契约中唯一缺少实机证明的路径：维护中的 native harness 在
readiness 后只向可见且 class/title/PID 精确匹配的窗口投递一个 `WM_CLOSE`，不激活窗口、不注入按键、
不直接 DestroyWindow 或终止进程。86.089 秒 `canonical-prototype-v28` 中，PID 21236 在 live frame 5
输出 sequence-one readiness，在 live frame/sample 356 输出 reason `window-close` 的 sequence-two
completion，exit 0、stderr/尾随输出为空、进程与 actor identity 不变、object policy idle；原 Escape、
forced-termination silence 和持续 capacity-one 会话仍精确，后者保留 12 个 Rejected 与 1,069 个
suppression frames。Prototype Rust、engine/GPU/resource 与 session schema/输出节奏均未改变。
Experiment 0112 将原生焦点中断接入同一有界实机会话证明：PID 18472 的精确可见窗口先收到
`WM_SETFOCUS`、`WM_KEYDOWN:W`、`WM_KILLFOCUS`，在暂停采样后再收到 `WM_SETFOCUS`，恢复后通过既有
Escape 退出。92.183 秒 `canonical-prototype-v29` 中，readiness live frame/sample 5 到 completion
1643 恰好新增 suspend/resume/reset 各一，记录 635 个 suspended samples、之后 1,002 个 Ready
samples，stall/render block 均为零且无 elapsed backlog；完整 actor state 与 readiness 逐字段相同，
object observation/interaction 保持 idle，stdout 仍恰为两值。Escape、WM_CLOSE、forced-silence 与
持续 capacity-one gate 均保持精确，后者仍有 12 个 Rejected 和 1,072 个 suppression frames；没有
修改产品输入/时钟策略、session schema、Runtime 或 engine/GPU/resource 结构。
Experiment 0113 补齐 Jump 生命周期的实机闭环：PID 2292 先收到一次 Space down，在零 stall 下等待
1,265.727 ms（超过既有 48-step 完整飞行），再由同一 native helper 投递 Space up/down，并在精确
104.278 ms 后投递 Escape。100.135 秒 `canonical-prototype-v30` 中，readiness live frame/sample 4
保持 grounded true、velocity 0、ground center 141,824；completion frame/sample 1,616 的 velocity
3,116 唯一反解为第二跳第 7 step，高度增量精确为 25,571、center 为 167,395。actor identity、XZ、
shape、Survey/yaw/epoch、clock reset/suspend/resume/stall、object idle state 与零 render block 均保持
精确，stdout 仍恰为两值、exit 0、stderr 空。focus gate 保留 645 个 suspended samples，持续
capacity gate 保留 12 个 Rejected 与 1,051 个 suppression frames；产品 Jump/input/time/session
schema、Runtime 与 engine/GPU/resource 结构未改变。
Experiment 0114 证明互补的 midair rejection：PID 3548 的同一可见窗口先收到 Space down/up，
208.749 ms 后收到第二次 Space down 与 W，再经 207.008 ms 收到 Escape。102.146 秒
`canonical-prototype-v31` 中，completion frame/sample 606 的 velocity -106 唯一反解为原始单 impulse
第 25 step，rise 51,050、center 192,874；若第二次 impulse 生效则不可能满足该整数轨迹。同批 W
同时产生 12 个 Walk steps、Z -384 Q9、clip 1/yaw 49,152，排除了第二次 input 未进入产品循环的假
阳性。actor identity/region/X/shape、clock discontinuity/stall、object idle、零 render block 与两值
clean exit 均精确；readmission/focus/sustained gate 分别保持 step 7、628 suspended samples、12 个
Rejected 与 1,054 个 suppression frames。acceptance-only native action schema 3 完整替换 schema 2，
无 alias；产品 Jump/input/session schema、Runtime 与 engine/GPU/resource 结构未改变。
Experiment 0115 按强制兼容清理节奏删除最后三个周期性历史请求：`simulation.status`、
`canonical.time.status` 与 `canonical.objects.query` 不再进入真实进程，两处 `retiredStatusGate` 及
`retiredStatus`/`retiredVerb` 报告字段一并删除，不保留 alias、fallback 或版本分支。既有 simulation、
presentation、object owner guard 现在是唯一缺失权威；当前 time set/step rollback、缺失必填 velocity
delta、velocity alias、未限定 object identity 等 strict payload 测试仍然保留。75.926 秒
`canonical-actor-v9`、43.215 秒 `canonical-frame-v12` 和 253.391 秒 `canonical-runtime-v17` 全部
通过；全量报告含 979 次 Sidecar invocation、5 warm/8 measured publication、492 handles/21 threads、
private bytes `423,276,544 -> 423,620,608`、2/2 lifecycle 与 24 files/25,346,327 bytes，三个最终报告
均不含旧 verb、旧字段或 `unknown_event`。产品、Runtime、engine/GPU/resource 结构均未改变。
Experiment 0116 补齐 reference-host 重复 down 抑制的实机链路：PID 20468 在 readiness 前向同一精确
可见窗口投递 E-down 且不释放，readiness 提交 orbit 1；之后重复 E-down 与 W-down 间隔 2.2142 ms，
205.2474 ms 后由 Escape 完成会话。109.679 秒 `canonical-prototype-v32` 中，最终位移严格为 11 个
orbit-one Walk steps：X -352 Q9、Z 0、clip 1/yaw 32,768、epoch `1 -> 24`，排除了重复 down 生成第二个
camera press edge。actor identity/region/shape 与 vertical velocity 0 保持精确，clock
reset/suspend/resume/stall 维持 `1/0/0/0`，Ready/sample 分别由 `2/3` 前进到 `28/29`，object idle、零
render block 与两值 clean exit 均成立。产品 `HostInput`、Win32 adapter、camera/locomotion policy、
session schema、Runtime 与 engine/GPU/resource 结构均未改变。
Experiment 0117 证明 Win32 key 全值在进入 `HostInput` 前不会被低字节截断：PID 16624 的同一精确可见
窗口在默认 orbit 0 readiness 后收到 `WPARAM=0x145`（325，低字节恰为 E/69），2.2231 ms 后收到 W，
再经 222.059 ms 收到 Escape。114.141 秒 `canonical-prototype-v33` 中，最终严格保持 orbit-zero
方向，产生 13 个 Walk steps、X 0/Z -416 Q9、clip 1/yaw 49,152；若 325 被截断为 E，则相机必转到
orbit 1 并产生负 X，因此该方向证据排除了 coercion。clock reset/suspend/resume/stall 保持
`1/0/0/0`，Ready/sample `1/2 -> 27/28`，actor identity/shape/vertical velocity 0、object idle、零
render block 与两值 clean exit 均精确。验收 session 的重复 startup/jump 比较被收敛到一个本地 helper
以满足 500 行质量门；产品 HostInput/Win32 adapter/camera/locomotion、session schema、Runtime 与
engine/GPU/resource 结构未改变。
Experiment 0118 消除了对象反馈验收夹具对首批模拟步数的隐式依赖。诊断中，即使把 F/Enter/方向键在
同一窗口线程暂停期间原子排队，旧夹具仍能返回合法但不同的反馈；根因是首个事务允许 `1..=8` steps，
移动会跨过 256-Q9 对象网格并改变 nearest 与 facing dot，而 focus reset 也不能精确强制单步。最终
`canonical-prototype-v34` 在 120.784 秒通过：PID 10244 于 base 静止三步，原子 F/Enter 跨度
0.0012 ms，选择 delta `(160,-32)` Q9 的 authored ID 496 并精确投影 Activated；PID 13120 于
`base+4` 静止一步，同样以 0.0012 ms 原子跨度选择 delta `(-224,-32)` Q9 的 authored ID 495 并精确
投影 Rejected。持续会话保持首次 consumption，D motion 后先释放 D 再提交第二次动作，保留 12 个
capacity Rejected 与 87 个 suppression frames。没有重试、阈值放宽、时间重置或动态接受反馈种类；
产品、reference-host、Runtime、renderer/GPU/resource/synchronization 均未改变。
Experiment 0119 补齐 Q/E 相反 camera press edges 的原生同批实机证明：PID 7632 的精确可见窗口线程
1728 在暂停期间原子排队 Q-down、E-down 与 W-down，两个投递间隔为 0.0012/0.0010 ms，完整 batch
跨度 0.0022 ms，恢复线程后 238.676 ms 由 Escape 完成。128.648 秒 `canonical-prototype-v35`
首次通过；readiness 与完成均保持 orbit 0，actor 精确产生 14 个负 Z Walk steps，X 0/Z -448 Q9、
clip 1/yaw 49,152。任一相反边缘丢失都会选择非零 camera candidate 并旋转移动轴，因此该方向证据
证明两边缘进入同一 ingest 后在纯 candidate 中抵消。clock reset/suspend/resume/stall 保持
`1/0/0/0`，Ready/sample `2/3 -> 39/40`，object idle、零 render block、两值 clean exit 均成立。
已达 500 行的会话文件按职责拆为共享进程 framing 与 session gate matrix；产品 HostInput、
camera/locomotion、session schema、Runtime 与 engine/GPU/resource 结构未改变。
Experiment 0120 按兼容与资源双重清理节奏删除三次只由历史 `fallback=true` 驱动的 invalid-bootstrap
进程启动、三个 `invalidDocument` 报告字段，以及 fallback/schema-1 单测断言；当前 schema-2 解码、路径
逃逸/非法投影/边界测试、missing source、corrupt payload、readiness/restart 全部保留。
`canonical-prototype-v36` 在 128.894 秒通过，`canonical-runtime-v18` 在 286.415 秒通过；后者含
1,108 次 Sidecar invocation、4 warm/8 measured publication、稳定 527 handles/24 threads、
private bytes `412,213,248 -> 411,742,208`、2/2 lifecycle 与 24 files/25,346,219 bytes，两个报告
均不再含 `invalidDocument`。guard 的兼容删除 owner 禁止旧探针回归。最终在验证与 commit hook 后删除
仓库内 `target/` 11,976 files/4,469,116,945 bytes 与 `out/` 263 files/441,628,335 bytes，共回收
12,239 files/4,910,745,280 bytes；未触碰全局缓存、`.task/`、source asset 或产品/Runtime/GPU 行为。
Experiment 0121 补齐 Q-only counter-clockwise camera wrap 的原生实机证明。0120 清理后首次 guard
明确发现仓库本地 Agility SDK 缺失，维护中的 `runseal :gpu-lab correctness` 恢复固定 1.619.4 并以
checksum `7ae6c64a0b95628a`、零 mismatch、零 D3D12 error 通过。随后
`canonical-prototype-v37` 首次运行在 139.716 秒通过：PID 6788 的精确可见窗口线程 18860 原子排队
Q-down/W-down，投递间隔与完整 batch span 均为 0.0011 ms，232.1975 ms 后 Escape 完成。readiness
保持 orbit 0，最终 actor 精确产生 14 个正 X Walk steps，X +448/Z 0 Q9、clip 1/yaw 0、epoch
`1 -> 35`；在唯一 locomotion key 为 W 的条件下，该方向只能来自 Q 将纯 camera candidate
`0 -> 3` 的 wrap。clock reset/suspend/resume/stall 保持 `1/0/0/0`，Ready/sample
`2/3 -> 40/41`，object idle、零 render block、两值 clean exit 与全部旧 gate 均成立；产品
HostInput、camera/locomotion、session schema、Runtime、traversal 与 engine/GPU/resource 结构未改变。
Experiment 0122 补齐 held E 在 release 后重新产生 camera press edge 的原生实机证明。
`canonical-prototype-v38` 首次运行在 138.736 秒通过：PID 1752 先以 startup E-down 发布 orbit-1
readiness，随后同一精确可见窗口线程 18524 原子排队 E-up/E-down/W-down；投递间隔
0.0016/0.0010 ms、完整 batch span 0.0026 ms，211.8739 ms 后 Escape 完成。最终 actor 精确产生
13 个正 Z Walk steps，X 0/Z +416 Q9、clip 1/yaw 16,384、epoch `1 -> 35`。若 E-up 或第二次
E-down 未形成 fresh edge，相机会留在 orbit 1 且 W 必产生负 X，因此该方向证据证明现有纯 candidate
已 `1 -> 2`。clock reset/suspend/resume/stall 保持 `1/0/0/0`，Ready/sample
`2/3 -> 40/41`，object idle、零 render block、两值 clean exit 与全部旧 gate 均成立；产品
HostInput、camera/locomotion、session schema、Runtime、traversal 与 engine/GPU/resource 结构未改变。
Experiment 0123 补齐 held Shift 释放后 W 仍保持 Walk 的原生实机证明。验收先暴露并修复了两个
helper 问题而未改产品：focus-discontinuity 的 W-down/focus-loss 改为在精确窗口线程暂停期间原子
排队，Run-release 则改为单一并发原生序列，避免第二个 PowerShell 启动开销先把 actor 推到有限边界。
`canonical-prototype-v39` 在 142.711 秒通过：PID 22072 依次收到 Shift-down、W-down、Shift-up、
Escape，前三段间隔为 4.6121/505.9149 ms，Escape 再晚 208.2274 ms。readiness 在 local Z
`-192` 提交 3 个 Run steps、clip 2/yaw 49,152/epoch 3；completion 保持同一 actor/region，在
local Z `-2304` 提交 Walk clip 1/yaw 49,152/epoch 19，总 Z delta `-2112 Q9`、X delta 0。
clock Ready/sample `2/3 -> 23/24`，reset/suspend/resume/stall 保持 `1/0/0/0`，object idle、零
render block、两值 clean exit 与全部旧 gate 均成立；HostInput、gait/presentation、session schema、
Runtime、traversal 与 engine/GPU/resource 结构未改变。
Experiment 0124 补齐 held W 期间重新按下 Shift 的 Walk→Run 原生再接纳证明。
`canonical-prototype-v40` 首次运行在 147.077 秒通过：PID 11268 在 W-down 后 508.8601 ms
收到 Shift-down，207.5121 ms 后收到 Escape。readiness 在 local Z `-64` 提交 Walk clip 1/yaw
49,152/epoch 3；completion 保持同一 actor/region，在 local Z `-1792` 提交 Run clip 2/yaw
49,152/epoch 19，总 Z delta `-1728 Q9`、X delta 0。clock Ready/sample `2/3 -> 24/25`，
reset/suspend/resume/stall 保持 `1/0/0/0`，object idle、零 render block、两值 clean exit 与全部
旧 gate 均成立；HostInput、gait/presentation、session schema、Runtime、traversal 与
engine/GPU/resource 结构未改变。
Experiment 0125 按兼容清理节奏删除 canonical actor admission 中两次纯历史
`simulation.actor.advance` 请求：缺少现行 `initial_step_velocity_delta_q16` 的 predecessor shape
与从未支持的 `initial_velocity_delta_q16` alias，并删除对应两个报告字段。现行 required-field
命令、非零 velocity ordering、invalid-presentation rollback 与 pending-window rollback 继续承担
process authority；既有 simulation-control removal guard 同时禁止旧探针回归且要求这些现行证据
保留。只提升直接消费该 gate 的 canonical actor report revision；产品 payload decoder、Runtime、
Prototype 与 engine/GPU/resource/synchronization 均未改变。`canonical-actor-v10` 在 85.111 秒
通过：admitted delta 16,384 使 velocity `0 -> 16,384`、center numerator
`141,824 -> 158,208`；pending candidate 保持 1 step/1 query、零 schedule/actor commit 且无
advance payload。最终 report 无旧字段，5 个 capture files 共 5,355,420 bytes；guard 为 0 Flavor
deny，init 通过。
Experiment 0126 补齐相反 locomotion 轴的原生抵消与释放再接纳证明。新增 session 在精确可见窗口
线程上原子排队 Shift/W/S；readiness 必须保持原点 Survey，随后只释放 S，completion 必须由仍 held
的 Shift/W 产生负 Z Run。验收先发现 PowerShell 请求 200ms sleep 而 Stopwatch 仅测得
197.0125ms，helper 改为单调 deadline 补足下界而未放宽阈值；后续不同旧 gate 又分别暴露 warm
Prototype 可能早于新 PowerShell helper 编译完成而发布 readiness。最终 startup helper 在 child
spawn 前预启动，等待下一个唯一 class/title 窗口并强制返回 PID 等于 child，没有 retry 或产品延迟。
`canonical-prototype-v41` 在修复后的首轮以 147.265 秒通过：PID 6140 的 window thread 13656
以 0.0022ms span 原子排队 Shift/W/S，readiness 为 `(0,0)`、Survey/yaw0/epoch1；S-up 后
209.9403ms 投递 Escape，completion 为 `(0,-832)`、13 个 Run steps、clip2/yaw49152/epoch27。
clock `1/2 -> 32/33`、零 block、对象 idle、两值 clean exit 与全部旧 gate 均成立；产品 HostInput、
locomotion/presentation、session schema、Runtime 与 engine/GPU/resource/synchronization 未改变。
Experiment 0127 将此前仅有单元证据的 A 键与对角 Walk 归一化提升为真实窗口证明。首轮 v42 在新
gate 前复现旧 `run-forward` startup race：仅先启动 PowerShell 进程并不能保证其 `Add-Type`
已在 warm Prototype 发布 readiness 前完成。传输现改为 helper 在类型准备后、找窗前输出唯一
ready marker，session owner 必须等到 marker 才 spawn child，最终实际窗口 PID 仍必须等于 child；
start-only 重复 helper 已删除，没有 retry、产品 sleep 或阈值放宽。显式握手后的首轮与职责拆分后
的最终工作树均通过全部旧 gate；最终 `canonical-prototype-v42` 为 157.404 秒。新 session 的 PID
1852 / thread 2472 以 0.0013ms interval/span 原子排队 W+A，readiness 为 `(-23,-23)`、Walk
clip1/yaw40960/epoch5；213.7342ms 后 Escape，completion 为 `(-299,-299)`，新增 12 个精确
23-Q9 对角步且 epoch 仍为 5。clock `4/5 -> 70/71`、零 block、对象 idle、两值 clean exit；
Flavor 0 deny / 5 个既有 warning，init 通过，report 1 file / 502,641 bytes。产品 HostInput、
locomotion/presentation、session schema、Runtime 与 engine/GPU/resource/synchronization
均未改变。
Experiment 0128 补齐 Shift/W/A 对角 Run 的真实窗口证明，并消除 startup 输入与首个 live
readiness frame 之间最后一个 response-before-post 缝隙。首轮 v43 在旧 E/W gate 得到静止
orbit-zero Survey；随后同步窗口响应探针仍在旧 camera re-press gate 丢失 E press edge，故该探针
被直接删除。现行 schema-4 传输允许单键 atomic batch，并在精确窗口线程恢复前排队每个 startup
请求的完整零延时 prefix；Run release/re-press 只原子化初始 Shift/W 或 W，后续 500ms 转换保持
原义，无 schema-3 decoder、retry、产品 sleep 或阈值放宽。定向真实进程先证明单 E、E/W、Space
以及两个 delayed Run 转换全部成立。最终 `canonical-prototype-v43` 在 161.489 秒通过全部旧
gate；新 session 的 PID 8092 / thread 7344 以 0.0016/0.0010ms interval、0.0026ms span 原子
排队 Shift/W/A。readiness 为 `(-45,-45)`、Run clip2/yaw40960/epoch5；207.613ms 后 Escape，
completion 为 `(-585,-585)`，新增 12 个精确 45-Q9 对角步且 epoch 仍为 5。clock
`4/5 -> 61/62`、零 block、对象 idle、两值 clean exit；Flavor 0 deny / 5 个既有 warning，
init 通过，report 1 file / 524,119 bytes。Rust 产品、Runtime、renderer/GPU/resource/
synchronization 均未改变。
Experiment 0129 修正了 schema-4 startup atomic prefix 的证据边界：窗口线程可能在当前 frame
已经完成消息泵后才被 helper 挂起，因此 queue-before-resume 不能证明 F/Enter 先于该 frame。
两个全量运行分别在既有 sustained/Rejected gate 观察到 `consumed=null` 与 `completed=false`；
没有增加 retry、等待或放宽阈值，而是直接删除 `"object-action"` startup request，把 Activated、
Rejected 和 capacity 首次消费都移到产品 readiness 之后并绑定确切 PID，最终 completion 与独立
source oracle 成为唯一动作权威。`canonical-prototype-v44` 在 163.431 秒通过全部旧 gate：
Activated PID 8908 以 0.0012ms span 消费 ID496，得到 12 Activated/64 suppression；Rejected
PID 3764 以 0.0270ms span 保留 ID495，得到 12 Rejected；sustained PID 1596 在 readiness 后
消费 ID496、移动到 local X 2400、保留排除后 ID505，得到 12 Activated/12 Rejected/2133
suppression。Flavor 0 deny / 5 个既有 warning，report 519,872 bytes；Rust 产品、Runtime、
renderer/GPU/source/resource/synchronization 均未改变。
Experiment 0130 执行逢 5 强制兼容清理，彻底删除 acceptance-only startup action 模式：
5 个 readiness 即强杀的 W/Shift-W/E-W/E/Space 进程检查、`StartupInput`/request switch、
PID-zero next-window 选择、`startupNativeInput` 报告字段以及失去调用者的命令期望全部移除。
首轮 v45 因旧 camera re-press startup E 未在 readiness 前生效而得到 orbit 0，次轮又以
199.2574ms 拒绝了声明 200ms 的 midair delayed key；没有重跑取巧或放宽阈值，而是让所有
动作先取得 idle readiness 和确切 PID，并让 delayed key/exit 共用 Stopwatch 下限 deadline。
最终 `canonical-prototype-v45` 在 144.642 秒通过：camera repeat/re-press 分别保持
267.5133/260.4763ms 后完成 orbit 1 的 `(-384,0)` 与 orbit 2 的 `(0,416)` Q9；Run
release/re-press 以 512.4148/514.1356ms 转为精确 Walk/Run；opposed input 保持
259.2241ms 后完成 12 个 Run 步；post-readiness diagonal Walk/Run 分别以
0.0012/0.0023ms span 完成 `(-299,-299)`/`(-585,-585)`；finite-boundary 在 readiness
后持 W 并存活 15,016.9422ms。最终耗时 144.642 秒，较 v44 减少 18.789 秒，report
缩小 77,834 bytes 至 442,038；Rust 产品、Runtime、renderer/GPU/source/
resource/synchronization 均未改变。
Experiment 0131 在不增加第二个长进程或产品输出的前提下，把唯一 finite-boundary workload
从 readiness 后单键 W 直接升级为确切 PID 的原子 Shift/W。`canonical-prototype-v46` 在
144.312 秒通过；边界 PID 17072 的可见窗口线程 10284 以 0.0013ms interval/span 顺序排队
Shift/W，并在同一 one-region 配置中存活 15,005.520ms。进程在证据清理前没有 stderr、
trailing output、application failure 或 completion；report 为 443,819 bytes。5 个产品边界
测试继续独立拥有 64-Q9 cardinal / 45-Q9 diagonal Run 的最大批次和逐轴削减权威，真实进程
证据只声明输入所有权与有界存活，不从无中间输出推断 actor 最终位置或持续 presentation。
全部 103 engine-runtime、45 Prototype、20 reference-host 测试通过，Flavor 0 deny / 5 个既有
warning；Rust 产品、Runtime、renderer/GPU/source/resource/synchronization 均未改变。
Experiment 0132 用正常 `WM_KEYUP:W` 完成证据直接替换了重复的 idle Escape 进程，没有增加
child 数。首轮 v47 拒绝了错误的 final Survey yaw-0 oracle；既有产品契约要求非零 W advance
提交的 yaw 49152 在静止时保留，因此只修正 oracle，未改产品或时序。最终
`canonical-prototype-v47` 在 144.561 秒通过：PID 20436 / thread 10864 精确收到
W-down/W-up/Escape，Walk hold 255.9837ms、release 后 stationary 252.1398ms；actor 完成
15 个 32-Q9 步至 `(0,-480)`，最终为 Survey clip0/yaw49152、零竖直速度，epoch `1 -> 609`。
clock ready/sample `4/5 -> 758/759`，无 suspend/resume/stall、零 render block、对象 idle、
stdout 恰为两值。report 446,287 bytes；全部 103 engine-runtime、45 Prototype、
20 reference-host 测试通过，Flavor 0 deny / 5 个既有 warning；Rust 产品、Runtime、
renderer/GPU/source/resource/synchronization 均未改变。
Experiment 0133 把现有 focus-discontinuity 进程的 W-only 批次直接升级为同一可见窗口线程上的
原子 Space/W/失焦批次，没有增加 child 或产品输出。首轮 focused guard 拒绝了五词 helper 名，
随后直接收敛为 `suspendWithActionBatch`，未增加 Flavor 例外。`canonical-prototype-v48` 在
144.949 秒通过：PID 2252 / thread 20452 依次收到 Space-down、W-down、`WM_KILLFOCUS`，
两键 interval 与完整 batch span 均为 0.0012ms。恢复后 clock 精确增加一次 suspend/resume 与
一次 reset，累计 740 个 suspended samples，并继续完成 1,206 个 Ready samples / 1,948 个
live frames；actor 仍逐字段等于 readiness，证明同批 Jump 边沿和 held W 都未进入恢复后的
非零模拟，且无 elapsed backlog、stall 或 render block。该结论不声称 HostInput 在最初 ingest
中立即删除 Space 边沿；它只由 Suspended/Reset 后的 Ready 进展与最终 actor 共同界定。report
447,445 bytes；全部 103 engine-runtime、45 Prototype、20 reference-host 测试通过，Flavor
0 deny / 5 个既有 warning；产品、Runtime、renderer/GPU/source/resource/synchronization
均未改变。
Experiment 0134 修复了 post-ready Activated 验收在 12-frame acknowledgement 边界上的
实机竞态。最初三个 v49 运行均在 focus sessions 之前失败；临时精确诊断一致显示 action
`committed=1`、`ineligible=0`、target 已清、Activated 恰为 12、Rejected 为 0，但
suppression 为 0，证明原 200ms Escape 恰落在绿色反馈结束处，不能保证后续 suppression frame。
最终只把验收 helper 的请求延长为 250ms，并严格验证观测区间 `[250,750]`ms；产品阈值和
12-frame 契约未变。`canonical-prototype-v49` 在 174.564 秒通过：PID 28412 / thread 31632
以 0.0016ms 原子 F/Enter 批次提交 authored ID 496，270.6458ms 后 Escape；完成值保留
12 Activated frames 并新增 2 suppression frames，committed identity/exclusion 精确、actor
静止、对象 pending/target/ack 清空、零 render block、stdout 恰为两值。report 446,569 bytes；
全部 103 engine-runtime、45 Prototype、20 reference-host 测试通过，Flavor 0 deny / 5 个
既有 warning；产品、Runtime、renderer/GPU/source/resource/synchronization 均未改变。
Experiment 0135 完成“逢 5”兼容清理：Experiment 0130 已删除 `startupNativeInput` 报告字段，
但 camera repeat/re-press、Run release/re-press、opposite locomotion、diagonal Walk/Run 和
forward-release 八个当前 oracle 仍各自保留一条旧 shape rejection。现已直接删除这些重复
分支，当前消息顺序、时间区间、actor/camera/presentation/object 语义均未改变；一个中央
removal guard 扫描全部八个 owner，禁止历史字段返回。`canonical-prototype-v50` 在
174.239 秒通过，16 个正常原生 session 均以恰好两值完成；代表性结果为 forward-release
16 步/`deltaZQ9=-512`、focus actor 精确不变、Activated 对象动作 12 绿帧后 2 suppression
frames。report 446,503 bytes；全部 103 engine-runtime、45 Prototype、20 reference-host
测试通过，Flavor 0 deny / 5 个既有 warning；产品、Runtime、renderer/GPU/source/resource/
synchronization 均未改变。
Experiment 0136 把同一 focus-discontinuity 进程的原子批次从 Space/W 扩为
Space/F/Enter/W/`WM_KILLFOCUS`，没有增加 child 或产品输出。`canonical-prototype-v51`
首轮在后续独立 invalid-key 进程的既有 clock-continuity oracle 偶发失败；代码与阈值不动的
原样复跑在 172.935 秒通过，未增加 retry。PID 5004 / thread 25564 依次投递四键，三个 interval
为 0.0014/0.0013/0.0012ms，完整 batch span 0.0039ms。完成值记录一次 suspend/resume、
一次 post-resume reset、88 个 suspended samples、156 个 Ready samples 与 246 个 live frames；
actor 精确等于 readiness，observation pending/target 均空，interaction pending/ack/consumed 均空，
committed/ineligible 均为 0，零 stall/render block，stdout 恰为两值。report 447,315 bytes；
全部 103 engine-runtime、45 Prototype、20 reference-host 测试通过，Flavor 0 deny / 5 个既有
warning；产品、Runtime、renderer/GPU/source/resource/synchronization 均未改变。
Experiment 0137 固定了 Q/E 与 simulation-bound intents 在失焦边界上的既有差异，没有修改
产品：组合政策测试明确证明同一 ingest 的 E-down/FocusLost 会清除 held，同时保留本 sample
的 pressed/released；camera candidate 提交恰好一次 orbit `0 -> 1`，下一空 ingest 边沿过期且
不会重复。focused camera tests 4/4 通过，Prototype 总数增至 46；没有增加原生 child、报告字段
或 camera/HostInput/main-loop 逻辑。`canonical-prototype-v52` 首轮在 176.068 秒通过，现有
原生 session/process 矩阵保持不变，report 447,306 bytes；全部 103 engine-runtime、46
Prototype、20 reference-host 测试通过，Flavor 0 deny / 5 个既有 warning；Runtime、
renderer/GPU/source/resource/synchronization 均未改变。
Experiment 0138 固定了已 held 相机键的互补失焦边界，没有修改产品：先以 E-down 提交
orbit `0 -> 1`，下一 ingest 的重复 E-down 因 held 被抑制，随后 FocusLost 只产生 release
cleanup；最终 held=false、pressed=false、released=true，camera candidate/commit 均保持
orbit 1，不会重复步进。focused camera tests 5/5 通过，Prototype 总数增至 47；没有增加原生
child、报告字段或 camera/HostInput/main-loop 逻辑。`canonical-prototype-v53` 首轮在
170.211 秒通过，现有原生 session/process 矩阵保持不变，report 447,346 bytes；全部 103
engine-runtime、47 Prototype、20 reference-host 测试通过，Flavor 0 deny / 5 个既有 warning；
Runtime、renderer/GPU/source/resource/synchronization 均未改变。
Experiment 0139 固定了相机键失焦清理后的重新准入，没有修改产品：E-down 先提交 orbit
`0 -> 1`，FocusLost 清 held、只产生 release 并保持 orbit 1；同一 HostInput owner 上后续
E-down 恢复为 held=true、pressed=true、released=false，并精确提交 orbit `1 -> 2`。focused
camera tests 6/6 通过，Prototype 总数增至 48；没有增加原生 child、报告字段或
camera/HostInput/main-loop 逻辑。`canonical-prototype-v54` 首轮在 175.160 秒通过，现有原生
session/process 矩阵保持不变，report 447,322 bytes；全部 103 engine-runtime、48 Prototype、
20 reference-host 测试通过，Flavor 0 deny / 5 个既有 warning；Runtime、renderer/GPU/source/
resource/synchronization 均未改变。
Experiment 0140 按兼容与资源双重清理节奏删除 Prototype session 中 6 个恒 false 的负向
`eventStream`/`eventHistory`/`copiedObjectState` 字段，以及 Rust/TypeScript 层 14 个断言、检查和
summary copy；session schema 从 v1 直接升至 v2，不留 decoder、alias、optional branch 或替代 flag。
现有 session guard 扫描 6 个当前 owner 并禁止旧字段返回。`canonical-prototype-v55` 首轮在
170.874 秒通过，439,342-byte report 含 58 个 v2 contract、零旧字段；`canonical-runtime-v19`
在 317.927 秒通过，7,528,196-byte report 含 2 个 v2 checkpoint、零旧字段，并保持 1,037 次
Sidecar invocation、4 warm/8 measured publications、499 handles、23 threads、private bytes
`409,800,704 -> 410,427,392`、2/2 lifecycle 与 24 artifacts / 25,346,264 bytes。全部 103
engine-runtime、48 Prototype、20 reference-host 测试通过，Flavor 0 deny / 5 个既有 warning。
受控提交后的 workspace-local `target/` 与 `out/` 共 7,740 files / 3,253,478,132 bytes，现
已删除；`.task/`、共享全局缓存与 tracked state 未改变。
Experiment 0141 强化既有 focus-discontinuity 进程而不增加 child 或产品输出：原子
Space/F/Enter/W/失焦批次恢复后，在同一 PID/window/thread 上原子投递 A-down，保持
272.3119ms 后 A-up，再静止 261.9928ms 后 Escape。`canonical-prototype-v56` 首轮在
180.909 秒通过；actor 在同一 generation/region 内从 `(0,0)` 精确移动到 `(-512,0)` Q9，
即 16 个 32-Q9 Walk 步，`deltaZQ9=0` 同时证明旧 W 未泄漏、新 A 已准入，最终以 Survey
clip 0/yaw 32768 停止。clock 恰增加一次 suspend/resume/reset，76 个 suspended samples，
零 stall/render block，stdout 恰为两值；442,700-byte report 与全部 103 engine-runtime、
48 Prototype、20 reference-host 测试通过，Flavor 0 deny / 5 个既有 warning。产品、
Runtime、renderer/GPU/source/resource/synchronization 均未改变。
Experiment 0142 复用现有 Jump-readmission 进程闭合 held Space 的失焦生命周期：第一次
Jump 落地但 Space 仍 held 后，同一 PID/window/thread 原子投递重复 Space-down 与失焦；
恢复后现有 Space-up/down 被重新准入并产生精确第二飞行。`canonical-prototype-v57` 首轮在
170.997 秒通过；线程 23692 的单键 batch span 为 0，第二次动作到 Escape 为 118.1957ms，
最终恰为第 7 步、速度 3116 Q16、上升 25571 Q16，actor identity/XZ/shape/Survey/epoch
均不变。clock 恰增加一次 suspend/resume/reset，84 个 suspended samples，零
stall/render block，stdout 恰为两值；445,067-byte report 与全部 103 engine-runtime、
48 Prototype、20 reference-host 测试通过，Flavor 0 deny / 5 个既有 warning。产品、
Runtime、renderer/GPU/source/resource/synchronization 均未改变。

## Project model

- [Repository ownership model](docs/architecture/repository-model.md)
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
runseal :prototype start
runseal :prototype status
runseal :prototype restart
runseal :prototype stop
runseal :canonical-prototype
runseal :canonical-actor
runseal :canonical-frame
runseal :canonical-resources
runseal :canonical-runtime
runseal :workbench start
runseal :workbench inspect
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

`runseal :prototype start` is the self-contained manual prototype entry. It deterministically cooks
the zero-origin `[-8,8]²` finite sandbox, declares inclusive `[-6,6]²` playable bounds in strict
bootstrap schema 2, and then uses
`sidecar.prototype.toml` to launch the application without an inspect endpoint. It becomes visible
and ready only after canonical content has rendered; close the window, press Escape, or use
`runseal :prototype stop` to end it. W/A/S/D moves relative to the current camera, hold Shift to run,
Q/E changes the committed camera orbit, Space requests one grounded Jump, and F acquires or clears
one read-only nearby-object target after the next committed movement step. Its source/window lifetime
is shown by static amber feedback. Enter acts only after the next committed step re-resolves that
exact target inside 512 Q9; a successful projected action receives a bounded 12-frame green
acknowledgement. No prior canonical acceptance output is required.

`runseal :canonical-prototype` is the focused real-process prototype workflow. It runs the
runtime/prototype/reference-host tests, cooks the six required signed centers, and proves
terminal missing/corrupt bootstrap failure, exact committed grounded gravity witness, exact stationary/native-W Walk
plus visible native-Shift+W Run and same-sample E+W camera-relative locomotion with transactional
Survey/Walk/Run selection and exact
committed eight-way facing, one
committed current actor authority, Q/E committed actor-relative camera orbit/frame ordering, typed render-block
consumption with zero normal-path blocks, one visible-window native Space action with exact
committed vertical trajectory and grounded-policy consumption plus a complete landing and exact
second native Space readmission across held-key focus cleanup and exact midair re-press rejection
with a Walk admission witness,
two exact-window atomic native F+Enter stationary observation/actions whose positive/negative-X
source fixtures are invariant across every allowed first batch, whose bounded results match an
independent source oracle, and whose exact Activated/Rejected targets commit only through the
successful frame, whose 250 ms post-action dwell proves at least one frame of suppression after
the exact 12-frame acknowledgement, plus sustained post-readiness motion/capacity rejection,
one exact camera-derived traversal schedule with prefetch disabled, explicitly activated held-W
finite-edge survival, exact native Escape and visible-window WM_CLOSE clean exits, native
same-batch Space/F/Enter/W focus-loss action/held-input suppression plus no-backlog resume and
same-process fresh-A Walk/release readmission, one exact atomic same-ingest opposite-Q/E
camera-edge cancellation with negative-Z Walk proof, direct restart equality, and Sidecar cleanup.

`runseal :canonical-frame` is the focused real-process GPU regression workflow. It cooks a fresh
minimal signed pair, checks strict committed CPU object lookup and exact terrain-position conversion
against an independent pack-byte oracle, checks the exact accepted canonical frame, immediately
replays it, and owns complete process cleanup. It does not replace end-to-end acceptance.

`runseal :canonical-actor` is the focused frame-safe actor GPU workflow. It proves typed
actor lifecycle/restart, fractional and coarse/nominal schedule/actor partition equality, complete
failure rollback through the canonical aggregate, advanced/render-blocked motion/presentation
candidate admission against published and non-prefetch
pending windows, one checked batch-entry vertical velocity delta with exact success/block rollback,
the final committed fixed-step grounded witness with blocked-candidate isolation, retained
rendering, dynamic candidate identity, alternating frame-slot
writes, cull/surface/shadow/occlusion participation, despawn/respawn clearing, frustum rejection,
outside-window rollback, and semantic capture.

`runseal :canonical-resources` is the focused deep resource/lifecycle workflow. It samples the warm
baseline before the first of 64 measured publications, rejects bounded active growth, requires at
least 60 seconds of post-workload handle stability and recovery, then proves 16 complete process
cycles.

`runseal :canonical-runtime` is the only end-to-end engine acceptance workflow. It cooks
signed terrain and schema-3 object sources directly, validates explicit presentation,
deterministic presentation time, fixed camera-visible directional object shadows, canonical
runtime and timeline ownership, successful-frame transactions, fixed normalized host input state,
strict configured canonical readiness, shared reference-host ownership, plain prototype
startup/restart/cleanup, exact committed-snapshot CPU object lookup, checked Q9 position conversion,
and bounded nearest selection, exact published-snapshot CPU terrain queries,
exact caller-owned vertical terrain contact and bounded transition witnesses, the explicit bounded
60 Hz simulation schedule and its frame/presentation independence, private fixed terrain-body
motion/translation/planar-first/batch contracts, one retained terrain-body generation lifecycle
with exact failure rollback and process reset, the sole explicit elapsed schedule/body dual commit
with coarse/nominal partition equality and complete rollback,
clear-only idle behavior and retired-control rejection, composition, fault rollback, and
traversal/prefetch/rollover without invoking an older experiment workflow. The v5 operator keeps a
4..8 publication state-driven warm, an 8-publication active-resource checkpoint, and two complete
lifecycle checkpoints in this full path;
the focused resource owner retains the 64-publication/60-second/16-cycle deep soak. Full acceptance
persists only representative captures, uses readback-only color/object-ID observations for repeated
hash assertions, and reports stage timings, operation counts, and artifact bytes.

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
