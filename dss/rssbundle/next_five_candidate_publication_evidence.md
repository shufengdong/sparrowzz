# 后续五个 OpenDSS 执行入口发布证据

## 固定输入与构建

- 上游：`dss-extensions/electricdss-tst` 提交
  `3b208397160213cae4a9e2d0a7d1aa3528ce26e1` 的未修改 `Version8/Distrib`。
- release `tensoreval` SHA-256：
  `e7e0a3cba88469a8fc191a927c067f57f0e47be7b39d941a5c8c860de0607f52`。
- 构建命令显式使用 `--no-default-features --features tensoreval-cli,with_dsa_mumps`。
- 所有 PSA/OpenDSS 比较通过 DSS plan 的 `line_model=exact` 合同执行；原生环境为
  OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5。

## Artifact 身份与搬移执行

| 官方入口 | 源 SHA-256 | solver | manifest 文件数 | manifest SHA-256 | 搬移输出行数 |
|---|---|---|---:|---|---:|
| `EPRITestCircuits/ckt5/Run_ckt5.dss` | `6fc218c3849f02c655fdbdedb6a35cd13cece50dcbd8bde5223c47d20726c9be` | Radial | 16 | `1ddbffbed1d51e233cfc33d51277ebb9a1eff7c5db0ef9e30cd265e9f544c921` | 34,588,428 |
| `Examples/Scripts/Storage-Quasi-Static-Example/Run_Demo1.dss` | `c94e1d5480e45bf20f2506e48e70ca1eca3fbb5c7ce45acef1317fb892c16999` | Radial | 17 | `e0e0a6b6e85e715c37b245562b75abb7c46b284171716e217cdf4e32177a9581` | 761,964 |
| `IEEETestCases/123Bus/Run_YearlySim.dss` | `7231261d7f13e119b18c03ebe217cdfae3864adc6b453bee858a8fcb796dbac7` | Radial | 11 | `9f16c24c72a0433213d84d0f8f2def6a1ea824f712974763016b9408c882dbbe` | 200,716 |
| `Examples/CIM/IEEE13_Assets.dss` | `679387c9d71611ba238761227dce69df670008517e3f3ba3cfef985b8a0ca92c` | Radial | 4 | `24fd32bc3a52c5e03b84755b607c6ec08e951e85159450e3bed5d64c292f56ff` | 86 |
| `EPRITestCircuits/ckt24/Run_Ckt24.dss` | `27c4e99f714c65576005e2574fd3873a18ee902fc1364c087bcb4f34db932f4c` | Newton-Raphson | 24 | `42562233494ab41974194f90a5e0ef28efe86163812067ddae473760377a6818` | 7,522 |

五个 bundle 均复制到独立 `/tmp` 父目录后直接执行成功。manifest 共声明 72 个文件；从
`sparrowzz` 提交后的 Git blob 逐一复算 SHA-256，零不匹配。Ckt24 官方 CRLF 资产
`AllocationFactors_Base.Txt` 由精确 `.gitattributes -text` 规则保留字节，避免新 checkout
破坏 manifest 哈希。

## 数值门禁

| 入口 | 最终状态 | 原生节点匹配 | 最大归一化误差 | RMS 误差 | 最大绝对误差 |
|---|---|---:|---:|---:|---:|
| Ckt5 | epoch 1，31,536,000 s，完整 8,760 点 | 3,437 / 3,437 | `8.20624e-4` | `1.67163e-4` | `0.200969 V` |
| Storage Demo | epoch 1，86,400 s，192 个 7.5 分钟点 | 3,437 / 3,437 | `9.34665e-4` | `1.79327e-4` | `0.227861 V` |
| IEEE123 Yearly | epoch 2，2,592,000 s，720 小时 | 278 / 278 | `2.10120e-6` | `1.15223e-6` | `0.00523409 V` |
| IEEE13 Assets | epoch 1，第二次 Solve | 41 / 41 | `2.75110e-3` | `4.37630e-4` | `6.33659 V` |
| Ckt24 | epoch 0，Snapshot | 7,522 / 7,522 | `3.10352e-2` | `3.20093e-3` | `7.31081 V` |

Ckt5 和 Storage 对共同的 3,437 个原生节点全部比较；PSA 另外显式保留 511 个 OpenDSS
节点清单不列出的相槽位，不用这些额外行替代原生键。IEEE13 的两个额外相槽位电压严格为零。
Ckt24 的最差点位于一个低压二次节点，采用锁定的 5% 大型二次网门限；全网 RMS 为 0.3201%。

Ckt5 官方脚本在完成最终 Solve 后请求不存在的 `ckt5mon` 导出，IEEE13 Assets 在第二次 Solve
后请求发行树中不存在的 UUID 文件；两者都只影响求解后外部文件副作用。原生数值读取保留已收敛的
最后电气状态，bundle manifest 则记录并忽略这些非电气副作用，不把它们描述为已生成的文件。

## Catalog 结论

`conversion_report.csv` 只把上述五个既有独立入口从 `unsupported` 改为 `converted`。
目录从 161/108/579 更新为 166 `converted`、103 `unsupported`、579
`not_standalone_entry`；独立入口分母保持 269。两个原拟选的 LVTestCaseNorthAmerican 脚本因包含
数百个跨场景清空、重建和求解顺序转换而未发布，未用路径白名单或静默删命令绕过。
