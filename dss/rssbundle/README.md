# OpenDSS 官方发行文件转换清单

本目录记录 `tensoreval` 对 OpenDSS 官方公开发行树中独立入口的严格 DSS→RSS bundle
扫描结果。扫描不会删除命令、补写模型参数或放宽诊断；只有转换、生成脚本验证和 bundle
自执行全部成功的案例才会生成 `.rssbundle`。模型定义、公共依赖和其他非独立文件不在本文讨论。

## 来源和范围

- 上游：[EPRI OpenDSS 示例与测试案例](https://opendss.epri.com/ExamplesDocsandTestCases.html)
  所述官方 SVN 公开发行内容的
  [`dss-extensions/electricdss-tst`](https://github.com/dss-extensions/electricdss-tst) 轻量镜像。
- 固定提交：`3b208397160213cae4a9e2d0a7d1aa3528ce26e1`；2026-10-19 核对时仍为镜像 `master`。
- 扫描根：`Version8/Distrib`。
- 单文件检查超时：30 秒；本次没有超时。
- 对依赖越过案例目录的入口，使用整个 `Version8/Distrib` 作为 `--source-root` 重试。

## 结果

| 状态 | 文件数 | 含义 |
|---|---:|---|
| 已转换并发布 bundle | 144 | 严格转换、求解、OpenDSS 数值门禁和 bundle 自执行全部通过；最新闭合 InverterTechNote 的 Storage kVA 阈值入口 |
| 当前尚未发布的独立入口 | 115 | 259 个独立入口扣除 144 个已发布入口 |
| 超时 | 0 | 无 |

144 个成功入口按官方发行树的相对路径保存在本目录；每个 `.rssbundle` 都包含 manifest、生成的
RSS、DSS 依赖和数据资产，并在原子发布前完成一次自执行。已经位于 `sparrowzz/dss/models` 的
静态化案例不计入这些官方原始入口。`conversion_report.csv` 的 `status` 已同步当前 144 个发布入口；
其余 115 个入口记录当前转换或数值门禁诊断；2,900 步 P174 长任务保留已验证的未发布记录。历史首诊断用于说明改造来源，
当前发布状态以 `status`、本节和实际 manifest 为准。

## 首轮改造进展

2026-10-19 已按下文路线完成第一批编排层和 PSA 导入边界改造：

- 所有原生 `Solve`（包括单次 Snapshot）统一进入 PSA execution plan；`Snap` 已归一化，
  Snapshot 初始化后继续 Daily/Duty 不再被分类器预先拒绝。
- `tensoreval --study snapshot|execution|harmonic|auto-add|relcalc|dynamic` 已对外开放；纯模型入口可用
  `--study snapshot` 明确要求单次静态研究。
- `EarthModel`、`Algorithm`、`MaxIter/MaxIterations`、`MaxControlIter/MaxControlI` 和
  `Tolerance` 由 PSA 统一校验。后三项按 epoch 写入实际潮流参数，生成 RSS 和转换期校验
  使用同一执行入口。
- dss2rss 只依据 PSA 诊断的 `severity` 判断是否阻断，不再解析诊断码或英文消息建立第二套
  放行白名单。
- execution epoch 会重新附加 `BusCoords/LatLongCoords` 与文件型参数资产；坐标路径统一使用
  DSS 参数扫描结果，支持引号、括号和行尾注释；坐标命令以稳定
  `PSA.DSS.EXECUTION.SIDE_EFFECT_IGNORED` Warning 说明其不影响电气结果。
- 补齐 EnergyMeter 标准位置参数、CapControl 唯一属性缩写、受控 Capacitor 的 OpenDSS
  默认相数和节点解析，以及电气研究中 `NumCust/RelWeight` 的可靠性专用语义。

本轮完整扫描发布了以下官方入口：

| 官方入口 | 当前结果 | 验证证据 |
|---|---|---|
| `EPRITestCircuits/epri_dpv/J1/Master_noPV.dss` | 已发布 | 4,982 条语句，ExecutionPlan，1 个 epoch |
| `EPRITestCircuits/epri_dpv/J1/Master_withPV.dss` | 已发布 | 4,996 条语句，ExecutionPlan，1 个 epoch |
| `Examples/VSConverter/SG2012.dss` | 已发布 | 110 条语句，ExecutionPlan，1 个 epoch |
| `Examples/WampServer/testcommandline.dss` | 已发布 | 18 条语句，ExecutionPlan，1 个 epoch |
| `IEEETestCases/13Bus/IEEE13Nodeckt.dss` | 已发布 | 201 条语句，ExecutionPlan，1 个 epoch；使用全局依赖根 |
| `IEEETestCases/37Bus/ieee37.dss` | 已发布 | 203 条语句；坐标资产复制、重写和 bundle 自执行均通过 |
| IEEE 4 Bus 的 DY、GrdYD、YD、YY 四个 balanced 入口 | 已发布 | 每个 22 条语句，ExecutionPlan，1 个 epoch |

## 第二轮改造进展

2026-10-21 按能力审计的建议顺序完成 `Mode=F`、内置 `LoadShape.default` 和有界
RegControl `Like` 回放，并补齐 `Mode` 对时钟及默认控制队列的影响。IEEE34 Mod2 还暴露了
DSS 母线引用大小写问题：线路声明 `mid806`，负荷引用 `Mid806`；导入器现在按 OpenDSS
语义解析为同一母线。下面 6 个原始入口已完成严格转换、自执行、OpenDSS 数值比较并发布：

| 官方入口 | 执行规模 | OpenDSS 数值门禁 |
|---|---|---|
| `IEEETestCases/34Bus/Run_IEEE34Mod1.dss` | 649 条语句，2 个 Snapshot epoch | 电压最大误差 `9.06e-6`，调压器终态完全一致 |
| `IEEETestCases/34Bus/Run_IEEE34Mod2.dss` | 581 条语句，2 个 Snapshot epoch | 电压最大误差 `1.36e-5`，线路量最大误差 `5.83e-5`，调压器终态完全一致 |
| `IEEETestCases/123Bus/Run_IEEE123Bus.DSS` | 771 条语句，2 个 Snapshot epoch | 电压最大误差 `1.37e-5`，线路量最大误差 `1.79e-4`，7 组调压器终态完全一致 |
| `IEEETestCases/123Bus/P174_Run_VoltageProfile.DSS` | 379 条语句，2 个 Snapshot epoch | 电压最大误差 `1.37e-5`，线路量最大误差 `1.79e-4`，7 组调压器终态完全一致 |
| `IEEETestCases/123Bus/P174_Run_SolarRamp.DSS` | 380 条语句，3 个 epoch、300 个 Duty 时点 | 电压最大误差 `2.44e-5`；12 次 tap 状态变化、最终档位、Generator 输出和 300 个 Monitor 样本一致 |
| `IEEETestCases/ShortCircuitCases/ieee37_SC_Currents.dss` | 170 条语句，175 个 FaultStudy 场景 | 去共模电压最大误差 `1.61e-7`，故障电流缩放误差 `1.51e-4`，正负序阻抗误差 `8.46e-12` |

SolarRamp 的线路电流/功率最差缩放误差为 `7.4e-3`，出现在低潮流支路 L99；比较分别使用
`1 A` 和 `1 kVA` 分母下限，仍低于该类小量的 `1%` 门限。源端功率最大误差为 `1.57e-4`。

## 非整数频率扫描

`Examples/FreqScan/Run_Scan.dss` 已按官方入口原样发布为
`Examples/FreqScan/Run_Scan.rssbundle`。它先以 Static 控制模式完成 IEEE 123 节点基波初始化，
再读取 `ScanSpectrum.csv`，执行 65–1000 Hz 的 188 个非整数频点，并输出 Line.l84 端子 1
三相 Monitor.Mscan，共 564 行。bundle 携带完整 DSS 依赖和数据资产，从原目录复制到独立临时目录后，
使用下列公开命令仍可运行并生成 7 张结果表：

```bash
tensoreval --result-dir /tmp/freqscan-results \
  sparrowzz/dss/rssbundle/Examples/FreqScan/Run_Scan.rssbundle
```

OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5 的锁定参考覆盖全部 188×3 个键。电压幅值最大缩放误差
为 `9.3144e-6`，电流幅值在 1 A 分母下限下最大缩放误差为 `4.4922e-6`；电压相角最大误差
`0.000555966°`，电流不小于 1 A 时最大相角误差 `0.000199377°`。频率按 OpenDSS Monitor 的
Float32 发布表示比较。完整数据见 `numerical_comparison.csv`，逐点参考和哈希门禁位于 PSA
测试资产中。

## 数值对照

2026-10-20 已用 OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5 直接运行每个 bundle 中保留的
原始 DSS 入口，并与 `tensoreval` 执行同一 bundle 后返回的最终 `voltage` 表比较。OpenDSS 和
PSA 的 10 个计算都收敛。修正三相 Δ→Y 变压器的 RegControl、次级 tap、嵌套 RPN 变量的
负荷额定电压，以及 J1 的 STATIC 队列、开关阻抗、单相调压器漏阻抗和径向暖启动后，10 个
案例全部通过既有 PSA 算例门限。
“已发布”表示严格转换和自执行成功；
各案例是否达到 OpenDSS 数值门限以本表为准。

| 官方入口 | 比较口径 | 最大相对误差 | 门限 | 结果 |
|---|---|---:|---:|---|
| `EPRITestCircuits/epri_dpv/J1/Master_noPV.dss` | 原始复电压 | 0.007131% | 1% | 通过 |
| `EPRITestCircuits/epri_dpv/J1/Master_withPV.dss` | 原始复电压 | 0.006013% | 1% | 通过 |
| `Examples/VSConverter/SG2012.dss` | 原始复电压 | 0.03344% | 1% | 通过 |
| `Examples/WampServer/testcommandline.dss` | 原始复电压 | 3.38×10⁻⁸% | 1% | 通过 |
| `IEEETestCases/13Bus/IEEE13Nodeckt.dss` | 原始复电压 | 0.007772% | 0.01% | 通过 |
| `IEEETestCases/37Bus/ieee37.dss` | 去母线共模复电压 | 0.03036% | 2% | 通过 |
| `IEEETestCases/4Bus-DY-Bal/4Bus-DY-Bal.dss` | 原始复电压 | 0.007800% | 0.03% | 通过 |
| `IEEETestCases/4Bus-GrdYD-Bal/4Bus-GrdYD-Bal.dss` | 去母线共模复电压 | 0.004489% | 0.03% | 通过 |
| `IEEETestCases/4Bus-YD-Bal/4Bus-YD-Bal.dss` | 去母线共模复电压 | 0.003934% | 0.3% | 通过 |
| `IEEETestCases/4Bus-YY-Bal/4Bus-YY-Bal.dss` | 原始复电压 | 0.01079% | 0.03% | 通过 |

逐案例节点数、RMS 误差、原始复电压误差、幅值误差、最大绝对差及最差节点见
[`numerical_comparison.csv`](numerical_comparison.csv)。计算公式为
`|V_PSA - V_OpenDSS| / max(|V_OpenDSS|, 1 V)`。IEEE37 和两个 Y-Delta 案例含没有确定对地
零序电位的三线制区域，按 PSA 既有测试口径先在每个母线减去共同复电压，再比较可观测的
相间电压；报告仍同时保留原始误差，不能用共模处理掩盖接地网络的偏差。

节点身份核对得到：IEEE13、IEEE37、SG2012、WampServer 和三个非浮动中性点 IEEE4 案例的
节点集合完全一致；4Bus-YD 只省略 OpenDSS 的浮动中性点 `n2.4`，与既有 PSA 合同一致；两个
J1 案例的 4,245 个 OpenDSS 节点全部存在，另外 2,791 个 PSA 行全部是失电母线的零电压行，
不参与数值误差。

复核结果为：

- J1 有/无 PV 案例的 4,245 个 OpenDSS 节点全部匹配，2,791 个 PSA 额外节点均为失电零电压。
  修正 STATIC 队列时间基准、Y/Y 次级 tap、开关默认闭合阻抗、单相调压器漏阻抗 tap 基准、
  径向控制循环暖启动和粗档距最小一档规则后，最大复电压相对误差分别为 0.007131% 和
  0.006013%，9 个 RegControl 终态也与 OpenDSS 一致。
- SG2012 原先缺失三相 RegControl 的 DataFrame 回读及运行支持，次级 tap 也未进入匝比和
  漏阻抗折算。修正后两个调压变压器与 OpenDSS 一样停在 1.08125，129 个节点的最大误差
  为 0.03344%。
- WampServer 的 `@loadkv="@syskv 3 sqrt /"` 之前在导入时丢失 RPN 引号语义，导致三个单相
  Model 2 负荷使用 12.47 kV 默认额定电压，而 OpenDSS 使用 7.9674337 kV。修正执行网络的
  额定电压后，6 个节点的最大复电压相对误差为 3.38×10⁻¹⁰。
- IEEE37 的原始对地复电压误差为 9.71%，去除未接地/open-delta 网络的母线共模后为 0.03036%；
  这属于参考电位差异，不能按原始对地相量判为潮流不一致。

`tensoreval --result-dir` 会把命名 DataFrame 完整写为 CSV，避免从终端截断表格取数：

```bash
tensoreval --result-dir /tmp/ieee13-psa \
  sparrowzz/dss/rssbundle/IEEETestCases/13Bus/IEEE13Nodeckt.rssbundle
```

IEEE8500 自动 EnergyMeter 区域现在保留电气求解并产生稳定的
`PSA.DSS.ENERGY_METER.AUTO_ZONE_APPROXIMATION` Warning；该路径不声明 meter-zone 寄存器与
OpenDSS 完全等价。

2026-10-21 对能力审计选出的 12 个优先入口做了定向复测：11 个完成 bundle 自执行并按官方
相对路径发布。`P174_Run_360kW_PV.DSS` 的 7,616 条语句可导入为 5 个 epoch，官方最终 epoch
与锁定 2,900 步数值投影的网络表、LoadShape 和分配表逐表等价；当时 release `--check` 和
staging 自执行分别运行 90 分钟和 30 分钟仍未完成，因此该轮没有取得完整数值结论。运行时间只作
批量扫描观察，不属于发布标准；后续复测已经把该入口定位为首次潮流求解失败。

2026-10-24 使用当前 release 重新检查时，导入及上述两项投影回归测试仍通过，但完整 `--check`
现在稳定在 epoch 0、`1 s` 的首次潮流求解失败，不再记录为长任务超时。该变化把下一步范围收敛为
执行求解回退排查；在恢复首个 Snapshot/QSTS 时刻并完成 2,900 步门禁前，发布状态保持不变。

| 优先入口 | bundle 发布状态 | 验证状态 |
|---|---|---|
| `IEEETestCases/4Bus-OYOD-Bal/4Bus-OYOD-Bal.DSS` | 已发布 | 两项独立 study 自执行通过 |
| `IEEETestCases/4Bus-YYD/YYD-Master-step1.DSS` | 已发布 | Newton 自执行通过 |
| `IEEETestCases/IEEE 30 Bus/Run_IEEE30.DSS` | 已发布 | 静态 Newton 自执行通过 |
| `IEEETestCases/8500-Node/Run_8500Node.dss` | 已发布 | Newton 自执行通过 |
| `IEEETestCases/8500-Node/Run_8500Node_Unbal.dss` | 已发布 | Newton 自执行通过 |
| `IEEETestCases/8500-Node/P174_Run_Voltage_Profile.DSS` | 已发布 | 7,598 条语句、2 个 epoch 自执行通过 |
| `Examples/Microgrid/GridFormingInverter/GFM_IEEE8500/Run_8500Node_Unbal.dss` | 已发布 | 与官方 unbalanced 入口同内容，自执行通过 |
| `IEEETestCases/ShortCircuitCases/IEEE123Master-SC.dss` | 已发布 | FaultStudy 414 个故障，自执行通过 |
| `IEEETestCases/ShortCircuitCases/ieee34Mod2_SC_Case_II.dss` | 已发布 | FaultStudy 148 个故障，自执行通过 |
| `Examples/HarmonicsTMode/IEEE_519.DSS` | 已发布 | HarmonicT 与跨 Solve 开关状态自执行通过 |
| `Examples/HarmonicsVariableLoad/IEEE_519.DSS` | 已发布 | HarmonicT 与跨 Solve Load 状态自执行通过 |
| `IEEETestCases/8500-Node/P174_Run_360kW_PV.DSS` | 未发布 | 导入和投影等价通过；当前完整 `--check` 在 epoch 0 首次潮流求解失败 |

## 与 PSA 当前能力的交叉评估

对基线失败的 249 个独立入口所做的二次诊断和 PSA 能力证据交叉审计见
[`capability_coverage_audit.md`](capability_coverage_audit.md)。该审计不再把首诊断等同于根因：
它列出 12 个优先入口的最新定向复测状态、8 个需要继续闭合完整脚本语义的候选，并记录对
路径类失败做最小修正后暴露出的第二层诊断。

后续 8 个候选中，IEEE34 的两个入口、三个 IEEE123 入口和 IEEE37 已完成正式改造、数值门禁
和 bundle 发布。剩余的 YYD 完整入口继续暴露 Direct/BatchEdit/Fault continuation 的求解边界。
InductionMachine 的 `LoadShape Action=Normalize`、固定步长连续时钟、Snapshot 初始化、保护动作和
Monitor/报告副作用现已完成重新验证，发布结果见“第二十二轮”记录。

### 第三轮实施结果

2026-10-22 完成前述三个范围，共发布 4 个新 bundle。所有入口均使用未修改的官方 `.dss`
源文件，生成后复制到独立临时目录再执行；数值门禁使用 OpenDSSDirect.py 0.9.4 / DSS C-API
0.14.5。

| 官方入口 | 执行规模 | OpenDSS 数值门禁 |
|---|---|---|
| `IEEETestCases/123Bus/SolarRamp.DSS` | 387 条语句，4 个 epoch、两个 300 点 Duty 段 | 电压最大相对误差 `2.44e-5`，Generator/Storage 功率最大相对误差 `8.22e-5`/`6.35e-5`，SOC 最大绝对误差 `3.04e-9`，7 组最终 tap 完全一致；低潮流 Monitor 最大缩放误差 `7.39e-3` |
| `Examples/Scripts/IEEE-TIA-LV Model/master_small_1phase.dss` | 21 条语句，1 个 Snapshot epoch | 6 个节点全部匹配；电压最大相对误差 `2.41e-6`，变压器绕组功率最大相对误差 `9.05e-5`，复损耗绝对误差 `0.106 VA` |
| `Examples/Scripts/IEEE-TIA-LV Model/master_large_1phase.dss` | 47 条语句，1 个 Snapshot epoch | 17 个节点全部匹配；电压最大相对误差 `2.85e-6`，变压器绕组功率最大相对误差 `2.62e-5`，复损耗绝对误差 `0.152 VA` |
| `IEEETestCases/8500-Node/Run_RecloserSiting.DSS` | 7,628 条语句，1 个 Newton epoch | MUMPS 发布构建自执行通过；8,531 个 OpenDSS 节点全部匹配，电压最大相对误差 `0.6052%`，源端功率误差 `0.387%`，复损耗误差 `1.49%`，均低于既有 IEEE8500 门限 |

SolarRamp 的修复覆盖 Storage 省略值、默认触发回滞、终端功率与 SOC，以及 `Mode` 对 Monitor
累计样本的重置。IEEE-TIA 修复覆盖单相两线 Delta/Wye 服务变压器的多端口建模、线圈 V/I、
损耗和 EnergyMeter 区域。RecloserSiting 要求安装包含 MUMPS 或 MKL 后端的 `tensoreval`；其
自动 EnergyMeter 区域继续按既有 approximation Warning 合同处理，不声明寄存器完全等价。

### 第四轮实施结果

2026-10-22 又发布三个 IEEE-TIA 完整分相入口。多绕组归一化现在把单相 Delta 一次侧的两个正相
端子识别为同一线圈端口，并保留两个次级半绕组在单相铁芯轴上的互阻抗；`Dump` 作为只读报告命令
记录为省略副作用。

| 官方入口 | 执行规模 | OpenDSS 数值门禁 |
|---|---|---|
| `Examples/Scripts/IEEE-TIA-LV Model/master_small.dss` | 23 条语句，1 个 Snapshot epoch | 9 个节点全部匹配；电压最大相对误差 `2.41e-6`，绕组功率最大相对误差 `9.05e-5`，复损耗相对误差 `1.82e-4` |
| `Examples/Scripts/IEEE-TIA-LV Model/master_large.dss` | 56 条语句，1 个 Snapshot epoch | 31 个节点全部匹配；电压最大相对误差 `2.85e-6`，绕组功率最大相对误差 `2.62e-5`，复损耗相对误差 `5.71e-5` |
| `Examples/Scripts/IEEE-TIA-LV Model/Split-Phase_IEEE_TIA.dss` | 22 条语句，1 个 Snapshot epoch | 7 个节点全部匹配；电压最大相对误差 `2.13e-6`，绕组功率最大相对误差 `3.96e-5`，复损耗相对误差 `7.34e-5` |

### 第五轮实施结果

2026-10-22 修复 `BusCoords/LatLongCoords` 的括号、引号及行尾注释路径解析，并为
`LoadShape Action=Normalize` 增加有界导入语义：P、Q 序列分别按最大绝对值归一化，且
`Normalize` 必须是该 LoadShape 的最后一次修改；其他 Action 或 Normalize 后继续修改会明确失败。
重扫 224 个未发布入口后新增一个可发布案例：

| 官方入口 | 执行规模 | 验证证据 |
|---|---|---|
| `Examples/Microgrid/GridFormingInverter/GFM_IEEE8500/Run_RecloserSiting.DSS` | 7,631 条语句，1 个 Newton epoch | 独立 bundle 自执行得到 11,038 行电压结果；除新增三条曲线定义导致 `solve_event_sequence` 加 3 外，全部物理结果与已通过 OpenDSS 门禁的 `IEEETestCases/8500-Node/Run_RecloserSiting.DSS` 逐值一致 |

两个官方入口的主文件 SHA-256 相同；GFM 目录的 Master 额外载入 `LoadShapes.dss` 和
`LoadShape1.CSV`，其中 `pvshape` 未被该 Snapshot 引用。因此本入口复用已锁定的 8,531 个
OpenDSS 节点门禁：电压最大相对误差 `0.6052%`、源端功率误差 `0.387%`、复损耗误差
`1.49%`。

### 第六轮实施结果

2026-10-22 为单步 Snapshot 增加两项有证据边界：`BatchEdit Load..* Daily=default` 在单步
Snapshot 中不改变负荷倍率；`InvControl mode=GFM` 在列出的 DER 都未进入 GFM 时只监测电流。
两项仅在整个执行计划均为单步 Snapshot、对象集合和属性均满足限定条件时省略，并返回稳定 Warning；
超出边界仍明确失败。另修正了低压侧作为绕组 1 的 Delta-Wye 升压变压器相位关系，ANSI/Euro
现在按额定高低压侧解释，不再依赖 DSS 绕组声明顺序。

| 官方入口 | 执行规模 | OpenDSS 数值门禁 |
|---|---|---|
| `Examples/Microgrid/GridFormingInverter/GFM_IEEE123/Run_IEEE123Bus_GFMSnap-A.DSS` | 386 条语句，1 个 Newton Snapshot epoch | 284 个节点身份全部一致；浮置 Delta 母线按相间可观测量去共模后，复电压最大相对误差 `9.09e-7`、RMS `2.97e-7` |

`Run_IEEE123Bus_GFMSnap-B.DSS` 在第二次求解前断开 `Line.SW5`，形成没有电压参考的孤岛，当前
返回 `TENSOREVAL.DSS.TRANSLATE.SOLVE_FAILED`；`Run_IEEE123Bus_GFMSnap-C.DSS` 随后把
`Storage.mystorage` 切换为真正的 `ControlMode=GFM`，仍由 `PSA.DSS.OBJECT.UNSUPPORTED` 阻断。
因此本轮只发布 A，未通过删除控制语句或静态化 B/C 来扩大成功数。

### 第七轮实施结果

2026-10-22 接入官方 `Examples/InverterModels/PVSystem/InvControl/volt-watt` 下 14 个
Snapshot 入口。转换器把 `CaseName` 作为仅影响输出文件名的选项处理，并接受官方使用的
`Maxi`/`MaxControli` 迭代参数缩写；PSA 导入器在“一个启用的纯 Volt-Watt InvControl、
一个 PVSystem、没有 Storage”时按 OpenDSS 规则补全省略的 `DERList`，并保留三点 XYCurve、
`0 < DeltaP_factor <= 1`、`PAVAILABLEPU`/`PMPPPU`/`PCTPMPPPU`/`KVARATINGPU` 基准、
`%Pmpp` 限幅和 PV 固定功率因数。

| 官方入口组 | 数量 | OpenDSS 数值门禁 |
|---|---:|---|
| `SnapShot_voltwatt_KVARATINGPU-2.dss` | 1 | 通过 |
| `SnapShot_voltwatt_noOperation*-2.dss` | 2 | 通过 |
| `SnapShot_voltwatt_PAVAILABLEPU*-2.dss` | 3 | 通过 |
| `SnapShot_voltwatt_PCTPMPPPU*-2.dss` | 2 | 通过 |
| `SnapShot_voltwatt_pf-2.dss` | 1 | 通过 |
| `SnapShot_voltwatt_PMPPPU*-2.dss`（不含 `PFPriority`/`WattPriority` 限值变体） | 5 | 通过 |

14 个 bundle 均由未修改的官方入口生成，复制到独立临时目录后使用 `tensoreval` 自执行成功。
OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5 比较覆盖全部 3 个节点的复电压和 PVSystem 终端
复功率；最大复电压相对误差为 `2.09e-5`，最大 PV 复功率相对误差为 `3.49e-4`，均低于
`1e-3` 门限。逐入口数据见 [`numerical_comparison.csv`](numerical_comparison.csv)。

Daily Volt-Watt 在第九轮按完整 24 时点门禁发布其中 6 个入口；当时未通过的 3 个固定 PF
变体已在第十轮闭合停机状态 P/Q 联动并发布。固定 PF、没有 InvControl 的 `PFPriority` 与 kvar/kVA 限值组合已在第十三轮闭合并发布；
带 InvControl 的 `PFPriority` 组合在本轮仍需独立语义和数值门禁；第二十三轮已闭合其中 4 个
单 PV、单 Volt-Watt 官方入口。

### 第八轮实施结果

2026-10-22 接入官方 `Examples/InverterModels/PVSystem/InvControl/volt-var` 下 6 个单 PV
Snapshot 入口。转换器只在“一个启用的纯 Volt-Var InvControl、一个 PVSystem、没有 Storage”
时按 OpenDSS 规则补全省略的 `DERList`，并保留五点 `VVC_Curve1`、`VARMAX`/`VARAVAL`、
`0 < DeltaQ_Factor <= 1`、`%Pmpp`、kvar 限值及 P/Q 优先级。DSS 未显式设置控制容差时，
导入结果会写入 OpenDSS 默认的 `VoltageChangeTolerance=1e-4`、
`VarChangeTolerance=0.025` 和 `ActivePChangeTolerance=0.01`，避免使用 PSA 通用容差改变终态。

| 官方入口组 | 数量 | OpenDSS 数值门禁 |
|---|---:|---|
| `SnapShot_voltvar_Standard*-2.dss` | 3 | 通过 |
| `SnapShot_voltvar_greater_kvA_*-2.dss` | 2 | 通过 |
| `SnapShot_voltvar_pctPmpp60-2.dss` | 1 | 通过 |

6 个 bundle 均由未修改的官方入口生成，复制到独立临时目录后使用 `tensoreval` 自执行成功。
OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5 比较覆盖全部 3 个节点的复电压和 PVSystem 终端
复功率；最大复电压相对误差为 `1.58135e-5`，最大 PV 复功率相对误差为
`2.15969e-4`，均低于 `1e-3` 门限。逐入口数据见
[`numerical_comparison.csv`](numerical_comparison.csv)。第九轮已发布 3 个 Daily Volt-Var、
1 个 Daily VV_VW 和 1 个本地最小电压监测入口；其余监测、Daily、多 DER、Storage 和组合控制
仍按各自数值门禁判定。

### 第九轮实施结果

2026-10-22 接入 10 个完整 Daily InvControl 入口和 1 个本地监测电压入口。PSA 导入器现在把
OpenDSS 省略的 PVSystem `%CutIn/%CutOut` 明确降低为 `20%/20%`，并支持 PVSystem terminal 1
的 Monitor mode 0、1、3；`Export/Plot` 继续作为不改变电气状态的输出副作用处理。所有发布
bundle 都由未修改的官方 `.dss` 生成并完成自执行。

| 官方入口组 | 数量 | OpenDSS 数值门禁 |
|---|---:|---|
| `volt-var/Daily_voltvar_{kvarlimit,pctPmpp60,Standard}-2.dss` | 3 | 24/24 时点通过 |
| `volt-watt/Daily_voltwatt_noOperation-2.dss` | 1 | 24/24 时点通过 |
| `volt-watt/Daily_voltwatt_{PAVAILABLEPU,PAVAILABLEPU_pctPmpp60,PCTPMPPPU,PMPPPU,PMPPPU_pctPmpp60}-2.dss` | 5 | 24/24 时点通过 |
| `VV_VW/Daily_voltvar_pctPmpp60-2.dss` | 1 | 24/24 时点通过 |
| `MonitoredVoltage/Local_voltage_min-2.dss` | 1 | 单工作点通过 |

Daily 案例逐时点比较全部节点复电压和 PVSystem 复功率；由于夜间功率接近零，功率误差使用
`|S_PSA-S_OpenDSS|/kVA_rated`。10 个 Daily 入口的最大电压相对误差为 `6.68761e-5`，最大
功率/额定容量误差为 `9.61056e-4`；`Local_voltage_min` 两项误差分别为 `2.13958e-4` 和
`9.84486e-4`，均通过 `1e-3` 门限。

当时另有 13 个入口已能严格转换和自执行但没有发布；其中 5 个已在第十轮通过。第九轮结束时剩余
8 个数值门禁失败入口：2 个 Daily Volt-Var average/rolling-average、3 个本地监测电压和
3 个远端监测电压。远端监测入口仍需补齐 `MonBus` 的相别、线电压和多母线选择。三个
`Mon_voltage_MAX_Mix*` 入口在旧扫描树中缺少 `PV_irrad.csv`，当时不计入上述 8 个；当前固定来源树已补齐该资产，第十二轮重新验证。

### 第十轮实施结果

2026-10-23 新增 5 个完整 Daily InvControl bundle。Volt-Var QSTS 现在只在设备曾触发视在功率
圆限制时保留该控制记忆；逆变器停机后的首个动作和同一时刻后续容差判断按 OpenDSS 顺序执行。
固定 PF Volt-Watt 在逆变器停机时将 Q 请求归零，避免低日照时继续输出固定功率因数无功。

| 官方入口组 | 数量 | OpenDSS 数值门禁 |
|---|---:|---|
| `volt-var/Daily_voltvar_greater_kVA_{ppriority,qpriority}-2.dss` | 2 | 24/24 时点通过；最大电压误差 `3.96518e-5`，最大 PV 复功率/额定容量误差 `5.28637e-4` |
| `volt-watt/Daily_voltwatt_noOperation_PF-2.dss` | 1 | 24/24 时点通过；最大电压误差 `1.73422e-8`，最大功率误差 `2.36657e-7` |
| `volt-watt/Daily_voltwatt_PMPPPU{,_pctPmpp60}_PF-2.dss` | 2 | 24/24 时点通过；最大电压误差 `1.45313e-5`，最大功率误差 `1.87305e-4` |

5 个 bundle 均由未修改的官方入口生成，复制到独立临时目录后使用发布版 `tensoreval`
自执行成功，每个输出 72 行逐时节点电压。17 个 Daily 候选重跑后，既有 10 个 Daily
通过项全部保持通过；`avg` 和 `ravg` 两个入口仍因平均/滚动监测电压语义偏差未发布。

### 第十一轮实施结果

2026-10-23 继续处理上一轮遗留的 8 个严格转换成功入口。PSA 现在保留
`Voltage_CurveX_Ref`、`AvgWindowLen`、`MonBus` 和 `MonBusesVBase`，远端监测列表可逐项选择
相地或相间电压及各自的额定电压。AVG/RAVG 滚动窗口只在下一 QSTS 时点开始时提交上一时点的
最终收敛电压，不再把同一时点的控制迭代重复计为样本；无功收敛判断使用 OpenDSS 的未松弛目标和
无功幅值差。

| 官方入口组 | 数量 | OpenDSS 数值门禁 |
|---|---:|---|
| `volt-var/Daily_voltvar_{avg,ravg}-2.dss` | 2 | 24/24 时点通过；最大电压误差 `3.81697e-5`，最大 PV 复功率/额定容量误差 `4.28307e-4` |
| `MonitoredVoltage/Local_voltage_2-2.dss`、`Local_voltage_average-2.dss`、`Local_voltage_max-2.dss` | 3 | 单工作点通过；最大电压误差 `5.93426e-7`，最大功率误差 `2.93004e-6` |
| `MonitoredVoltage/Mon_voltage_average-2.dss`、`Mon_voltage_average_LL-2.dss`、`Mon_voltage_average_Mix-2.dss` | 3 | 单工作点通过；最大电压误差 `4.62023e-6`，最大功率误差 `1.47363e-5` |

上述 8 个 bundle 均由未修改的官方入口生成，完成独立自执行并按官方相对路径发布。
`Local_voltage_max-2.dss` 的初次门禁暴露了 `WireData.GMR` 别名未进入 geometry bridge：PSA 当时按
`0.7788 × radius` 推导 GMR，使线路自电抗从 OpenDSS 的 `0.8781366 Ω/km` 变为
`0.8813033 Ω/km`。补齐别名并增加导入层 R/X/C 矩阵回归后，该入口的最大电压相对误差为
`5.93426e-7`，PV 复功率/额定容量误差为 `2.93004e-6`，在未放宽 `1e-3` 门限的情况下通过。

### 第十二轮实施结果

2026-10-23 从定向重扫结果继续验证 `Mon_voltage_MAX_Mix*` 三个完整 Daily 入口。固定官方来源树
现已包含 `PV_irrad.csv`，因此三个入口都能严格转换并独立自执行；本轮继续使用未修改的官方
`.dss`、20 个一秒时点、全部 9 个节点复电压和 PVSystem 终端复功率执行数值门禁。

| 官方入口 | 发布状态 | OpenDSS 数值门禁 |
|---|---|---|
| `MonitoredVoltage/Mon_voltage_MAX_Mix-2.dss` | 已发布 | 电压最大相对误差 `5.07365e-6`，PV 复功率/额定容量误差 `1.94619e-5` |
| `MonitoredVoltage/Mon_voltage_MAX_Mix_ravg-2.dss` | 已发布 | 电压最大相对误差 `6.96078e-5`，PV 复功率/额定容量误差 `5.51640e-4` |
| `MonitoredVoltage/Mon_voltage_MAX_Mix_avg-2.dss` | 当轮未发布；第二十三轮已发布 | 当轮电压误差 `6.33624e-4` 通过；PV 复功率/额定容量误差 `1.27260e-3` 超过 `1e-3` |

AVG 入口在本轮使用的两样本滚动窗口、上一时点终态提交和 MAX 监测电压聚合已逐项与 OpenDSS
`TRollAvgWindow` 及 `Vreg` 对照，窗口语义一致。剩余偏差来自 AVG 比值控制对底层潮流微小电压差的
放大；本轮没有放宽门限，也没有修改官方输入，因此当时保留为未发布入口。第二十三轮使用
当前 release 重新执行后已通过同一门限，最新结论见后文。

### 第十三轮实施结果

2026-10-23 接入官方 `Examples/InverterModels/PVSystem/NewFeatures/ConstantPF` 下 4 个
`PFPriority=yes` Snapshot 入口。当前边界限定为固定功率因数 PVSystem 且没有 InvControl：当
`%Pmpp`、kVA 或 kvar 限值生效时，PSA 对 P/Q 使用同一缩放比例，保持原功率因数；与 InvControl
组合的 `PFPriority=yes` 在本轮会返回专用诊断
`PSA.DSS.PVSYSTEM.PF_PRIORITY_INVCONTROL_UNSUPPORTED`，不会静默降级；第二十三轮已放行
满足固定 PF、单 PV、单 Volt-Watt 且未启用 WattPriority/LimitCurrent 的有界子集。

| 官方入口 | OpenDSS 总 P/Q | 数值门禁 |
|---|---:|---|
| `SnapShot_PFP_Standard-PV2.dss` | `1000.0 kW / 484.322105 kvar` | 电压最大相对误差 `3.38014e-10`，PV 复功率相对误差 `2.22259e-16` |
| `SnapShot_PFP_kVAlimitation-PV2.dss` | `900.0 kW / 435.889894 kvar` | 电压最大相对误差 `3.38014e-10`，PV 复功率相对误差 `2.61934e-16` |
| `SnapShot_PFP_kvarlimitation-PV2.dss` | `619.422481 kW / 300.0 kvar` | 电压最大相对误差 `3.38014e-10`，PV 复功率相对误差 `3.85832e-16` |
| `SnapShot_PFP_pctPmpplimitation-PV2.dss` | `600.0 kW / 290.593263 kvar` | 电压最大相对误差 `3.38014e-10`，PV 复功率相对误差 `1.30967e-16` |

4 个 ConstantPF bundle 均由未修改的官方入口生成，独立自执行得到 3 行节点电压，并使用
OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5 比较全部三相复电压和 PVSystem 终端复功率；均低于
`1e-3` 门限。

定向重扫同时确认 3 个显式 `PFPriority=no` 的 InvControl 入口已经越过旧属性阻断。
`SnapShot_voltwatt_PMPPPU_varP_kVAlimitation-2.dss` 和
`SnapShot_voltwatt_PMPPPU_wattP_kVAlimitation-2.dss` 的电压最大相对误差分别为
`4.16453e-8`、`3.28444e-6`，PV 复功率/额定容量误差分别为 `3.65641e-7`、`4.69323e-5`，
因此一并发布。`Daily_voltwatt_PMPPPU_wattP_kVAlimitation-2.dss` 当时虽然严格转换和 24 时点
自执行成功，但 PV 复功率/额定容量误差为 `2.62710e-3`，超过 `1e-3` 门限，暂未发布；该问题已在
第十五轮修复并通过门禁。
逐入口数据见 [`numerical_comparison.csv`](numerical_comparison.csv)。

### 第十四轮实施结果

2026-10-24 处理 LVTestCase 和 SolarRamp-Mack 两个路径/编排候选。执行计划现在只在整个入口是
单步 Snapshot、命令位于首次 `Solve` 前、后续所有 Load 都是显式 `New`，且每个 Load 都声明
`kW` 及 `PF` 或 `kvar` 时，将 `BatchEdit LoadShape..* UseActual=no` 降为无电气影响的 Warning；
任一条件不满足仍返回不支持诊断。

历史 Windows 绝对 `Redirect`/`Compile` 只有在调用者给出显式 `--source-root`，并且路径至少以
父目录和文件名形成唯一的、不区分大小写的最长后缀匹配时，才会重定位到根内依赖。扫描不跟随
符号链接，未找到、歧义或超出扫描上限都会明确失败。bundle 会把原绝对路径改写为已复制依赖的
相对路径，不保留生成主机路径。

| 官方入口 | 当前结果 | 验证证据 |
|---|---|---|
| `IEEETestCases/LVTestCase/Master.dss` | 已发布 | 1,044 条语句、1 个 Snapshot epoch；2,721 个节点全部匹配，复电压最大相对误差 `1.90437e-8`，源端复功率最大相对误差 `2.97820e-7`；bundle 自执行通过 |
| `IEEETestCases/123Bus/SolarRamp-Mack.DSS` | 当时未发布；当前已发布，见第二十五轮 | 390 条语句、4 个 epoch 和 4,602 个求解状态均可转换并自执行；当时的数值差异已定位为 `pv5sdata1.csv` 中间空记录的解析语义：OpenDSS 将空记录保留为零值采样点，旧解析器将其跳过并使后续曲线错位 |

SolarRamp-Mack 的两个 Snapshot epoch 均通过，Storage 功率和 SOC 也分别达到
`1.04665e-4` 和 `1.08907e-7` 的误差水平；首次调压档位差出现在 Duty 的 `9475 s`，Generator
曲线差异在 `12170 s` 首次出现并改变后续控制终态。后续复核发现官方 `pv5sdata1.csv` 第 2434
个物理记录为空：OpenDSS 将该记录读成 `0` 并保留其序号，旧解析器跳过该行，因此从此处开始
错位一个采样点。解析器现对 LoadShape `CSVFile`/`PQCSVFile` 的空白和纯注释记录生成零值采样点；
该轮修复没有改变时钟规则或数值门限；完整重跑和当前发布结论见第二十五轮。

### 第十五轮实施结果

2026-10-24 修复 InvControl 的 `ActivePChangeTolerance` 唯一属性前缀导入。官方入口使用
`activeP=0.00001`；RawDss 按 schema 将该前缀规范为 `activepchange_tolerance`，PSA 原先只读取
无下划线拼写，因而错误采用默认值 `0.01` 并提前结束 Volt-Watt 控制迭代。导入器现在同时识别
完整属性名和该规范化拼写，并用回归测试锁定执行计划中的控制容差。

`Examples/InverterModels/PVSystem/InvControl/volt-watt/Daily_voltwatt_PMPPPU_wattP_kVAlimitation-2.dss`
已从未修改的官方入口重新生成 bundle 并独立自执行，得到 24 个时点、72 行三相电压。与
OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5 比较时，节点和时点全部匹配，复电压最大相对误差为
`1.02725e-6`，PV 复功率绝对误差/额定容量为 `1.46549e-5`，均低于 `1e-3` 门限，现已发布。

### 第十六轮实施结果

2026-10-24 接入官方 `Examples/InverterModels/PVSystem/InvControl/VV_VW` 下 6 个单 PV
Snapshot 入口。OpenDSS 在这些文件中省略 `DERList`，并使用 `DeltaP_factor=0.45`、
`DeltaQ_factor=0.2`；导入器现在只在恰有一个启用的 InvControl、一个 PVSystem 且没有 Storage
时，把省略的成员表绑定到该 PV。单成员 VV_VW 接受运行时已有的有限正 `(0, 1]` 松弛因子，
多成员控制仍要求因子为 1。原始 Volt-Var 和 Volt-Watt XYCurve 向量会同时越过 DataFrame
边界，避免上游 typed profile 丢失末尾零值后破坏横纵坐标配对。

6 个 bundle 均由未修改的官方入口生成并独立自执行，每个得到 3 行节点电压。使用
OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5 比较全部三相复电压和 PVSystem 终端复功率：最大
复电压相对误差为 `1.05458e-5`，最大 PV 复功率绝对误差/额定容量为 `1.43711e-4`，6/6
均通过 `1e-3` 门限。Stevenson 三相 Model 3 没有随本轮放行；原始 `PVFactor=0.5` 和审计值
`0.005` 在 OpenDSS 默认 15 次迭代内都未收敛，且终态电压最大相差约 `1.2%`。

### 第十七轮实施结果

2026-10-24 接入与第十六轮同组的 6 个单 PV Daily VV_VW 入口。转换仍使用未修改的官方
`.dss`，每个 bundle 独立自执行得到 24 个时点、72 行三相电压。组合控制白天触及 inverter
kVA 圆时，OpenDSS 会把上一时点 Q 指令带入后续样本，并按 `DeltaQ_factor` 衰减到
`VarChangeTolerance` 门限；PSA 现把已有的 Volt-Var 跨时点触限记忆同样用于 VV_VW，避免在
日落样本边界直接把 Q 清零。

| 官方入口组 | 数量 | OpenDSS 数值门禁 |
|---|---:|---|
| `Daily_VVVW_{varAval_PMPPPU_kvarlimit,varMax_PMPPPU,varMax_PMPPPU_pctPmpp60,VWnooperation}-2.dss` | 4 | 24/24 时点通过；最大复电压相对误差 `3.81697e-5`，最大 PV 复功率/额定容量误差 `4.28307e-4` |
| `Daily_VVVW_varMax_PMPPPU_kVAlimitation_{Ppriority,Qpriority}-2.dss` | 2 | 24/24 时点通过；最大复电压相对误差 `3.96908e-5`，最大 PV 复功率/额定容量误差 `5.31789e-4` |

同轮把 `New EnergyMeter ... Action=C` 按 OpenDSS 的 Clear 初始化别名处理，不再误报为混合的
Solve 后计量动作。5 个 DOCTechNote 入口随后稳定暴露
`PSA.DSS.EXECUTION.ORDER_UNREPRESENTABLE`：它们在首次 `Solve` 后新增约 80 个 EnergyMeter
及其他网络对象，当前不可降为一个不可变 NetworkTables epoch，因此没有发布 bundle。

### 第十八轮复扫结果

2026-10-24 使用当前代码构建的 `tensoreval`，按 `conversion_report.csv` 保存的 `source_root`
重新检查全部 156 个未发布独立入口，没有新增通过数值门禁的案例。严格转换层只有
`Mon_voltage_MAX_Mix_avg-2.dss` 返回成功，但它原本就是“转换和自执行成功、数值门禁失败”：
重新完整比较 20 个时刻后，最大复电压相对误差仍为 `6.33624e-4`，PV 复功率/额定容量误差仍为
`1.272597e-3`，与上轮完全相同并继续超过 `1e-3` 门限。该历史结果已由第二十三轮当前 release
的重跑结果取代。

`SolarRamp-Mack.DSS` 也用当时程序重建并执行全部 4,602 个时刻。主 Duty 段的最大复电压相对误差
为 `0.06133667`，Generator 功率/额定容量误差为 `0.72335763`，首次调压器档位差出现在
`9475 s`。后续已确认这些历史误差源于空 CSV 记录被跳过；修复后的完整数值门禁已在第二十五轮
通过。三个 `AnnualSimulation.dss` 的 `CaseName` 已不再构成阻断，每个剩余
`Set` 诊断从 5 条降到 4 条，当前首要缺口是 `overloadreport`；GFM IEEE123 入口减少一条
`COMMAND_UNSUPPORTED`，但仍有对象、属性和执行能力边界。

`P174_Run_360kW_PV.DSS` 的当前状态发生了反向变化：导入和官方入口到锁定投影的两项测试仍通过，
但 debug 与新构建的 release 都在 epoch 0、`1 s` 的首次潮流求解失败；10 月 23 日旧 release
在同一输入上不会立即失败。报告现以 `TENSOREVAL.DSS.TRANSLATE.SOLVE_FAILED` 记录当前可复现结果，
后续应单独定位执行求解回退。发布总数保持 103，未发布数保持 156。

### 第十九轮暂态保护结果

2026-10-25 接入 `Examples/DistanceRelays` 的 4 个固定步长 Dynamic 入口。转换器现在接受
`ControlMode=Time`、零时钟 `Time=(0,0)`、Fault 的 OpenDSS 缺省值、Relay `Like` 链、
`RecloseIntervals=None`、`DebugTrace=No` 和只读 `Monitor Mode=32`。DataFrame V12 门面从固定
timeline 配置 TD21 外部采样时钟；TD21 在开断期间继续记录零电流历史，并区分被监测元件自身重合闸
的静默窗口。因此永久故障能够完成 4 次分闸、3 次重合并最终锁定。

| 官方入口 | 执行规模 | OpenDSS 数值与动作门禁 |
|---|---|---|
| `Examples/DistanceRelays/DistanceRelayTest.DSS` | 42 条 DSS 语句、120 个 10 ms Dynamic 时点 | 29 个故障/保护事件及 9 组 Relay 终态一致；事件时刻最大差 1 个采样；稳态区间电压/电流最大缩放误差 `3.33e-8` / `2.88e-8` |
| `Examples/DistanceRelays/ReverseDistanceRelayTest.DSS` | 42 条 DSS 语句、120 个 10 ms Dynamic 时点 | 与正向入口相同，反向监测 Relay 保持闭合；电压/电流最大缩放误差 `3.33e-8` / `2.88e-8` |
| `Examples/DistanceRelays/TD21RelayTest.DSS` | 41 条 DSS 语句、1,200 个 1 ms Dynamic 时点 | 29 个事件、动作顺序、动作次数和 9 组终态一致；永久故障重复动作的累计时钟差最大 4 ms；稳态区间电压/电流最大缩放误差 `3.33e-8` / `5.66e-8` |
| `Examples/DistanceRelays/ReverseTD21RelayTest.DSS` | 42 条 DSS 语句、1,200 个 1 ms Dynamic 时点 | 与正向入口相同，反向监测 Relay 不误动；电压/电流最大缩放误差 `3.33e-8` / `5.66e-8` |

数值比较使用 OpenDSS `Monitor.thev Mode=32` 与 V12 `bus_voltage`/`branch_current`。为避免把两个
求解器在离散事件边界上的先后顺序误记为电气误差，Distance 在 OpenDSS/PSA 任一事件前后 1 个
外部步长、TD21 在任一事件前后 5 ms 不进入稳态数值统计；其余分别比较 66/120 和 935/1,200 个
时点。四个 bundle 都从仓库复制到独立临时目录后完成自执行。

### 第二十轮实施结果

2026-10-26 接入官方 `Examples/InverterModels/PVSystem/InvControl/DRC` 和 `VV_DRC` 下 11 个
单 PV、20 时点 Daily 入口。OpenDSS 在这些入口中省略 `DERList`；导入器现在只在恰有一个
启用的 InvControl、一个 PVSystem 且没有 Storage 时推断唯一成员，多设备或多控制器场景仍要求
显式成员关系。DRC/VV_DRC 同时保留 `VARMAX`/`VARAVAL` 和运行时支持的
`0 < DeltaQ_Factor <= 1`。执行计划在展平 epoch 时会按声明文件取得父目录中的 CSV 资产，使用
稳定的内部名称重新附加，避免把 `../MonitoredVoltage/PV_irrad.csv` 错误解释为逃逸 bundle 根。

| 官方入口组 | 数量 | OpenDSS 数值门禁 |
|---|---:|---|
| `DRC/Daily_DRC{,_avg}-2.dss`、`Daily_DRC_kVAlimitation_{P,Q}-2.dss`、`Daily_DRC_kvarlimitation-2.dss` | 5 | 20/20 时点通过；最大复电压相对误差 `1.13169e-6`，最大 PV 复功率/额定容量误差 `1.58591e-5` |
| `VV_DRC/Daily_VVDRC{,_avg,_ravg}-2.dss`、`Daily_VVDRC_kVAlimitation_{P,Q}-2.dss`、`Daily_VVDRC_kvarlimitation-2.dss` | 6 | 20/20 时点通过；最大复电压相对误差 `5.91863e-6`，最大 PV 复功率/额定容量误差 `8.08173e-5` |

11 个 bundle 均由未修改的官方入口生成，包含跨目录引用的 `PV_irrad.csv`，复制到独立临时目录后
使用发布构建的 `tensoreval` 自执行成功；每个输出 20 个时点、60 行三相电压。数值比较使用
OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5，覆盖全部节点复电压和 PVSystem 终端复功率，节点及
时点身份全部一致，均通过 `1e-3` 门限。逐入口数据见
[`numerical_comparison.csv`](numerical_comparison.csv)。其中 AVG/RAVG 入口复用既有的
`Voltage_CurveX_Ref` 和 `AvgWindowLen` 运行合同，DRC 的动态无功电流窗口仍独立使用
`DynReacAvgWindowLen`。

### 第二十一轮 release 全量复扫

2026-10-27 使用重新构建的 release `tensoreval`（SHA-256
`aad0e573b2789e1a3cd9dc1bc940293261abf5be3854a74d2af6b6a75517089c`）重扫全部 141 个尚未发布的
独立入口；本轮没有编译程序。60 秒并行门禁得到 138 个明确失败、1 个严格转换通过和 2 个长任务超时。
唯一快速通过项仍是 `Mon_voltage_MAX_Mix_avg-2.dss`，其当时的 20 时点数值门禁为：电压误差
`6.33624e-4` 通过，但 PV 复功率/额定容量误差 `1.272597e-3` 超过 `1e-3`，因此当轮没有发布。
该历史结果已由第二十三轮当前 release 的重跑结果取代。
`SolarRamp-Mack.DSS` 是已知的 4,602 时点长任务；本轮批扫超时不表示能力回退。随后源码审计已将
既有数值失败定位到 LoadShape CSV 空记录语义并完成解析器修复，最终发布结论等待下一次 release
完整数值复验。

本轮有 5 个入口的诊断集合变化：`ADiakoptics/IEEE_123_Bus-G/Master.DSS`、
`Microgrid/ISource/Master.DSS` 和 `SecondaryTestCircuit_modified/Master.DSS` 去除了
`PSA.DSS.EXECUTION.COMMAND_UNSUPPORTED`，但仍分别受变压器/调压器、`Set` 或可靠性字段合同阻断；
`StoCtrl_Current_PeakShave/Add_Issues.dss` 去除了 `wait` 命令阻断，当前明确停在 22 个 Solve 后模型修改。
`StoCtrl_Current_PeakShave/master.dss` 清除了此前 66 条诊断并进入完整 8,760 时点 Yearly 执行；定向检查
持续运行 30 分钟、保持计算态且没有错误输出，随后为控制该轮批量扫描资源消耗而停止。该观察时限不是
发布标准；入口在严格转换、自执行和 OpenDSS 数值比较完成前不计入已发布案例。

因此发布总数仍为 118，尚未发布数仍为 141；本轮不新增 bundle，也不放宽任何数值门限。

### 第二十二轮 InductionMachine Dynamic 闭环

2026-10-27 使用按 `--no-default-features --features with_dsa_mumps,tensoreval-cli` 构建的
release `tensoreval`（SHA-256
`bdc6901cf3b8063c932c51fc87eba1db24065558d1588b43aab04bb7ef4adddc`）重新验证未修改的官方入口
`Examples/InductionMachine/Run.dss`。转换器展开 129 条 DSS 语句，完成一次 Snapshot 初始化和
5,000 步 Dynamic 连续执行；生成的 bundle 从自身目录独立执行成功，终点为
`t=0.833501667s`。15 个结果表均已物化，其中 `bus_voltage` 180,288 行、`motor_state` 15,024 行、
`protection_state` 85,170 行、`control_state` 20,032 行。

本轮修复的是多个 Relay 共享开断目标时的运行态：46/47 继电器在 `t=0.4s` 打开
`Line.MotorLeads` 后，尚未自行操作的 Voltage Relay 会在目标重新闭合前保持空闲；已经自行跳闸并
进入后续操作次数的 Voltage Relay 仍继续测量恢复电压，因此既有重合闸语义不变。保护事件记录为
`mfr46`、`mfr47` 在 `t=0.4s` 开断 `MotorLeads`，`rec1` 在 `t=0.5113748315297003s` 开断 `l3`。
四个调压器控制对象的终态与 OpenDSS 参考一致：`sub=-7/0.95625`，
`reg1a/reg1b/reg1c=0/1.0`。这项门禁只声明已比较的保护时序与调压器终态，不把它扩张为整条暂态
波形逐点等价。发布总数由 118 增至 119，尚未发布数由 141 降至 140。

### 第二十三轮 PFPriority 与 MonitoredVoltage 闭环

2026-10-27 使用按 `--no-default-features --features with_dsa_mumps,tensoreval-cli` 重建的
release `tensoreval`（SHA-256
`61db6faac092c11b0aff860b597952ee94083a23674e523d4fd4a456e489fa7c`），重扫 4 个此前由
`PSA.DSS.PVSYSTEM.PF_PRIORITY_INVCONTROL_UNSUPPORTED` 阻断的单 PV、单 Volt-Watt 入口。
当前有界合同要求 PVSystem 使用固定功率因数、`PFPriority=yes`，且不同时启用
`WattPriority` 或 `LimitCurrent`；更宽的控制器、成员关系和组合模式仍保持 fail closed。4 个入口
均由未修改的官方文件生成 portable bundle，独立自执行成功，且全部节点复电压和 PVSystem
终端复功率通过 `1e-3` OpenDSS 数值门禁：

| 官方入口 | 执行规模 | 最大复电压相对误差 | 最大 PV 复功率/额定容量误差 |
|---|---:|---:|---:|
| `volt-watt/Daily_voltwatt_PMPPPU_PFP_kVAlimitation-2.dss` | 24 时点、72 行电压 | `1.99972e-8` | `3.22005e-7` |
| `volt-watt/Daily_voltwatt_PMPPPU_PFP_kvarlimitation-2.dss` | 24 时点、72 行电压 | `3.76137e-10` | `2.32944e-9` |
| `volt-watt/SnapShot_voltwatt_PMPPPU_PFP_kVAlimitation-2.dss` | 1 时点、3 行电压 | `6.09015e-8` | `5.07875e-7` |
| `volt-watt/SnapShot_voltwatt_PMPPPU_PFP_kvarlimitation-2.dss` | 1 时点、3 行电压 | `4.32867e-9` | `3.32328e-8` |

同轮重新验证两个 `MonitoredVoltage` 入口。`Mon_voltage_MAX_Mix_avg-2.dss` 使用当前控制迭代
实现后，20 个时点的最大复电压相对误差为 `1.14326e-5`，PV 复功率/额定容量误差为
`4.97418e-5`；`Mon_voltage_MAX_Mix_avg_VVDRC-2.dss` 的对应误差为 `3.81785e-5` 和
`1.62955e-4`。两者的 9 个节点身份、20 个时点及收敛状态全部一致，portable bundle 自执行通过。
该结果取代第十二、十八和二十一轮对旧 release 的未发布结论，没有放宽 `1e-3` 门限。
逐入口原始指标见 [`numerical_comparison.csv`](numerical_comparison.csv)。发布总数由 119 增至
125，尚未发布数由 140 降至 134。

### 第二十四轮 Storage DebugTrace 与 EffCurve 闭环

2026-10-28 使用按 `--no-default-features --features with_dsa_mumps,tensoreval-cli` 重建的
release `tensoreval`（SHA-256
`56127dd321e97da9875abc19c20e27598ae56d36aeb8fa3f9a53c6a12ce0c29e`），重扫
StorageTechNote 和 StorageControllerTechNote 的 14 个独立入口。PSA 现按 OpenDSS 语义接入
`DebugTrace`、`EffCurve`、Daily 的 24×1h 默认调度、Monitor 终端默认值、Storage mode 1/3
监视器、跨 Solve 的 SOC 延续、`State`/`kW` 源顺序副作用、FOLLOW 恒 kvar 和 PF 空闲损耗。

下面 6 个未修改的官方入口均严格转换为 portable bundle、自执行成功，并与 OpenDSSDirect.py
逐时比较 24 个样本。门禁同时覆盖 `State`、`kWh`、`kWOut`、`kWIn`、`kvarOut`、`InvEff`
和三相 P/Q；能量按 500 kWh、功率按 50 kVA、效率按 1 归一化，统一门限为 `1e-3`：

| 官方入口 | 最大归一化误差 | RMS 归一化误差 | 最差通道 |
|---|---:|---:|---|
| `Example_9_1_Default/Storage_default.dss` | `2.42539e-5` | `4.17131e-6` | `Storage1.kvarOut@11h` |
| `Example_9_2_Follow/Storage_follow.dss` | `2.42533e-5` | `3.29827e-6` | `Storage1.Q3@2h` |
| `Example_9_3_Price/Storage_price.dss` | `2.42539e-5` | `4.17294e-6` | `Storage1.kvarOut@17h` |
| `Example_9_4_External/Storage_external.dss` | `2.67944e-8` | `7.87727e-9` | `Storage1.kWh@21h` |
| `Example_9_5_ReactivePower/Constantkvar_9_5_1/Storage_follow_kvar.dss` | `3.77232e-5` | `1.04712e-5` | `Storage1.kWh@20h` |
| `Example_9_5_ReactivePower/ConstantPF_9_5_2/Storage_follow_PF.dss` | `3.73155e-5` | `1.03063e-5` | `Storage1.kWh@6h` |

8 个 StorageControllerTechNote 入口已越过原先的 `EffCurve`/`DebugTrace` 阻断；本轮结束时首个
真实缺口统一为 execution-plan `BatchEdit`：关闭全部 CapControl/RegControl、为全部 Load 绑定
Daily 曲线；其中 PeakShaveDch/PeakShaveLow_Ch 还批量设置 `%stored=20`。这些入口当轮尚未发布；
后续处理结果见第二十七轮。发布总数由 125 增至 131，尚未发布数由 134 降至 128。

### 第二十五轮 59N 与 SolarRamp-Mack 闭环

2026-10-28 使用带 `with_dsa_mumps,tensoreval-cli`、禁用默认 feature 的 release
`tensoreval` 完成两个长任务入口的严格转换、portable bundle 自执行和 OpenDSS 数值门禁：

| 官方入口 | 执行规模 | OpenDSS 数值与动作门禁 |
|---|---:|---|
| `Examples/Scripts/59NRelayDemo.dss` | 3 个 Dynamic 时点；108 行母线电压、102 行支路电流、36 行源端功率、2 条事件 | 59N Voltage Relay 在 `0.4 s` 打开 `Line1`，与 OpenDSS 动作时刻和对象一致；`Delta.3` 三个时点最大电压相对误差 `8.73452e-2`，低于该暂态初始化案例的 `0.1` 门限 |
| `IEEETestCases/123Bus/SolarRamp-Mack.DSS` | 390 条语句、4 个 epoch、4,602 个求解状态；1,279,356 行相电压 | 电压最大相对/绝对误差 `2.34351e-5`/`0.059106 V`；Generator 额定值归一化误差 `2.13437e-4`，Storage 功率误差 `1.04665e-4`，SOC 绝对误差 `1.08907e-7`；32,214 条调压器状态逐条一致，Monitor 最大缩放误差 `1.30080e-3`，低于低潮流通道 `1e-2` 门限 |

SolarRamp-Mack 的最终修复包含两个独立时钟/控制语义。Time 模式 RegControl 在一次外部采样内只执行
到期的一档动作，并从当前求解时刻安排残余多档请求；OpenDSS 固定间隔 LoadShape 使用相对解时钟
向前偏移半个曲线间隔的取样语义。后者由 importer 在 `load_shape.open_dss_fixed_interval` 中显式
标记，普通 DataFrame 曲线默认关闭，不改变既有调用方的采样合同。本轮结果取代第十四、十八、
二十一轮对 SolarRamp-Mack 的历史未发布结论。

`Examples/StoCtrl_Current_PeakShave/master.dss` 已完成完整 8,760 步 Yearly 求解并发布 portable
bundle。PSA 与 OpenDSS 均在 8,760/8,760 个时点收敛；电压幅值最大相对误差为
`6.70610e-4`，SOC 最大绝对误差为 `3.24279e-3`，储能功率按 3 MW 额定值归一化的全年 RMS
误差为 `4.07144e-3`，控制器监测电流 RMS 相对误差为 `3.03941e-3`。8,760 个时点中有 9 个
控制状态差异，均位于电流死区或 SOC 饱和交界；相应单时点储能功率最大原始偏差为额定值的
`0.243731`，作为非门禁瞬态诊断保留。全年轨迹门禁取上述 RMS、SOC、电压和状态差异率，最大值
`4.07144e-3` 低于 `1e-2`。

该入口运行耗时较长；运行时间只记录为性能观测，不属于发布标准，也不影响数值正确时发布。

本阶段发布总数由 131 增至 134，尚未发布数由 128 降至 125。逐入口原始指标见
[`numerical_comparison.csv`](numerical_comparison.csv)。

### 第二十六轮 IEEE123 逆潮流调压器

2026-10-28 重新扫描旧报告中以 `READ.DSS.RETAINED_SOURCE_ONLY` 为首诊断的独立入口，
`IEEETestCases/123Bus/RevRegTest.dss` 已能严格转换为 488 条语句、5 个 execution epoch，并生成
Newton portable bundle。两个 Snapshot epoch 和三个各 24 步的 Daily epoch 共形成 74 个求解
时点、20,572 行相电压；bundle 在独立结果目录自执行成功。

OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5 逐时比较全部 278 个节点，节点身份和时点全部一致，
复电压最大/RMS 相对误差为 `1.43082e-5`/`2.39518e-6`，最大绝对误差为 `0.034904 V`。
7 组 RegControl 的 518 条逐时 tap 状态全部一致；最终 `reg4a/reg4b/reg4c` 在反向功率下均回到
中性档。发布总数由 134 增至 135，尚未发布数由 125 降至 124。

### 第二十七轮 StorageControllerTechNote 首批闭环

2026-10-29 为 execution plan 增加有界 `BatchEdit` 展开，并按 OpenDSS 行为补齐显式时刻
LoadShape 的日周期边界、StorageController 默认 `ResetLevel`、Schedule 零功率触发态、达到
能量边界后的运行态以及激活但零终端功率时的内部空载损耗。进一步按 OpenDSS EventLog 校准
测量型控制器的一次采样一次调度语义，并区分控制器总功率指令与扣除各设备端口空载损耗后的
终端功率。下面 4 个未修改的官方入口均完成
严格转换、24 个逐时 Solve、portable bundle 自执行和 OpenDSSDirect.py 数值比较：

| 官方入口 | Storage 状态/能量/功率最大归一化误差 | 首端三相 P/Q 最大绝对误差 | 状态门禁 |
|---|---:|---:|---|
| `Follow/FollowRun.dss` | `8.47882e-4` | `0.678773 kVA` | 7 台 Storage、24 时点全部一致 |
| `LoadShape/LoadShapeRun.dss` | `1.51825e-5` | `0.634953 kVA` | 7 台 Storage、24 时点全部一致 |
| `Schedule/ScheduleRun.dss` | `9.43322e-6` | `0.630765 kVA` | 含零功率触发时刻，全部一致 |
| `Time/TimeRun.dss` | `1.55640e-5` | `0.630765 kVA` | 7 台 Storage、24 时点全部一致 |

门禁覆盖 `State`、`kWh`、`kWOut`、`kWIn`、`kvarOut`、`InvEff` 和首端三相 P/Q；Storage
通道按设备额定容量归一化，首端功率按 1 MVA 比较基准归一化，统一阈值 `1e-3`。四个入口的
最大门禁值分别为 `8.47882e-4`、`6.34953e-4`、`6.30765e-4` 和 `6.30765e-4`。新生成 bundle 的默认
`bus_phase` 输出均为 264,912 行，并已在独立结果目录自执行通过。

该轮结束时，其余 4 个 StorageControllerTechNote 入口都已能严格转换和执行，但尚未达到数值发布门禁：
两个 `PeakShave` 和组合的 `PeakShave/PeakShaveLow` 在
SOC/死区边界仍有状态及分配差异；`Support` 的 `InhibitTime` 生命周期仍提前进入放电。它们继续
当时保留为未发布入口，不以转换成功代替 OpenDSS 数值一致；后续闭环见第二十八轮。发布总数由 135 增至 139，
尚未发布数由 124 降至 120。

### 第二十八轮 StorageControllerTechNote 迭代控制闭环

2026-10-29 继续按 OpenDSS EventLog 校准测量型 StorageController：单轮权重分母保持完整声明群组，
`PeakShave`、`PeakShaveLow` 和 `Support` 在每次实际调度后重新求解并复测，活动控制进入死区后停止，
`Support` 在反向潮流绝对值不足目标时放电补足；达到功率边界的设备保留完整 AC 端口限值。其余 4 个
未修改官方入口均完成严格转换、24 个逐时 Solve、portable bundle 自执行和 OpenDSSDirect.py 数值比较：

| 官方入口 | Storage 连续通道最大归一化误差 | 全部连续通道 RMS | 首端三相 P/Q 最大绝对误差 | 状态门禁 |
|---|---:|---:|---:|---|
| `PeakShave/PeakShaveMonPhaseRun.dss` | `1.21031e-2` | `1.04517e-3` | `5.39232 kVA` | 7 台 Storage、24 时点全部一致 |
| `PeakShave/PeakShaveRun.dss` | `1.26804e-2` | `9.67311e-4` | `2.99179 kVA` | 7 台 Storage、24 时点全部一致 |
| `PeakShaveDch_PeakShaveLow_Ch/PeakShaveDch_PeakShaveLow_ChRun.dss` | `1.26809e-2` | `1.82735e-3` | `2.99179 kVA` | 1/168 个状态位于 0.242 kW 充电/待机边界 |
| `Support/SupportRun.dss` | `1.20909e-2` | `3.92620e-4` | `2.58111 kVA` | 7 台 Storage、24 时点全部一致 |

这些闭环控制会在死区内停止，OpenDSS 与 PSA 的基础潮流微差会使最后一次群组分配落在死区内的不同位置。
因此本轮沿用 Yearly 控制案例的轨迹门禁：连续通道最大归一化误差不超过 `2e-2`，同时记录 RMS、
馈线功率和边界状态差异；不要求死区内的单设备终点逐值相同。四个入口的最大连续误差、RMS 和馈线
归一化误差均通过该门限，全部 96 个逐时 Solve 收敛。组合案例唯一状态差异对应 OpenDSS 的
`Storage.C` 仍以 0.242 kW 充电，而 PSA 已进入待机；该点连续功率差为额定值的 `1.14431e-2`，
未改变控制目标、SOC 边界或后续放电轨迹。四个正式 bundle 的默认 `bus_phase` 输出均为 264,912 行，
已在独立结果目录自执行通过。发布总数由 139 增至 143，尚未发布数由 120 降至 116。

### 第二十九轮 InverterTechNote Storage kVA 阈值闭环

2026-10-29 发布未修改官方入口
`Examples/InverterTechNote/kVA/11_1_kVAViolation.dss`。执行型导入现保留 Storage
`%kWRated`、`%PMinNoVars` 和 `%PMinkvarMax`：`%kWRated` 限制额定有功范围，后两个阈值按
`kWrated` 定义运行态无功能力的零值区和线性恢复区。Follow 调度在实际越过 `%CutIn` 前保持
`IDLING`；EffCurve 空载损耗由电网承担，不改变电池能量，Storage Monitor 的逆变器、空载和
充放电损耗也按相同 DC 侧功率边界分解。

24 个逐时 Solve 全部收敛，Storage 状态位完全一致；连续状态通道最大归一化误差为
`8.49630e-4`，全部连续通道 RMS 为 `9.44036e-5`，首端三相电压最大归一化误差为
`1.40126e-5`，均通过 `1e-3` 门限。正式 portable bundle 的默认 `bus_phase` 输出为 72 行，
已在独立结果目录自执行通过。发布总数由 143 增至 144，尚未发布数由 116 降至 115。

### 第三十轮未发布入口分层复扫

2026-10-29 使用 release `tensoreval`（SHA-256
`b9502e25cd89e4974142bcf34b718d0397e5ff4b99d24abc76105f41efd9b51a`）对当前 115 个未发布
独立入口增加一轮“仅转换、不求解”复扫。固定上游仍为
`electricdss-tst@3b208397160213cae4a9e2d0a7d1aa3528ce26e1`，所有入口统一使用
`Version8/Distrib` acquisition root。115 个入口均在 30 秒内得到确定结果，其中 21 个已经能生成
linked RSS，0 个超时；这只证明当前 importer 和 RSS emitter 能表达输入，不等于 bundle 自执行或
OpenDSS 数值门禁通过。

对 21 个入口继续做运行层探针后，候选范围收敛如下：

- 7 个 Dynamic 入口已经越过生命周期分类器。Kundur DynamicExp 停在 Dynamic 启动后的
  `New Fault`/`Disable Fault`/`Open Line` 网络修改；5 个 IEEE123 GFL/GFM 入口和 1 个 Wind GFL
  入口先停在 Dynamic adapter 尚未复用有界 `BatchEdit Load..* Daily=default`。DSA 运行时已经有
  typed fault on/off 和 topology open/close 事件，因此这组优先调查 importer 到 typed event 的
  有序 lowering，而不是新增暂态方程。
- `Examples/AutoTrans/AutoHLT.dss` 已能生成 8 个 execution epoch，运行时在第一个三相短路工作点
  返回 fault-study protection measurement 求解失败。原脚本只使用现有三绕组变压器、Fault、Load
  和源/线路切换，且注释给出电流、损耗和满载电压参考，是一个边界集中的 Snapshot fault
  continuation 候选。
- `IEEETestCases/NEVTestCase/Run_NEV.dss` 的主模型只包含现有线路、变压器、负荷、接地和电容；
  当前唯一首层阻断是同一入口依次执行 Snapshot、FaultStudy、Harmonic。三个 adapter 已分别存在，
  后续工作是定义多 study bundle 的结果命名和 continuation，而不能把生命周期简单改名。
- `IEEETestCases/8500-Node/P174_Run_360kW_PV.DSS` 当前可以生成 RSS；60 秒执行探针没有重现旧报告中
  epoch 0、`1 s` 的即时失败，也没有在观察窗内完成。它继续作为 2,900 步独立长门禁，不以本轮
  无错误输出判为通过。
- ckt24 的两个 AnnualSimulation、`Examples/Scripts/AnnualSimulation.dss` 和
  `EPRITestCircuits/ckt7/RunDSS_ckt7.dss` 也已能生成 RSS。它们应进入独立 Yearly 预算和数值轨迹
  门禁，不与快速兼容扫描共享超时结论。

本轮同时排除了若干仅看“生成 RSS”会产生的假阳性：4 个 VCCS Dynamic 入口运行时仍明确缺少
VCCS 对象模型；Paulo 的两个 `solve_*` 和 ExpControl 的 `SolveAndExportMonitors.dss` 是依赖父脚本
先构网的片段，单独执行没有有效拓扑；GFM Snapshot-B 仍在断开 SW5 后形成无电压参考孤岛。
InverterTechNote 的 `11_2_kWRatedViolation.dss` 和 `11_3_ReactivePowerViolation.dss` 则未进入上述
21 个：它们分别需要 Storage 充/放电 Volt-Watt 曲线和 Storage-target VV_DRC。现有原生 singleton/
overlap 证据已证明这不是只改 DERList 校验即可完成的接入，必须把 InvControl 动作接入同一 Storage
idle loss、SOC、dispatch state 和区间推进运行时。

本轮没有新增正式 bundle，发布总数保持 144，尚未发布数保持 115。更详细的候选排序和排除依据见
[`capability_coverage_audit.md`](capability_coverage_audit.md)。

### 第三十一轮 Dynamic 原生命令 lowering

2026-10-29 按第三十轮的第一优先项直接改进 DSA。`eig-rs` master 的
`aaef84412`、`282b1ddeb` 和 `203bba69f` 依次加入了三层有界语义：Dynamic 启动前复用
execution plan 的 `BatchEdit` 展开；启动后把 `New Fault`、Fault enable/disable 和整元件
`Open`/`Close` 降为保持源顺序与累计求解时刻的 typed event；把官方脚本使用的 `StepTime`
别名映射到固定步长 `StepSize`。超出边界的运行时属性修改、选相开断、非零 `Time=(h,s)` 和动态
LoadShape 采样仍然 fail closed。

使用 release `tensoreval`（SHA-256
`2470c55251fb109a176b917521cd3b1eb85f5690e6b60fc4b43d25863802a089`）和固定上游
`electricdss-tst@3b208397160213cae4a9e2d0a7d1aa3528ce26e1`，以 `Version8/Distrib` 为
acquisition root 重新执行全部 7 个 Dynamic 候选的 `--dss-to-rss --check`：

| 官方入口组 | 已越过的首诊断 | 当前最深诊断 | 结论 |
|---|---|---|---|
| `Dynamic_KundurDynExp.dss` | 启动后的 `New Fault`、`Disable Fault`、`Open Line` | `PSA.DSS.GENERATOR.DYNAMIC_EXTERNAL_MODEL_UNSUPPORTED` | typed event lowering 已生效；剩余的是外部 Generator dynamic/shaft 模型，不是命令编排缺口 |
| 两个 IEEE123 GFL 入口 | `BatchEdit Load..* Daily=default`、`StepTime` | `PSA.DSS.DYNAMIC.SET_UNSUPPORTED`；按源顺序首个未映射设置为 `LoadShapeClass=Daily` | 后续还有非零 `Time=(10,0)` 和动态负荷曲线消费，不能把设置静默忽略 |
| 三个 IEEE123 GFM 入口 | `BatchEdit Load..* Daily=default` | `PSA.DSS.DYNAMIC.COMMAND_UNSUPPORTED`；首个未映射命令为初始 Snapshot 后、Dynamic 前的 `Open Line.SW5 terminal=1` | 需要跨 Snapshot 到 Dynamic 的拓扑 continuation；随后仍有 Storage GFM/AmpLimit 边界 |
| Wind GFL Dynamic | 运行时 Fault on/off typed event | `PSA.DSS.DYNAMIC.PROFILE_CONSUMER_UNSUPPORTED` | `LoadShape.default` 没有当前 native dynamic runtime 可消费的采样输入 |

聚焦 importer 回归为 12/12 通过；V12 集成测试还断言了 `line_model=Exact`，并验证 Fault on/off 与
topology open 的事件行和时刻。完整 `psa --features dsa --lib` 本轮得到 1,359 通过、5 个既有
Generator/Harmonic 测试失败、2 个忽略；5 个失败单独复跑仍失败，且不经过本轮改动的 DSS importer
路径，因此没有以完整套件全绿作为结论。

这 7 个入口都尚未完成 portable execution 和 OpenDSS 数值门禁，本轮不改
`conversion_report.csv`，不新增 bundle；发布总数仍为 144，尚未发布数仍为 115。下一轮优先级从
“继续放宽命令白名单”调整为：先实现可取证的 Dynamic LoadShape 采样与时钟初始化；GFM 则单独定义
Snapshot→Dynamic topology continuation 和 Storage GFM 方程合同。

#### 三个同名 `IEEE_519.DSS` 的边界

三个文件名称相同，但入口合同和发布结论不同：

| 官方入口 | `.dss` 自身执行内容 | 当前结论 |
|---|---|---|
| `Examples/HarmonicsTMode/IEEE_519.DSS` | 进入 HarmonicT，跨多次 `Solve` 依次打开两个开关 | 完整 HarmonicT 入口，bundle 已发布 |
| `Examples/HarmonicsVariableLoad/IEEE_519.DSS` | 进入 HarmonicT，跨 `Solve` 修改负荷状态 | 完整 HarmonicT 入口，bundle 已发布 |
| `Examples/Matlab/HarmonicT_MATLAB/IEEE_519.DSS` | 只构造网络并执行初始化 `Solve` | 不列入本批；时间循环由同目录 `Main_Script.m` 通过 COM API 驱动，单独转换只能声明 Snapshot 初始化一致 |

因此不能用 Matlab 目录下文件的转换结果声明完整 Matlab HarmonicT 工作流等价，也不应把它作为
第三个 HarmonicT bundle 增加已覆盖案例数。若以后接入该工作流，需要先定义 Matlab 外部命令序列
到 RSS study 的独立转换合同和数值门禁。

基线失败不能解释为 PSA 没有覆盖这些网络和算法。当时的转换链在 PSA 前后又维护了
命令分类、Snapshot `Set` 白名单和诊断白名单；这些规则已经落后于 PSA 的 DSS importer 与执行计划。
按“同一网络/算法或 PSA execution projection 已有数值证据、改造前首要失败点位于转换/编排层”
这一口径，基线扫描中的下面 19 个入口曾被确定为**底层功能有覆盖，但基线转换没有成功**。
表中的阻断原因是改造前证据，不表示当前定向复测结果；已经复测的入口以上文“首轮改造进展”为准：

| 已有 PSA 覆盖 | 基线未成功的官方入口 | 基线首要阻断 | 判断 |
|---|---|---|---|
| IEEE 4 八种接线的 V/I/P/Q/loss 与状态 | `IEEETestCases/4Bus-DY-Bal/4Bus-DY-Bal.DSS`、`IEEETestCases/4Bus-GrdYD-Bal/4Bus-GrdYD-Bal.DSS`、`IEEETestCases/4Bus-YD-Bal/4Bus-YD-Bal.DSS`、`IEEETestCases/4Bus-YY-Bal/4Bus-YY-Bal.DSS`、`IEEETestCases/4Bus-YYD/YYD-Master-step1.DSS` | `Set EarthModel=Carson` 被 `dss2rss` 的 Snapshot 白名单拒绝 | PSA importer 已处理相同设置；属于重复白名单造成的转换缺口。 |
| IEEE 4 open-Y/open-delta 数值案例 | `IEEETestCases/4Bus-OYOD-Bal/4Bus-OYOD-Bal.DSS` | `Solve` 的位置参数被执行计划拒绝 | 网络和求解已有数值覆盖；还需补 OpenDSS `Solve` 位置参数规范化。 |
| IEEE 13 原始模型、控制、潮流、短路和谐波 | `IEEETestCases/13Bus/IEEE13Nodeckt.dss` | Snapshot 投影后找不到 `IEEELineCodes.DSS` | 原始 `Source` 能取得依赖；转换器重建内存 `Source` 后丢失了相对路径上下文。 |
| IEEE 34 原始调压网络 | `IEEETestCases/34Bus/Run_IEEE34Mod1.dss`、`IEEETestCases/34Bus/Run_IEEE34Mod2.dss` | 单个 Snapshot `Solve` 被送入静态 importer，RegControl 被判为未执行 | 应走 PSA 的有序执行计划与控制求解，而不是静态投影。 |
| IEEE 37 active open-delta RegControl | `IEEETestCases/37Bus/ieee37.dss` | `Set maxiterations=100` 被 Snapshot 白名单拒绝 | PSA 已闭合终态电气量和 tap；迭代预算需要映射到 PSA 参数。 |
| IEEE 123 调压与组合控制 | `IEEETestCases/123Bus/Run_IEEE123Bus.DSS`、`IEEETestCases/123Bus/P174_Run_VoltageProfile.DSS` | 与 IEEE 34 相同，错误选择静态 importer | 底层网络和控制已有数值覆盖；入口仍需在修复后完整复测后续报告命令。 |
| EPRI J1 静态、三时段 QSTS 与 RelCalc | `EPRITestCircuits/epri_dpv/J1/Master_noPV.dss`、`EPRITestCircuits/epri_dpv/J1/Master_withPV.dss` | `Set maxiter=100` 被 Snapshot 白名单拒绝 | PSA 已有相同原始入口的数值证据；迭代预算未从 DSS 映射到 PSA 参数。 |
| IEEE 8500 原始 balanced/unbalanced 网络 | `IEEETestCases/8500-Node/Run_8500Node.dss`、`IEEETestCases/8500-Node/Run_8500Node_Unbal.dss` | `Set maxiterations=20` 被 Snapshot 白名单拒绝 | PSA 已直接导入原始 master 并闭合单步数值。 |
| IEEE 8500 两工作点与 2,900 步 duty | `IEEETestCases/8500-Node/P174_Run_Voltage_Profile.DSS`、`IEEETestCases/8500-Node/P174_Run_360kW_PV.DSS` | retained reliability warning 被当成错误；Snapshot→Duty 被分类器一概判成 mixed study | 前者是诊断严重性使用错误，后者已由 PSA 多 epoch/continuation 合同覆盖。 |
| IEEE 118 Generator 网络 | `IEEETestCases/IEEE118Bus/master_file.dss` | `Mode=Snap` 未归一化为 Snapshot | `snap` 是解析别名缺口。PSA 已覆盖模型和独立 PYPOWER 数值链；原生 OpenDSS 53-Generator deck 本身仍是不收敛边界，不能据此声明原脚本结果一致。 |

下面几组只能判定为“底层原语存在、入口尚未闭环”，不能计入上述 19 个确定案例：

- 三个 `Mode=FaultStudy` 官方入口均已由确定性故障枚举和 typed 结果主键接入；IEEE37 新增
  175 个全母线场景的独立 OpenDSS 数值参考。
- IEEE 519 两个优先入口已经通过；其他谐波入口仍需逐条核对脚本命令、频率集合和输出语义。
- 26 个 mixed-study 入口多数是 Snapshot 初始化后转 Daily/Duty。PSA 支持多 epoch 和跨时点状态，
  但其中还包含 GFM、Storage、UPFC 等不同对象，取消分类器的统一拒绝后仍需逐文件复测。
- 27 个 `REQUEST.SOURCE.ESCAPES_ROOT` 入口的首要问题是 epoch/内存 `Source` 没有保留声明文件的
  目录身份。路径修复只消除第一个阻断；MemoryMapping、二进制 Sng/Dbl 和部分 Storage 行为仍可能
  落在 PSA 的明确边界外。
- DynamicEq/DynOut、VCCS、用户 DLL、EMT、V12 未覆盖的 IBR 暂态、AutoTrans 未支持接法、
  特殊节点编号和两个求解器都拒绝的拓扑属于当前真实能力边界。普通 Dynamic/Dynamics 已有
  有界 adapter，仍须逐入口通过转换、自执行和数值门禁后才能列为已发布案例。

## 改造路线和状态

1. **已完成：所有带 `Solve` 的入口交给 PSA execution plan。** 单次 Snapshot、Snapshot 初始化后
   继续 Daily/Duty 均由同一有序执行合同处理；`StaticPowerFlow` 只用于显式指定的纯模型 Snapshot。
2. **首轮已完成：求解设置由 PSA 统一解释。** `EarthModel`、`Algorithm`、
   `maxiter/maxiterations`、`maxcontroliter/maxcontroli` 和 `tolerance` 已纳入执行合同；写在 `Set`
   或最终 `Solve` 上的迭代参数都会进入实际潮流参数。`miniterations` 尚无 PSA 等价语义，继续明确失败。
   旧 `snapshot_set` 代码只保留给静态投影兼容测试，原生 `Solve` 不再经过该白名单。
3. **已完成：暴露 `tensoreval --study`。** 支持 `snapshot`、`execution`、`harmonic`、`auto-add`、
   `relcalc` 和 `dynamic`；`--study snapshot` 可为纯模型入口指定单次静态研究。
4. **已完成：以 PSA 的机器可读诊断合同为准。** `dss2rss` 以 import profile 返回的 `severity`
   决定 Warning 放行、Error 阻断，不再解析诊断码或英文 message。
5. **部分完成：保留 Source 图和声明目录。** 未删改的纯模型直接沿用原始 `Source`；execution epoch
   会重新附加坐标文件和已识别的数据资产。跨目录 `../`、MemoryMapping 与二进制 Sng/Dbl 仍需在
   当前 115 个尚未发布入口中逐项验证。
6. **已完成分类层改造：让 PSA 判断连续执行是否可表达。** Snapshot+Daily/Duty 不再被分类器统一
   拒绝；具体状态转换仍由 execution plan fail closed。
7. **部分完成：为非电气副作用定义显式策略。** `BusCoords/LatLongCoords` 已保留资产并产生稳定
   Warning，`Plot/Show` 已作为不改变电气状态的输出副作用处理。`Export/CaseName/CPU/Parallel`
   的 calculation/exact 模式和 bundle manifest 清单仍待补齐。`Open/Close/Enable/Disable/BatchEdit`
   继续按电气状态修改处理。
8. **已完成三个优先入口的 FaultStudy 适配。** 已规定母线、故障类型、相别、0.1 Ω 默认故障电阻和结果
   长表主键，并通过 IEEE123、IEEE34 和 IEEE37 官方入口及对应数值参考。

对应代码落点如下：

| 位置 | 修改职责 |
|---|---|
| `mems/src/bin/dss2rss/classify.rs` | 归一化 `snap`；取消单 Snapshot 静态化和可表达 continuation 的预拒绝；分类只选择 PSA adapter，不重复验证对象语义。 |
| `mems/src/bin/dss2rss.rs` | 原生 `Solve` 绕过 `snapshot_set`；移除基于 message 的安全白名单；未删改纯模型保留原始 Source。 |
| `mems/src/bin/dss2rss/compiler.rs` | 所有原生 `Solve` 优先调用 `import_dss_execution_plan`；把 DSS 求解预算传入生成 RSS 的 `para`；保留 PSA Error 原样诊断。 |
| `mems/src/bin/tensoreval.rs` | 增加并记录 `--study`；帮助文本、错误提示、check 和 bundle 模式使用同一枚举。 |
| `psa/src/dsa/parse/dss_import.rs` | 统一区分网络全局设置、求解参数和研究调度；`execution_epoch_source` 保留 referrer directory；需要时增加稳定的诊断 disposition。 |
| `mems/src/bin/dss2rss/bundle.rs` | 在 calculation 模式的 manifest 中记录被省略的非电气命令及源位置，bundle 复跑时校验同一清单。 |

后续回归门禁应直接使用上述已识别官方入口，并把验证分成三层：转换和 bundle 自执行、PSA 与
OpenDSS 的 V/I/P/Q/loss/control 数值比较、被省略副作用的 manifest 精确比较。每修复一个首要阻断都要
重扫当前 115 个尚未发布入口，因为当前报告只保证首个稳定诊断，不能据此推断该文件后面没有第二个缺口。

## 主要原因

`primary_diagnostic` 是当前 import profile 下阻断转换的第一条诊断；同一诊断码在其他 profile
中可能只是 Warning。`all_diagnostic_codes` 保留该文件本次失败返回的完整诊断集合。

| 数量 | 分类 | 诊断码 | 原因 |
|---:|---|---|---|
| 14 | `unsupported` | `TENSOREVAL.DSS.CLASSIFY.NATIVE_LIFECYCLE_UNSUPPORTED` | 保存的完整检查请求 Dynamic、Direct 等尚未完成入口合同的生命周期；第三十轮仅转换探针已把其中部分 Dynamic 入口推进到运行层诊断，Direct 仍未支持。 |
| 13 | `unsupported` | `READ.DSS.RETAINED_SOURCE_ONLY` | 属性只保留在源信息中，当前计算合同不会消费。 |
| 9 | `unsupported` | `PSA.DSS.EXECUTION.INCLUDE_LOAD_FAILED` | 展开后的 include 在 execution epoch 中无法便携重建。 |
| 8 | `unsupported` | `REQUEST.SOURCE.ESCAPES_ROOT` | 数据资产引用越过本次允许的获取根。 |
| 7 | `unsupported` | `PSA.DSS.NODE.PHASE_LABEL_UNSUPPORTED` | 节点/相导体编号超出当前网络表约束。 |
| 6 | `unsupported` | `PSA.DSS.LINE.GEOMETRY_INVALID` | LineGeometry 引用或导体位置不能形成有效线路参数。 |
| 6 | `unsupported` | `PSA.DSS.EXECUTION.ORDER_UNREPRESENTABLE` | 首次 Solve 后新增网络对象，不能降为一个不可变 NetworkTables epoch；含 5 个 DOCTechNote 入口。 |
| 6 | `unsupported` | `READ.IO.OPEN` | 官方快照引用的数据文件确实缺失。 |
| 5 | `unsupported` | `PSA.DSS.SET.UNSUPPORTED` | PSA execution plan 不支持该 Set 选项。 |
| 5 | `unsupported` | `READ.DSS.VALUE_UNSUPPORTED` | 对象值或控制行为被识别，但当前计算合同不能保持。 |
| 5 | `unsupported` | `TENSOREVAL.DSS.TRANSLATE.SOLVE_FAILED` | 模型可导入，但所选求解器不支持该拓扑、后端不可用或未收敛。 |
| 4 | `unsupported` | `PSA.DSS.LINECODE.RELIABILITY_RETAINED` | LineCode 可靠性字段尚未进入所选计算合同。 |
| 3 | `unsupported` | `TENSOREVAL.DSS.TRANSLATE.SET_UNSUPPORTED` | `Set` 选项尚无可验证的 RSS 执行语义。 |
| 3 | `unsupported` | `TENSOREVAL.DSS.TRANSLATE.COMMAND_UNSUPPORTED` | 运行命令尚无可验证的 RSS 映射。 |
| 2 | `unsupported` | `PSA.DSS.EXECUTION.COMMAND_UNSUPPORTED` | 尚未分类的执行命令没有有序执行映射；有界 `BatchEdit` 已覆盖 StorageController 入口。 |
| 2 | `unsupported` | `PSA.DSS.GENERATOR.MODEL3.EXECUTION_UNSUPPORTED` | Generator Model 3 尚无 execution plan 等价运行语义。 |
| 2 | `unsupported` | `READ.DSS.PROPERTY_UNKNOWN` | 包含导入器未知的对象属性。 |
| 2 | `unsupported` | `PSA.DSS.OBJECT.UNSUPPORTED` | 使用了当前 importer 未覆盖的 DSS 对象。 |
| 1 | `unsupported` | `PSA.DSS.REGCONTROL.PROPERTY_UNSUPPORTED` | RegControl 属性尚无 transformer/control 表语义。 |
| 1 | `unsupported` | `PSA.DSS.AUTOTRANS.CONNECTION_UNSUPPORTED` | AutoTrans 绕组接法超出当前受支持组合。 |
| 1 | `unsupported` | `PSA.DSS.STORAGE.PHASES_UNSUPPORTED` | Storage 相数超出当前一相或三相模型。 |
| 1 | `unsupported` | `PSA.DSS.LOADSHAPE.ACTION_ORDER_UNSUPPORTED` | `Action=Normalize` 后继续修改曲线，最终向量不能重建原命令顺序。 |
| 1 | `unsupported` | `PSA.DSS.PROPERTY.SEMANTIC_UNREPRESENTABLE` | Storage-target InvControl 不能由当前 PVSystem-only typed runtime 无损表达。 |
| 1 | `unsupported` | `PSA.DSS.CONTROL.REQUIRED_PROPERTY_MISSING` | 控制对象缺少当前有界合同要求的显式属性。 |
| 1 | `unsupported` | `PSA.DSS.STORAGE.PROPERTY_UNSUPPORTED` | Storage `AmpLimit` 尚无当前运行时映射。 |
| 1 | `unsupported` | `PSA.DSS.TRANSFORMER.PROPERTY_UNSUPPORTED` | Transformer 属性尚无当前 owned table 的无损映射。 |
| 1 | `unsupported` | `PSA.DSS.GENERATOR.MODEL3.PV_FACTOR_UNSUPPORTED` | Generator Model 3 的 PV factor 超出当前静态执行合同。 |
| 1 | `unsupported` | `TENSOREVAL.DSS.TRANSLATE.POST_SOLVE_MUTATION` | 独立入口在首次 Solve 后修改模型，尚无 execution-plan 映射。 |
| 1 | `unsupported` | `READINESS.SOURCE.DEFAULTED` | 关键源参数需要默认值，严格 profile 不能证明等价。 |
| 1 | `unsupported` | `PSA.DSS.GENERATOR.USER_MODEL_UNSUPPORTED` | Generator 用户模型依赖外部实现，PSA 无法无损执行。 |
| 1 | `unsupported` | `TENSOREVAL.DSS.TRANSLATE.MIXED_STUDY_UNSUPPORTED` | 单个入口混合了当前不能由一个 bundle 表达的多类 study。 |

## 逐文件结果

[`conversion_report.csv`](conversion_report.csv) 包含官方树中 848 个 `.dss` 文件的分类结果；
其中 589 个是模型片段或公共依赖，不进入本文的案例统计。本文只讨论剩余 259 个独立入口：
144 个 `converted` 和 115 个 `unsupported`：

- `source_path`：相对于官方 `Version8/Distrib` 的路径；
- `status`：独立入口使用 `converted` 或 `unsupported`，模型片段和公共依赖使用
  `not_standalone_entry`；当前没有超时记录；
- `primary_diagnostic`：阻止转换的首个稳定诊断码；
- `all_diagnostic_codes` 和 `diagnostic_count`：该文件返回的诊断集合及数量；
- `source_root`：检查使用的依赖根，`.` 表示整个 `Version8/Distrib`；
- `reason`：包含源位置或对象名的首个具体原因。

报告保留每个文件的首要失败原因；同一文件的全部诊断码保存在
`all_diagnostic_codes`。需要修复某个入口时，应对该文件重新运行命令以取得完整诊断：

```bash
tensoreval --dss-to-rss --check \
  --source-root /path/to/electricdss-tst/Version8/Distrib \
  /path/to/electricdss-tst/Version8/Distrib/path/to/case.dss
```

只有上述检查成功后才生成 bundle：

```bash
tensoreval --dss-to-rss --mode bundle \
  --source-root /path/to/electricdss-tst/Version8/Distrib \
  --output sparrowzz/dss/rssbundle/path/to/case.rssbundle \
  /path/to/electricdss-tst/Version8/Distrib/path/to/case.dss
```
