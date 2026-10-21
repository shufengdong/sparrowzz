# OpenDSS 官方案例转换失败与 PSA 能力覆盖审计

本报告回答一个比“首个诊断是什么”更严格的问题：当前转换失败的官方入口中，哪些已经有
PSA 数值能力和参考证据，只是尚未由 `tensoreval --dss-to-rss` 正确接入；哪些在修复首个
路径或解析问题后仍会遇到真实能力边界。

## 审计基线

- 上游版本：`dss-extensions/electricdss-tst` 提交
  `3b208397160213cae4a9e2d0a7d1aa3528ce26e1`。
- 扫描根：`Version8/Distrib`。本报告只追踪基线转换失败的 249 个独立入口；模型片段、
  公共依赖和其他非独立文件不纳入本报告的数量或结论。
- 命令：`tensoreval --dss-to-rss --check --source-root <Version8/Distrib> <entry>`。
- 基线结果：10 个入口转换并自执行成功，249 个独立入口转换失败。
- 每个失败入口先记录全部诊断，不以首诊断推断后续能力；对路径和词法类失败，另在临时
  副本中只修正引用位置或行尾注释，再次执行以取得第二层诊断。
- 能力覆盖必须至少有 PSA 运行时 consumer 和测试；“参考覆盖”还要求锁定 OpenDSS 输出
  或官方入口的等价投影证据。只有存在同类 API，不算案例已覆盖。

## 优先接入的 12 个入口及定向复测

下面 12 个入口已有直接或同入口等价投影的数值证据。基线失败点位于转换编排、研究适配、
求解后端选择或严格导入门面，不需要先新增一种电气元件方程。2026-10-21 的定向复测已让
12 个入口全部通过严格转换；其中 11 个完成 bundle 自执行并按官方相对路径发布，2,900 步
P174 入口完成转换和官方输入到锁定数值投影的逐表等价检查。该入口的完整 `--check` 在 release
构建运行 90 分钟后仍未结束，人工停止，因此不把本次运行记为完整求解通过，也不发布半成品
bundle。消除 bundle 编译与 staging 自执行的重复求解后，又对唯一一次 staging 自执行重试
30 分钟；执行仍未完成，临时目录已清理。

| 入口 | 已有能力证据 | 定向复测结果 |
|---|---|---|
| `IEEETestCases/4Bus-OYOD-Bal/4Bus-OYOD-Bal.DSS` | 两个 OYOD 工作点均有官方入口到 PSA 投影的逐输出证据；IEEE4 连接矩阵参考测试通过 | 严格转换和两项独立 study 自执行通过，bundle 已发布 |
| `IEEETestCases/4Bus-YYD/YYD-Master-step1.DSS` | `yyd_step1` 的 V/I/P/Q/loss/control 锁定参考已存在 | Newton 自执行通过，bundle 已发布 |
| `IEEETestCases/IEEE 30 Bus/Run_IEEE30.DSS` | 官方入口的 Model 3 静态 Newton 投影与 OpenDSS 输出逐文件一致 | 静态 Newton 自执行通过，bundle 已发布 |
| `IEEETestCases/8500-Node/Run_8500Node.dss` | 原始 balanced deck 的 execution profile、控制终态和电气量已有测试 | Newton 自执行通过，bundle 已发布；自动 EnergyMeter 区域以显式 approximation Warning 保留 |
| `IEEETestCases/8500-Node/Run_8500Node_Unbal.dss` | 原始 unbalanced deck 的 execution profile、控制终态和电气量已有测试 | Newton 自执行通过，bundle 已发布；Monitor 标准位置参数已规范化 |
| `IEEETestCases/8500-Node/P174_Run_Voltage_Profile.DSS` | 两个有序 epoch 和 100%/40% 工作点已有数值参考 | 7,598 条语句、2 个 epoch 自执行通过，bundle 已发布 |
| `IEEETestCases/8500-Node/P174_Run_360kW_PV.DSS` | 2,900 步 Duty、曲线长度、调压器/电容状态及采样数值均有参考 | 7,616 条语句、5 个 epoch 严格转换通过；投影逐表等价；完整长时序未完成，bundle 未发布 |
| `Examples/Microgrid/GridFormingInverter/GFM_IEEE8500/Run_8500Node_Unbal.dss` | 文件 SHA-256 与上面的官方 unbalanced 入口完全相同 | 与同内容 unbalanced 入口使用同一路径，自执行通过，bundle 已发布 |
| `IEEETestCases/ShortCircuitCases/IEEE123Master-SC.dss` | IEEE123 全母线 414 个故障参考测试通过 | FaultStudy adapter 自执行通过，bundle 已发布 |
| `IEEETestCases/ShortCircuitCases/ieee34Mod2_SC_Case_II.dss` | IEEE34 全母线 148 个故障参考测试通过 | FaultStudy adapter 自执行通过，bundle 已发布 |
| `Examples/HarmonicsTMode/IEEE_519.DSS` | HarmonicT 的时间×频率、LoadShape/Storage/Fault 状态物化已有 OpenDSS 参考 | `Open Line` 跨 Solve 快照物化后自执行通过，bundle 已发布 |
| `Examples/HarmonicsVariableLoad/IEEE_519.DSS` | 同一 HarmonicT 运行时已覆盖逐快照状态变化 | Load 属性跨 Solve 快照物化后自执行通过，bundle 已发布 |

这些入口的转换改造已经按 IEEE4 后端/多 study、IEEE30 静态 Newton 路由、FaultStudy
adapter、IEEE519 快照物化和 IEEE8500 严格门面的顺序完成。IEEE8500 仍是性能和完整性
长门禁；尤其 2,900 步 Duty 不进入普通快速测试。

## 核心能力已有，但不能直接承诺完整转换的 8 个入口

下面 8 个入口的主要网络或算法已经覆盖，但官方脚本还包含没有完成无损映射的命令、对象
或输出。它们是下一层候选，不应与上面的 12 个混为“修一个首诊断即可通过”。

| 入口 | 已覆盖部分 | 仍需闭合的部分 |
|---|---|---|
| `IEEETestCases/34Bus/Run_IEEE34Mod1.dss` | IEEE34 潮流、调压器、QSTS、谐波和 FaultStudy | 原始 execution import 仍把 RegControl 当作静态忽略；EnergyMeter 自动区域需明确输出合同 |
| `IEEETestCases/34Bus/Run_IEEE34Mod2.dss` | 同上 | 除 RegControl 外还有 LoadShape 引用未解析，需修复后再探测 |
| `IEEETestCases/123Bus/Run_IEEE123Bus.DSS` | IEEE123 网络、RegControl/CapControl 和多工况 QSTS 有 OpenDSS 参考 | `BusCoords` 行尾注释解析错误修正后，仍有 RegControl `Like` 位置合同和同文件两次 `Compile`/两项研究 |
| `IEEETestCases/123Bus/P174_Run_VoltageProfile.DSS` | 100%/40% 负荷工作点和连续控制能力已覆盖 | 继承 master 中的 RegControl `Like` 与 EnergyMeter/Plot 输出边界 |
| `IEEETestCases/123Bus/P174_Run_SolarRamp.DSS` | Generator Duty、300 点曲线和 IEEE123 控制能力已覆盖 | 同一 master 的 `Like`、运行后 Monitor/Plot，以及原始入口的完整数值门禁尚未建立 |
| `IEEETestCases/4Bus-YYD/YYD-Master.DSS` | YYD 潮流和显式短路数值核心均存在 | 一个脚本先做潮流，再创建 Fault、批量禁用负荷并进入 Direct；需要跨研究状态 lowering |
| `IEEETestCases/ShortCircuitCases/ieee37_SC_Currents.dss` | IEEE37 网络和一般 FaultStudy 核心存在 | 尚无这个 IEEE37 `Mode=F` 入口的全量短路参考，不能用 IEEE37 潮流证据代替 |
| `Examples/InductionMachine/Run.dss` | `IndMach012`、有限动态电机和有界故障/过流保护运行时已有参考 | PSA 明确不声明完整 OpenDSS Dynamic Mode 等价；需先核对该入口的 Relay、5001 步时钟和 Monitor 通道 |

## 首诊断不能直接判定能力覆盖

### 27 个跨目录资产失败

所有被拒绝的 `../` 资产都能在官方 `Version8/Distrib` 树中找到，因此
`REQUEST.SOURCE.ESCAPES_ROOT` 确实暴露了 acquisition root 与声明文件目录处理错误。但在临时
副本中只把资产复制到入口目录后，27 个文件没有一个直接转换成功：

- 11 个 DRC/VV_DRC 案例继续失败于隐式 `DERList`、`avgwindowlen`、
  `voltage_curvex_ref` 等有界 InvControl 合同；多数还使用当前 DRC 子集不接受的
  `DeltaQ_Factor=0.2`。
- 8 个 StorageControllerTechNote 案例继续失败于每个 Storage 的 `EffCurve`。控制器的
  Follow、LoadShape、PeakShave、PeakShaveLow、Schedule、Support 和 Time 模式虽已存在，
  但不能忽略设备效率曲线后声称官方结果一致。
- 8 个 MemoryMappingLoadShapes 案例还涉及 MemoryMapping、Sng/Dbl 二进制或专用文本加载
  语义，超出当前 LoadShape 文件合同。

所以这 27 个都是“路径层有 bug”，但不是“能力已经完整覆盖”的案例。

### 23 个 `READ.DSS.VALUE_DEFAULTED`

这 23 个均来自 InvControl 监测电压或 Volt-Var 示例。额定电压推断是一个导入缺口，但全部
入口还同时省略显式 `DERList`，多数使用 `DeltaQ_Factor=0.2`，并含 `Set CaseName` 或旧式
`Set maxi`。PSA 的 AVG/MIN/MAX、远端监测和 Volt-Var 测试只能证明对应原语存在，不能证明
这些官方入口已经处于当前有界子集。

### 25 个 `READ.IO.OPEN`

- 19 个是 `BusCoords` 引号、括号或行尾注释被并入文件名。只修正词法后，除 IEEE123
  `Run_IEEE123Bus.DSS` 进入上面的候选层外，其余立即暴露 Dynamic lifecycle、LoadShape
  `Action`、Storage `AmpLimit`、WindGen 对象或 BatchEdit 等真实边界。
- 6 个引用的 `LoadShape1.CSV`、`solarramp.csv` 或 `Windoutput.txt` 在相应官方案例目录中确实
  缺失，不能由转换器猜测或从同名文件静默替代。

### 其他明确能力缺口

- 36 个以 `PSA.DSS.PROPERTY.SEMANTIC_UNREPRESENTABLE` 为首诊断的入口主要使用非 1 的
  InvControl `deltaP/deltaQ`、超出有界曲线基准或成员关系。
- 11 个 PVSystem 入口使用 `PFPriority` 等没有无损映射的属性。
- 9 个 Storage 入口使用 `%kWrated`、`EffCurve` 或 `DebugTrace`；前两项会改变电气结果。
- 11 个 LoadShape 入口使用 `Action`；它不是普通 Daily/Yearly/Duty 曲线采样的别名。
- GFM、DynEq、VCCS、用户 DLL、Distance/TD21/59N Relay、AutoTrans 未支持接法和特殊节点
  编号仍属于明确边界。
- IEEE118 原始 53 Generator deck 虽有静态替代投影，但原生 generator 版本自身没有完成
  有界参考生成，不能列为“转换后应与 OpenDSS 一致”的确定案例。

## 建议的实现门禁

1. 为每个候选保存“原入口 SHA-256、转换模式、PSA 证据 case、首诊断、修复后诊断”关系，
   避免只维护诊断码白名单。
2. 上述 12 个入口已通过严格转换，11 个已发布 bundle；下一步逐个补齐 OpenDSS 的
   V/I/P/Q/loss、控制终态或 Fault/Harmonic 专用结果比较，并为 2,900 步 Duty 配置独立长任务预算。
3. 每消除一个首诊断都重扫 249 个独立失败入口，并对受影响文件继续探测第二层诊断；成功数量相同
   不代表改造没有改变失败原因。
4. 对 Plot、Show、Export、FileEdit、BusCoords 等副作用采用 manifest 记录；电气状态修改
   `Open/Close/Edit/BatchEdit` 必须进入执行计划，不能按输出副作用忽略。
5. `Mode=FaultStudy`、HarmonicT 快照和一个文件内多 study 应各自有独立 adapter，不应塞入
   普通 QSTS execution profile。

## 本次验证

- `cargo test -p psa --features dsa --test dss_import ieee4_transformer_connection_matrix_matches_locked_native_references`：1 项通过。
- `cargo test -p psa --features dsa --test dss_import ieee8500_original_decks_lower_to_single_step_static_control_execution_profiles`：1 项通过，157.19 秒。
- `cargo test -p psa --features dsa --test short_circuit fault_study_matches_opendss_ieee`：IEEE13/34/123 共 3 项通过。
- `cargo test -p psa --features dsa --test dss_import p6_harmonic_t_materializes_loadshape_state_before_each_frequency_scan`：1 项通过。
- `cargo test -p psa --features mumps-static --test dss_post_solve_actions -- --nocapture`：
  9 项通过，包含 Newton 执行计划、流式结果和求解后动作。
- `cargo test -p mems --features with_dsa,tensoreval-cli --bin tensoreval -- --nocapture`：
  48 项通过，包含 DSS→RSS 分类、生成、bundle 单次自执行、Redirect 后的数据资产读取和
  Newton 后端缺失时的明确拒绝。
