# 最终小候选组发布与终局扫描证据

## 固定输入和通用小修

- 上游固定为 `dss-extensions/electricdss-tst` 提交
  `3b208397160213cae4a9e2d0a7d1aa3528ce26e1` 的 `Version8/Distrib`，未修改官方输入。
- 发布 `tensoreval` SHA-256 为
  `b170bbe060821fda49a63a714ff41c6b968180d5e16a46912b80b8766ef7504e`；构建命令为
  `cargo build -p mems --release --no-default-features --features tensoreval-cli,with_dsa_mumps --bin tensoreval`。
- 通用修复只在 DSS `Clear` 时结束 LoadShape Normalize 审计对象生命周期，并支持已由原生属性
  清单锁定的 ExpControl `DERList` 成员别名和 `QBias`、`VRegMin`、`VRegMax`、`QMaxLead`、
  `QMaxLag`、`PreferQ` 默认值。官方 `Examples/ExpControl/Master.dss` 随后仍要求单相
  `VRegTau=300` 动态轨迹；现有 reference-backed 运行时只支持三相动态，因此未发布该案例，
  也未把数值语义扩展伪装成小修。

## SecondaryTestCircuit_modified 发布

| 官方入口 | 源 SHA-256 | manifest 声明文件数 | manifest SHA-256 | 搬移输出 |
|---|---|---:|---|---:|
| `IEEETestCases/SecondaryTestCircuit_modified/Master.DSS` | `3745d7e7c67cc2f02094f2a72b8acfdc3938318a70cfb85e8a6f3267d2527033` | 82 | `ef8c3f03d8e29c1ff5524362f8c40e1721037aaf2ca0c187700823babddd00b3` | 118×11 |

schema-v2 bundle 的实际目录严格为 82 个 `.files[]` 加 `manifest.json`。bundle 在原目录执行成功，
复制到独立 `/tmp` 父目录并改名后也执行成功；两次各产生 118 行，排序后逐字节一致。catalog
校验覆盖 177 个 projected bundles，报告 zero completeness diagnostics。

原生参考由 OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5 连续生成两次，CSV 均为
`27bebc9ac245a85720ff981d5544fdd281639ca5395645c0a04080b748786f4c`。显式
`line_model=exact` 的 Rust consumer 比较全部 118 个 native bus-phase 复电压和三相源端 P/Q：

| 指标 | 实测 | 门限 |
|---|---:|---:|
| 最大复电压相对误差 | `2.9768622780833826e-2` | `3.1e-2` |
| RMS 复电压相对误差 | `8.558778979334618e-3` | `9e-3` |
| 源相复功率最大相对误差 | `7.97605042875059e-2` | `8.1e-2` |

该证据只覆盖一个 Snapshot 终态，不声明 QSTS、8,760 小时性能或跨机器 SLA。

## 终局扫描结论

发布后 catalog 为 177 `converted`、92 `unsupported`、579 `not_standalone_entry`，独立入口
分母仍为 269。`candidate_scan_2026-11-08-final.json` 用上述 release 工具重新扫描全部 92 个
`unsupported`：85 个转换失败，7 个只能完成 linked conversion，严格 check 为 0 个成功。

- 唯一 scope 已确认且只剩一个新诊断的完整入口是
  `EPRITestCircuits/epri_dpv/M1/Master_NoPV.dss`；4,279 条语句和两个 epoch 均已导入，但 epoch 0
  被现有 QSTS 求解器以“network topology is unsupported”拒绝。radial 和 Newton 都已独立探测失败，
  这不是解析器小修。
- 其余 6 个 conversion-success 文件均为 `circuit_bootstrap=false` 的会话片段，依赖先前已存在的
  circuit，不能作为独立发布入口。
- 其他低 gap-count 项涉及 Direct/Time/Multi-Actor 生命周期、1–3 之外的相导体标签、缺失上游
  资产、执行后拓扑重构、动态设备/控制对象或混合 Fault/Harmonic 生命周期；都需要新的运行时、
  数据模型或数值 oracle。

因此在“不修改官方输入、必须独立 strict check、必须有原生 exact 数值门禁”的口径下，已经没有
能通过小修改发布的剩余案例。
