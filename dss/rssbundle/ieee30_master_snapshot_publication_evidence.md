# IEEE30 Master Snapshot 发布证据

## 身份与范围

- 官方入口：`IEEETestCases/IEEE 30 Bus/Master.dss`
- 固定上游提交：`3b208397160213cae4a9e2d0a7d1aa3528ce26e1`
- 官方源文件 SHA-256：`bb0b864c3e63955cad3c6f289832421c24cc0d4dccaf9f66c3be51e8c22d1aba`
- 发布边界：显式 Snapshot、Newton-Raphson、`line_model=exact`；不改变模型定义文件的自动分类，
  不把 wrapper `Run_IEEE30.DSS` 或 ADiakoptics sibling 纳入本入口。

## 锁定原生数值证据

现有 IEEE30 fixture 中的 `Master.dss` 与官方入口字节完全相同。PSA 可执行投影只显式补齐导入
合同并移除不改变电气结果的电压基准命令；`projection_evidence.json` 固定了两者分别由原生
OpenDSS NCIM 执行后的输出比较，以下七个文件逐字节一致：对象清单、节点电压、端口电流、端口
功率、元件损耗、控制状态和汇总。

- projection evidence SHA-256：
  `0cec2f3ed95ecff5824ab8d2555142a2321ae43f14e330584386571b1ecf8c45`
- native metadata SHA-256：
  `6ba487b970bcf3191d0f579b4fa1d4f79bd2c2e6b15d627634d0c97ceb8a88d5`
- native reference checksum manifest SHA-256：
  `82bdf55fef92edcd59982e40441a9d1124c06a3e696e3cc066963b5c2b97d895`
- Consumer：`dss_import::ieee30_snapshot_matches_locked_native_reference`
- PSA 门禁：显式 `line_model="exact"` 和 `max_iter=100`；比较 90 个节点、264 个端口电流、
  264 个端口功率、65 个元件损耗/功率，以及汇总与控制状态。

## release 与 portable bundle 门禁

本任务没有生产代码变更，因此复用按以下口径构建的现有 release：

```text
cargo build -p mems --release --no-default-features --features tensoreval-cli,with_dsa_mumps --bin tensoreval
```

`target/release/tensoreval` SHA-256：
`876ce8e214cec6ad56c541a658406648b3b5eca63cfb618e600459e0596dcc34`。

严格检查确认 101 条 DSS 语句、Snapshot、1 个 solve epoch 和 `circuit_bootstrap=true`。生成的
schema v2 bundle 使用 Newton-Raphson，`main.rss` 显式设置 `line_model=exact`；closure 只含 Master
与 `Lines.DSS`、`Transformers.DSS`、`Loads.DSS`、`Capacitors.DSS`、`Generators.DSS`，不含
`Run_IEEE30.DSS`。

bundle 复制到独立临时父目录后，从 bundle 根目录自执行成功；`voltage.csv` 含表头及全部 90 个
母线相结果。对 closure 搜索固定上游路径、工作区绝对路径和 `/tmp` 均无匹配。

发布位置：`IEEETestCases/IEEE 30 Bus/Master.rssbundle/`。

## 目录计数

只将本入口的 catalog 行由 `not_standalone_entry` 改为 `converted`。发布后 CSV 状态为
155 `converted`、108 `unsupported`、585 `not_standalone_entry`。独立入口分母由 262 增至
263，尚未发布的独立入口仍为 108。
