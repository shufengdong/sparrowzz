# 配电网仿真工具 TensorEval

本目录保存经过回归测试验证的 RustScriptSimulation（RSS）脚本（以下简称为 RSS 脚本），
统一使用 `.rss` 扩展名。
请在 eig-rs 工作区根目录运行命令，因为脚本中的 DSS 路径相对于进程工作目录解析。

从现有 OpenDSS 模型和 Python 仿真脚本迁移时，请参阅
[从 OpenDSS Python 脚本迁移到 RSS](docs/opendss_to_rss.md)。
完整命令行模式和暂态入口见 [tensoreval 命令行与暂态仿真参考](docs/tensoreval_cli_reference.md)；
语法、函数、schema、兼容范围和智能体编写指南见 [`docs/README.md`](docs/README.md)。

## 定位

`tensoreval` 不是 OpenDSS 的替代求解器，而是把 OpenDSS 工作流编译进 Rust 原生运行时
的编译器与运行时：`import_dss` 把声明式 `.dss` 模型转成 21 张类型化网络表，`ds_*`
函数执行分析，`--dss-to-rss` 产出可移植的 RSS 脚本或 `.rssbundle`。数值上它以
OpenDSS 为 oracle 复现结果（复电压相对误差门限 `1e-3`），而不是宣称超越它；日常
仿真、DataFrame 后处理和流程编排由 RSDss 承担，OpenDSS 保留为独立基准生成器。

## 相比 Python + OpenDSS 的优势

- **执行模型分离**：声明式模型（`import_dss`）与命令式执行（`ds_*`）拆开，同一份
  `model.dss` 可同时喂给 OpenDSS 作 oracle、喂给 tensoreval 作日常仿真。
- **原生执行，无 Python / OpenDSS 运行时**：单一 Rust 二进制，无需 `opendssdirect`、
  DSS C-API 的版本管理；核心计算是原生 Rust + MUMPS/MKL，拼接与索引等内核快于解释型
  循环（见 [`../rspower/bench/PF_PERFORMANCE.md`](../rspower/bench/PF_PERFORMANCE.md)）。
- **失败关闭 + 有界**：`import_dss` 是转换器而非解释器，无法无损表达的 OpenDSS 命令或
  对象会被拒绝并返回诊断码，而不是静默忽略；URL 下载、ExecutionStudy 子集均有边界。
- **类型化 DataFrame 契约**：20 张网络表固定顺序、`para.result` 契约、每张结果表的
  列/类型/单位由 schema 与机器可读 catalog 固定，可被一致性检查校验。
- **可打包部署**：`--dss-to-rss --mode bundle` 把模型、脚本与外部数据冻结为自包含的
  `.rssbundle`。
- **超出 OpenDSS 的分析面**：状态估计、故障定位、恢复重构、保护配合审计、Monte
  Carlo、电机网络动态等入口在 OpenDSS/Python 生态中不原生提供。

边界：纯潮流求解速度与编译型 OpenDSS 内核大致相当，优势集中在后处理、流程编排与
工程化；兼容面有界，未覆盖对象会 fail-closed 拒绝，迁移需先拆分模型与 `Solve`。

## tensoreval 功能入口

对外工具只有 `tensoreval`，按输入类型选择功能：

| 输入与参数 | 功能 | 主要输出 |
|---|---|---|
| `tensoreval --df FILE.rss` | 执行 RSDss DataFrame 图；可调用全部 `ds_*` 分析 | 脚本最深依赖层的具名 DataFrame |
| `tensoreval FILE.dss` | 自动分类并直接执行受支持的 DSS 研究 | 研究对应的规范结果表 |
| `tensoreval FILE.rssbundle` | 校验并执行便携 bundle | bundle `main.rss` 声明的结果表 |
| `tensoreval --dss-to-rss FILE.dss` | 只转换为 linked RSS 或 bundle | `.rss` 文件或 `.rssbundle` 目录 |
| `tensoreval FILE.rss` | 普通 Tensor 表达式图或 RustScript VM | Tensor/标量调试文本 |

`--result-dir DIR --output-format csv|parquet` 将每张具名 DataFrame 完整落盘；终端表格仅供查看，
可能截断。输入组合、参数约束、退出码和结果文件命名见
[tensoreval 命令行参考](docs/tensoreval_cli_reference.md)。

## 暂态仿真

新增暂态能力是有界 RMS/phasor 网络动态 V12，共有两条主要使用路径：

- 已有 OpenDSS Dynamic/Dynamics 入口时，直接运行 `.dss`，或先用
  `tensoreval --dss-to-rss --study dynamic` 生成可审查的 RSS/bundle；
- 已有类型化网络和事件表时，在 `--df` 脚本中调用固定 29 参数的 `ds_network_dynamics`。

DSS 命令流必须保持固定正步长和连续时钟。显式表入口必须提供严格递增的 `timeline.time_s`，
并按固定位置传入网络、事件、保护、故障、spec 和 para 表。两条路径统一输出可选的 15 张 V12
长表，覆盖电压、支路电流、电源功率、PV/Storage/电机/Generator 状态、事件、求解轨迹、故障、
保护、FRT、普通控制、设备汇总和逐 owner 连接审计。同一 `time_s` 可出现多个 `sequence`，
动作前后状态不能只按时间去重。

这是 PSA 已取证的有界暂态子集，不是 EMT，也不代表完整 OpenDSS Dynamic Mode。逐项输入、输出、
单位和失败条件见 [暂态仿真参考](docs/tensoreval_cli_reference.md#5-暂态仿真的三种入口)。

## 与 mems DataFrame 流程引擎（DFF）结合

[`mems`](../../mems/README.md) 的 DFF 是 eig-rs 工作区的 DataFrame 流程引擎：
`DffModel` 定义可持久化流程（节点、边、触发方式、保存策略），编译为 `DfFlow` 运行时
计划，由调度器触发、执行器执行，结果落库或走 Web 服务。RSS/RSDss 脚本通过
`DfNodeType::TensorEval` 节点 + `TE_PARA_DF` 参数位作为流程中的一个节点嵌入：节点输入
边的别名成为脚本变量，脚本末层变量成为节点输出。由此获得：

- **配网分析成为流程编排的一等节点**：DSA 脚本与 `Source`/`Transform`/`Sql`/`Solve`/
  `NLP`/`MILP`/`Wasm` 节点混编成一条流水线，而不是孤立的命令行计算。
- **类型化 DataFrame 在节点间显式流动**：潮流输出的 `bus_phase` 等结果直接沿边进入
  下游变换、优化或持久化，无需 CSV/pickle 中转。
- **同源零拷贝运行时**：DSA、DataFrame 变换与优化共享 polars 与 `eig_expr`/`eig_aoe`，
  结果不跨 Python 序列化边界。
- **触发与持久化**：`DfTriggerType`（定时/手动/依赖）让 QSTS/时序潮流成为可周期性重跑
  的持续仿真任务，结果按 `DfSaveMode` 落库。
- **与优化/求解混编闭环**：潮流或状态估计结果可作为 `NLP`/`MILP` 节点的初值与约束
  输入，形成"分析 → 优化"闭环。

## 脚本

| 脚本 | 计算 |
| --- | --- |
| `bmopf_4bus_power_flow.rss` | BMOPF JSON 4Bus-DY-Bal 辐射状配电网潮流 |
| `bmopf_ieee13_power_flow.rss` | BMOPF JSON IEEE13 辐射状配电网潮流 |
| `bmopf_ieee34_power_flow.rss` | BMOPF JSON IEEE34 辐射状配电网潮流 |
| `ieee13_power_flow.rss` | IEEE13 辐射状配电网潮流 |
| `ieee34_power_flow.rss` | IEEE34 辐射状配电网潮流 |
| `ieee123_power_flow.rss` | IEEE123 辐射状配电网潮流 |
| `ieee13_harmonic.rss` | IEEE13 1/3/5/7 次谐波 |
| `ieee34_harmonic.rss` | IEEE34 1/3/5/7 次谐波 |
| `IEEE13Nodeckt_harmonic_run.rss` | 从完整 IEEE13 谐波 DSS 入口生成的链接脚本 |
| `ieee34Mod1_harmonic_run.rss` | 从完整 IEEE34 谐波 DSS 入口生成的链接脚本 |
| `ieee13_short_circuit.rss` | IEEE13 母线 652 单相接地短路 |
| `ieee34_short_circuit.rss` | IEEE34 母线 810 单相接地短路 |
| `4bus_daily_qsts.rss` | 4Bus-DY-Bal 三时点 Daily QSTS |
| `4bus_dy_run.rss` | 从完整 4Bus-DY-Bal Daily DSS 入口生成的链接脚本 |
| `ieee13_daily_qsts.rss` | IEEE13 全负荷三时点 Daily QSTS |
| `case14_4bus_master_slave_pf.rss` | IEEE case14 主网与 4Bus-DY-Bal 配网显式展开三轮的单 PCC 主从分裂案例 |

主从分裂案例的每轮主网计算都是独立 Prog，通过
[`runpf_bundle.txt`](runpf_bundle.txt) 递归装入相同的 RSPower 依赖，避免在案例中重复维护
函数清单。共享清单仍需写在每个 `{ ... }` 内，使对应 VM 获得完整函数定义。

`models/*_static`、`models/*_harmonic` 和 `models/4bus_daily_qsts` 保存测试所用的
OpenDSS 模型。基础模型保留网络定义，`*_run.dss` 是可由 OpenDSS 直接执行的完整入口，
包含计算模式、必要参数和 `Solve`。模型同时补齐 PSA 严格导入所需的线路额定值。

## OpenDSS 算例与转换结果

| OpenDSS 执行入口 | RSS 结果 | 说明 |
| --- | --- | --- |
| `models/ieee13_static/IEEE13Nodeckt_run.dss` | `ieee13_power_flow.rss` | IEEE13 Snapshot 潮流；已有脚本与生成结果等价，不重复保存 `_run.rss` |
| `models/ieee34_static/ieee34Mod1_run.dss` | `ieee34_power_flow.rss` | IEEE34 Snapshot 潮流；已有脚本与生成结果等价，不重复保存 `_run.rss` |
| `models/ieee123_static/IEEE123Master_run.dss` | `ieee123_power_flow.rss` | IEEE123 Snapshot 潮流；已有脚本与生成结果等价，不重复保存 `_run.rss` |
| `models/4bus_daily_qsts/4bus_dy_run.dss` | `4bus_dy_run.rss` | 三个一小时时步的 Daily 执行计划；RSS 调用 DSS 执行计划门面 |
| `models/ieee13_harmonic/IEEE13Nodeckt_harmonic_run.dss` | `IEEE13Nodeckt_harmonic_run.rss` | 完整内联 IEEE13 模型，先求解基频，再计算 1/3/5/7 次谐波 |
| `models/ieee34_harmonic/ieee34Mod1_harmonic_run.dss` | `ieee34Mod1_harmonic_run.rss` | 完整内联 IEEE34 模型，先求解基频，再计算 1/3/5/7 次谐波 |

三个生成的链接 RSS 保留 DSS 入口路径和源树摘要，运行时以表中 `.dss` 文件为权威输入。
`4bus_daily_qsts.rss`、`ieee13_harmonic.rss` 和 `ieee34_harmonic.rss` 则显式组装
DataFrame，便于查看和修改每张输入表。

回归测试会重新转换静态入口，并将静态潮流、Daily QSTS 和谐波结果逐节点、逐相、
逐时点或逐谐波与 OpenDSS 复电压基准比较，最大允许相对误差为 `1e-3`。

官方 OpenDSS 发行文件的便携转换结果集中在 [`rssbundle`](rssbundle/README.md)，
其中列出 131 个已发布入口、基线未支持文件的原因和与 OpenDSS 的数值对照。官方
`Examples/FreqScan/Run_Scan.dss` 已转换为可搬移 bundle，完整重放 188 个非整数频点并以
564 行 Monitor 数据与 OpenDSS 逐点比较。
针对 249 个独立入口中后续 8 个候选的定向分析见
[`rssbundle/capability_coverage_audit.md`](rssbundle/capability_coverage_audit.md)：IEEE34 的两个入口、
三个 IEEE123 入口和 IEEE37 已补齐 `LoadShape.default`、RegControl `Like` 有序回放、`Mode=F`
规范化、Mode 时钟/控制队列和 DSS 母线名大小写语义，完成 OpenDSS 数值门禁并发布 bundle。
YYD 完整入口仍需要跨研究 Direct 专用合同。InductionMachine 可进入新增的有界
Dynamic/Dynamics V12 adapter，但仍须通过该入口的保护、Monitor 和数值门禁，不能只按语法成功计入。
`Examples/DistanceRelays` 的 4 个固定步长 Distance/TD21 入口已经完成上述门禁并发布 bundle。

## 运行

完整 OpenDSS 执行文件可先严格转换为 RSS；详细边界见
[DSS 到 RSS 严格转换](docs/dss_to_rss.md)：

```bash
tensoreval --dss-to-rss \
  --output sparrowzz/dss/4bus_dy_run.rss \
  sparrowzz/dss/models/4bus_daily_qsts/4bus_dy_run.dss
```

潮流或短路脚本可直接运行：

```bash
tensoreval --df sparrowzz/dss/ieee13_power_flow.rss
```

BMOPF JSON 潮流脚本同样从工作区根目录运行：

```bash
tensoreval --df sparrowzz/dss/bmopf_ieee13_power_flow.rss
```

QSTS 脚本也可直接运行：

```bash
tensoreval --df sparrowzz/dss/4bus_daily_qsts.rss
```

该脚本用 0、3600 和 7200 秒的负荷曲线样本求解三个一小时时步；结果时间是每个时步
的结束时刻，即 3600、7200 和 10800 秒。

已发布的 Distance 暂态保护 bundle 可直接运行并完整落盘：

```bash
tensoreval --result-dir /tmp/distance-dynamic \
  sparrowzz/dss/rssbundle/Examples/DistanceRelays/DistanceRelayTest.rssbundle
```

输出目录包含 `bus_voltage.csv`、`branch_current.csv`、`event.csv`、`protection_state.csv`、
`device_protection_connection.csv` 等 15 张 V12 结果表。直接运行受支持的 Dynamic `.dss` 使用相同
结果合同。

联合潮流案例需要主网数值求解器和配网后端：

```bash
tensoreval --df sparrowzz/dss/case14_4bus_master_slave_pf.rss
```

谐波计算应使用 release 构建：

```bash
tensoreval --df sparrowzz/dss/ieee13_harmonic.rss
```

通过 `cargo make tensoreval-native` 构建后，也可以直接运行：

```bash
target/release/tensoreval --df sparrowzz/dss/ieee13_power_flow.rss
```
