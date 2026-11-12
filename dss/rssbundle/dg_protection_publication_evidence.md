# DG Protection Snapshot 发布证据

日期：2026-11-12

## 固定来源与范围

- 官方入口：`IEEETestCases/DG_Protection/DG_Prot_Fdr.dss`。
- 上游：`electricdss-tst@3b208397160213cae4a9e2d0a7d1aa3528ce26e1`，根目录
  `Version8/Distrib`。
- 入口 SHA-256：`d19abd4047f04549d0421aa63cbef4b7ae215f41091d41280d656bcdf9aad6dc`。
- 发布位置：`IEEETestCases/DG_Protection/DG_Prot_Fdr.rssbundle/`。

入口保持字节不变。执行合同覆盖一个 Snapshot、三相故障、`Transformer.Tg` 端子 2 的原生
开路状态，以及三个 Recloser、两个 Fuse 和一个 Relay 的禁用终态。本次不声明 Monitor 通道、
保护动作时序、Dynamic、GFM 或年度/QSTS 性能等价。

## 有界实现合同

- 只有显式限定的初始 `Open Transformer.<name> <terminal>` 可以进入执行计划；不限定类名、
  非法端子和其他未支持对象继续失败关闭。
- 未指定导体的 Transformer 端子动作遵循 OpenDSS：只切换相导体并保留中性导体。官方
  `Transformer.Tg` 端子 2 的状态锁定为 `[true,true,true,false]`。
- Recloser `New ... Like=...` 只允许后续直接写 `enabled`；其他属性、嵌套 Like 和动作副作用
  继续失败关闭。
- `Vsource.WindGen1` 使用 `X0=1e99` 表示三线电源。它在变压器开路后成为独立电源孤岛；短路
  Norton 系统只在这种无其他设备的三相孤立源母线上补一个零均值公共模基准。
- 活动 Fault 且没有启用 Fuse/Recloser/Relay 时，最终 Snapshot 发布已经计算出的故障后母线
  和支路表；存在启用保护时仍保留原保护时序合同。

## 构建、严格检查与 artifact

发布可执行文件使用以下显式 feature 边界构建：

```text
cargo build -p mems --release --no-default-features \
  --features tensoreval-cli,with_dsa_mumps --bin tensoreval
```

- release `tensoreval` SHA-256：
  `4ed3eb8976b4705f6629a50b5d3540ecaa9f004fc7314697f0efd982d78dab59`。
- 严格检查识别 69 条 DSS 语句、1 个 solve epoch、`circuit_bootstrap=true`。
- bundle schema：v2；study：`execution_plan`；兼容性：`opendss-native`；求解器：
  `newton-raphson`。
- source tree SHA-256：
  `6e5dc08d8c1cf82baf4c0350a9654d5863201f9690c51f0e5f260d7fe5e3c8a8`。
- `manifest.json` SHA-256：
  `27b2639b0068951915b7bdf245aaa18a63cbb94da22f2a7fb197341c67f545b8`。

原位置与复制到独立临时父目录并改名后的 bundle 都自执行成功，均输出 26×11 的
`voltage.csv`。两份文件均为 2,671 字节、27 行（含表头），SHA-256 均为
`c4a6024f721f8d02e2f24b12fc8c776abe9567d4b0d546babe38f1daa0f8309f`，且逐字节一致。
artifact 全量检索未发现采集根、`/tmp` 或工作树绝对路径。

## OpenDSS 原生数值门禁

锁定参考由 OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5 直接执行未修改官方入口生成；PSA 不参与
期望值生成。PSA 比较显式设置 `line_model="exact"`。

| 指标 | 结果 |
|---|---:|
| 母线相复电压键 | 26 |
| 非故障零电压键最大相对误差 | `0.005259176861490091` |
| 最差键 | `Bm.2` |
| 原生幅值小于 1 V 的故障母线键，PSA 最大绝对幅值 | `0.0765114979168519 V` |
| OpenDSS 收敛迭代 / 控制迭代 | `2 / 1` |
| 禁用保护对象 | `3 Recloser + 2 Fuse + 1 Relay` |
| `Transformer.Tg` 端子 2 导体状态 | `[open, open, open, closed]` |

门禁使用 `0.6%` 的复电压相对误差上限；实际最大值约 `0.526%`。原生接近零的故障母线电压
单独使用 `0.2 V` 绝对上限，避免用接近零的分母放大数值噪声。

参考生成器、版本、行数和所有输出哈希保存在 PSA 的
`tests/bmopf/opendss_inputs/dg_protection/`，专用回归同时校验来源哈希、禁用控制状态、端子状态和
全部 26 个节点。

目录只把该入口一行从 `unsupported` 改为 `converted`。发布后为 191 `converted`、79
`unsupported`、578 `not_standalone_entry`。
