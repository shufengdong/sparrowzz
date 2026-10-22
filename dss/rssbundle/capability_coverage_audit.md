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
- 当前结果：51 个入口已经发布 bundle，208 个独立入口尚未转换；`conversion_report.csv`
  的 `status` 已同步当前发布状态，未发布入口已用当前程序重扫；2,900 步 P174 保留已验证的
  未发布记录。
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

## 后续 8 个候选入口的定向分析

2026-10-21 使用当前 `tensoreval` 重新执行这 8 个官方入口，并按建议顺序实现分类、导入与
执行语义。IEEE34、IEEE123 和 IEEE37 共 6 个入口现已使用未修改的官方源文件通过严格转换、
bundle 自执行和 OpenDSS 数值比较，并按官方相对路径发布。另两个入口仍有明确能力边界。

结论分成三层：

- IEEE34 Mod1/Mod2、三个 IEEE123 入口和 IEEE37 均已完成改造及发布。
- YYD 完整入口和 InductionMachine 入口仍有跨研究或动态能力缺口，不能按转换器别名修复处理。

| 入口 | 当前原文件结果 | 最小探测及结果 | 判断 |
|---|---|---|---|
| `IEEETestCases/34Bus/Run_IEEE34Mod1.dss` | 通过；649 条语句，ExecutionPlan，2 个 epoch | 两个工作点的 V/I/P/Q/loss 已比较，电压最大误差 `9.06e-6`，6 组 tap 一致 | 已发布 |
| `IEEETestCases/34Bus/Run_IEEE34Mod2.dss` | 通过；581 条语句，ExecutionPlan，2 个 epoch | 已物化原生 `LoadShape.default`，并修复 `mid806`/`Mid806` 大小写解析；电压最大误差 `1.36e-5`，线路量最大误差 `5.83e-5`，6 组 tap 一致 | 已发布 |
| `IEEETestCases/123Bus/Run_IEEE123Bus.DSS` | 通过；771 条语句，ExecutionPlan，2 个 epoch | 有界回放 RegControl `Like` 后续 Edit 并按 Clear 隔离历史；电压最大误差 `1.37e-5`，7 组 tap 一致 | 已发布 |
| `IEEETestCases/123Bus/P174_Run_VoltageProfile.DSS` | 通过；379 条语句，ExecutionPlan，2 个 epoch | 100%/40% 工作点电压最大误差 `1.37e-5`，7 组 tap 一致 | 已发布 |
| `IEEETestCases/123Bus/P174_Run_SolarRamp.DSS` | 通过；380 条语句，3 个 epoch、300 点 Duty | `Mode` 重置时钟并恢复 Duty 的 Time 控制队列；12 次 tap 变化、最终档位、Generator 和 300 个 Monitor 样本一致 | 已发布 |
| `IEEETestCases/ShortCircuitCases/ieee37_SC_Currents.dss` | 通过；170 条语句，FaultStudy，175 个场景 | `F` 与 `FaultStudy` 统一规范化；全母线去共模电压误差 `1.61e-7`、故障电流缩放误差 `1.51e-4`、正负序阻抗误差 `8.46e-12` | 已发布 |
| `IEEETestCases/4Bus-YYD/YYD-Master.DSS` | `Mode=Direct` 无适配器 | 临时改成 Snapshot 后依次暴露 `Show/Export/FileEdit`、`BatchEdit`、Fault 原生默认值；继续等价展开后，epoch 1 在 fault-study protection measurement 求解失败 | 中到大改造；需要有序副作用策略、BatchEdit 展开、Fault 默认值和 Direct/显式故障专用执行合同，不能把 Direct 简单改名为 Snapshot 或 FaultStudy |
| `Examples/InductionMachine/Run.dss` | 两次 Dynamic solve 均被生命周期分类器拒绝 | `LoadShape Action=Normalize` 已在导入期物化；原脚本仍含两次 Dynamic solve、5001 步、IndMach012、临时故障、Fuse/Recloser、Voltage/46/47 Relay 和 Monitor mode 3 | 大改造；需要专用 Dynamic adapter、曲线预处理、保护动作时序和 Monitor 通道合同，现阶段不能承诺完整等价转换 |

### 近期 6 个入口的已完成改造

1. IEEE37 的 `F`/`FaultStudy` 已在分类器和 PSA FaultStudy 扫描中统一规范化，并增加 175 个
   全母线故障场景的 OpenDSS 锁定参考。
2. `LoadShape.default` 已按 DSS C-API 0.14.5 的 24 点、1 小时固定间隔曲线物化；仅在活动
   `Clear` 之后存在引用且用户没有显式声明同名曲线时注入。
3. RegControl `Like` 已支持单一非 Like 源、一次 named `New` 以及后续受支持的 `Edit`、`More`
   和属性引用，并按最后一个 `Clear` 隔离对象历史。
4. `Mode` 赋值现在重置执行时钟，并按 OpenDSS 默认令 Duty 使用 Time 队列、其他已接入模式
   使用 Static 队列；同一语句后写的 `ControlMode` 仍可覆盖。
5. DSS 母线引用在生成 NetworkTables 时按已声明母线名做大小写无关归一化，避免径向求解器
   把 `Mid806` 负荷漏出 `mid806` 母线。
6. 六个入口完成 V/I/P/Q/loss/control 或 FaultStudy 专用数值门禁后才正式发布 bundle。

### 两个较远入口的前置条件

- YYD 需要定义 `Direct` 的结果表和 continuation 语义。第一段普通潮流的 Show/Export 是输出
  副作用，第二段 Fault、负荷禁用和 Direct solve 是电气状态变化；两者必须保持顺序。当前把
  Direct 改成 Snapshot 后出现的 fault-study protection measurement 失败说明求解路径也尚未闭合。
- InductionMachine 不能复用普通 QSTS 名称来绕过 Dynamic。验收至少要覆盖 0.3 s 故障投入、
  约 0.4 s 保护隔离、5001 个动态步、转差率/电机状态、Fuse/Recloser/三类 Relay 动作和原脚本
  请求的 Monitor 通道。`Action=Normalize` 已按 OpenDSS 在导入期处理曲线数据；剩余门禁集中在 Dynamic 和保护时序。

## 第三轮实施结果

2026-10-22 完成原定三个范围，新增 4 个按官方相对路径发布的 bundle：

1. `IEEETestCases/123Bus/SolarRamp.DSS` 已物化 Storage 的 OpenDSS 省略值，并闭合默认触发回滞、
   有功端口损耗、SOC 和 `Mode` 重置 Monitor 样本。两个 300 点 Duty 段的电压、Generator、
   Storage、SOC、tap 和 Monitor 数值门禁通过。
2. `Examples/Scripts/IEEE-TIA-LV Model/master_small_1phase.dss` 与
   `master_large_1phase.dss` 已闭合单相两线 Delta/Wye 服务变压器的多端口拓扑、线圈 V/I、损耗和
   EnergyMeter。两者分别以 21/47 条语句生成单 epoch bundle；最大电压相对误差为
   `2.41e-6`/`2.85e-6`，变压器绕组功率最大相对误差为 `9.05e-5`/`2.62e-5`。
3. `IEEETestCases/8500-Node/Run_RecloserSiting.DSS` 使用 MUMPS 发布配置完成 7,628 条语句的 Newton
   转换、自执行和 OpenDSS 对比。8,531 个 OpenDSS 节点全部匹配，最大电压误差 `0.6052%`、
   源端功率误差 `0.387%`、复损耗误差 `1.49%`，均满足既有 IEEE8500 门限。

这 4 个入口不再属于待实现范围。RecloserSiting 的自动网状 EnergyMeter 区域仍返回明确的
approximation Warning，电气结果已比较，但不据此声明 meter-zone 寄存器与 OpenDSS 完全等价。

## 第四轮实施结果

三个 IEEE-TIA 完整分相入口已经发布：

- `master_small.dss` 和 `master_large.dss` 分别以 23/56 条语句完成单 epoch 自执行；OpenDSS 的
  9/31 个节点全部匹配，最大电压相对误差为 `2.41e-6`/`2.85e-6`。
- `Split-Phase_IEEE_TIA.dss` 以 22 条语句完成单 epoch 自执行；7 个节点全部匹配，最大电压
  相对误差为 `2.13e-6`。
- 三个入口的绕组功率最大相对误差不超过 `9.05e-5`，复损耗相对误差不超过 `1.82e-4`。

改造补齐了单相 Delta 两线一次侧在多绕组校验和归一化中的合法映射，并修复了两个次级半绕组
因外部端子标签不同而丢失互阻抗的问题。`Dump` 被识别为只读报告副作用，转换时保留稳定 Warning，
不参与 PSA 电气状态。

`Examples/Matlab/HarmonicT_MATLAB/IEEE_519.DSS` 从优先范围移除。它与已经发布的
`Examples/HarmonicsTMode/IEEE_519.DSS`、`Examples/HarmonicsVariableLoad/IEEE_519.DSS` 哈希不同，
但 `.dss` 文件只完成初始化 `Solve`；实际 HarmonicT 循环位于同目录 `Main_Script.m`，由 Matlab
COM API 发起。转换单个 `.dss` 只能证明 Snapshot 初始化，不能声明完整 Matlab 工作流等价。

## 第七轮：14 个 Volt-Watt Snapshot 入口

官方 `Examples/InverterModels/PVSystem/InvControl/volt-watt` 下 14 个单步 Snapshot 入口已从
`unsupported` 改为 `converted` 并发布 bundle。改造覆盖输出型 `CaseName`、`Maxi`/`MaxControli`
缩写、单 PV 的隐式 `DERList`、三点 Volt-Watt 曲线、有限正 `DeltaP_factor`、四种 Y 轴基准、
`%Pmpp` 与可用功率基准的区别，以及固定 PF 随每轮松弛后有功命令同步变化。

每个 bundle 都在独立临时目录完成自执行，并与 OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5
比较 3 个节点的复电压和 PV 终端复功率。14/14 通过 `1e-3` 门限；最大电压误差
`2.09e-5`，最大 PV 复功率误差 `3.49e-4`。Daily、多 DER、Storage，以及
`PFPriority`/`WattPriority`/kvar-kVA 限值组合仍需各自执行合同和数值门禁。

## 首诊断不能直接判定能力覆盖

### 当前重扫新增入口

修复坐标路径扫描及 `LoadShape Action=Normalize` 后，对 224 个未发布入口重新执行严格检查。
`Examples/Microgrid/GridFormingInverter/GFM_IEEE8500/Run_RecloserSiting.DSS` 新增为第 36 个
发布 bundle。它与已发布 IEEE8500 RecloserSiting 的主入口哈希相同，只多载入未被 Snapshot
引用的归一化 `pvshape`；两个 bundle 的 11,038 行结果中全部物理字段逐值一致，事件序号固定
相差 3。当时其余 223 个入口的首诊断已写入 `conversion_report.csv`；当前状态以上方统计和报告为准。

坐标路径修复后，`READ.IO.OPEN` 从历史 25 个降至 6 个；这 6 个均是官方快照实际缺少
`LoadShape1.CSV`、`solarramp.csv` 或 `Windoutput.txt`。`Action=Normalize` 现支持 P/Q 分别按
最大绝对值归一化，并要求它是该 LoadShape 的最后一次修改；其他 Action 仍明确拒绝。

### GFM IEEE123 Snapshot 分层结果

`Run_IEEE123Bus_GFMSnap-A.DSS` 中的默认 Daily 曲线绑定和未激活 GFM 的 InvControl 对单步
Snapshot 无电气作用。导入器只在可证明的单步边界省略这两条命令；修正低压 Delta 绕组 1 的
ANSI 相位方向后，386 条语句严格转换、自执行和 284 节点 OpenDSS 比较均通过，已作为第 37 个
bundle 发布。去除浮置 Delta 母线的任意公共模后，最大复电压相对误差为 `9.09e-7`。

同目录 B/C 没有被一并放行：B 断开 SW5 后的第二个 epoch 是无电压参考孤岛；C 又把 Storage
切为真正的 GFM 电压源。前者当前求解器拒绝该拓扑，后者仍缺少 Storage GFM 稳态合同。

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

### 11 个 `READ.DSS.VALUE_DEFAULTED`

当前剩余 11 个主要来自 InvControl 监测电压或 Volt-Var 示例。额定电压推断仍是导入缺口，
多数还使用当前有界子集不接受的 `DeltaQ_Factor=0.2`。`CaseName`、`Maxi` 和 `MaxControli`
已不再构成阻断；PSA 的 AVG/MIN/MAX、远端监测和 Volt-Var 测试仍不能单独证明整个入口闭环。

### 25 个 `READ.IO.OPEN`

- 历史 19 个由 `BusCoords` 引号、括号或行尾注释并入文件名导致的失败已经修复。重扫后这些
  入口继续暴露 Dynamic lifecycle、Storage `AmpLimit`、WindGen 对象、InvControl 或 BatchEdit
  等真实边界；其中 GFM IEEE8500 RecloserSiting 已完成发布。
- 6 个引用的 `LoadShape1.CSV`、`solarramp.csv` 或 `Windoutput.txt` 在相应官方案例目录中确实
  缺失，不能由转换器猜测或从同名文件静默替代。

### 其他明确能力缺口

- 当前 27 个以 `PSA.DSS.PROPERTY.SEMANTIC_UNREPRESENTABLE` 为首诊断的入口主要使用
  组合 VV_VW 的非 1 `deltaP`、非 1 `deltaQ`、超出有界曲线基准或成员关系。纯 Volt-Watt
  已支持有限的 `0 < DeltaP_factor <= 1`。
- 11 个 PVSystem 入口使用 `PFPriority` 等没有无损映射的属性。
- 9 个 Storage 入口使用 `%kWrated`、`EffCurve` 或 `DebugTrace`；前两项会改变电气结果。
- `LoadShape Action=Normalize` 已实现有界导入；其他 Action 以及 Normalize 后继续修改曲线仍没有
  可证明等价的执行合同。
- GFM、DynEq、VCCS、用户 DLL、Distance/TD21/59N Relay、AutoTrans 未支持接法和特殊节点
  编号仍属于明确边界。
- IEEE118 原始 53 Generator deck 虽有静态替代投影，但原生 generator 版本自身没有完成
  有界参考生成，不能列为“转换后应与 OpenDSS 一致”的确定案例。

## 建议的实现门禁

1. 为每个候选保存“原入口 SHA-256、转换模式、PSA 证据 case、首诊断、修复后诊断”关系，
   避免只维护诊断码白名单。
2. 首批 12 个入口已通过严格转换，11 个已发布 bundle；第二批 8 个候选中有 6 个完成
   V/I/P/Q/loss、控制终态或 FaultStudy 专用结果比较并发布 bundle。未发布的 2,900 步 Duty
   需要独立长任务预算；YYD 完整入口和 InductionMachine 则分别等待 Direct/Fault continuation
   与 Dynamic/保护动作执行合同，不能降低门禁后发布。
3. 每消除一个首诊断都重扫当前 208 个尚未发布入口，并对受影响文件继续探测第二层诊断；成功数量相同
   不代表改造没有改变失败原因。
4. 对 Plot、Show、Export、FileEdit、BusCoords 等副作用采用 manifest 记录；电气状态修改
   `Open/Close/Edit/BatchEdit` 必须进入执行计划，不能按输出副作用忽略。
5. `Mode=FaultStudy`、HarmonicT 快照和一个文件内多 study 应各自有独立 adapter，不应塞入
   普通 QSTS execution profile。

## 本次验证

- IEEE-TIA 三个完整分相入口均由未修改的官方文件生成 bundle，并复制到独立临时目录自执行；
  OpenDSS 数值比较覆盖全部节点、源端功率、三个绕组功率和全网复损耗。
- `cargo test -p psa --features dsa --lib dsa::parse::transformer_impedance::tests::`：10 项通过，
  包含单相 Delta 两线一次侧、次级半绕组互阻抗和既有多绕组矩阵边界。
- `cargo test -p psa --features dsa --test dss_import execution_plan_ignores_plot_between_solve_epochs`：
  1 项通过，锁定 `Plot`、`Show`、`Dump` 的只读报告副作用语义。
- `IEEETestCases/123Bus/SolarRamp.DSS` 从未修改的官方入口生成 387 条语句、4 个 epoch 的 bundle，
  在独立临时目录自执行通过；两个 300 点 Duty 段的电压、Generator/Storage、SOC、tap 和 Monitor
  已与 OpenDSSDirect.py 逐项比较。
- IEEE-TIA 两个单相入口分别以 21/47 条语句生成单 epoch bundle，并在独立临时目录自执行；
  OpenDSS 的 6/17 个节点全部匹配，变压器绕组功率和复损耗通过数值门禁。
- `IEEETestCases/8500-Node/Run_RecloserSiting.DSS` 使用含 MUMPS 的 release `tensoreval` 完成
  7,628 条语句的严格检查和 bundle 自执行；OpenDSS 的 8,531 个节点全部匹配，电压、源端功率和
  复损耗误差分别为 `0.6052%`、`0.387%` 和 `1.49%`。
- `cargo test -p psa --features dsa --lib
  dsa::output::power_flow::tests::single_phase_two_wire_winding_flow_reports_coil_voltage_and_current`：
  1 项通过，锁定单相两线变压器绕组的线圈电压和电流输出语义。
- 后续 6 个入口均由未修改的官方入口生成 bundle，并分别完成仓库内 bundle 自执行：
  IEEE34 Mod1/Mod2 各 2 个 epoch，IEEE123 Run/VoltageProfile 各 2 个 epoch，SolarRamp
  3 个 epoch、300 个 Duty 时刻，IEEE37 FaultStudy 175 个场景。
- IEEE34、IEEE123 和 SolarRamp 已逐工作点比较 OpenDSS 的电压、线路电流/功率、源端功率、
  loss、调压器终态或 Monitor；IEEE37 已比较全母线故障的去共模电压、故障电流和正负序阻抗。
- `cargo test -p psa --features dsa --test regcontrol_property_boundaries`：5 项通过，包含
  RegControl `Like` 后续属性回放和 `Clear` 历史隔离。
- `cargo test -p psa --features dsa --test short_circuit native_ieee37_faultstudy_alias_matches_opendss_all_buses`：
  1 项通过。
- `cargo test -p mems --features with_dsa,tensoreval-cli --bin tensoreval -- --nocapture`：
  49 项通过，包含 `Mode=F` 分类和 DSS→RSS 生成、自执行回归。
- `LoadShape.default`、Mode 时钟/控制队列重置和母线名大小写归一化的定向导入测试分别通过。
- IEEE37 官方输入、OpenDSS 锁定参考和 PSA 证据的 SHA-256 清单校验通过；所有新增 manifest
  通过 JSON 解析，根仓库和 `sparrowzz` 的 `git diff --check` 通过。
- 完整 `dss_import` 集成测试套件尚不能记为全套通过：既有测试
  `dynamic_profile_rejects_incomplete_or_non_gfl_pv_contract` 的拒绝预期与当前 PSA Dynamic
  导入行为不一致；其余长测试运行到 J1 三时刻案例时人工停止。本次涉及的定向测试均已单独通过。

以下是首批 12 个入口改造时保留的验证记录：

- `cargo test -p psa --features dsa --test dss_import ieee4_transformer_connection_matrix_matches_locked_native_references`：1 项通过。
- `cargo test -p psa --features dsa --test dss_import ieee8500_original_decks_lower_to_single_step_static_control_execution_profiles`：1 项通过，157.19 秒。
- `cargo test -p psa --features dsa --test short_circuit fault_study_matches_opendss_ieee`：IEEE13/34/123 共 3 项通过。
- `cargo test -p psa --features dsa --test dss_import p6_harmonic_t_materializes_loadshape_state_before_each_frequency_scan`：1 项通过。
- `cargo test -p psa --features mumps-static --test dss_post_solve_actions -- --nocapture`：
  9 项通过，包含 Newton 执行计划、流式结果和求解后动作。
- `cargo test -p mems --features with_dsa,tensoreval-cli --bin tensoreval -- --nocapture`：
  当时 48 项通过，包含 DSS→RSS 分类、生成、bundle 单次自执行、Redirect 后的数据资产读取和
  Newton 后端缺失时的明确拒绝。
