# PV CurrentkvarLimite Volt-Var Snapshot 发布证据

## 身份与范围

- 官方入口：`Examples/InverterModels/PVSystem/CurrentkvarLimite/PV_currentkvarLimit_VV.dss`
- 固定上游提交：`3b208397160213cae4a9e2d0a7d1aa3528ce26e1`
- 官方 CRLF 源文件 SHA-256：`ae05f52408aff3795dba85ffe655e16b4711f13aa9daff30b0d524cd3a5af353`
- 发布边界：显式 Snapshot、`line_model=exact`、Radial；不把模型定义文件自动归类为入口，
  不外推到 Daily、多 DER 或其他 InvControl 生命周期。

## 独立原生数值证据

- Case id：`pv_var_capability.current_kvar_limit_voltvar`
- OpenDSSDirect.py：`0.9.4`
- DSS C-API：`0.14.5`
- 原生 reference metadata SHA-256：`3ab77a07ee3209373c9d554c62e689039c3b86c9c6626bb976f0b265101020a8`
- 原生求解收敛：56 次迭代、28 次控制迭代；对象清单包含一个启用的
  `InvControl.VoltVar`。
- 比较范围：全部 3 个母线相复电压，以及全部 3 个非中性 PV 相的有功、无功输出。
- 实际最大归一化复电压误差：`3.3801558328329703e-10`，门限 `1e-9`。
- 实际最大 P/Q 额定容量归一化误差：`3.6963573337257865e-6`，门限 `1e-5`，
  分母为 PV 的 `1,200,000 VA` 额定容量。原生每相 Q 约 `-0.806 var`，PSA 每相约
  `+3.630 var`；两者的停止尾差均处于源文件显式 `VarChangeTolerance=1e-5` 的量级。
  发布门限使用总额定容量基准，数值上与该容差对齐，但不等同于控制器的每相停止判据。
- Consumer：`dss_import::pv_current_kvar_voltvar_snapshot_matches_locked_native_reference`；
  同一 helper 中两个固定 kvar sibling 仍使用 `1e-6 W/var` 绝对门限。

## release 与 portable bundle 门禁

本任务没有生产代码变更，因此复用按以下口径构建的现有 release：

```text
cargo build -p mems --release --no-default-features --features tensoreval-cli,with_dsa_mumps --bin tensoreval
```

`target/release/tensoreval` SHA-256：
`876ce8e214cec6ad56c541a658406648b3b5eca63cfb618e600459e0596dcc34`。

未修改官方入口的严格检查确认 15 条 DSS 语句、Snapshot、1 个 solve epoch 和
`circuit_bootstrap=true`。schema v2 bundle 被复制到第二个临时父目录后执行成功；
`voltage.csv` 恰含表头和 3 行母线相结果。对完整 bundle closure 搜索固定上游路径、
`/tmp` 和工作区绝对路径均无匹配。

发布位置：
`Examples/InverterModels/PVSystem/CurrentkvarLimite/PV_currentkvarLimit_VV.rssbundle/`。

## 模型定义入口扫描与 Kundur 结论

`model_only_snapshot_candidate_scan_2026-11-07.json` 固定了 42 个原 catalog
`not_standalone_entry`、且文件自身创建 Circuit 的入口；报告设置 `publishes_cases=false`，
路径使用 `<SOURCE_ROOT>` 与 `<SCAN_OUTPUT>` 占位符。使用同一 release 和显式
`--study snapshot` 后，7 个入口完成转换和严格检查。本轮只发布经过独立原生数值门禁的 VV
入口；其余 6 个保持原状态。规范化报告 SHA-256：
`e9d41f07a04945e3d63818685ccd3fd36b1e2477e65d53e44e731a44cf04b4f2`。

Kundur DynamicExp 的进一步探测证明 Monitor 导出只是首层阻断。移除输出命令后会暴露
`DynamicEq`/`DynOut` 外部 Generator 模型、Monitor `Term`/mode 语义；即使临时改用内置 H/D
模型，5 秒末原生与 PSA 转速分别约为 `0.6255 rad/s` 与 `3.8242 rad/s`。因此它不是下一项
小修改候选，未发布也未修改生产代码。

## 目录计数

只将本入口的 catalog 行由 `not_standalone_entry` 改为 `converted`。发布后 CSV 状态为
154 `converted`、108 `unsupported`、586 `not_standalone_entry`。独立入口分母由 261 增至
262，尚未发布的独立入口仍为 108。
