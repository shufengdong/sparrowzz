# PV current kvar limit 负 kvar 案例发布证据

## 身份与范围

- 官方入口：`Examples/InverterModels/PVSystem/NewFeatures/varCapability/PV_currentkvarLimit_kvarNEG.dss`
- 固定上游提交：`3b208397160213cae4a9e2d0a7d1aa3528ce26e1`
- 官方源文件 SHA-256：`8f3e3389ded5a057984f3efbdfce1962508bda51fea52b6c5bdc7ea457d35a79`
- 发布边界：显式 Snapshot、`line_model=exact`、Radial；不外推到外部 Python sweep 或其他生命周期。

## 独立原生数值证据

- Case id：`pv_var_capability.current_kvar_limit_negative`
- OpenDSSDirect.py：`0.9.4`
- DSS C-API：`0.14.5`
- 原生求解：收敛，2 次迭代。
- 负向能力：写入 `kvar=-500`，按 `kvarmaxabs=300` 限制为每相约 `-100 kvar`、三相合计
  `-300 kvar`；正向 sibling 的三相合计为 `+200 kvar`，两个分支独立取证。
- 比较范围：全部 3 个母线相复电压，以及全部 3 个非中性 PV 相的有功、无功输出。
- 实际最大归一化复电压误差：`3.3801475356637357e-10`，门限 `1e-9`。
- 实际最大有功绝对误差：`5.820766091346741e-11 W`，门限 `1e-6 W`。
- 实际最大无功绝对误差：`1.4551915228366852e-11 var`，门限 `1e-6 var`。
- Consumer：`dss_import::pv_negative_kvar_limit_snapshot_matches_locked_native_reference`。

## release 与 portable bundle 门禁

本任务没有生产代码变更，因此复用已按以下命令构建的现有 release：

```text
cargo build -p mems --release --no-default-features --features tensoreval-cli,with_dsa_mumps --bin tensoreval
```

`target/release/tensoreval` SHA-256：
`876ce8e214cec6ad56c541a658406648b3b5eca63cfb618e600459e0596dcc34`。

官方入口检查结果为 12 条 DSS 语句、Snapshot、1 个 solve epoch。schema v2 bundle 被复制到第二个
临时父目录后执行成功；`voltage.csv` 包含表头和 3 行母线相结果。对完整 bundle closure 搜索固定上游
路径、`/tmp` 和工作区绝对路径均无匹配。

发布位置：
`Examples/InverterModels/PVSystem/NewFeatures/varCapability/PV_currentkvarLimit_kvarNEG.rssbundle/`。

## 目录计数

该入口原状态为 `not_standalone_entry`，因此发布后 CSV 状态为 153 `converted`、108 `unsupported`、
587 `not_standalone_entry`。独立入口分母从 260 增至 261，尚未发布的独立入口仍为 108。
