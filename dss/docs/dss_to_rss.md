# DSS 到 RSS 严格转换

`dss2rss` 把一个本地 OpenDSS `.dss` 入口文件转换成可由 `tensoreval --df` 执行的
`.rss` 脚本。转换器在写文件前展开并审计本地 `Redirect`/`Compile` 文件树，按 PSA
当前契约导入全部电气内容，并用所选 PSA 求解器实际求解一次。任一步失败都不会留下部分
可执行脚本。

## 1. 命令

从工作区根目录运行：

```bash
cargo run -p mems --features tensoreval-cli,with_dsa --bin dss2rss -- \
  path/to/run.dss --output path/to/run.rss
```

随后执行生成文件：

```bash
cargo run -p mems --features tensoreval-cli,with_dsa --bin tensoreval -- \
  --df path/to/run.rss
```

也可以把 `.dss` 入口直接交给 `tensoreval`。后缀匹配不区分大小写，不要求 `--df`：

```bash
cargo run -p mems --features tensoreval-cli,with_dsa --bin tensoreval -- \
  path/to/run.dss
```

只转换为同目录同名的 `path/to/run.rss`，不执行生成脚本和输出潮流结果时使用：

```bash
cargo run -p mems --features tensoreval-cli,with_dsa --bin tensoreval -- \
  --dss-to-rss path/to/run.dss
```

`--dss2rss`、`--dsstorss` 和 `--dss_to_rss` 是 `--dss-to-rss` 的等价别名，输出和错误
处理合同完全相同。

该参数仍执行严格导入和求解校验。目标 RSS 已存在时会拒绝覆盖；需要选择输出路径或覆盖时，
使用上面的独立 `dss2rss --output ... [--force]` 命令。

此路径与独立 `dss2rss` 共用同一个分类、导入和求解实现。一个 `.dss` 必须单独运行，
也不能同时使用 `--input`。直接执行会遵守 DSS 的显式 `Algorithm`；未指定时根据 PSA
已验证的拓扑选择 Radial 或 Newton，不会在 Radial 失败后自动切换 Newton。

只检查而不写文件时使用 `--check`。输出文件存在时转换器拒绝覆盖；显式使用 `--force`
才能替换。`--solver auto` 是默认值；`--solver radial` 和 `--solver newton-raphson` 可
显式覆盖选择。Newton 还要求构建包含可用的
MUMPS 或 MKL 后端。

## 2. 转换路径

转换器根据 DSS 命令流选择一种执行路径：

| DSS 输入 | 生成的 RSS | 条件 |
|---|---|---|
| Daily/Yearly/Duty/PeakDay/LD1/LD2 单个或多个 `Solve` | `ds_radial_dss_plan` 或 Newton 版本 | 整个命令流满足 PSA ExecutionStudy 合同；LD1/LD2 必须显式设置 `LDCurve` |
| 一个普通 Snapshot `Solve` | `import_dss` 静态模型后调用 `ds_radial_pf` 或 Newton 版本 | 入口本身可由 `import_dss` 读取，或只包装一个可读取的 `Redirect/Compile` 静态模型 |
| `Mode=Harmonic/HarmonicT` | `ds_dss_harmonic_plan` | 满足 PSA 原生谐波执行合同 |
| `Mode=AutoAdd` | `ds_dss_auto_add` | 满足 PSA 有界单次 AutoAdd 合同 |
| Snapshot 后执行 `RelCalc` | `ds_dss_relcalc` | 满足 PSA 径向分区可靠性合同 |

转换器先解析完整命令流再分类。专用生命周期优先于其准备阶段的 Snapshot；一个文件中
混合 QSTS、Harmonic、AutoAdd 或 RelCalc 会以
`MEMS.DSS.TRANSLATE.MIXED_STUDY_UNSUPPORTED` 失败。只有模型定义、没有研究命令的入口
不会由 `dss2rss` 猜测研究类型。

为保持已有生成脚本合同，Snapshot 和 ExecutionPlan 的 linked RSS 把所选 `bus_phase`
结果绑定为终端变量 `voltage`；`tensoreval case.dss` 的直接执行结果名仍为 `bus_phase`。
Harmonic、AutoAdd 和 RelCalc 的终端变量名与各自结果名完全相同。

Snapshot 投影当前只移除以下语句：

- `Set VoltageBases=...`：PSA 从已导入网络得到电压基准；
- `CalcV` / `CalcVoltageBases`：对应工作已包含在网络构造和潮流入口中；
- 一个无参数 `Solve`，或只带 `Mode=Snapshot`、`ControlMode=Static/Off` 的 `Solve`；
- `BusCoords`：只影响 OpenDSS 图形布局，不影响生成脚本返回的 `bus_phase` 电气结果。

`Set DefaultBaseFrequency`、对象声明、`Edit`、`More`、属性引用、变量以及本地 include
继续留在模型中。多个 Snapshot `Solve`、求解后的模型修改和其他运行时命令不会被忽略。

生成的 RSS 保留 `.dss` 文件作为权威输入，路径写成规范化绝对路径。ExecutionPlan 指向
原执行入口；Snapshot 指向经过验证、可由现有 `import_dss` 读取的静态模型入口。移动或
删除相关 DSS 文件树后，生成脚本不能运行。它不是把所有 DataFrame 内联进单个文本文件。

每个 linked RSS 的头部记录 `mode=linked`、分类、规范化源入口、依赖树 SHA-256 和 PSA
合同版本。依赖树来自 PSA 导入器实际读取的 `Redirect`/`Compile` 和数据资产清单；哈希按
规范化相对路径排序并覆盖原始文件字节。头部哈希用于溯源，linked 运行时仍重新读取原文件；
运行前强制验签属于后续 bundle 校验能力。

直接执行 `.dss` 不要求 Snapshot 入口和模型分文件；若直接计算成功，但入口无法安全归约
成一个持久 RSS 可引用的静态模型，`dss2rss --output` 会用
`MEMS.DSS.TRANSLATE.STATIC_ENTRY_UNAVAILABLE` 拒绝生成，并提示拆分模型文件和运行入口。

## 3. 失败诊断

失败输出包含全部可获得的稳定诊断码、源文件/行号或对象名以及原因。例如原始 IEEE13
案例的 `RegControl` 调压语义若不能由当前 PSA Snapshot 路径执行，转换器会报告：

```text
DSS import produced findings that can change OpenDSS results
- [READ.DSS.VALUE_UNSUPPORTED] IEEE13Nodeckt.dss:29 (regcontrol.Reg1): regcontrol Reg1: ...
- [READ.DSS.VALUE_UNSUPPORTED] IEEE13Nodeckt.dss:33 (regcontrol.Reg2): regcontrol Reg2: ...
- [READ.DSS.VALUE_UNSUPPORTED] IEEE13Nodeckt.dss:37 (regcontrol.Reg3): regcontrol Reg3: ...
```

PSA 导入器的部分发现原本是 Warning。`dss2rss` 会把可能改变 OpenDSS 结果的 Warning
提升为转换失败。当前只放行已验证的源 `pu/angle` 默认值、`emergamps` 默认值，以及
OpenDSS 自己对 `switch=yes` 忽略的线路阻抗字段。

常见失败码：

| 诊断码 | 含义 |
|---|---|
| `MEMS.DSS.TRANSLATE.COMMAND_UNSUPPORTED` | 命令没有 RSS/PSA Snapshot 等价语义 |
| `MEMS.DSS.TRANSLATE.SET_UNSUPPORTED` | `Set` 选项尚未映射 |
| `MEMS.DSS.TRANSLATE.SOLVE_ARGUMENT_UNSUPPORTED` | Snapshot `Solve` 带有未映射参数 |
| `MEMS.DSS.TRANSLATE.POST_SOLVE_MUTATION` | `Solve` 后仍修改模型，且不能进入 execution plan |
| `MEMS.DSS.TRANSLATE.MULTIPLE_SNAPSHOT_SOLVES` | 多个 Snapshot barrier 尚无连续状态合同 |
| `MEMS.DSS.TRANSLATE.SOLVE_FAILED` | 导入成功，但所选 PSA 求解器未能完成计算 |
| `MEMS.DSS.TRANSLATE.STATIC_ENTRY_UNAVAILABLE` | 可直接计算，但没有可供生成 RSS 使用的单一静态模型入口 |
| `MEMS.DSS.CLASSIFY.STUDY_REQUIRED` | 入口只定义模型，没有可分类的研究命令 |
| `MEMS.DSS.CLASSIFY.NATIVE_LIFECYCLE_UNSUPPORTED` | 请求 FaultStudy、Dynamic、M1/M2/M3/MonteFault 等尚无原生 adapter 的生命周期 |
| `MEMS.DSS.TRANSLATE.MIXED_STUDY_UNSUPPORTED` | 一个命令流混合了没有 continuation 合同的研究类型 |
| `MEMS.DSS.TRANSLATE.GENERATED_RSS_FAILED` | 生成文本已形成，但通过同一 expr_df/PSA 门面自校验失败 |

另外还会原样返回 PSA 的 `PSA.DSS.*`、`READ.DSS.*` 和 `READINESS.*` 诊断。调用方应以
退出码和诊断码判断结果，不能只搜索提示文本。

## 4. 数值一致性

转换成功表示同一个 DSS 文件已经通过生成脚本所引用的 PSA 路径导入并求解。HarmonicT、
AutoAdd 和 RelCalc 在返回脚本前还会实际解析并执行生成的 RSS，结果表顺序分别固定为
`6/4/4` 张；空 Monitor 和未选中 AutoAdd 均保留固定 schema。仓库测试还会：

1. 用 OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5 直接运行
   `models/ieee13_static/IEEE13Nodeckt_run.dss`，锁定 41 个节点的复电压；
2. 从同一个入口文件生成 IEEE13 Snapshot RSS；
3. 重新解析并执行生成文本；
4. 按 `bus + phase` 与锁定结果连接，检查全部节点覆盖和 `1e-3` 最大相对误差。

`tensoreval` 的直接 `.dss` 入口还原地使用 PSA 仓库中的三个输入及其 OpenDSS 基准：

| PSA DSS 输入 | 覆盖语义 | 比较结果 |
|---|---|---|
| `generator_voltage_models_reference.dss` | 单相 Generator Model 1 | Generator 母线复电压 |
| `generator_model7_balanced_reference.dss` | 三相不平衡负荷、Generator Model 7 | Generator 母线三相复电压 |
| `pvsystem_balanced_extended_reference.dss` | 三相不平衡负荷、PVSystem | PV 母线三相复电压 |

这些测试调用 `tensoreval` 实际使用的 `run_dss` 路径，按 `bus + phase` 比较 `v_re/v_im`，
最大相对误差门限为 `5e-6`。输入和 CSV 都从 `psa/tests/bmopf` 原地读取，不维护第二份基准。
运行命令：

```bash
cargo test -p mems --features tensoreval-cli,with_dsa --bin tensoreval psa_ -- \
  --test-threads=1
```

对应的独立基准保存在
`models/ieee13_static/IEEE13Nodeckt_run_opendss_voltage.csv`。OpenDSS 与原有 PSA 基准之间
另行核对得到的最大复电压相对差为 `1.86e-4`，低于上述验收门限。

该保证只覆盖被测试的分析、对象组合和结果表。业务模型仍应按
[验证与故障排查](validation_and_troubleshooting.md)保留独立 OpenDSS oracle。转换器发现
不支持项时应扩展 PSA 语义和对照测试，不能通过删除诊断或放宽白名单绕过。
