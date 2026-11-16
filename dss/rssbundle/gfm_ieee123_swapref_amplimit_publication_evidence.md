# IEEE123 GFM SwapRef/AmpLimit 发布证据

本批只提升固定上游版本中的三个未修改入口，执行模型仍标识为
`psa_gfm_approx_v1`，`source_model=opendss_builtin_gfm`，且
`numerical_parity=false`。主动 GFM 时段只做结构、所有权、连续性和限流门禁；OpenDSS 数值比较仅
覆盖冻结的非 GFM 窗口。

## 固定来源与工具链

- 上游提交：`3b208397160213cae4a9e2d0a7d1aa3528ce26e1`。
- 入口 SHA-256：
  - `GFM_IEEE123/Run_IEEE123Bus_GFMDailySwapRef.DSS`：`56efe1adc9adb9f0dd7345ea090568e8d12d15a6febf9f5f3de1acc318e785cd`；
  - `GFM_AmpsLimit_123/Run_IEEE123Bus_GFMDailySwapRef.DSS`：`c87125a1172d53ca5c60994f8f65941738955f3120748d44a49a00785193fa18`；
  - `GFM_AmpsLimit_123/Run_IEEE123Bus_GFMDaily.DSS`：`609882f2e46acbc2331be32825a9fc3321317b56530454797bd4846285769c2d`。
- 独立原生证据使用 OpenDSSDirect.py `0.9.4` / DSS C-API `0.14.5`；
  `metadata.json`、`native_bus_phase.csv`、`native_device.csv` 的 SHA-256 分别为
  `465b1c6b9ab9abe6d1aca3f41392a23f9018d237b330119af6db93bbd003b358`、
  `40b5c1b36fb80230684b1b12d58be6265a7dc053a0cc9ac75ecef1b9e417e5bf`、
  `b332e6b45810418040568d02fcfb6813701759631d563e93b0d46b552eb05b79`。
- 发布构建命令为
  `cargo build -p mems --release --no-default-features --features tensoreval-cli,with_dsa_mumps --bin tensoreval`；
  可执行文件 SHA-256 为
  `2878f90035d8387eee419e39e2337218c751992ea5746621e63a2a4f450bbf86`；内嵌
  `version-rs` 身份为提交 `4d677d55a`，提交时间 `2026-11-16 03:38:02 +00:00`。

## 执行、所有权与限流

三个入口均完成 25 次求解。两个 SwapRef bundle 各输出 49 条 `island_status`，已完成 barrier
的所有权序列均为 `None → Storage.mystorage → PVSystem.myPV → Storage.mystorage`；AmpLimit
Daily 输出 31 条，所有权序列为 `None → None → Storage.mystorage → None`。两个 SwapRef
各有 49 条 `ENERGIZED_CONVERGED`；AmpLimit Daily 有 29 条 `ENERGIZED_CONVERGED` 和 2 条
`GFM_CAPACITY_INSUFFICIENT`。三者均无 `DEENERGIZED_NO_REFERENCE`，且没有把 capacity 结果
重标为普通收敛。

AmpLimit Daily 的 `Storage.mystorage` 在 typed `island_status` 中有 2 条
`i_limited=true` 的 capacity 终止记录；它们在所有公共支路达到限流时保留最后一个已接受的
切换前解，而不是发布为普通收敛。有限且处于 active Storage GFM 状态的最大观测电流为
`320.0171993056328 A`。声明的平衡 Delta `AmpLimit=500 A` 先规范化为
`500/sqrt(3) A` Delta 支路能力；OpenDSS Storage GFM 适配器把端口改为 FourLeg 对地支路，
再映射为等效的 `500 A` 公共支路/导体限值。这是一次保持拓扑语义的约定转换。
`AmpLimitGain=0.76` 保持来源信息，不进入 `psa_gfm_approx_v1` 方程。

## 有界精确线路数值窗口

所有 PSA/OpenDSS 数值断言显式设置 `line_model="exact"`。两个 SwapRef 入口只比较 step 0；
AmpLimit Daily 只比较并网 step 0..10。归一化复电压门限为 `1e-5`，设备复功率门限通常为
`3 VA`；两个 SwapRef 的 step 0 `PVSystem.myPV` 因已锁定的 OpenDSS 端口残差局部放宽到
`4 VA`。

| 案例 | 比较步 | 最大归一化复电压误差 | 最大设备复功率误差 |
|---|---:|---:|---:|
| IEEE123 SwapRef | 0 | `5.803924977559147e-7` | `3.7459818143985375 VA` |
| AmpLimit 目录 SwapRef | 0 | `5.803924977559147e-7` | `3.7459818143985375 VA` |
| AmpLimit Daily | 0..10 | `5.523403508204881e-6` | `1.6974251806441745 VA` |

## Bundle 与结果身份

| 案例 | source tree | manifest | main.rss | bus_phase Parquet | island_status Parquet |
|---|---|---|---|---|---|
| IEEE123 SwapRef | `dba60efe8f4875996a800e1a165000cccd1919894b7edeb9318cc5ebeb609a3c` | `26bec20ed208cd83d8600a71d70fd8048be2585951c6d4c041a6c77c48f1e07c` | `56980e4d11d37dfca3ee9cfa4911e426c78f165d8ee5b1877ef16a69abc626e1` | `50b4fe950a86491d05d8a1f91644f3d14dd39ccde4ae88520165283161b99242` | `93aac04c8afd379867404fa3809f79e35a99d0ac1d2e4c6f3b72bb49596ea06a` |
| AmpLimit SwapRef | `eb3e4f3662864de4321e6fd0a61444ced48bb3c9e8179c5b871cca897c135404` | `c42c46d6e50cdd0a663840a534cb5996cfcaec286772fe684eb83d25a743d35a` | `9876bd167bc9cb1fdcddcefadb9daa7dbd0b10a9e94f4aeef565db34c573fce5` | `dd23e0a28ba578844a5b67ecd947f178be7100b61df3ea5445181ca948d6df2d` | `93aac04c8afd379867404fa3809f79e35a99d0ac1d2e4c6f3b72bb49596ea06a` |
| AmpLimit Daily | `4c09fe1223a18bbd2bf35c5620f6b9e11d4016a99fc69aa53041e07a7d4b3c09` | `4521d667c80aaf6ef8e28e0dfee6f1d05c0eddedecf8351f495ad355c847222c` | `83eccff0a4ba9eb8df52704cc5aa655966d89785f941a9e93252a8a4f8839145` | `4a692a43fe3e94d990974b42d62f193da7e160ab97a72b9db3ecf61e67aa2837` | `726e655569f2a08875eaeffb1da8bba4f514b16ac13da82ba55acd490de64b00` |

三个 bundle 共用 `managed.rss` SHA-256
`30f4b4492f42d0f629532b5823740786d5d723de4c378ff9d9e04ef6d1d863a4`。候选目录原样安装后，
又在改名父目录中、上游采集根不可见时重新执行；两组 `bus_phase` / `island_status` Parquet
逐字节一致，bundle 内绝对构建路径匹配数为 0。发布检查器还直接读取两组 Parquet，核对
完整 schema/dtype、每次 284 个唯一且顺序稳定的 bus/phase 坐标、有限乘数、终止岛全 NaN
电压、epoch/step/time 顺序，以及逐 attempt 的 island ordinal、outcome/reason、限流标志、
reference kind/device、owner 和模型身份。AmpLimit Daily 的 capacity 精确发生在全局 attempt
12、13（epoch 2 的局部 step 1、2）；此时所有 GFM 支路已转为 PQ，故公共
`reference_kind=NONE`，但仍保留 `Storage.mystorage` 与 `psa_gfm_approx_v1` 作为 capacity
证据。检查器还拒绝符号链接或哈希不符的 `--tensoreval`，并确认搬移前后语义一致。

## Catalog 与范围边界

`conversion_report.csv` 仅提升上述三行，保持 848 行；当前计数为 196 `converted`、74
`unsupported`、578 `not_standalone_entry`。除三条目标行外的 845 行规范化 SHA-256 为
`00b5884edec592aa6cccb3283f61232151cee45d5ae970e92777ccf905206641`。两个既有未加引号的
reason 逗号按末列自由文本规范化后参与该摘要，源 CSV 字节未改写。

本批不声明 OpenDSS 主动 GFM 数值等价、任意 GFM 参考调度或同时多参考、`AmpLimitGain`/`Kp`/
`PITol`/`kVDC` 的电气语义、IEEE8500 GFM、Dynamic 或 Snapshot-to-Dynamic GFM、径向 GFM，
也不代表最终十五入口审计已关闭。
