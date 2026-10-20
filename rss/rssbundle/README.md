# OpenDSS 官方发行文件转换清单

本目录记录 `tensoreval` 对 OpenDSS 官方公开发行树中全部 `.dss` 文件的严格 DSS→RSS
bundle 扫描结果。扫描不会删除命令、补写模型参数或放宽诊断；只有转换、生成脚本验证和
bundle 自执行全部成功的案例才会生成 `.rssbundle`。

## 来源和范围

- 上游：[EPRI OpenDSS 示例与测试案例](https://opendss.epri.com/ExamplesDocsandTestCases.html)
  所述官方 SVN 公开发行内容的
  [`dss-extensions/electricdss-tst`](https://github.com/dss-extensions/electricdss-tst) 轻量镜像。
- 固定提交：`3b208397160213cae4a9e2d0a7d1aa3528ce26e1`；2026-10-19 核对时仍为镜像 `master`。
- 扫描根：`Version8/Distrib`。
- 文件总数：848，其中 `Examples` 642 个、`IEEETestCases` 116 个、`EPRITestCircuits` 90 个。
- 单文件检查超时：30 秒；本次没有超时。
- 对依赖越过案例目录的入口，使用整个 `Version8/Distrib` 作为 `--source-root` 重试。

## 结果

| 状态 | 文件数 | 含义 |
|---|---:|---|
| 已转换并发布 bundle | 10 | 严格转换、求解和 bundle 自执行全部通过 |
| 非独立入口 | 589 | 模型定义或公共依赖片段，没有独立研究；它们不是可直接运行的入口脚本 |
| 不支持 | 249 | 严格检查遇到当前转换合同不能无损保持的内容 |
| 超时 | 0 | 无 |

10 个成功入口按官方发行树的相对路径保存在本目录；每个 `.rssbundle` 都包含 manifest、生成的
RSS、DSS 依赖和数据资产，并在原子发布前完成一次自执行。已经位于 `sparrowzz/dss/models` 的
静态化案例不计入这 10 个官方原始入口。

## 首轮改造进展

2026-10-19 已按下文路线完成第一批编排层和 PSA 导入边界改造：

- 所有原生 `Solve`（包括单次 Snapshot）统一进入 PSA execution plan；`Snap` 已归一化，
  Snapshot 初始化后继续 Daily/Duty 不再被分类器预先拒绝。
- `tensoreval --study snapshot|execution|harmonic|auto-add|relcalc` 已对外开放；纯模型入口可用
  `--study snapshot` 明确要求单次静态研究。
- `EarthModel`、`Algorithm`、`MaxIter/MaxIterations`、`MaxControlIter/MaxControlI` 和
  `Tolerance` 由 PSA 统一校验。后三项按 epoch 写入实际潮流参数，生成 RSS 和转换期校验
  使用同一执行入口。
- dss2rss 只依据 PSA 诊断的 `severity` 判断是否阻断，不再解析诊断码或英文消息建立第二套
  放行白名单。
- execution epoch 会重新附加 `BusCoords/LatLongCoords` 与文件型参数资产；坐标命令以稳定
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

## 数值对照

2026-10-20 已用 OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5 直接运行每个 bundle 中保留的
原始 DSS 入口，并与 `tensoreval` 执行同一 bundle 后返回的最终 `voltage` 表比较。OpenDSS 和
PSA 的 10 个计算都收敛。修正三相 Δ→Y 变压器的 RegControl、次级 tap，以及嵌套 RPN 变量的
负荷额定电压后，8 个案例通过既有 PSA 算例门限，另有 2 个 J1 案例仍存在待修正的数值差异。
“已发布”表示严格转换和自执行成功；
各案例是否达到 OpenDSS 数值门限以本表为准。

| 官方入口 | 比较口径 | 最大相对误差 | 门限 | 结果 |
|---|---|---:|---:|---|
| `EPRITestCircuits/epri_dpv/J1/Master_noPV.dss` | 原始复电压 | 1.4486% | 1% | 不通过 |
| `EPRITestCircuits/epri_dpv/J1/Master_withPV.dss` | 原始复电压 | 1.3676% | 1% | 不通过 |
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

- J1 有/无 PV 案例的共同节点仍有 1.3676%/1.4486% 的实际电压差，需继续核对
  调压器模型、对象状态和控制后的求解终态。三相 Δ→Y 的修正没有扩展到 J1 的单相调压器
  和三相 Y→Y 变压器。试验将 Y→Y 二次 tap 计入匝比后，静态无 PV 误差降至 0.666%，
  有 PV 误差降至 1.057%；但 P58 三时段 QSTS 在第三时段的调压动作后无法收敛，
  因此未保留该改动。修复需同时处理径向潮流的迭代稳定性与单相调压器的终态差异。
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
  sparrowzz/rss/rssbundle/IEEETestCases/13Bus/IEEE13Nodeckt.rssbundle
```

纯模型 `IEEETestCases/123Bus/IEEE123Master.dss` 另用 `--study snapshot` 完成了 364 条语句的
定向检查和 bundle 自执行；它没有原生研究命令，因此在无 `--study` 的全量入口扫描中仍归为
非独立入口。IEEE8500 的 EnergyMeter 自动分区仍会影响电表区域结果，继续失败关闭。

## 与 PSA 当前能力的交叉评估

基线扫描的 `0/848` 不能解释为 PSA 没有覆盖这些网络和算法。当时的转换链在 PSA 前后又维护了
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

基线扫描中另有 3 个模型入口不是执行失败，而是当时的转换器要求一个尚未暴露给用户的
`--study`：

- `IEEETestCases/123Bus/IEEE123Master.dss`；
- `IEEETestCases/8500-Node/Master.dss`；
- `IEEETestCases/8500-Node/Master-unbal.dss`。

这三个文件本来就是模型定义。当前 `tensoreval` 已暴露 `--study`，其中
`IEEE123Master.dss` 已通过 `--study snapshot` 定向验证；两个 IEEE8500 master 仍需在后续
全量复测中确认其完整对象与拓扑边界。

下面几组只能判定为“底层原语存在、入口尚未闭环”，不能计入上述 19 个确定案例：

- 3 个 `Mode=FaultStudy` 入口使用 OpenDSS 的全网隐式故障枚举。PSA 已有 FaultStudy 数值核心，
  但要求显式 typed fault rows；必须先定义确定性的枚举和结果身份，不能只改模式名。
- 2 个 IEEE 519 入口已经生成 RSS，最后被 harmonic command contract 拒绝。谐波求解已覆盖，
  仍需逐条核对脚本中的命令和输出语义。
- 26 个 mixed-study 入口多数是 Snapshot 初始化后转 Daily/Duty。PSA 支持多 epoch 和跨时点状态，
  但其中还包含 GFM、Storage、UPFC 等不同对象，取消分类器的统一拒绝后仍需逐文件复测。
- 27 个 `REQUEST.SOURCE.ESCAPES_ROOT` 入口的首要问题是 epoch/内存 `Source` 没有保留声明文件的
  目录身份。路径修复只消除第一个阻断；MemoryMapping、二进制 Sng/Dbl 和部分 Storage 行为仍可能
  落在 PSA 的明确边界外。
- Dynamic/Dynamics、VCCS、DynEq、完整 IBR 暂态、AutoTrans 未支持接法、特殊节点编号和两个求解器
  都拒绝的拓扑属于当前真实能力边界。

## 改造路线和状态

1. **已完成：所有带 `Solve` 的入口交给 PSA execution plan。** 单次 Snapshot、Snapshot 初始化后
   继续 Daily/Duty 均由同一有序执行合同处理；`StaticPowerFlow` 只用于显式指定的纯模型 Snapshot。
2. **首轮已完成：求解设置由 PSA 统一解释。** `EarthModel`、`Algorithm`、
   `maxiter/maxiterations`、`maxcontroliter/maxcontroli` 和 `tolerance` 已纳入执行合同；写在 `Set`
   或最终 `Solve` 上的迭代参数都会进入实际潮流参数。`miniterations` 尚无 PSA 等价语义，继续明确失败。
   旧 `snapshot_set` 代码只保留给静态投影兼容测试，原生 `Solve` 不再经过该白名单。
3. **已完成：暴露 `tensoreval --study`。** 支持 `snapshot`、`execution`、`harmonic`、`auto-add` 和
   `relcalc`；`--study snapshot` 可为纯模型入口指定单次静态研究。
4. **已完成：以 PSA 的机器可读诊断合同为准。** `dss2rss` 以 import profile 返回的 `severity`
   决定 Warning 放行、Error 阻断，不再解析诊断码或英文 message。
5. **部分完成：保留 Source 图和声明目录。** 未删改的纯模型直接沿用原始 `Source`；execution epoch
   会重新附加坐标文件和已识别的数据资产。跨目录 `../`、MemoryMapping 与二进制 Sng/Dbl 仍需用
   全量扫描逐项验证。
6. **已完成分类层改造：让 PSA 判断连续执行是否可表达。** Snapshot+Daily/Duty 不再被分类器统一
   拒绝；具体状态转换仍由 execution plan fail closed。
7. **部分完成：为非电气副作用定义显式策略。** `BusCoords/LatLongCoords` 已保留资产并产生稳定
   Warning。`Plot/Show/Export/CaseName/CPU/Parallel` 的 calculation/exact 模式和 bundle manifest
   清单尚未实现。`Open/Close/Enable/Disable/BatchEdit` 仍按电气状态修改处理。
8. **待完成：FaultStudy 适配器。** 需要先规定母线/故障类型的确定性枚举、故障电阻默认值和结果
   长表主键，再调用现有 `ds_fault_study`；完成前保持 fail closed。

对应代码落点如下：

| 位置 | 修改职责 |
|---|---|
| `mems/src/bin/dss2rss/classify.rs` | 归一化 `snap`；取消单 Snapshot 静态化和可表达 continuation 的预拒绝；分类只选择 PSA adapter，不重复验证对象语义。 |
| `mems/src/bin/dss2rss.rs` | 原生 `Solve` 绕过 `snapshot_set`；移除基于 message 的安全白名单；未删改纯模型保留原始 Source。 |
| `mems/src/bin/dss2rss/compiler.rs` | 所有原生 `Solve` 优先调用 `import_dss_execution_plan`；把 DSS 求解预算传入生成 RSS 的 `para`；保留 PSA Error 原样诊断。 |
| `mems/src/bin/tensoreval.rs` | 增加并记录 `--study`；帮助文本、错误提示、check 和 bundle 模式使用同一枚举。 |
| `psa/src/dsa/parse/dss_import.rs` | 统一区分网络全局设置、求解参数和研究调度；`execution_epoch_source` 保留 referrer directory；需要时增加稳定的诊断 disposition。 |
| `mems/src/bin/dss2rss/bundle.rs` | 在 calculation 模式的 manifest 中记录被省略的非电气命令及源位置，bundle 复跑时校验同一清单。 |

第一批回归门禁应直接使用上述 19 个官方入口，并把验证分成三层：转换和 bundle 自执行、PSA 与
OpenDSS 的 V/I/P/Q/loss/control 数值比较、被省略副作用的 manifest 精确比较。每修复一个首要阻断都要
重扫全部 848 个文件，因为当前报告只保证首个稳定诊断，不能据此推断该文件后面没有第二个缺口。

## 主要原因

`primary_diagnostic` 是当前 import profile 下阻断转换的第一条诊断；同一诊断码在其他 profile
中可能只是 Warning。`all_diagnostic_codes` 保留该文件本次失败返回的完整诊断集合。

| 数量 | 分类 | 诊断码 | 原因 |
|---:|---|---|---|
| 589 | `not_standalone_entry` | `TENSOREVAL.DSS.CLASSIFY.STUDY_REQUIRED` | 模型或公共定义片段，没有可独立执行的研究/Solve；应由入口脚本通过 Redirect/Compile 使用。 |
| 36 | `unsupported` | `PSA.DSS.PROPERTY.SEMANTIC_UNREPRESENTABLE` | 对象属性超出当前 PSA DataFrame/执行合同，无法无损表达。 |
| 27 | `unsupported` | `REQUEST.SOURCE.ESCAPES_ROOT` | 数据资产引用被转换器判定为越过允许的获取根。 |
| 25 | `unsupported` | `TENSOREVAL.DSS.CLASSIFY.NATIVE_LIFECYCLE_UNSUPPORTED` | 请求 FaultStudy、Dynamic、MonteCarlo 等尚无转换适配器的生命周期。 |
| 23 | `unsupported` | `READ.DSS.VALUE_DEFAULTED` | 导入需要补默认值，所选严格 profile 不能证明与 OpenDSS 等价。 |
| 18 | `unsupported` | `READ.DSS.RETAINED_SOURCE_ONLY` | 属性虽保留在源信息中，但当前计算合同不会消费它。 |
| 16 | `unsupported` | `READ.IO.OPEN` | 官方快照缺少引用文件、路径大小写不匹配或文件无法打开。 |
| 12 | `unsupported` | `PSA.DSS.LOADSHAPE.PROPERTY_UNSUPPORTED` | LoadShape 属性尚无被当前时序合同消费的无损映射。 |
| 11 | `unsupported` | `PSA.DSS.EXECUTION.INCLUDE_LOAD_FAILED` | 展开后的 include 在 execution epoch 中无法便携重建。 |
| 11 | `unsupported` | `PSA.DSS.PVSYSTEM.PROPERTY_UNSUPPORTED` | PVSystem 属性尚无无损映射。 |
| 9 | `unsupported` | `PSA.DSS.STORAGE.PROPERTY_UNSUPPORTED` | Storage 属性尚无无损映射。 |
| 7 | `unsupported` | `PSA.DSS.GENERATOR.MODEL3.EXECUTION_UNSUPPORTED` | Generator Model 3 尚无 execution plan 等价运行语义。 |
| 7 | `unsupported` | `PSA.DSS.NODE.PHASE_LABEL_UNSUPPORTED` | 节点/相导体编号超出当前网络表约束。 |
| 7 | `unsupported` | `PSA.DSS.LINECODE.RELIABILITY_RETAINED` | LineCode 可靠性字段尚未进入所选计算合同。 |
| 6 | `unsupported` | `TENSOREVAL.DSS.TRANSLATE.SOLVE_FAILED` | 模型可导入，但所选 PSA 求解器不支持该拓扑或未收敛。 |
| 5 | `unsupported` | `PSA.DSS.EXECUTION.POST_SOLVE_ACTION_MIXED_MUTATION` | Solve 后动作与模型修改混合，执行顺序无法无歧义保持。 |
| 4 | `unsupported` | `PSA.DSS.SET.UNSUPPORTED` | Set 选项没有无损的 ExecutionStudy DataFrame 语义。 |
| 4 | `unsupported` | `PSA.DSS.LINE.GEOMETRY_INVALID` | LineGeometry 引用或导体位置不能形成有效线路参数。 |
| 4 | `unsupported` | `READ.DSS.VALUE_UNSUPPORTED` | 对象值或控制行为被识别，但当前计算合同不能保持。 |
| 4 | `unsupported` | `PSA.DSS.EXECUTION.ORDER_UNREPRESENTABLE` | 构造、设置和运行命令顺序不能降为当前不可变 epoch。 |
| 3 | `unsupported` | `PSA.TRANSFORMER_MATRIX.DIMENSION_MISMATCH` | 变压器矩阵维数与绕组或相数不一致。 |
| 2 | `unsupported` | `TENSOREVAL.DSS.TRANSLATE.GENERATED_RSS_FAILED` | RSS 已生成，但验证执行失败，因此不发布 bundle。 |
| 1 | `unsupported` | `PSA.DSS.CONSTRUCTION.PROPERTY_UNKNOWN` | 构造对象包含 PSA 尚未知的属性。 |
| 1 | `unsupported` | `PSA.DSS.AUTOTRANS.CONNECTION_UNSUPPORTED` | AutoTrans 绕组接法超出当前受支持组合。 |
| 1 | `unsupported` | `READ.DSS.PROPERTY_UNKNOWN` | 包含导入器未知的对象属性。 |
| 1 | `unsupported` | `PSA.DSS.INVCONTROL.MON_VOLTAGE_CALC_INVALID` | InvControl 监测电压聚合设置不满足当前合同。 |
| 1 | `unsupported` | `PSA.DSS.SPECTRUM.PROPERTY_UNSUPPORTED` | Spectrum 属性尚无谐波合同映射。 |
| 1 | `unsupported` | `PSA.DSS.OBJECT.UNSUPPORTED` | 使用了当前 importer 未覆盖的 DSS 对象。 |
| 1 | `unsupported` | `READINESS.SOURCE.DEFAULTED` | 关键源参数需要默认值，严格 profile 不能证明等价。 |
| 1 | `unsupported` | `PSA.DSS.GENERATOR.USER_MODEL_UNSUPPORTED` | Generator 用户模型依赖外部实现，PSA 无法无损执行。 |

## 逐文件结果

[`conversion_report.csv`](conversion_report.csv) 对 848 个文件逐一记录：

- `source_path`：相对于官方 `Version8/Distrib` 的路径；
- `status`：`converted`、`not_standalone_entry`、`unsupported` 或 `timeout`；
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
  --output sparrowzz/rss/rssbundle/path/to/case.rssbundle \
  /path/to/electricdss-tst/Version8/Distrib/path/to/case.dss
```
