# DSS 到 RSS 严格转换

`tensoreval --dss-to-rss` 把一个本地 OpenDSS `.dss` 入口文件转换成可由
`tensoreval --df` 执行的
`.rss` 脚本。转换器在写文件前展开并审计本地 `Redirect`/`Compile` 文件树，按 PSA
当前契约导入全部电气内容，并用所选 PSA 求解器实际求解一次。任一步失败都不会留下部分
可执行脚本。

## 1. 命令

直接计算 DSS：

```bash
tensoreval path/to/run.dss
```

只转换为 linked RSS，不执行生成脚本：

```bash
tensoreval --dss-to-rss --output path/to/run.rss path/to/run.dss
```

随后执行生成文件：

```bash
tensoreval --df path/to/run.rss
```

省略 `--output` 时生成同目录同名的 `path/to/run.rss`：

```bash
tensoreval --dss-to-rss path/to/run.dss
```

`--dss2rss`、`--dsstorss` 和 `--dss_to_rss` 是 `--dss-to-rss` 的等价别名，输出和错误
处理合同完全相同。

该参数仍执行严格导入和求解校验。目标 RSS 已存在时会拒绝覆盖；使用 `--output` 选择路径，
使用 `--force` 覆盖已有输出。

直接计算和转换共用同一个分类、导入和求解实现。一个 `.dss` 必须是唯一位置参数。
直接计算不接受 `--input`；转换时 `--input NAME=FILE` 只与 `--mode bundle` 联用。
直接执行会遵守 DSS 的显式 `Algorithm`；未指定时根据 PSA
已验证的拓扑选择 Radial 或 Newton，不会在 Radial 失败后自动切换 Newton。

只检查而不写文件时使用 `tensoreval --dss-to-rss --check INPUT.dss`。输出文件存在时转换器拒绝覆盖；显式使用 `--force`
才能替换。`--solver auto` 是默认值；`--solver radial` 和 `--solver newton-raphson` 可
显式覆盖选择。Newton 还要求构建包含可用的
MUMPS 或 MKL 后端。

只有模型定义、没有 `Solve` 或专用研究命令的文件必须显式指定研究类型。例如：

```bash
tensoreval --dss-to-rss --check --study snapshot path/to/model.dss
```

`--study` 支持 `snapshot`、`execution`、`harmonic`、`auto-add` 和 `relcalc`。该参数选择
PSA adapter，不跳过对应 adapter 的严格语义检查。

需要把 DSS 文件树、数据资产和外部输入一起交付时，生成便携 bundle：

```bash
tensoreval --dss-to-rss --mode bundle \
  --output path/to/case.rssbundle path/to/run.dss

tensoreval path/to/case.rssbundle
```

默认 `--mode linked` 仍生成单个 `.rss`。`--mode bundle` 生成一个目录，包含
`manifest.json`、`main.rss`、`dss/` 以及可选的 `inputs/`。DSS 源文件和文件型数据资产都按
原始 source-root 相对路径复制到 `dss/`；manifest 使用 `dss-source` 和 `data-asset` 角色区分二者。
这种布局让 Redirect 引入的 DSS 仍能在 acquisition root 内读取 CSV。转换器在临时目录中完成
路径重写、清单校验和一次自执行后再原子发布。早期 schema v1 bundle 的顶层 `assets/` 仍可读取。
`--source-root` 可显式指定依赖树边界；`--force` 只在新 bundle 完整验证后替换旧目录。
本页后续章节定义清单字段、路径约束和验证顺序。

## 2. 转换路径

转换器根据 DSS 命令流选择一种执行路径：

| DSS 输入 | 生成的 RSS | 条件 |
|---|---|---|
| Daily/Yearly/Duty/PeakDay/LD1/LD2 单个或多个 `Solve` | `ds_radial_dss_plan` 或 Newton 版本 | 整个命令流满足 PSA ExecutionStudy 合同；LD1/LD2 必须显式设置 `LDCurve` |
| 一个普通 Snapshot `Solve` | `ds_radial_dss_plan` 或 Newton 版本 | 使用 PSA 的单 epoch Snapshot 执行计划，保留控制和有序命令语义 |
| 没有研究命令的纯模型 + `--study snapshot` | `import_dss` 后调用 `ds_radial_pf` 或 Newton 版本 | 原始模型入口必须满足 StaticPowerFlow 导入合同 |
| `Mode=Harmonic/HarmonicT` | `ds_dss_harmonic_plan` | 满足 PSA 原生谐波执行合同 |
| `Mode=AutoAdd` | `ds_dss_auto_add` | 满足 PSA 有界单次 AutoAdd 合同 |
| Snapshot 后执行 `RelCalc` | `ds_dss_relcalc` | 满足 PSA 径向分区可靠性合同 |

转换器先解析完整命令流再分类。专用生命周期优先于其准备阶段的 Snapshot；Snapshot 初始化
后继续 Daily/Yearly/Duty 等 QSTS 由同一个 execution plan 表达。一个文件中混合 QSTS、
Harmonic、AutoAdd 或 RelCalc 仍会以 `TENSOREVAL.DSS.TRANSLATE.MIXED_STUDY_UNSUPPORTED`
失败。只有模型定义、没有研究命令的入口不会由转换器猜测研究类型，必须使用 `--study`。

为保持已有生成脚本合同，Snapshot 和 ExecutionPlan 的 linked RSS 把所选 `bus_phase`
结果绑定为终端变量 `voltage`；`tensoreval case.dss` 的直接执行结果名仍为 `bus_phase`。
Harmonic、AutoAdd 和 RelCalc 的终端变量名与各自结果名完全相同。

静态 Snapshot 投影只用于没有原生 `Solve` 的显式静态路径或兼容性处理。原生 `Solve` 不再
进入该投影，而是统一交给 execution plan。投影可移除以下语句：

- `Set VoltageBases=...`：PSA 从已导入网络得到电压基准；
- `CalcV` / `CalcVoltageBases`：对应工作已包含在网络构造和潮流入口中；
- 一个无参数 `Solve`，或只带 `Mode=Snapshot`、`ControlMode=Static/Off` 的 `Solve`；
- `BusCoords` / `LatLongCoords`：只影响 OpenDSS 图形布局，不影响生成脚本返回的 `bus_phase`
  电气结果；文件参数可使用引号、括号及行尾注释，转换器按 DSS 词法取得规范化路径；
- `Plot`、`Show` 和 `Dump`：只读取模型或已求解状态生成图形/报告，不改变电气状态；转换诊断和
  执行计划诊断会以 Warning 保留省略原因及源位置。

`Set DefaultBaseFrequency`、对象声明、`Edit`、`More`、属性引用、变量以及本地 include
继续留在模型中。多个 Snapshot `Solve`、求解后的模型修改和其他未列出的运行时命令不会被忽略。

生成的 RSS 保留 `.dss` 文件作为权威输入，路径写成规范化绝对路径。ExecutionPlan 指向
原执行入口；Snapshot 指向经过验证、可由现有 `import_dss` 读取的静态模型入口。移动或
删除相关 DSS 文件树后，生成脚本不能运行。它不是把所有 DataFrame 内联进单个文本文件。

ExecutionPlan 会按源码顺序保留 `Set` 和最终 `Solve` 上的执行设置。
`MaxIter/MaxIterations`、`MaxControlIter/MaxControlI` 和 `Tolerance` 会覆盖对应 epoch 的 PSA
潮流参数；`Algorithm` 决定 Radial 或 Newton adapter，`EarthModel` 由 PSA 统一校验。
`MinIterations` 当前没有 PSA 等价参数，因此仍返回明确的不支持诊断，不会被静默忽略。

整个执行计划都由单步 Snapshot 构成时，PSA 还会在可证明的窄边界省略
`BatchEdit Load..* Daily=default`，以及只监测仍处于 GFL/PQ 状态 DER 的
`InvControl mode=GFM`。两者分别返回
`PSA.DSS.EXECUTION.SNAPSHOT_BATCH_EDIT_IGNORED` 和
`PSA.DSS.EXECUTION.INERT_GFM_MONITOR_IGNORED` Warning。非 Snapshot、多步求解、额外
InvControl 属性、控制器后续修改或任一成员切换为 `ControlMode=GFM` 都不适用该规则。

`LoadShape Action=Normalize` 在导入期把 P、Q 序列分别除以各自最大绝对值。该 Action 必须是
该 LoadShape 的最后一次修改；其他 Action 或 Normalize 后继续修改曲线会返回明确诊断。

每个 linked RSS 的头部记录 `mode=linked`、分类、规范化源入口、依赖树 SHA-256 和 PSA
合同版本。依赖树来自 PSA 导入器实际读取的 `Redirect`/`Compile` 和数据资产清单；哈希按
规范化相对路径排序并覆盖原始文件字节。头部哈希用于溯源，linked 运行时仍重新读取原文件；
bundle 则复制这些文件，并在解析或执行 RSS 前强制验证清单、路径、文件哈希和输入 schema。

纯模型使用 `--study snapshot` 时保留原始 `Source` 和依赖目录，不为删除
`Set VoltageBases`/`CalcVoltageBases` 重建扁平内存源。带原生 `Solve` 的入口则由 linked RSS
直接引用 execution plan 入口。

## 3. 失败诊断

失败输出包含全部可获得的稳定诊断码、源文件/行号或对象名以及原因。例如原始 IEEE13
案例的 `RegControl` 调压语义若不能由当前 PSA Snapshot 路径执行，转换器会报告：

```text
DSS import produced findings that can change OpenDSS results
- [READ.DSS.VALUE_UNSUPPORTED] IEEE13Nodeckt.dss:29 (regcontrol.Reg1): regcontrol Reg1: ...
- [READ.DSS.VALUE_UNSUPPORTED] IEEE13Nodeckt.dss:33 (regcontrol.Reg2): regcontrol Reg2: ...
- [READ.DSS.VALUE_UNSUPPORTED] IEEE13Nodeckt.dss:37 (regcontrol.Reg3): regcontrol Reg3: ...
```

转换器以 PSA 在所选 import profile 下给出的机器可读 `severity` 为唯一边界：Warning 保留并
输出，Error 阻断转换。转换器不再根据诊断码或英文消息维护第二套白名单；若某项 Warning
实际会改变所选研究结果，应在 PSA 修正严重性合同。

常见失败码：

| 诊断码 | 含义 |
|---|---|
| `TENSOREVAL.DSS.TRANSLATE.COMMAND_UNSUPPORTED` | 命令没有 RSS/PSA Snapshot 等价语义 |
| `TENSOREVAL.DSS.TRANSLATE.SET_UNSUPPORTED` | `Set` 选项尚未映射 |
| `TENSOREVAL.DSS.TRANSLATE.SOLVE_ARGUMENT_UNSUPPORTED` | Snapshot `Solve` 带有未映射参数 |
| `TENSOREVAL.DSS.TRANSLATE.POST_SOLVE_MUTATION` | `Solve` 后仍修改模型，且不能进入 execution plan |
| `TENSOREVAL.DSS.TRANSLATE.MULTIPLE_SNAPSHOT_SOLVES` | 多个 Snapshot barrier 尚无连续状态合同 |
| `TENSOREVAL.DSS.TRANSLATE.SOLVE_FAILED` | 导入成功，但所选 PSA 求解器未能完成计算 |
| `TENSOREVAL.DSS.TRANSLATE.STATIC_ENTRY_UNAVAILABLE` | 可直接计算，但没有可供生成 RSS 使用的单一静态模型入口 |
| `TENSOREVAL.DSS.CLASSIFY.STUDY_REQUIRED` | 入口只定义模型，没有可分类的研究命令 |
| `TENSOREVAL.DSS.CLASSIFY.NATIVE_LIFECYCLE_UNSUPPORTED` | 请求 FaultStudy、Dynamic、M1/M2/M3/MonteFault 等尚无原生 adapter 的生命周期 |
| `TENSOREVAL.DSS.TRANSLATE.MIXED_STUDY_UNSUPPORTED` | 一个命令流混合了没有 continuation 合同的研究类型 |
| `TENSOREVAL.DSS.TRANSLATE.GENERATED_RSS_FAILED` | 生成文本已形成，但通过同一 RSDss/PSA 门面自校验失败 |
| `TENSOREVAL.RSS.BUNDLE.PATH_INVALID` / `TENSOREVAL.RSS.BUNDLE.PATH_ESCAPE` | 清单路径不规范，或解析后越过 bundle 根目录 |
| `TENSOREVAL.RSS.BUNDLE.HASH_MISMATCH` | bundle 内文件内容与清单中的 SHA-256 不一致 |
| `TENSOREVAL.RSS.BUNDLE.INPUT_SCHEMA_MISMATCH` | 外部输入文件的列名或数据类型与清单不一致 |
| `TENSOREVAL.RSS.BUNDLE.RESULT_MISMATCH` | 执行结果的名称或顺序与清单声明不一致 |

另外还会原样返回 PSA 的 `PSA.DSS.*`、`READ.DSS.*` 和 `READINESS.*` 诊断。调用方应以
退出码和诊断码判断结果，不能只搜索提示文本。

## 4. OpenDSS 官方发行文件扫描

仓库使用 OpenDSS 官方 SVN 公开发行内容的 `electricdss-tst` 镜像，对固定提交
`3b208397160213cae4a9e2d0a7d1aa3528ce26e1` 的 `Version8/Distrib` 做了完整扫描。
扫描覆盖 848 个 `.dss` 文件：`Examples` 642 个、`IEEETestCases` 116 个、
`EPRITestCircuits` 90 个。每个文件都实际执行严格 `--check`；依赖越过当前案例目录时，
以整个 `Version8/Distrib` 作为 `--source-root` 重试。

基线重扫结果是：10 个官方原始入口通过严格检查、生成 bundle 并完成构建期自执行；589 个文件
是没有独立研究命令的模型或公共依赖片段；另有 249 个独立入口因当时未映射的生命周期、对象
属性、路径、执行顺序或求解问题失败，没有文件超时。后续只在这 249 个独立入口中做定向改造，
目前另有 27 个入口通过并发布，总计 37 个官方原始入口。新增入口包括 Snapshot/QSTS、FaultStudy、
HarmonicT，以及 `Examples/FreqScan/Run_Scan.dss` 的 188 点非整数频率扫描；不包含对官方文件的
删改或静态化包装。

纯模型 `IEEE123Master.dss` 另用 `--study snapshot` 完成定向检查和 bundle 自执行，但因原文件
没有研究命令，不计入无 `--study` 扫描的 10 个可运行入口。实时清单、全部剩余诊断和 IEEE8500
EnergyMeter 自动分区边界见[官方发行文件转换清单](../rssbundle/README.md)。

[官方发行文件转换清单](../rssbundle/README.md)汇总各诊断码；
[`conversion_report.csv`](../rssbundle/conversion_report.csv)逐一列出 848 个文件、状态、
全部诊断码和带源位置的首要原因。以后升级转换能力时，应使用同一固定输入重新扫描；只有
检查、生成和 bundle 自执行都成功的案例才能改为 `converted`。

## 5. 数值一致性

转换成功表示同一个 DSS 文件已经通过生成脚本所引用的 PSA 路径导入并求解。HarmonicT、
AutoAdd 和 RelCalc 在返回脚本前还会实际解析并执行生成的 RSS，结果表顺序分别固定为
`6/4/4` 张；空 Monitor 和未选中 AutoAdd 均保留固定 schema。仓库测试还会：

1. 用 OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5 直接运行
   `models/ieee13_static/IEEE13Nodeckt_run.dss`，锁定 41 个节点的复电压；
2. 从同一个入口文件生成 IEEE13 Snapshot RSS；
3. 重新解析并执行生成文本；
4. 按 `bus + phase` 与锁定结果连接，检查全部节点覆盖和 `1e-3` 最大相对误差。

需要完整导出命名结果表时，使用 `--result-dir`。每个结果写入 `DIR/结果名.csv`，bundle 的
`voltage` 因而写入 `DIR/voltage.csv`：

```bash
tensoreval --result-dir /tmp/case-psa path/to/case.rssbundle
```

官方发行扫描的基线 10 个成功 bundle 已完成最终母线复电压对照，并全部通过已有 PSA 案例门限。
EPRI J1 无/有 PV 案例的最大复电压相对误差分别为 0.007131% 和 0.006013%；SG2012 的两个
三相 RegControl 最终 tap 均与 OpenDSS 一致，129 个节点最大相对误差为 0.03344%。完整节点
集合、比较口径和误差见[官方发行文件转换清单](../rssbundle/README.md)及
[`numerical_comparison.csv`](../rssbundle/numerical_comparison.csv)。

FreqScan bundle 另以 Monitor 结果对照：188 个频点、3 相、564 行均有唯一稳定键；电压幅值最大
缩放误差 `9.3144e-6`，电压相角最大误差 `0.000555966°`，不小于 1 A 的电流相角最大误差
`0.000199377°`。复制整个 bundle 后仍可用 `tensoreval --result-dir DIR INPUT.rssbundle` 运行，
证明脚本不依赖生成机绝对路径。

`tensoreval` 的直接 `.dss` 入口还原地使用 PSA 仓库中的三个输入及其 OpenDSS 基准：

| PSA DSS 输入 | 覆盖语义 | 比较结果 |
|---|---|---|
| `generator_voltage_models_reference.dss` | 单相 Generator Model 1 | Generator 母线复电压 |
| `generator_model7_balanced_reference.dss` | 三相不平衡负荷、Generator Model 7 | Generator 母线三相复电压 |
| `pvsystem_balanced_extended_reference.dss` | 三相不平衡负荷、PVSystem | PV 母线三相复电压 |

这些测试调用 `tensoreval` 实际使用的 `run_dss` 路径，按 `bus + phase` 比较 `v_re/v_im`，
最大相对误差门限为 `5e-6`。输入和 CSV 都从 `psa/tests/bmopf` 原地读取，不维护第二份基准。
可直接用公开入口复核运行结果：

```bash
tensoreval path/to/reference-case.dss
```

对应的独立基准保存在
`models/ieee13_static/IEEE13Nodeckt_run_opendss_voltage.csv`。OpenDSS 与原有 PSA 基准之间
另行核对得到的最大复电压相对差为 `1.86e-4`，低于上述验收门限。

该保证只覆盖被测试的分析、对象组合和结果表。业务模型仍应按
[验证与故障排查](validation_and_troubleshooting.md)保留独立 OpenDSS oracle。转换器发现
不支持项时应扩展 PSA 语义和对照测试，不能通过删除诊断或放宽白名单绕过。
