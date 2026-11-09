# IEEE8500 Master-unbal Snapshot 发布证据

## 身份与范围

- 官方入口：`IEEETestCases/8500-Node/Master-unbal.dss`
- 固定上游提交：`3b208397160213cae4a9e2d0a7d1aa3528ce26e1`
- 官方入口与 PSA fixture SHA-256：
  `8cbb671b1e353b72d445f361547bd43ac8ed3c90878d7c8dced26ba3bc55f043`
- 发布边界：显式 Snapshot、Radial、`line_model=exact`；不包含
  `Run_8500Node_Unbal.dss` wrapper、GFM duplicate 或其他 IEEE8500 study。

## 锁定原生数值证据

现有 `psa/tests/bmopf/opendss_inputs/ieee8500/native_reference/unbalanced/` 由
OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5 对字节相同的 fixture 生成并收敛：8,531 个节点电压、
24,195 个端口电流、24,195 个端口功率、7,275 个元件损耗、7,280 个对象、3,579 个控制状态和
11 个汇总值。

- native metadata SHA-256：
  `509da79a2559a9fff130c18b4e20acc23a9a490767ccee64dd22670ac47dcbc6`
- native reference checksum manifest SHA-256：
  `0223d4cd37c9be035a17306b74e28bc43bd53fc7adba441a718b0a9cda4071f1`
- Consumer：
  `dss_import::ieee8500_original_decks_lower_to_single_step_static_control_execution_profiles`
- PSA 门禁：显式 `line_model="exact"`、`max_iter=500`、`max_control_iter=100`；比较全部
  8,531 个原生节点，并比较 15,882 个可映射端口电流、15,882 个端口功率、7,244 个元件
  损耗/功率、源端汇总、复损耗、9 个启用电容器、12 个调压器与 1,190 个变压器 tap 组。
  电压相对复数门限为 2%；电流和功率使用绝对值加 5% 相对值门限；总复损耗门限为 5%。

## release 与 portable bundle 门禁

本任务没有生产代码变更，因此复用按以下口径构建的现有 release：

```text
cargo build -p mems --release --no-default-features --features tensoreval-cli,with_dsa_mumps --bin tensoreval
```

`target/release/tensoreval` SHA-256：
`876ce8e214cec6ad56c541a658406648b3b5eca63cfb618e600459e0596dcc34`。

严格检查确认 7,590 条 DSS 语句、Snapshot、1 个 solve epoch 和 `circuit_bootstrap=true`。生成的
schema v2 bundle 使用 Radial，`main.rss` 显式设置 `line_model=exact`；manifest 的 14 个文件哈希
全部自洽，closure 包含 Master、11 个 Redirect/BusCoords 依赖、`main.rss` 和 `managed.rss`。

bundle 复制到独立临时父目录后，从 bundle 根目录自执行成功；`voltage.csv` 含表头及 11,038 行
完整 PSA `bus_phase` 结果。原生 consumer 将 8,531 个 OpenDSS 节点作为该结果的必含子集逐项
数值比较。closure 对固定上游路径、工作区绝对路径、`/tmp`、wrapper 和 GFM sibling 的扫描均无匹配。

发布位置：`IEEETestCases/8500-Node/Master-unbal.rssbundle/`。

## 目录计数

只将本入口的 catalog 行由 `not_standalone_entry` 改为 `converted`。发布后 CSV 状态为
156 `converted`、108 `unsupported`、584 `not_standalone_entry`。独立入口分母由 263 增至
264，尚未发布的独立入口仍为 108。
