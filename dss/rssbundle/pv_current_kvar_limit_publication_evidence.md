# PV current kvar limit 正 kvar 案例发布证据

## 身份与范围

- 官方入口：`Examples/InverterModels/PVSystem/NewFeatures/varCapability/PV_currentkvarLimit_kvar.dss`
- 固定上游提交：`3b208397160213cae4a9e2d0a7d1aa3528ce26e1`
- 官方源文件 SHA-256：`9134bf1c0f17a06a9cd94781f625536d86ee76f29bc240904a1ff5078e4bd009`
- 发布边界：显式 Snapshot、`line_model=exact`、Radial；不覆盖负 kvar sibling 或其他生命周期。

## 原生数值证据

- OpenDSSDirect.py：`0.9.4`
- DSS C-API：`0.14.5`
- 原生求解：收敛，2 次迭代。
- 比较范围：全部 3 个母线相复电压，以及全部 3 个非中性 PV 相的有功、无功输出。
- 门限：最大归一化复电压误差 `1e-9`；有功/无功最大绝对误差 `1e-6 W/var`。
- 消费测试：`dss_import::pv_kvar_limit_snapshot_matches_locked_native_reference`，通过。

## release 与 portable bundle 门禁

release 构建命令：

```text
cargo build -p mems --release --no-default-features --features tensoreval-cli,with_dsa_mumps --bin tensoreval
```

`target/release/tensoreval` SHA-256：
`876ce8e214cec6ad56c541a658406648b3b5eca63cfb618e600459e0596dcc34`。

官方入口检查结果为 12 条 DSS 语句、Snapshot、1 个 solve epoch。生成的 schema v2 bundle 被移动到
第二个临时父目录后，从 bundle 根目录执行成功；`voltage.csv` 包含表头和 3 行母线相结果。对完整
bundle 搜索固定上游临时目录绝对路径无匹配，说明执行不依赖原始源树位置。

发布位置：
`Examples/InverterModels/PVSystem/NewFeatures/varCapability/PV_currentkvarLimit_kvar.rssbundle/`。

## 未发布边界

`PV_currentkvarLimit_kvarNEG.dss` 未包含在本次数值参考与 portable bundle 门禁中，目录状态保持
`not_standalone_entry`。后续必须独立锁定负 kvar 原生参考并重复相同门禁，才能发布。
