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
- 当前结果：182 个入口已经发布 bundle，88 个独立入口尚未发布；`conversion_report.csv`
  的 `status` 已同步当前发布状态。2,900 步 P174、8-epoch AutoHLT、8,760 点 Ckt7 与两快照
  GFMSnap-B 均已完成
  严格检查、官方路径 bundle 生成、搬移后独立自执行和锁定 OpenDSS 数值比较。
- 每个失败入口先记录全部诊断，不以首诊断推断后续能力；对路径和词法类失败，另在临时
  副本中只修正引用位置或行尾注释，再次执行以取得第二层诊断。
- 能力覆盖必须至少有 PSA 运行时 consumer 和测试；“参考覆盖”还要求锁定 OpenDSS 输出
  或官方入口的等价投影证据。只有存在同类 API，不算案例已覆盖。

## 优先接入的 12 个入口及定向复测

下面 12 个入口已有直接或同入口等价投影的数值证据。基线失败点位于转换编排、研究适配、
求解后端选择或严格导入门面，不需要先新增一种电气元件方程。2026-10-21 的定向复测中，
11 个入口当时完成 bundle 自执行并按官方相对路径发布；2,900 步 P174 入口完成导入和官方输入到
锁定数值投影的逐表等价检查。当时完整 `--check` 在 release 构建运行 90 分钟后仍未结束，
消除重复求解后再次运行 30 分钟仍未完成。这里的运行时限只用于控制扫描资源，不是发布标准。
后续完整 8,760 时点运行和 OpenDSS 年度轨迹比较均已完成，
`Examples/StoCtrl_Current_PeakShave/master.dss` 现已发布；其完整数值门禁见 README 和
`numerical_comparison.csv`。

2026-10-24 当前 release 复测时，P174 的两项导入/投影测试继续通过，但完整 `--check` 已稳定
在 epoch 0、`1 s` 的首次潮流求解失败。该入口因此仍不发布，并从性能长门禁转为执行求解回退
排查对象。

2026-11-03 当前 release 已恢复该入口：7,616 条语句、5 个 epoch 和 2,900 步 Duty 均执行完成，
径向 `line_model=Exact` bundle 搬移后独立自执行产生 32,054,352 行电压。全部 2,900 个时点的
9 个锁定参考节点电压最大相对误差为 `2.41171923822849e-5`，入口现已发布；首批 12 个入口至此
全部闭合。

| 入口 | 已有能力证据 | 定向复测结果 |
|---|---|---|
| `IEEETestCases/4Bus-OYOD-Bal/4Bus-OYOD-Bal.DSS` | 两个 OYOD 工作点均有官方入口到 PSA 投影的逐输出证据；IEEE4 连接矩阵参考测试通过 | 严格转换和两项独立 study 自执行通过，bundle 已发布 |
| `IEEETestCases/4Bus-YYD/YYD-Master-step1.DSS` | `yyd_step1` 的 V/I/P/Q/loss/control 锁定参考已存在 | Newton 自执行通过，bundle 已发布 |
| `IEEETestCases/IEEE 30 Bus/Run_IEEE30.DSS` | 官方入口的 Model 3 静态 Newton 投影与 OpenDSS 输出逐文件一致 | 静态 Newton 自执行通过，bundle 已发布 |
| `IEEETestCases/8500-Node/Run_8500Node.dss` | 原始 balanced deck 的 execution profile、控制终态和电气量已有测试 | Newton 自执行通过，bundle 已发布；自动 EnergyMeter 区域以显式 approximation Warning 保留 |
| `IEEETestCases/8500-Node/Run_8500Node_Unbal.dss` | 原始 unbalanced deck 的 execution profile、控制终态和电气量已有测试 | Newton 自执行通过，bundle 已发布；Monitor 标准位置参数已规范化 |
| `IEEETestCases/8500-Node/P174_Run_Voltage_Profile.DSS` | 两个有序 epoch 和 100%/40% 工作点已有数值参考 | 7,598 条语句、2 个 epoch 自执行通过，bundle 已发布 |
| `IEEETestCases/8500-Node/P174_Run_360kW_PV.DSS` | 2,900 步 Duty、曲线长度、调压器/电容状态及采样数值均有参考 | 7,616 条语句、5 个 epoch、2,900 步严格检查和搬移自执行通过；26,100 个锁定节点电压门禁通过，bundle 已发布 |
| `Examples/Microgrid/GridFormingInverter/GFM_IEEE8500/Run_8500Node_Unbal.dss` | 文件 SHA-256 与上面的官方 unbalanced 入口完全相同 | 与同内容 unbalanced 入口使用同一路径，自执行通过，bundle 已发布 |
| `IEEETestCases/ShortCircuitCases/IEEE123Master-SC.dss` | IEEE123 全母线 414 个故障参考测试通过 | FaultStudy adapter 自执行通过，bundle 已发布 |
| `IEEETestCases/ShortCircuitCases/ieee34Mod2_SC_Case_II.dss` | IEEE34 全母线 148 个故障参考测试通过 | FaultStudy adapter 自执行通过，bundle 已发布 |
| `Examples/HarmonicsTMode/IEEE_519.DSS` | HarmonicT 的时间×频率、LoadShape/Storage/Fault 状态物化已有 OpenDSS 参考 | `Open Line` 跨 Solve 快照物化后自执行通过，bundle 已发布 |
| `Examples/HarmonicsVariableLoad/IEEE_519.DSS` | 同一 HarmonicT 运行时已覆盖逐快照状态变化 | Load 属性跨 Solve 快照物化后自执行通过，bundle 已发布 |

这些入口的转换改造已经按 IEEE4 后端/多 study、IEEE30 静态 Newton 路由、FaultStudy
adapter、IEEE519 快照物化和 IEEE8500 严格门面的顺序完成。IEEE8500 的完整性验证仍需长时间
运行；尤其 2,900 步 Duty 不进入普通快速测试，但运行时间本身不决定是否发布。

## 后续 8 个候选入口的定向分析

2026-10-21 使用当前 `tensoreval` 重新执行这 8 个官方入口，并按建议顺序实现分类、导入与
执行语义。IEEE34、IEEE123 和 IEEE37 共 6 个入口现已使用未修改的官方源文件通过严格转换、
bundle 自执行和 OpenDSS 数值比较，并按官方相对路径发布。另两个入口仍有明确能力边界。

结论分成三层：

- IEEE34 Mod1/Mod2、三个 IEEE123 入口和 IEEE37 均已完成改造及发布。
- YYD 完整入口已按显式有序 study sequence 闭合；InductionMachine 入口仍有动态能力缺口，不能按转换器别名修复处理。

| 入口 | 当前原文件结果 | 最小探测及结果 | 判断 |
|---|---|---|---|
| `IEEETestCases/34Bus/Run_IEEE34Mod1.dss` | 通过；649 条语句，ExecutionPlan，2 个 epoch | 两个工作点的 V/I/P/Q/loss 已比较，电压最大误差 `9.06e-6`，6 组 tap 一致 | 已发布 |
| `IEEETestCases/34Bus/Run_IEEE34Mod2.dss` | 通过；581 条语句，ExecutionPlan，2 个 epoch | 已物化原生 `LoadShape.default`，并修复 `mid806`/`Mid806` 大小写解析；电压最大误差 `1.36e-5`，线路量最大误差 `5.83e-5`，6 组 tap 一致 | 已发布 |
| `IEEETestCases/123Bus/Run_IEEE123Bus.DSS` | 通过；771 条语句，ExecutionPlan，2 个 epoch | 有界回放 RegControl `Like` 后续 Edit 并按 Clear 隔离历史；电压最大误差 `1.37e-5`，7 组 tap 一致 | 已发布 |
| `IEEETestCases/123Bus/P174_Run_VoltageProfile.DSS` | 通过；379 条语句，ExecutionPlan，2 个 epoch | 100%/40% 工作点电压最大误差 `1.37e-5`，7 组 tap 一致 | 已发布 |
| `IEEETestCases/123Bus/P174_Run_SolarRamp.DSS` | 通过；380 条语句，3 个 epoch、300 点 Duty | `Mode` 重置时钟并恢复 Duty 的 Time 控制队列；12 次 tap 变化、最终档位、Generator 和 300 个 Monitor 样本一致 | 已发布 |
| `IEEETestCases/ShortCircuitCases/ieee37_SC_Currents.dss` | 通过；170 条语句，FaultStudy，175 个场景 | `F` 与 `FaultStudy` 统一规范化；全母线去共模电压误差 `1.61e-7`、故障电流缩放误差 `1.51e-4`、正负序阻抗误差 `8.46e-12` | 已发布 |
| `IEEETestCases/4Bus-YYD/YYD-Master.DSS` | 通过；`Snapshot → Direct` 两阶段 schema-v3 sequence | DirectFault continuation、原始 Solve 序号和两阶段 `bus_phase` 兼容聚合均已锁定 | 已发布；公开 `voltage` 仍为 36 行 |
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

- YYD 已定义 `Direct` 的结果表和 continuation 语义，并以 schema-v3 `Snapshot → Direct` 顺序
  保存。第一段和第二段 `bus_phase` 通过 manifest compatibility aggregate 继续发布为单一
  `voltage` 表；没有把 Direct 改名为 Snapshot 或 FaultStudy。
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

## 第八轮：6 个 Volt-Var Snapshot 入口

官方 `Examples/InverterModels/PVSystem/InvControl/volt-var` 下 6 个单 PV Snapshot 入口已从
`unsupported` 改为 `converted` 并发布 bundle。改造沿用纯 InvControl 的有界隐式成员规则：
仅当一个启用的纯 Volt-Var 控制器面对一个 PVSystem 且没有 Storage 时，省略的 `DERList`
才绑定该 PV。导入器保留原始五点曲线、`VARMAX`/`VARAVAL`、有限正 `DeltaQ_Factor`、
`%Pmpp`、kvar 限值与 P/Q 优先级。

定向数值比较同时暴露并修复了一个终态差异：DSS 未写容差时，控制配置此前没有保留
OpenDSS 默认 `VarChangeTolerance=0.025`，运行时会退回 PSA 通用默认 `1e-6`，使控制继续迭代
并越过 OpenDSS 停止点。现在三个 OpenDSS 默认值 `VoltageChangeTolerance=1e-4`、
`VarChangeTolerance=0.025`、`ActivePChangeTolerance=0.01` 都显式进入控制配置。

6/6 bundle 在搬移后独立自执行通过，并与 OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5
比较全部节点复电压和 PV 终端复功率。最大电压相对误差 `1.58135e-5`，最大 PV 复功率
相对误差 `2.15969e-4`，均通过 `1e-3` 门限。远端或相别监测、Daily、多 DER、Storage 和
组合模式不在本轮结论内。

## 第九轮：10 个 Daily InvControl 和 1 个本地监测入口

本轮先修正 PVSystem 的 OpenDSS 默认逆变器阈值：省略 `%CutIn/%CutOut` 时，执行型导入明确
写入 `0.20/0.20`，不再由 PSA 通用 IBR 默认值代替。Monitor 目标解析和 QSTS 采样也扩展到
PVSystem terminal 1 的 mode 0、1、3，使官方 Daily 文件中的电流、功率和状态 Monitor 能够
保留在执行计划中。

17 个 Daily 候选均使用未修改的官方入口完成严格转换和 24 时点比较，其中 10 个通过：3 个
Volt-Var、6 个 Volt-Watt 和 1 个 VV_VW。所有时点的节点身份一致；最大复电压相对误差
`6.68761e-5`，最大 PV 复功率/额定容量误差 `9.61056e-4`。`Local_voltage_min-2.dss` 也通过
单工作点门禁，两个误差为 `2.13958e-4` 和 `9.84486e-4`。这 11 个入口均已按官方路径发布。

第九轮结束时未发布的 13 个可转换入口保留真实门禁结果：4 个 Daily Volt-Var 的 average、rolling-average
或额定容量优先级终态不一致；3 个固定 PF Daily Volt-Watt 的 P/Q 联动不一致；3 个本地
监测入口的功率/额定容量误差略高于 `1e-3`；3 个远端监测入口还缺少完整 `MonBus` 相别、
线电压和多母线选择语义。另有 3 个 `Mon_voltage_MAX_Mix*` 入口引用官方发行树中缺失的
`PV_irrad.csv`，尚不能建立原始输入门禁。

## 第十轮：视在功率受限 Volt-Var 与固定 PF Volt-Watt

2026-10-23 对第九轮的 5 个控制终态失败入口完成闭合。两个 1010 kVA Volt-Var 案例在日间
触发视在功率圆限制；运行时现在保留该控制记忆，并在停机后的每个 QSTS 时刻先执行一次动作，
再按 OpenDSS 电压/无功双容差决定是否继续。三个固定 PF Volt-Watt 案例在逆变器停机时将
无功请求归零。5 个入口均完成 24/24 时点 OpenDSS 数值比较、独立 bundle 自执行并按官方路径发布。

本轮最大电压相对误差 `3.96518e-5`，最大 PV 复功率/额定容量误差 `5.28637e-4`；第九轮
已经发布的 10 个 Daily 入口全部复测通过。当前可严格转换但未通过数值门禁的入口降为 8 个：
2 个 Daily average/rolling-average、3 个本地监测电压和 3 个远端监测电压。

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
- StorageControllerTechNote 的 `EffCurve`、DebugTrace 和有界 `BatchEdit` 已进入实际执行合同。
  8 个入口现已全部完成 24 时点 OpenDSS 数值门禁并发布。第二十八轮补齐完整群组权重、
  迭代复测、活动死区、Support 反向潮流方向和饱和端口限值；PeakShave、PeakShaveLow 与
  Support 使用 `2e-2` 连续轨迹门限，并显式记录死区边界状态差异。
- InverterTechNote `kVA/11_1_kVAViolation.dss` 已在第二十九轮接入 Storage `%kWRated`、
  `%PMinNoVars/%PMinkvarMax`、切入前空闲态和 EffCurve DC 侧损耗分解；24 时点最大连续状态
  归一化误差 `8.49630e-4`，三相电压最大归一化误差 `1.40126e-5`，bundle 已发布。
- 8 个 MemoryMappingLoadShapes 案例在该轮审计时还涉及 MemoryMapping、Sng/Dbl 二进制或专用
  文本加载语义；它们现已在第二、第三批全部通过严格转换、原生数值和搬移 bundle 门禁。

所以当时这 27 个都是“路径层有 bug”，但不能仅凭路径修复声明能力完整覆盖；其中 8 个
MemoryMappingLoadShapes 完整入口现已另行完成发布门禁。

### 3 个 `READ.DSS.VALUE_DEFAULTED`

当前剩余 3 个都是 `Mon_voltage_MAX_Mix*` 入口。它们引用官方发行树中缺失的 `PV_irrad.csv`，
同时使用多 DER 混合监测；转换器不能补造辐照度序列后声明与 OpenDSS 一致。`CaseName`、
`Maxi` 和 `MaxControli` 已不再构成阻断。

### 25 个 `READ.IO.OPEN`

- 历史 19 个由 `BusCoords` 引号、括号或行尾注释并入文件名导致的失败已经修复。重扫后这些
  入口继续暴露 Dynamic lifecycle、Storage `AmpLimit`、WindGen 对象、InvControl 或 BatchEdit
  等真实边界；其中 GFM IEEE8500 RecloserSiting 已完成发布。
- 6 个引用的 `LoadShape1.CSV`、`solarramp.csv` 或 `Windoutput.txt` 在相应官方案例目录中确实
  缺失，不能由转换器猜测或从同名文件静默替代。

### 其他明确能力缺口

Distance/TD21 Relay 已不再是统一能力缺口。`Examples/DistanceRelays` 的 4 个官方 Dynamic
入口已完成固定时钟、故障生命周期、方向性、重合闸/锁定、9 组 Relay 终态和 Monitor V/I
门禁并发布 bundle。该结论只覆盖完整三相端口、整端开断和 V12 有界 RMS/phasor 合同；超出该
边界的 Distance/TD21 配置仍由导入器明确拒绝。

- 当前仅 1 个入口以 `PSA.DSS.PROPERTY.SEMANTIC_UNREPRESENTABLE` 为首诊断：
  `11_3_ReactivePowerViolation.dss` 的 Storage-target VV_DRC。单 PV 和已取证 multi-target/overlap
  PVSystem 合同已闭合；Storage member 仍等待统一状态运行时。
- PFPriority 的 4 个有界官方入口已在第二十三轮发布，不再属于当前未发布分组。
- `%kWrated`、`EffCurve` 和 `DebugTrace` 已被第二十四、二十九轮消费；当前剩余 Storage 首诊断集中在
  GFM `AmpLimit` 以及 `11_2` 的 Storage Volt-Watt 充/放电曲线和成员类型。
- `LoadShape Action=Normalize` 已实现有界导入；其他 Action 以及 Normalize 后继续修改曲线仍没有
  可证明等价的执行合同。
- GFM 的部分动态命令 lowering、DynamicExp 启动后网络事件、VCCS、用户 DLL、AutoTrans 未支持接法
  和特殊节点编号仍属于明确边界；59N Relay 已在第二十五轮发布，不再列为缺口。
- IEEE118 原始 53 Generator deck 虽有静态替代投影，但原生 generator 版本自身没有完成
  有界参考生成，不能列为“转换后应与 OpenDSS 一致”的确定案例。

## 第三十轮：115 个未发布入口的 DSA 候选分层

2026-10-29 使用 release `tensoreval`（SHA-256
`b9502e25cd89e4974142bcf34b718d0397e5ff4b99d24abc76105f41efd9b51a`）重新读取固定
`electricdss-tst` revision，并对 115 个未发布独立入口先执行“仅转换、不求解”探针。统一使用
`Version8/Distrib` 作为 acquisition root；每个入口限时 30 秒，115/115 得到确定结果，21 个生成
linked RSS，0 个超时。随后对其中 16 个非 Yearly、非 P174 入口执行 30 秒完整 `--check`，16 个均
返回明确运行层诊断；另对 ckt7 和 P174 做 60 秒观察，两者都进入长执行且没有即时错误输出。

这轮的目的不是增加发布数，而是把“parser/emitter 已经能表达”与“DSA runtime 能完成且数值一致”
分开。没有 OpenDSS 数值门禁的入口继续保持 `unsupported`。

| 候选组 | 入口数 | 当前最深诊断与已有原语 | 判断与顺序 |
|---|---:|---|---|
| Dynamic 原生命令 lowering | 7 | `Dynamic_KundurDynExp.dss` 已到启动后的 `New Fault`/`Disable Fault`/`Open Line`；5 个 IEEE123 GFL/GFM 和 1 个 Wind GFL 先停在 Dynamic 对 `BatchEdit Load..* Daily=default` 的命令拒绝。运行时已有 typed fault on/off、topology open/close 和 fixed-clock continuation | **第一优先**。先复用 execution plan 的有界 BatchEdit 展开，再把启动后的 Fault/开关命令映射为 typed dynamic event；每消除一层都重跑，不能预先宣称 GFM/Wind 对象已闭合 |
| AutoHLT Snapshot fault continuation | 1 | 第三十轮时 `AutoHLT.dss` 已生成 8 个 epoch，但第一个三相短路工作点在 `2 s` 返回 fault-study protection measurement 求解失败 | **已闭合**。第四十轮已锁定 fault current、winding I/P/loss/voltage 和非故障母线电压，bundle 已发布；见下文 |
| NEV 多 study bundle | 1 | 主模型是已支持的无源网络；入口按 Snapshot→FaultStudy→Harmonic 顺序执行，当前只由 `MIXED_STUDY_UNSUPPORTED` 阻断 | **第三优先**。三个 adapter 已独立存在；需要结果命名、study 顺序和状态 continuation 合同，不能把 mode 互相别名化 |
| P174 2,900 步 Duty | 1 | 当前仅转换通过；60 秒内未重现旧版 epoch 0 即时失败，也未完成。已有完整 2,900 步 locked projection | **独立长门禁**。先固定当前 release 哈希跑完整首 Snapshot 和 2,900 步，再决定是求解回退、性能问题还是已经恢复；观察超时不算失败或通过 |
| Yearly ckt24/ckt7 | 4 | 两个 ckt24 AnnualSimulation、Scripts AnnualSimulation 和 ckt7 已能生成 RSS；ckt7 60 秒内持续执行 | **独立长门禁**。需要 8,760 时点收敛、状态和轨迹门禁；不适合普通快速复扫 |
| Storage-target InvControl | 2 | `11_2_kWRatedViolation.dss` 缺 `voltWattCH_curve` 与 Storage Volt-Watt member；`11_3_ReactivePowerViolation.dss` 缺 Storage-target VV_DRC。已有 P2-E-057/064 singleton/overlap 原生证据 | **高可信但大改造**。必须统一 Storage 与 IBR 的控制动作、idle loss、SOC、dispatch state 和区间推进；禁止只放宽 DERList 校验 |

### 本轮排除的生成期假阳性

- 4 个 VCCS Dynamic 入口虽然能生成 RSS，完整检查仍返回 `PSA.DSS.OBJECT.UNSUPPORTED`；需要新增
  VCCS 电气/动态模型，不是 importer 接线工作。
- `Examples/ExpControl/SolveAndExportMonitors.dss` 只含 Solve/Export，必须由 `Master.dss` 在同一
  OpenDSS 进程中先构网；把它当独立入口会得到无效拓扑。Paulo 的 `solve_snapshot.dss` 和
  `solve_daily.dss` 同样是父脚本片段。
- `Run_IEEE123Bus_GFMSnap-B.DSS` 的无电压参考被动孤岛已经闭合并发布；显式打开 SW5 后 32 个
  母线相及设备终端输出为精确零，主动 GFM 孤岛仍 fail closed。
- GIC、NCIM、VCCS、WindGen QSTS、Storage GFM/AmpLimit 等真实对象或方程缺口没有因为生成 RSS
  的数量变化而自动晋级为近期候选。

### 下一轮验证门禁

1. Dynamic lowering 必须先用 importer 单元测试证明 BatchEdit 在 Dynamic 启动前物化，且 post-start
   Fault/Open/Disable 被保留为有时标、稳定顺序的 typed event；再运行至少 Kundur 与一个 GFL 入口。
2. AutoHLT 必须逐 epoch 比较三相短路电流、复损耗和满载端电压；所有 OpenDSS 对标参数显式使用
   `line_model="exact"`。
3. NEV 必须为每个 study 使用独立结果表主键和 manifest 记录，不能用最后一个 study 的成功覆盖前面
   study 的失败。
4. P174/Yearly 使用单独长任务预算，并记录 release SHA、完成步数、首次失败时刻和收敛状态；不能以
   批量扫描超时更新 `conversion_report.csv`。
5. Storage-target InvControl 在 runtime 统一前继续保持现有证据化 fail closed；任何实现必须消费既有
   P2-E-057/064 oracle，而不是删除拒绝测试。

## 第三十一轮：Dynamic lowering 实施与再分层

第三十轮第一优先项已经在 `eig-rs` master 的 `aaef84412`、`282b1ddeb`、`203bba69f` 落地。
实现只接入已有可验证原语：启动前有界 `BatchEdit` 物化，启动后的 Fault on/off 和整元件 topology
open/close typed event，以及 `StepTime` 到 `StepSize` 的固定步长别名。V12 集成测试显式使用
`line_model=Exact`，验证事件顺序和累计时刻；运行时任意属性编辑、选相开断、非零 clock reset 与
未采样的动态曲线继续明确拒绝。

固定 `electricdss-tst@3b208397160213cae4a9e2d0a7d1aa3528ce26e1`，使用 release
`tensoreval`（SHA-256
`2470c55251fb109a176b917521cd3b1eb85f5690e6b60fc4b43d25863802a089`）对 7 个候选重新执行完整
`--check`，结果如下：

| 候选 | 修复前首阻断 | 修复后首阻断 | 后续判断 |
|---|---|---|---|
| Kundur DynamicExp | post-start Fault/Line mutation | `PSA.DSS.GENERATOR.DYNAMIC_EXTERNAL_MODEL_UNSUPPORTED` | 已证明命令层闭合；必须新增或接入外部 Generator dynamic/shaft 模型后才能继续 |
| IEEE123 GFL、GFL DynExp | prestart BatchEdit | `PSA.DSS.DYNAMIC.SET_UNSUPPORTED`，对应首个未映射 `LoadShapeClass=Daily` | `Time=(10,0)` 与 LoadShape 采样共同构成新的有界合同，不能把设置当作无副作用 |
| IEEE123 GFM、CannotPickUpLoad、AmpLimit | prestart BatchEdit | `PSA.DSS.DYNAMIC.COMMAND_UNSUPPORTED`，对应 Snapshot 后的 `Open Line.SW5 terminal=1` | 先定义 Snapshot→Dynamic 拓扑 continuation；Storage GFM 与 AmpLimit 仍是后续真实方程边界 |
| Wind GFL Dynamic | prestart BatchEdit | `PSA.DSS.DYNAMIC.PROFILE_CONSUMER_UNSUPPORTED` | Fault 生命周期已被 typed event 消费；下一缺口是 `LoadShape.default` 的动态采样输入 |

聚焦 Dynamic importer 测试 12/12 通过。完整 DSA 库测试为 1,359 通过、5 个既有失败、2 个忽略；
失败集中在 Generator 动态参数/连接和 Harmonic Monitor state，单独复跑仍失败，不经过本轮 importer
改动路径。由于 7 个官方入口仍全部 fail closed，未执行 OpenDSS 数值比较，也不改变发布状态：
`conversion_report.csv` 保持 144 个 `converted`、115 个 `unsupported`。

### 下一轮更新后的优先级

1. 对 GFL/Wind 先定义 `LoadShapeClass=Daily`、`LoadShape.default` 与 Dynamic 初始时钟的采样合同；
   必须用官方 10 h 初值和故障前后 V/I/P/Q 轨迹取证，不能只让命令通过。
2. 对 GFM 把初始 Snapshot、`Open Line.SW5`、第二次 Snapshot 和 Dynamic 启动建成显式 continuation；
   之后再分别处理 Storage GFM 电压源和 `AmpLimit`，避免把拓扑与设备方程混为一个诊断。
3. Kundur 只有在 Generator dynamic/shaft 外部模型有可执行替代或原生 adapter 后再晋级；命令 lowering
   已不再是该入口的首要缺口。
4. 任一入口只有在 portable bundle 自执行和 OpenDSS 数值门禁均通过后，才更新
   `conversion_report.csv` 与 144/115 计数。

## 第三十二轮：Dynamic Daily 启动与绝对时钟

`eig-rs` master 的 `afa322e4c`、`ef8d75811`、`7a93cfee8` 已完成第三十一轮第一优先项的有界
实现：固定间隔 STEP `Load.daily` 只在整个 Dynamic 窗口常量时物化 P/Q，`LoadShapeClass=Daily`
控制是否激活曲线，`Time=(h,s)` 保留为绝对起点，后续 typed event 继续使用绝对时标。超出范围的
Yearly/Duty、PVSystem Daily、运行中曲线转换及非 Load consumer 均有稳定拒绝诊断。

OpenDSSDirect 锁定证据为：10 h 的官方 Daily 样本倍率是 `0.88`，未激活
`LoadShapeClass=Daily` 时同一 `daily` 引用保持 unity。PSA 的数值对标集成测试使用
`line_model=Exact`，同时断言 0.88 启动种子及 `36000.2 s` 的故障事件。大绝对时刻的固定步长
检查改为与投影网格比较，避免 f64 消减把均匀 1 ms 网格误报为非均匀。

固定 `electricdss-tst@3b208397160213cae4a9e2d0a7d1aa3528ce26e1`，使用 release
`tensoreval`（SHA-256
`90874a816b7865852e0cb40df3133c2da3e07cdc93008bf5b8dddfadfbc0402b`）重新完整检查 7 个候选：

| 候选 | 第三十一轮阻断 | 第三十二轮最深阻断 | 判断 |
|---|---|---|---|
| Kundur DynamicExp | Generator 外部动态模型 | `PSA.DSS.GENERATOR.DYNAMIC_EXTERNAL_MODEL_UNSUPPORTED` | 不受本轮 Load Daily 合同影响 |
| IEEE123 GFL Daily | `LoadShapeClass=Daily` | `PSA.DSS.PVSYSTEM.DYNAMIC_VALUE_INVALID`，`safevoltage=0` | 已证明 Daily/10 h/Load 启动闭合；下一步需锁定零 SafeVoltage 的原生语义 |
| IEEE123 GFL Daily DynExp | `LoadShapeClass=Daily` | `PSA.DSS.PVSYSTEM.PROPERTY_UNSUPPORTED` | 已到官方 `DynamicEq`/`DynOut` 用户动态表达式边界，不能由普通 GFL adapter 代替 |
| IEEE123 GFM、CannotPickUpLoad、AmpLimit | `Open Line.SW5 terminal=1` | `PSA.DSS.DYNAMIC.COMMAND_UNSUPPORTED` | topology continuation 仍是首阻断 |
| Wind GFL Dynamic | Load Daily consumer | `PSA.DSS.OBJECT.UNSUPPORTED` | 已越过负荷曲线，首阻断推进到 `WindGen.wtg1` 对象/方程 |

所有探针退出码均为 1，没有候选达到 portable execution，更没有进入 OpenDSS 数值门禁。聚焦测试
为 20/20、6/6、PSA 集成 4/4、MEMS 门面 1/1；完整 DSA 库为 1,375 通过、5 个既有
Generator/Harmonic 失败、2 个忽略，失败集合未变化。因此 `conversion_report.csv` 仍保持
144 个 `converted`、115 个 `unsupported`。

### 下一轮更新后的优先级

1. 先用 OpenDSSDirect 锁定 `SafeVoltage=0` 在原生 PV 动态中的禁用/阈值语义；若能有界表达，再推进
   非 DynExp GFL，并要求 portable execution 与故障前后 V/I/P/Q 数值门禁。
2. `DynamicEq`/`DynOut`、WindGen 和 Generator 外部模型是独立真实方程边界，不因 Daily 启动完成而
   降低门禁；分别评估实现成本与可取证参考后再排序。
3. 三个 GFM 入口仍先处理 Snapshot→Dynamic topology continuation，再暴露 Storage GFM/AmpLimit，
   不把拓扑闭合与设备方程混成同一改动。
4. 只有 portable bundle 自执行和 OpenDSS 数值门禁均通过，才修改发布清单与 144/115 计数。

## 第三十三轮：Native `SafeVoltage=0` 与下一层动态合同

固定上游的两个 IEEE123 GFL Daily 入口都为 `Storage.mystorage` 和 `PVSystem.myPV` 显式设置
`SafeVoltage=0`。OpenDSSDirect 的 Native 属性回读保持零值；结合官方动态属性的百分数阈值语义，
这不是缺失值或禁用设备的哨兵，而是合法下边界。`eig-rs` master 的 `079a77dc2`、`bfc608a8c`
因此只做有界扩展：PVSystem/Storage 动态设备规格接受有限 `[0,1]`，DSS importer 接受
`[0,100]` 并按百分数精确缩放。负数、超过上界、NaN 和正负无穷继续返回原有设备级诊断；
共享逆变器提交内核对齐 Native 的 `MinVS=0` 低压停机旁路，并由正电压暂降与零电压测试锁定
零最小电压不会错误离线。

固定 `electricdss-tst@3b208397160213cae4a9e2d0a7d1aa3528ce26e1`，使用 release
`tensoreval`（SHA-256
`3a733f94e50c5dcbed6854cf3cd2638e99753bf1e39a60eb20cf9e8127fa3007`）重新检查非 DynExp
`Run_IEEE123Bus_GFLDaily.DSS`。退出码为 1，但最深诊断已从
`PSA.DSS.PVSYSTEM.DYNAMIC_VALUE_INVALID: safevoltage=0` 推进为
`PSA.DSS.PVSYSTEM.DYNAMIC_CONTROL_MODE_UNSUPPORTED: dynamic PVSystem requires explicit
ControlMode=GFL`。官方 PVSystem 对象没有显式 `ControlMode`，同一脚本却有
`InvControl.StoCtrl ... mode=GFM`；下一轮必须先用 Native 运行和属性/状态轨迹确认默认控制模式、
InvControl 作用域及 PV/Storage 各自的实际模式，再决定是否能安全补全 importer 合同。

本轮聚焦门禁全部通过：PV 4/4、Storage 3/3、共享逆变器 5/5、SafeVoltage importer 2/2、
动态快照初始化 4/4、MEMS 表达式门面 1/1。完整 `dss_import` 集成为 454 通过、11 个既有失败；
完整 DSA 库为 1,379 通过、5 个既有 Generator/Harmonic 失败、2 个忽略，失败集合不经过本轮路径。
入口尚未生成可自执行 portable bundle，也未进入显式 `line_model=Exact` 的 OpenDSS V/I/P/Q
数值门禁，因此 `conversion_report.csv` 保持 144 个 `converted`、115 个 `unsupported`。

## 第三十四轮：Native 默认 GFL、惰性 GFM monitor 与 PV 曲线边界

锁定的 DSS C-API 0.14.5 源码将 PVSystem/Storage `ControlMode` 表示为
`GFL=false`、`GFM=true`，构造默认值为 `false`。OpenDSSDirect 最小探针在创建
PVSystem/Storage 后均回读 `GFL`；创建
`InvControl.ctrl DERList=[PVSystem.pv Storage.s] mode=GFM` 后，控制器回读 `GFM`，两个成员仍为
`GFL`。源码中 GFM controller 分支还要求成员自身 `GFM_Mode` 为真。这三项证据共同证明：官方
入口中省略的设备模式使用 Native 默认 GFL，`InvControl ... mode=GFM` 不会隐式改变成员模式，
在所有成员保持 GFL 时没有电气动作。

`eig-rs` master 的 `b14678959` 只为省略的 PVSystem/Storage `ControlMode` 提供 GFL 默认；显式
GFM/非法值继续 fail closed。`ecfa1c75f` 将事件序列上的惰性证明复用于 Dynamic execution，
保留 Warning `PSA.DSS.EXECUTION.INERT_GFM_MONITOR_IGNORED`。证明要求成员完整解析为已声明的
PVSystem/Storage，控制器不可变，成员没有显式 `ControlMode=GFM`，且声明只含
`DERList`、`mode`、`enabled`；控制器变更、成员转 GFM、未知成员或 `eventlog=yes` 都不省略。

固定 `electricdss-tst@3b208397160213cae4a9e2d0a7d1aa3528ce26e1`，使用 release
`tensoreval`（SHA-256
`016d399c82c93dd9778a9bd733fd64385421b0b136b0016b91b631098893cc95`）重新检查非 DynExp
`Run_IEEE123Bus_GFLDaily.DSS`。退出码为 1；最深诊断从设备
`PSA.DSS.PVSYSTEM.DYNAMIC_CONTROL_MODE_UNSUPPORTED` 推进为
`PSA.DSS.DYNAMIC.PROFILE_CONSUMER_UNSUPPORTED`，具体边界是 PVSystem 的 `pvshape` LoadShape
引用在当前 native dynamic runtime 没有采样输入。该入口因此已越过本轮默认值和控制器编排层，
下一缺口是 PV 动态 profile consumer，而不是 GFM 设备方程。

聚焦门禁全部通过：默认 GFL 1/1、惰性 GFM 3/3、动态快照初始化 4/4、MEMS 表达式门面 1/1。
完整 `dss_import` 为 459 通过、10 个既有失败；完整 DSA 库为 1,379 通过、5 个既有
Generator/Harmonic 失败、2 个忽略。本轮没有 portable bundle，也未进入显式
`line_model=Exact` 的故障窗 V/I/P/Q 及 PVSystem/Storage 状态比较，故
`conversion_report.csv` 仍为 144 个 `converted`、115 个 `unsupported`。

## 第三十五轮：Snapshot 初始化后的 PV Daily 为惰性 provenance

OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5 的隔离双路径参考锁定了同一 PV Daily 声明的两种
行为：10:00 的 Daily 单步在 `36001 s` 得到 `IrradianceNow=0.5`、400 kW；Snapshot 初始化后
切换 Dynamics 的 1 ms 步在 `36000.001 s` 仍为 `IrradianceNow=1.0`、800 kW。两条路径均
收敛。由此可知，官方入口中的 Daily 引用在当前 Snapshot→Dynamics 生命周期只保留声明来源，
不是动态运行时采样输入。

`45a430a24` 只对已执行 Snapshot、活动 `LoadShapeClass=Daily`、固定间隔且非 UseActual 的轮廓，
以及已准入、无 TShape/P-T/EffCurve 依赖的完整 GFL PV 发出
`PSA.DSS.DYNAMIC.PVSYSTEM_PROFILE_INERT` Warning；静态 800 kW 工作点不变。所有未证明变体
继续返回原诊断。`884a088de` 加入独立 generator、两个 DSS 输入、两行 Native CSV 和 manifest
消费测试；生成器/公共输入/动态输入/CSV SHA-256 依次为
`30d8ae3fb92ff739ec1957dc53869af7862b0f4a8b6e4a8d8b962ce1208ea993`、
`ce66b96159ea2d4fb676a85c7c195ba14e0e7f6de2f2bc1b922b39108e14616e`、
`52ab404066665c96f6fae1722f484e19da0ace5d38bdd39dd839354a7385eb9b`、
`8f35b46755f6a2ce7c7d00933c1c788d4e4fa48cd1d4566fb02c29335addbba6`。manifest 计数由
729 增至 730，且 generator `--check`、40 项 manifest 测试均通过。

收尾代码审阅指出仅记录“曾经 Snapshot”会误准入 Snapshot 后才声明/编辑的 PV，或 Clear 后重建
的网络。`ac06e0a68` 将该状态改为可失效证明：后续结构/电气声明变更会清除它，新的 Snapshot
可重新建立；官方已有明确 importer 合同的 `Edit Storage state=Discharging` 初始状态动作不清除
PV 证明。新增三类负测锁定 PV 后声明、后编辑和 Clear/rebuild 均返回原 profile consumer 诊断。

固定 `electricdss-tst@3b208397160213cae4a9e2d0a7d1aa3528ce26e1`，使用 release
`tensoreval`（SHA-256
`7447540d8dca0a57998e2f3653578ebee20dab91d186f2fe87692f426a60c7f3`）重新检查
`Run_IEEE123Bus_GFLDaily.DSS`。退出码为 1；PV profile consumer 已越过，新的首个阻断诊断是
`PSA.DSS.OBJECT.UNSUPPORTED`，target 为 `EnergyMeter.Feeder`。完整诊断还包含 6 个后续 Monitor
边界：两个 mode 1 功率 Monitor，以及 PVSystem/Storage 各自的 mode 0 电气 Monitor 和 mode 3
状态 Monitor，当前均没有对应的 V12 动态结果投影。本轮不扩大设计去处理这些独立对象合同。

聚焦测试为 Dynamic execution 23/23、Native 参考 1/1、manifest 40/40、Snapshot 初始化 4/4、
MEMS 门面 1/1；宽 `dynamic_profile_` 过滤为 26/28，失败仍是两个既有 Relay 用例。完整
`dss_import` 为 460/470，10 个既有失败；完整 DSA 库为 1,382 通过、5 个既有
Generator/Harmonic 失败、2 个忽略（共 1,389）。入口未通过 portable 自执行，也没有执行要求 `line_model=Exact` 的
OpenDSS 数值比较，因此 `conversion_report.csv` 保持 144 个 `converted`、115 个 `unsupported`。

### 下一轮更新后的优先级

1. 先判定 `EnergyMeter.Feeder` 在 Dynamic 转换中是可审计的惰性声明，还是必须连续维护寄存器的
   运行时对象；只有 Native 生命周期与当前 V12 输出合同证明等价时才允许省略或投影。
2. 分别为 PVSystem/Storage 锁定 mode 0/1/3 Monitor 的公开通道、采样时刻和终端方向；不得用
   “Export 是副作用”推导 Monitor 声明无语义。
3. 上述对象边界闭合并生成 portable bundle 后，以 `line_model=Exact` 比较故障窗 V/I/P/Q 与
   PVSystem/Storage 状态；两层发布门禁都通过前不改变 144/115 计数。

## 第三十六轮：Dynamic EnergyMeter 惰性证明与 Monitor 边界

固定官方 IEEE123 GFL Daily 的有表/无表双路径使用 OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5。
在 Snapshot、200 ms 故障前、70 ms 故障中和 200 ms 清除后检查点，全部母线复电压、损耗、
PVSystem/Storage 电流、功率和公开内部状态的最大绝对差均为 0；EnergyMeter 在 470 个自动
Dynamic 步内没有采样。独立五路径锁定参考给出相同结论，并证明显式 `Meters.Sample()` 会把
`kWh/Max kW` 从 `0/-1e50` 改为 `2.500150565056e-4/900.0542034202`，Daily 则自动累计
`900.0531994212 kWh`。因此“自动 Dynamic 中惰性”不能外推为显式 meter 请求也可省略。

`383eddb13` 实现有界准入：只有有效且可失效的 Snapshot 初始化证明存在时，EnergyMeter 的
标准边界才作为 provenance 保留并发出 `PSA.DSS.DYNAMIC.ENERGY_METER_INERT`。声明属性仅限
`element/terminal/basefreq/enabled`；运行期变更、Action、ZoneList、Sample、AllocateLoads、
meter export、保存和复位请求全部拒绝。代码复用既有 QSTS 边界解析，未增加 meter runtime、
寄存器或结果 schema。最终审阅提交 `3647d118f` 增加 quoted 和 parser-variable meter export 的
RED→GREEN 回归，并把导出/持久化选择器切换到 tokenization 与变量替换后的参数，防止通用 Export
省略隐藏显式 meter 请求。`47c0cfc37` 加入五行 Native reference；四个 SHA-256 为
`80d4f4b1e109716e16c0e011d929a9019c095667f4ef94ee34420b29c131d7a8`、
`cb4bee3633e5a859c504a338ed9932d6c73ffd1b837b7f0760e9a57baad6adcd`、
`21f96b6f18d9359df6413fcdf33a311ec7e7f35e50600c7688c2496516e9f9b4`、
`b2f211ae7cb3ee14e322e6578ea41e32a402b6be0d63ea2c58b29bf1f5d019a1`；manifest 计数为
731/731，`--check` 和 40 项 manifest 测试通过。

release `tensoreval` SHA-256 为
`b6367a867cdc7789839c94948673fbf84f31e3b015754842f19320c591d24e11`。固定官方入口完整检查退出码
为 1，最深诊断推进到 `PSA.DSS.DYNAMIC.MONITOR_MODE_UNSUPPORTED`；按源文件声明顺序，首个
对象是 mode 1 的 `Monitor.pvmonitor`，后续是 `Monitor.pvvi`/`pvstatevars` 和对应 Storage
mode 1/0/3。下一轮应分别锁定 mode 1 功率通道、mode 0 V/I 通道和 mode 3 状态通道的采样时刻、
极性与 V12 投影，不能把 Monitor 声明等同于末尾的 Export 副作用。

验证计数：Dynamic execution 27/27、Native reference consumer 1/1、Snapshot 初始化 4/4、
MEMS façade 1/1、manifest 40/40；完整 `dss_import` 461 通过、10 个既有失败；完整 DSA lib
1,386 通过、5 个既有 Generator/Harmonic 失败、2 个忽略。由于 portable execution 和显式
`line_model=Exact` 的 Native 故障窗数值门禁都未通过，`conversion_report.csv`、bundle 树和
144 converted / 115 unsupported 均保持不变。

本轮完整基线的失败集合逐项为：`dss_import` 的
`dss_autotrans_series_tap_regcontrol_reaches_qsts_runtime`、
`execution_profile_fails_closed_for_unsupported_or_incomplete_time_semantics`、
`execution_profile_rejects_spurious_one_shot_reclose_interval`、
`generator_dynamic_import_rejects_unsupported_runtime_boundaries`、
`invcontrol_voltvar_disjoint_controllers_execute_independently`、
`invcontrol_voltvar_member_order_does_not_change_per_device_capability`、
`protected_dynamic_doc_multishot_rejects_invalid_reclose_sequences`、
`protected_dynamic_profile_admits_inert_voltage_relay_without_curves`、
`protected_dynamic_profile_rejects_doc_reclose_intervals`、
`volt_var_keeps_pvsystem_model_one_above_vmaxpu`；DSA lib 的
`generator_invalid_specs_devices_and_samples_fail_closed`、
`generator_network_rejects_noncanonical_phases_and_wrong_connection`、
`generator_parameter_bases_match_hand_calculation`、
`prepared_dynamic_v10_generator_state_schema_order_and_deenergized_zeros`、
`harmonic_monitor_state_modes_reuse_fundamental_state_and_match_opendss`。它们与上一轮集合相同，
新增 EnergyMeter 单元与消费测试均不在失败路径中。

### 下一轮更新后的优先级

1. 先用 Native reference 分别锁定 PVSystem/Storage mode 0、1、3 的列名、单位、方向和每步采样
   顺序，再决定复用 V12 结果表还是新增有版本的投影；不得只放宽 importer。
2. Monitor 声明闭合后必须先生成并搬移执行 portable bundle，再以 `line_model=Exact` 比较官方
   故障前/中/后的 V/I/P/Q 及 PVSystem/Storage 状态。
3. 两个发布门禁完成前继续保持 144/115；EnergyMeter 的显式采样、寄存器导出和 QSTS 累计仍是
   独立运行时合同，不属于本轮惰性准入。

## 第三十七轮：Dynamic V13 Monitor 闭合与 GFL IEEE123 发布

PVSystem/Storage mode 0、1、3 的 Native 通道、单位、终端方向和每步采样顺序已由
OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5 锁定。V13 投影保持 470 个 1 ms 动态时点，覆盖
200 ms 故障前、70 ms 故障中和 200 ms 清除后；6 个 Monitor 共 31,960 行。未建模 mode 3
状态统一标记 `validity=unmodeled` 并输出 `NaN`，PVSystem/Storage 的 Native `it` 不再由端口
电流幅值替代。

固定官方入口已通过 406 条 DSS 语句的严格检查，生成 schema v2 portable bundle，并从原目录
搬移到独立临时目录自执行，得到 16 张结果表。数值比较显式使用 `line_model=Exact`：mode 0
V/I 和 mode 1 P/Q 的最大额定值缩放误差为 `4.05983e-2`，低于 `15%` 门限；mode 3 已建模状态
最大缩放误差为 `3.46374e-2`，低于 `20%` 门限；Storage `kWh` 最大缩放误差为
`2.05525e-7`。逐点原生参考、比较摘要、版本和 release SHA-256 均随 bundle 发布并由 PSA 测试消费。

两个发布门禁均已闭合，`conversion_report.csv` 已把该入口改为 `converted`。发布计数由
144/115 更新为 145/114；剩余 GFL DynExp 和 GFM 入口仍需各自的 DynamicEq/GFM 运行时合同，
不能由本次普通 GFL V13 结果外推。

## 2026-11-02 Stevenson Generator Model 3 Snapshot 闭合

未修改的官方 `Examples/Stevenson/StevensonPflow-3ph.dss` 使用 Generator Model 3 和
`PVFactor=0.5`。本轮只接受单个全新 Snapshot epoch，并要求 `line_model=Exact` 与 Newton；
`PVFactor` 作为 provenance 保留，不扩展为 time-series 控制状态，也不外推到未经审计的取值。
OpenDSS reference `stevenson_model3_snapshot` 导出全部 15 个节点电压、58 行端子电流和 58 行
端子功率；consumer 比较 42 行非接地 Line/Vsource/Generator 电流与 42 行对应功率，并比较
Generator 聚合功率、11 行元件损耗和全网复损耗；最大归一化复数误差分别为
`4.7449e-5`、`5.0376e-4`、`5.2226e-4`、`2.6188e-4`、`9.8037e-4` 和
`9.6221e-5`，均低于 `1e-3`。

schema v2 bundle 只暴露 `bus_phase` 电压结果，详细 I/P/Q/loss 由 PSA reference test 保留。
使用现成 release `tensoreval`（SHA-256
`ddda420b926d0e080127bcc93efc462fb64b50cab1b9e3df08df4ad02a914198`）将完整 bundle 搬移到独立
临时目录后自执行成功，输出 15 行、11 列，且 closure 中没有来源树绝对路径。报告状态已由
`unsupported` 改为 `converted`，当前计数由 145/114 更新为 146/113。ADiakoptics IEEE30、
time-series Model 3、未经审计的 PVFactor 和 multi-epoch continuation 继续保持 unsupported。

## 2026-11-03 IEEE8500 P174 360 kW PV Duty 闭合

未修改的官方 `IEEETestCases/8500-Node/P174_Run_360kW_PV.DSS` 已使用现成 release
`tensoreval`（SHA-256 `a4faa17d7990dec8bd5ebcf7d20a9dcbc37a44b1d6de4f9aa1207195b023801b`）
完成 7,616 条语句、5 个 solve epoch 的严格检查。径向 execution plan 使用
`line_model=Exact`，第 5 个 epoch 完整执行 2,900 个 1 秒 Duty 时点。

schema v2 bundle 按官方相对路径生成，搬移到独立临时目录后自执行成功，输出 32,054,352 行、
11 列的 `voltage.csv`，且 closure 不含来源树绝对路径。锁定 OpenDSS `p57_duty/trajectory.csv`
中的 2,900 时点 × 9 节点电压全部匹配：最大相对误差 `2.41171923822849e-5`、RMS 相对误差
`4.03956012716511e-6`、最大绝对误差 `0.178942628966979 V`，低于 `2%` 门限。
`conversion_report.csv` 已由 `unsupported` 改为 `converted`，当前计数由 146/113 更新为 147/112。

## 2026-11-03 AutoHLT 8-epoch 三绕组变压器闭合

官方 `Examples/AutoTrans/AutoHLT.dss` 已使用现成 release `tensoreval`（SHA-256
`bd81fbc8a0634f56e845327d67cd428e002fa172925b6ecf6c5e48bfd307b5d3`）完成 46 条语句、8 个 solve
epoch 的严格检查。schema v2 bundle 按官方相对路径生成，搬移到独立临时目录后自执行成功，输出
96 行、11 列的 `voltage.csv`，包含全部 8 个 epoch，且 closure 不含来源树绝对路径。

数值门禁显式使用 `line_model=exact`。5 个非故障 epoch 的 60 个母线相复电压全部匹配锁定
OpenDSS reference：最大相对误差 `2.94510954372887e-5`、RMS 相对误差
`6.7477168074739e-6`、最大绝对误差 `2.73751317027705 V`，低于 `0.5%` 门限。PSA 聚焦回归另锁定
3 个故障 epoch 的 HL/HT/LT 三相短路电流，以及非故障 epoch 的绕组电压、电流、复损耗和满载端
电压；`bus_phase` 在故障 epoch 保留保护测量前潮流状态，此合同未被当作故障母线电压比较。
`conversion_report.csv` 已由 `unsupported` 改为 `converted`，当前计数由 147/112 更新为 148/111。

## 2026-11-04 Ckt7 8760 点 Yearly 闭合

官方 `EPRITestCircuits/ckt7/RunDSS_ckt7.dss` 已使用 release `tensoreval`（SHA-256
`46edf137c06179c1984c2aae269bc260b514c29b39e1ed9769f277a7a17dc177`）完成两个 Snapshot solve
epoch 和全部 8,760 个 Yearly 时点。schema v2 bundle 在仓库内与独立临时目录均自执行成功；迁移前后
原生参考 CSV 与比较 JSON 的 SHA-256 完全相同。

数值门禁显式使用 `line_model=exact`，只比较最终 epoch 2、`31536000 s` 的结果，不把它扩大为
中间轨迹对齐声明。2,452 个母线相复电压的最大/RMS 相对误差为
`0.0005814825338454038` / `0.0003498953036292401`；14 个 Monitor 复数量的最大/RMS 相对误差为
`0.0007173422750251094` / `0.0005788494690267307`，均通过 `1%` 门限。
`conversion_report.csv` 已由 `unsupported` 改为 `converted`，当前计数由 148/111 更新为 149/110。

## 2026-11-04 发布后 110 个入口的有界复扫

冻结报告 [`candidate_scan_2026-11-04.json`](candidate_scan_2026-11-04.json) 对当前 110 个
`unsupported` 入口全部执行 30 秒 linked conversion，并只对 7 个转换成功者追加 60 秒
`--check`。结果为 103 个转换诊断、7 个转换成功后检查诊断、0 个超时、0 个完整检查成功；扫描未
修改 `conversion_report.csv`，也不构成任何新案例的发布声明。扫描使用按发布 feature 构建并增加
解析器证据输出的 release `tensoreval`，SHA-256 为
`382ce2670964ccb5df9d8f7c4c6106ec93dbef5d3892aa6d2f527ce6febffb57`。

| 顺位 | 入口 | 有界证据 | 结论与剩余工作 |
|---:|---|---|---|
| 1 | `Examples/Microgrid/GridFormingInverter/GFM_IEEE123/Run_IEEE123Bus_GFMSnap-B.DSS` | 依赖展开后的解析器事件流明确确认 `Circuit` bootstrap；converter 报告 388 条 DSS 语句、2 个 solve epoch、366 条依赖展开语句；linked conversion `0.043737 s` 成功，生成 RSS SHA-256 `39f434fd5020a1d633aa356beb16c2459dcf1a860cb568023bb98f258b66b71c`；check `0.179204 s` 后保留 1 个 fresh 阻断 `TENSOREVAL.DSS.TRANSLATE.SOLVE_FAILED` | **下一个正式候选**。闭合打开 `line.sw5` 后 epoch 1 的 QSTS 拓扑求解，再以 `line_model=exact` 对两个 Snapshot 的最终节点电压、控制终态及 GFM/Storage/PV 结果做 OpenDSS 门禁；随后仍须完成 portable bundle 自执行和搬移验证 |
| 2 | `Examples/Paulo_Example/DSSFiles/solve_snapshot.dss` | linked conversion 成功；check 返回同类 `SOLVE_FAILED` | 文件仅含 `solve`，依赖既有会话电路，不是独立发布入口；应更正分类或与其主入口建立复合合同 |
| 3 | `Examples/ExpControl/SolveAndExportMonitors.dss` | linked conversion 成功；check 返回同类 `SOLVE_FAILED` | 文件注释明确要求电路和基准解已存在，不是独立发布入口；不应直接发布 |

首选源文件 SHA-256 为
`b025cb063e83c8feadfb98ba2eac891b3534ee14849abb80268f577941067786`。即使修复当前唯一的深层
求解诊断，也必须重新执行严格转换、完整检查、OpenDSS 数值对标和 bundle 迁移门禁，不能从本次
有界扫描直接推导发布状态。

## 2026-11-05 GFMSnap-B 发布与 109 个入口复扫

GFMSnap-B 使用 release `tensoreval`（SHA-256
`1c6dd45ca03d2fb73214e7fb23f7ce3c064a7d6e9ca56a332bb71104755f3d50`）完成 388 条语句、两个
Snapshot epoch、正式 bundle 和迁移副本自执行。最终 epoch 1 显式使用 `line_model=exact`：252 个
带电节点的最大/RMS 归一化复电压误差为 `4.112314026489773e-6` / `2.3217050758919202e-6`；32 个
失电节点的 PSA 电压为精确零，OpenDSS 残差最大 `0.005418309046707934 V`，低于 `0.01 V` 判据。
`conversion_report.csv` 计数由 149/110 更新为 150/109。

冻结报告 [`candidate_scan_2026-11-05.json`](candidate_scan_2026-11-05.json)（SHA-256
`c05a640861c1c86660f300adf6278d2959188ed721fdeeaf583514b199fd3dfb`）对剩余 109 个入口全部完成
有界扫描：103 个 linked conversion 诊断、6 个转换成功后检查诊断、0 个超时、0 个完整检查成功。
6 个转换成功者均依赖预置会话状态，不能作为独立 bundle 发布。

排除会话片段后，当前最小的完整工程候选是
`Examples/Dynamic_Expressions/Dynamic_KundurDynExp.dss`。它显式创建 Circuit，fresh 阻断仅为
`PSA.DSS.DYNAMIC.MONITOR_REQUEST_UNSUPPORTED`；下一步应把运行期 Monitor 显式采样及指定缓冲区
Export/Show/Plot 接入已有 Dynamic Monitor 结果合同，并以 OpenDSS 锁定 5 秒动态轨迹。
GFMSnap-C 是同系列下一入口，但有两个 fresh 阻断：执行开始后的 Storage `ControlMode` 修改顺序无法
由单一不可变 NetworkTables 表达，且 `Storage.ControlMode=GFM` 尚无无损导入映射。

## 2026-11-07 Stevenson 正序 Snapshot 发布

未修改的官方 `Examples/Stevenson/StevensonPflow.dss` 使用 `Set cktmodel=pos`、单相正序网络和
Generator Model 3。本轮准入严格限定为单个全新 Snapshot epoch、Newton 求解和
`line_model=exact`，不外推到 time-series、multi-epoch continuation 或其他正序生命周期。
现成 release `tensoreval`（SHA-256
`3355a4ae97767be49bfc5a7e32186d93b34131810d98f48dacc10f63da18f4ca`）完成 25 条语句和 1 个
solve epoch 的严格检查。schema v2 bundle 搬移后从 bundle 根目录独立自执行，输出 5 行电压，
closure 不含来源树或工作区绝对路径。

OpenDSS 数值门禁比较电压、端子电流、端子功率、Generator 聚合功率、元件损耗和全网复损耗；最大
归一化复数误差分别为 `4.706012215396644e-5`、`5.060234324821709e-4`、
`5.134890000010807e-4`、`2.6197402537890573e-4`、`9.737775970918348e-4` 和
`9.558586294051992e-5`，均低于 `1e-3`。`conversion_report.csv` 已由 `unsupported` 改为
`converted`，当前计数由 150/109 更新为 151/108；既有三相 Stevenson bundle 未修改。

冻结报告
[`candidate_scan_2026-11-06-post-stevenson.json`](candidate_scan_2026-11-06-post-stevenson.json)
（SHA-256 `197a90ce76d670de6d68456038d3e5dda83c0280c954c41007776e2c12f6e0be`）对剩余 108 个入口
完成有界复扫：102 个 linked conversion 诊断、6 个转换成功后检查诊断、0 个超时、0 个完整检查成功。
6 个转换成功者均依赖预置会话状态，不是独立发布入口；排除这些片段后，下一完整工程候选仍为
`Examples/Dynamic_Expressions/Dynamic_KundurDynExp.dss`。

## 2026-11-07 PV 正 kvar 能力 Snapshot 发布

未修改的官方
`Examples/InverterModels/PVSystem/NewFeatures/varCapability/PV_currentkvarLimit_kvar.dss` 已按显式
Snapshot、`line_model=exact` 和 Radial 求解发布为 schema v2 portable bundle。固定上游提交为
`3b208397160213cae4a9e2d0a7d1aa3528ce26e1`，源文件 SHA-256 为
`9134bf1c0f17a06a9cd94781f625536d86ee76f29bc240904a1ff5078e4bd009`。

release `tensoreval` SHA-256 为
`876ce8e214cec6ad56c541a658406648b3b5eca63cfb618e600459e0596dcc34`；严格检查确认 12 条语句、
1 个 solve epoch。搬迁后的 bundle 独立自执行并输出 3 行母线相电压，closure 未包含来源树绝对路径。
锁定 OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5 参考在 2 次迭代内收敛；3 个母线相电压通过
`1e-9` 最大归一化复数误差门限，3 个 PV 相的有功/无功通过 `1e-6 W/var` 最大绝对误差门限。

`conversion_report.csv` 仅将该正 kvar 行由 `not_standalone_entry` 改为 `converted`。发布数由 151
增至 152；因为该入口原先不在 259 个独立入口统计内，本轮把独立入口总数扩为 260，尚未发布数仍为
108。相邻 `PV_currentkvarLimit_kvarNEG.dss` 未经本次数值门禁，继续保持 `not_standalone_entry`，
不得由正 kvar 结果外推。

## 2026-11-07 PV 负 kvar 能力 Snapshot 发布

未修改的官方
`Examples/InverterModels/PVSystem/NewFeatures/varCapability/PV_currentkvarLimit_kvarNEG.dss` 已按显式
Snapshot、`line_model=exact` 和 Radial 求解发布为 schema v2 portable bundle。源文件 SHA-256 为
`8f3e3389ded5a057984f3efbdfce1962508bda51fea52b6c5bdc7ea457d35a79`；固定上游提交仍为
`3b208397160213cae4a9e2d0a7d1aa3528ce26e1`。

本案例使用独立 OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5 reference，并在 2 次迭代内收敛。
`kvar=-500` 通过 `kvarmaxabs=300` 被限制为三相合计 `-300 kvar`，与正向 sibling 的
`kvarmax=200` 分支分开验证。3 个母线相电压最大归一化复数误差为
`3.3801475356637357e-10`；PV 相最大有功/无功绝对误差分别为
`5.820766091346741e-11 W` 和 `1.4551915228366852e-11 var`。

release binary SHA-256 为 `876ce8e214cec6ad56c541a658406648b3b5eca63cfb618e600459e0596dcc34`。
严格检查确认 12 条语句和 1 个 solve epoch；搬迁 bundle 自执行输出 3 行电压，closure 不含来源树、
`/tmp` 或工作区绝对路径。目录只将 NEG 行由 `not_standalone_entry` 改为 `converted`：发布数由
152 增至 153，独立入口分母由 260 增至 261，尚未发布数仍为 108。

## 2026-11-07 CurrentkvarLimite Volt-Var Snapshot 发布

未修改的官方
`Examples/InverterModels/PVSystem/CurrentkvarLimite/PV_currentkvarLimit_VV.dss` 已按显式
Snapshot、`line_model=exact` 和 Radial 求解发布。源文件 SHA-256 为
`ae05f52408aff3795dba85ffe655e16b4711f13aa9daff30b0d524cd3a5af353`。现有 release
严格检查 15 条语句和 1 个 solve epoch；搬迁 bundle 自执行输出 3 行电压，closure 无绝对路径。

独立 OpenDSS reference 在 56 次迭代、28 次控制迭代后收敛。3 个母线相电压最大归一化误差为
`3.3801558328329703e-10`；PV 相 P/Q 以 1.2 MVA 为基准的最大归一化误差为
`3.6963573337257865e-6`，通过 `1e-5` 门限。catalog 只修改该 VV 行，当前计数为
154 `converted`、108 `unsupported`、586 `not_standalone_entry`；独立入口分母为 262。

新的 42 条模型入口扫描报告设置 `publishes_cases=false`：7 条可完成显式 Snapshot 转换和检查，
本轮仅 VV 具备完整独立数值与搬移证据。Kundur 的深入探测则推翻了“只补 Monitor”判断：其后还有
外部 Generator DynamicEq/DynOut 和 Monitor 端口/mode 合同，内置模型替代的 5 秒转速轨迹也与
原生明显分离，因此降级为需要专用动态模型闭合的中大型候选。

## 2026-11-07 IEEE30 Master Snapshot 发布

模型入口 `IEEETestCases/IEEE 30 Bus/Master.dss` 已按显式 Snapshot、Newton-Raphson 与
`line_model=exact` 发布。官方源 SHA-256
`bb0b864c3e63955cad3c6f289832421c24cc0d4dccaf9f66c3be51e8c22d1aba` 与 PSA 锁定 fixture
完全相同；严格检查覆盖 101 条语句、1 个 solve epoch 和 Circuit bootstrap。

已有 `projection_evidence.json` 证明官方 Master 与 PSA 可执行投影的七类原生输出逐文件字节一致，
`dss_import::ieee30_snapshot_matches_locked_native_reference` 则以显式 Exact 线路模型比较 90 个节点、
264 个端口电流、264 个端口功率、65 个元件损耗/功率以及汇总和控制状态。portable bundle 的
closure 只包含 Master 与五个 Redirect 依赖；不含 `Run_IEEE30.DSS`。catalog 当前为
155 `converted`、108 `unsupported`、585 `not_standalone_entry`，独立入口分母为 263。

## 2026-11-07 IEEE8500 Master-unbal Snapshot 发布

模型入口 `IEEETestCases/8500-Node/Master-unbal.dss` 已按显式 Snapshot、Radial 与
`line_model=exact` 发布。官方源与已有 PSA fixture 的 SHA-256 都是
`8cbb671b1e353b72d445f361547bd43ac8ed3c90878d7c8dced26ba3bc55f043`；严格检查覆盖
7,590 条语句、1 个 solve epoch 和 Circuit bootstrap。

既有 unbalanced native reference 覆盖 8,531 个节点电压、24,195 个端口电流/功率、7,275 个
元件损耗、7,280 个对象、3,579 个控制状态与 11 个汇总值；Exact consumer 逐项要求全部原生节点
存在。portable closure 共 14 个 manifest 文件，不含 wrapper/GFM sibling 和绝对路径；搬移后完整
PSA `bus_phase` 输出 11,038 行。catalog 当前为 156 `converted`、108 `unsupported`、584
`not_standalone_entry`，独立入口分母为 264，尚未发布数仍为 108。

## 2026-11-07 五个模型入口 Snapshot 批量闭合

两个 CurrentkvarLimite 固定 kvar 模型入口、GFM IEEE8500 unbalanced Master、官方 IEEE8500
balanced Master 和 GFM balanced Master 均已按显式 Snapshot、Radial、`line_model=exact` 发布。
CurrentkvarLimite 的两个 exact-source consumer 锁定全部 3 个复电压与 3 个 PV 相 P/Q；其中
`kvarNEG` 官方源仍是正向 `kvar=500`，不作为负 kvar 证据。

IEEE8500 balanced/unbalanced 共用既有原生 reference consumer，分别覆盖 8,531 个节点、
15,882 个端口电流、15,882 个端口功率以及元件功率/损耗和汇总。GFM balanced 的唯一电气源差异
是一个没有消费者的 `LoadShapes.dss` redirect；portable closure 保留该文件和 `LoadShape1.CSV`，
搬移结果与官方 balanced Master 按 `bus + phase` 对齐的 11,038 个复电压最大归一化误差为
`1.2399434592283917e-16`，属于一 ULP 量级。

五条 catalog 行由 `not_standalone_entry` 改为 `converted` 后，当前为 161 `converted`、108
`unsupported`、579 `not_standalone_entry`；独立入口分母为 269。完整证据见
[`five_model_snapshot_publication_evidence.md`](five_model_snapshot_publication_evidence.md)。

## 2026-11-08 五个执行入口闭合

Ckt5、Storage Quasi-Static Demo、IEEE123 `Run_YearlySim`、IEEE13 Assets 与 Ckt24 已从当前
`unsupported` 集合闭合。通用改造覆盖未引用无效导体过滤、`Ins` 电缆缩写、Solve RPN 表达式、
序分量 LineCode 相数扩展、布尔 `RegControl.DebugTrace`，以及仅限 `New EnergyMeter` 的幂等 Clear；
被引用无效导体、显式矩阵猜测扩维、非法布尔值和后续 Meter Clear 仍保持失败关闭。

五个 portable bundle 均从独立 `/tmp` 父目录执行成功；72 个 manifest 文件从 `sparrowzz` HEAD
Git blob 复算 SHA-256 后零不匹配。固定 OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5 比较覆盖
3,437 / 3,437 / 278 / 41 / 7,522 个原生最终节点，最大归一化复电压误差依次为
`8.20624e-4`、`9.34665e-4`、`2.10120e-6`、`2.75110e-3` 和 `3.10352e-2`。
catalog 当前为 166 `converted`、103 `unsupported`、579 `not_standalone_entry`。
完整证据见
[`next_five_candidate_publication_evidence.md`](next_five_candidate_publication_evidence.md)。

## 2026-11-08 第二批五个执行入口闭合

Ckt24 MemoryMapping 示例的 non-MM、CSV-P、CSV-PQ 三个完整 Daily 入口、Paulo IEEE13 Daily 和
Matlab IEEE 519 初始化 Snapshot 已从 `unsupported` 集合闭合。通用改造包括：把 BusMarker 归入
输出副作用；保留 `MinIterations`；按 OpenDSS 默认补全 SwtControl `Normal`/`State`/`Reset` 并接受
`Lock=Open`；Meter 电压基准忽略只由 disabled 对象形成的 ghost bus；把 LoadShape
`MemoryMapping` 限定为不改变曲线样本的存储提示。非法布尔值、真实引用的 ghost bus 和未表示的
动态/外部编排仍然失败关闭。

五个 portable bundle 搬移执行分别输出 752,200 / 1,025 / 24 / 752,200 / 752,200 行电压；
103 个 manifest 声明 Git blob 复算 SHA-256 后零不匹配。显式 `line_model=exact` 的锁定参考覆盖
三个 Ckt24 各 7,522 个最终节点、Paulo 两个 epoch 各 41 个节点、IEEE 519 的 24 个节点，以及
每个状态的三相源端 P/Q。Matlab 案例只证明 `.DSS` 文件自身的初始化 Snapshot，不证明外部
`Main_Script.m` 谐波循环。

catalog 当前为 171 `converted`、98 `unsupported`、579 `not_standalone_entry`。完整证据见
[`next_five_batch_2_publication_evidence.md`](next_five_batch_2_publication_evidence.md)。

## 2026-11-08 第三批五个执行入口闭合

Ckt24 MemoryMapping 示例剩余的 TXT-P、TXT-PQ、DBL-P、SNG-P 和旧式直接 SNG 五个完整
100 步 Yearly 入口已从 `unsupported` 集合闭合。通用改造为 LoadShape 增加有界的小端
float32/float64 资产物化，并把直接及嵌入式 `SngFile`/`DblFile` 纳入同一依赖来源与 bundle
改写合同；非法宽度、空文件、非有限值、缺失文件和超限点数继续失败关闭。

五个 portable bundle 搬移并改名后均独立执行，分别输出 752,200 行电压；schema-v2 manifest
共声明 123 个文件，从 `sparrowzz` Git blob 复算 SHA-256 后零不匹配，二进制资产与官方源
逐字节一致。显式 `line_model=exact` 的锁定参考对每例比较 7,522 个最终母线相及三相源端
P/Q，最大/RMS 复电压相对误差为 `1.92567e-2` / `2.08902e-3`，源功率最大误差为
`1.20918e-2`。

catalog 当前为 176 `converted`、93 `unsupported`、579 `not_standalone_entry`。这些入口只
执行 100 个一小时时点，不能据此声明 8,760 小时性能。完整证据见
[`next_five_batch_3_publication_evidence.md`](next_five_batch_3_publication_evidence.md)。

## 2026-11-08 YYD Direct 故障 continuation 闭合

`IEEETestCases/4Bus-YYD/YYD-Master.DSS` 最初由未修改官方入口生成 schema-v2 portable
bundle，本批已在同一路径提升为 schema-v3 study sequence。执行计划保留 Snapshot 初始化和后续 Direct 单相接地故障两个有序 epoch；Direct
epoch 使用专用 `DirectFault` 投影，full、stream、trace-stream 和 diagnosed API 返回一致的
故障后电压、支路和 fault state。禁用的 Fault 声明不会扩大“单个启用故障”合同，非法计划在
数值执行前失败关闭。

原目录和搬移改名后的 bundle 均输出 36 行 `bus_phase`，规范化 CSV SHA-256 为
`9dbd2d7f5f742e13b7de4492b3a6202f92a1597a7f6a35dab117b5e1a671853a`。最终故障 epoch
相对 OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5 的最大去共模复电压误差为
`5.258653887720258e-9`，故障电流误差为 `3.763079292555971e-8`。没有故障后合同的
Snapshot source/transformer/conductor/loss/violation/summary 表不会作为 Direct 结果发布。

catalog 当前为 178 `converted`、91 `unsupported`、579 `not_standalone_entry`。该合同只覆盖
官方两 epoch YYD 案例，不扩展为任意多故障、保护动作或 8,760 小时 QSTS 性能声明。

## 2026-11-09 EPRI M1 无 PV Snapshot 终态闭合

`EPRITestCircuits/epri_dpv/M1/Master_NoPV.dss` 已从 `unsupported` 集合闭合。固定官方入口的
4,279 条语句降为两个有序 Snapshot epoch；显式 `line_model=exact` 的最终态门禁比较 3,153 个
原生母线相、三组接口复功率、总复损耗、M1 tap 和三个电容器状态。电压最大/RMS 归一化误差为
`9.141043688582145e-5` / `2.7928383278329492e-5`，接口功率最大归一化误差为
`6.054447326642671e-5`，总损耗最大归一化误差为 `2.8505834749390895e-4`；tap 和电容状态一致。

PSA 额外保留的 25 个禁用开关端点全部为精确零，门禁按固定键集合失败关闭；M1 变压器两侧功率
使用具有固定 SourceBus→temp 方向的 `branch_flow`，因为该变压器不出现在
`transformer_winding_flow` 投影中。portable bundle 原地和搬移改名后均输出 6,356 行且字节相同。

当时 catalog 为 180 `converted`、89 `unsupported`、579 `not_standalone_entry`，独立入口分母
为 269。完整证据见
[`epri_m1_no_pv_publication_evidence.md`](epri_m1_no_pv_publication_evidence.md)。该闭合只覆盖无
PV 入口最终 Snapshot；不覆盖 PV、可靠性、Monitor 通道、QSTS 轨迹或 8,760 小时年度性能。

## 2026-11-11 NEV 多 study 与 WATTPF Snapshot

`IEEETestCases/NEVTestCase/Run_NEV.dss` 现由 schema-v3 bundle 依次执行 Snapshot、FaultStudy 和
Harmonic，九张公开结果在原路径与搬移路径的 Parquet 文件逐字节一致。Snapshot 的全部 192 个
OpenDSS/PSA `(bus, phase)` 键完全一致，最大归一化复电压误差 `2.21931e-5`，最大角度误差
`0.000707223°`。清单校验覆盖 12 个文件和 48 个固定输入，并禁止最后一个 Harmonic 成功掩盖
缺失或失败的 FaultStudy。

`Examples/InverterModels/PVSystem/InvControl/watt-pf_watt-var/dss/SnapShot_wattpf.dss` 从 model-only
入口以显式 Snapshot 发布。它锁定省略 `kVA` 的 OpenDSS 500 kVA 默认、singleton DERList、
`DeltaQ_Factor=-1`、重复 X 的 PF 符号阶跃及能力圆投影前的可用有功基准。三相最大归一化复电压
误差为 `2.95244e-7`，搬移输出字节一致。

最终 release 已把 YYD、IEEE123/IEEE37 FaultStudy、FreqScan 和两个既有 HarmonicT bundle 在原路径
提升为 schema-v3 study sequence；原地与搬移执行全部通过，公开结果列表及行数不变。前四例输出
逐文件字节一致；两个 HarmonicT 例的稳定键、schema 和非浮点值一致，独立浮点重算差异小于
`1e-12`。

当前 catalog 为 182 `converted`、88 `unsupported`、578 `not_standalone_entry`。完整命令、
SHA-256 和候选替换说明见
[`multi_study_sequence_batch_publication_evidence.md`](multi_study_sequence_batch_publication_evidence.md)。

## 建议的实现门禁

1. 为每个候选保存“原入口 SHA-256、转换模式、PSA 证据 case、首诊断、修复后诊断”关系，
   避免只维护诊断码白名单。
2. 首批 12 个入口已通过严格转换并全部发布 bundle；第二批 8 个候选中有 6 个完成
   V/I/P/Q/loss、控制终态或 FaultStudy 专用结果比较并发布 bundle。YYD Direct/Fault
   continuation 已按上述有界合同发布；InductionMachine 仍等待 Dynamic/保护动作执行合同，
   不能降低门禁后发布。
3. 每消除一个首诊断都重扫当前 88 个尚未发布入口，并对受影响文件继续探测第二层诊断；成功数量相同
   不代表改造没有改变失败原因。
4. 对 Plot、Show、Export、FileEdit、BusCoords 等副作用采用 manifest 记录；电气状态修改
   `Open/Close/Edit/BatchEdit` 必须进入执行计划，不能按输出副作用忽略。
5. `Mode=FaultStudy`、HarmonicT 快照和一个文件内多 study 应各自有独立 adapter，不应塞入
   普通 QSTS execution profile。

## 本次验证

- `IEEETestCases/8500-Node/P174_Run_360kW_PV.DSS` 使用现成 release 完成 7,616 条语句、5 个 epoch
  和 2,900 步 Duty 的严格检查；官方路径 bundle 搬移后独立自执行，输出 32,054,352 行电压。
  与锁定 OpenDSS 轨迹比较 26,100 个节点电压，最大相对误差 `2.41171923822849e-5`。

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
- `IEEETestCases/123Bus/RevRegTest.dss` 已完成 5 个 epoch、74 个求解时点和 20,572 个节点复电压
  的 OpenDSS 逐点比较；最大相对误差 `1.43082e-5`，7 组调压器的 518 条 tap 状态全部一致，
  Newton portable bundle 自执行通过。
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
