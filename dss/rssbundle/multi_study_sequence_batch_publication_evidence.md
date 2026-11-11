# Multi-study sequence batch publication evidence

本批固定上游为 `electricdss-tst@3b208397160213cae4a9e2d0a7d1aa3528ce26e1`，发布构建命令显式使用
`--no-default-features --features tensoreval-cli,with_dsa_mumps`。release `tensoreval` SHA-256 为
`1bcfef83385e58ab582836cbcb64966d8c96166464153f95ddcecf416f8210e8`。

## 新增 bundle

| 官方入口 | bundle | schema / study | 原路径与搬移路径 |
|---|---|---|---|
| `IEEETestCases/NEVTestCase/Run_NEV.dss` | `IEEETestCases/NEVTestCase/Run_NEV.rssbundle` | v3 / `study_sequence` | 九张 Parquet 结果逐文件字节一致 |
| `Examples/InverterModels/PVSystem/InvControl/watt-pf_watt-var/dss/SnapShot_wattpf.dss` | 同路径 `SnapShot_wattpf.rssbundle` | v2 / `static_power_flow` | `voltage.parquet` 字节一致 |

NEV manifest SHA-256 为 `196241f68a1a451b24bf0139d79fd7919aa309028bea485c8bda25d30dfbd04d`。
checker 验证了 12 个清单文件、48 个固定输入及其 SHA-256，阶段严格为
`Snapshot → FaultStudy → Harmonic`，公开结果顺序为：

1. `s0_bus_phase`：192 行；
2. `s1_fault_result`：285 行；
3. `s2_bus_voltage`：384 行；
4. `s2_branch_flow`：2098 行；
5. `s2_conductor_flow`：362 行；
6. `s2_bus_sequence`：300 行；
7. `s2_branch_sequence`：564 行；
8. `s2_monitor`：0 行稳定 schema；
9. `s2_monitor_save`：0 行稳定 schema。

NEV Snapshot 的 OpenDSSDirect/PSA 键集合均为完整 192 个 `(bus, phase)`，无缺失、重复或额外键；
最大归一化复电压误差 `2.21931e-5`，最大角度误差 `0.000707223°`，分别低于锁定的
`1e-3` 和 `0.01°` 门限。FaultStudy 与 Harmonic 阶段由同一 schema-v3 调用连续执行，独立输出和搬移输出
哈希完全相同；`psa/ci/check_nev_study_sequence.py` 对缺失 FaultStudy、阶段错序、非收敛、频率错误、
键集合错误、三类数值越界和 manifest/source 文件哈希错误均 fail-closed。

WATTPF manifest SHA-256 为 `a450266c29de0f948beda3540d36c5639b213892a5165ece6362d5ef56a733e7`，
输出 `voltage.parquet` SHA-256 为 `7ff55e67644a1f6c478a3ff034b882073f56ae77a32940e68a706b86bc8a04f4`。
官方模型省略 `kVA`、`DERList` 和 `DeltaQ_Factor`，并以重复 X 值表达 PF 符号阶跃；PSA 保留 OpenDSS 的
500 kVA、唯一 PVSystem 和 `DeltaQ_Factor=-1` 默认值，并在能力圆投影前用可用有功计算 WATTPF 无功目标。
OpenDSSDirect 与 PSA 三相键集合完全一致，最大复电压绝对误差 `0.00230571 V`、归一化误差
`2.95244e-7`、幅值误差 `0.00195421 V`、角度误差 `8.97758e-6°`。

## 既有 bundle 兼容性

最终 release 同时在原目录和搬移改名目录执行了六个既有 schema-v2 bundle，公开结果列表和行数均
保持不变：

| bundle | 公开结果与行数 | 搬移比较 |
|---|---|---|
| `IEEETestCases/4Bus-YYD/YYD-Master.rssbundle` | `voltage` 36 | Parquet 字节一致 |
| `IEEETestCases/ShortCircuitCases/IEEE123Master-SC.rssbundle` | `fault` 414 | Parquet 字节一致 |
| `IEEETestCases/ShortCircuitCases/ieee37_SC_Currents.rssbundle` | `fault` 175 | Parquet 字节一致 |
| `Examples/FreqScan/Run_Scan.rssbundle` | 7 张表；`bus_voltage` 52,264、`branch_flow` 103,400、`conductor_flow` 0、`bus_sequence` 40,044、`branch_sequence` 77,832、`monitor` 564、`monitor_save` 0 | 全部 Parquet 字节一致 |
| `Examples/HarmonicsTMode/IEEE_519.rssbundle` | 6 张表；`bus_voltage`/`bus_sequence` 各 1,080、`branch_flow`/`branch_sequence` 各 1,890、`conductor_flow` 0、`monitor` 135 | 稳定键完全一致；独立求解最大差异仅 `voltage_im=1.13687e-13 V`、`voltage_angle=8.88178e-16°`，其他电压量与 THD 精确一致 |
| `Examples/HarmonicsVariableLoad/IEEE_519.rssbundle` | 6 张表；`bus_voltage`/`bus_sequence` 各 720、`branch_flow`/`branch_sequence` 各 1,260、`conductor_flow` 0、`monitor` 90 | 全部 Parquet 字节一致 |

兼容门禁还锁定了 ordinary grounded-Wye、未物化 synthetic return 和显式 neutral 三种 RegControl
量测语义，避免 full-conductor neutral 扩展使既有 IEEE123 基波控制失效。

两个 bundle 均扫描确认不包含 `/tmp`、仓库或 worktree 绝对路径。目录报告由
`180 converted / 89 unsupported / 579 not_standalone_entry` 更新为
`182 converted / 88 unsupported / 578 not_standalone_entry`。

## 候选替换说明

原计划的 `EPRITestCircuits/epri_dpv/K1/Master_NoPV.dss` 已完成 `Kron`、neutral、SetKVBase 和变压器
BatchEdit 等浅层缺口，但完整导体网络只能使用 Newton，且 epoch 0 的 QSTS Newton 数值不收敛，因此没有
伪装成已发布案例。全量重扫当时 89 个 unsupported 入口后只有 NEV 通过；随后对 42 个 model-only 入口
显式施加 Snapshot，选择了可由小范围 native-default 修正闭合且具原生数值对标价值的 WATTPF 案例作为
第二个新增 bundle。
