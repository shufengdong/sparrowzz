# EPRI M1 无 PV Snapshot 发布证据

## 身份与范围

- 官方入口：`EPRITestCircuits/epri_dpv/M1/Master_NoPV.dss`
- 固定上游仓库：`dss-extensions/electricdss-tst`
- 固定上游提交：`3b208397160213cae4a9e2d0a7d1aa3528ce26e1`
- 入口 SHA-256：`ed1622057bbbb2bd25f3f86e0befeb6252ca3d17db7b301a0acf42ab8ff9d08a`
- 完整来源树 SHA-256：`3f38d2ebbefaab9058dea97f336aa6973b17bb730b271cbdbe7698390e76617b`
- 发布前父仓库锁定的 `sparrowzz` 提交：`9ee902e`
- bundle 首次发布提交：`f22018e`（`sparrowzz`）
- 父仓库实现与门禁提交：`84c84d04c`
- 发布位置：`EPRITestCircuits/epri_dpv/M1/Master_NoPV.rssbundle/`

发布合同是无 PV 官方入口的两个有序 Snapshot epoch，并只比较第二个 epoch 的最终电气状态。
不包含同目录 PV 入口、可靠性指标、Monitor 通道逐样本等价、QSTS 轨迹或 8,760 小时年度性能。

## 构建、bundle 与搬移证据

发布可执行文件按以下口径构建：

```text
cargo build -p mems --release --no-default-features --features tensoreval-cli,with_dsa_mumps --bin tensoreval
```

- `tensoreval` SHA-256：`3552ee2f31556d7e6f64ab47ab10d73ea8fd92c932080349dfe44368b9b3295c`
- schema：v2
- 严格检查：4,279 条 DSS 语句、2 个 solve epoch、`circuit_bootstrap=true`
- manifest 声明文件数：15
- 来源 closure SHA-256：`3f38d2ebbefaab9058dea97f336aa6973b17bb730b271cbdbe7698390e76617b`
- `manifest.json` SHA-256：`0cf85c5cd72cfc6e392c8d008f4dcc80e362e21b32f7711f51eadeb193b53bc2`
- 线路模型：公开 `main.rss` 和托管入口均显式使用 `exact`

原目录与复制到独立临时父目录并改名后的 bundle 都从公开 `main.rss` 自执行成功，各产生
6,356×11 的结果；两份结果均为 719,506 字节，SHA-256 都是
`fc4823a26c4a68567ee8bc2c0167c791b938ff657a3ee45ec406be0d38293ed8`。closure 扫描不含固定
上游临时目录、工作区绝对路径或 `.worktrees` 路径。

## OpenDSS 与 PSA 最终态门禁

原生参考由 OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5 对固定官方来源生成并收敛。原生 CSV
包含 3,153 个母线相电压，其 SHA-256 为
`12f2046abbcf9783e791325838668171f991ec7e78f3d62c5515c37273955228`。比较 JSON SHA-256 为
`c58f0f57352bdca786173915b7910952641ded562170d643e01e8605eeab21b3`，门禁 metadata SHA-256 为
`01c0069b92c2c5a30ded8132e18307002a152982a1ec47f706322660c5930929`。

| 指标 | 数量 | 最大绝对误差 | 最大归一化复误差 | RMS 归一化复误差 | 最差键 |
|---|---:|---:|---:|---:|---|
| 母线相电压 | 3,153 | `0.02045447826164971 V` | `9.141043688582145e-5` | `2.7928383278329492e-5` | `0x008e6fb0_drop-237-2 / 1` |
| 接口复功率 | 3 | `357.55199515845607 VA` | `6.054447326642671e-5` | `3.934058746697034e-5` | `line.temp2 / terminal-1` |
| 全网复损耗 | 1 | `381.6491567651887 VA` | `2.8505834749390895e-4` | `2.8505834749390895e-4` | `total` |

M1 调压 tap 的 OpenDSS 与 PSA 值均为 `1.01875`，绝对误差为零；电容器终态精确一致：
`Cap1=[0]`、`Cap2=[1]`、`Cap3=[1]`。

M1 substation transformer 参与求解并出现在 `branch_flow`，但不出现在 PSA 的
`transformer_winding_flow` 投影。门禁仍要求 winding 输出表存在，并按固定 SourceBus→temp
方向从 `branch_flow` 求和：绕组 1 使用 `sum(S_from)`，绕组 2 使用 `-sum(S_to)`。若相集合或
方向不匹配，门禁失败关闭。

OpenDSS 的 `AllBusNames` 不列出 25 个禁用开关端点，PSA 则显式保留它们且电压精确为零。门禁
固定这 25 个母线相键，要求每个键存在并精确为零；任何集合漂移、缺失或非零值均失败，活动节点
不会被该规则排除。

## 目录变化

`conversion_report.csv` 只将本入口由 `unsupported` 改为 `converted`。发布后状态为 180
`converted`、89 `unsupported`、579 `not_standalone_entry`，独立入口分母仍为 269。
