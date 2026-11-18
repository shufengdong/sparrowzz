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
  `c49f525e34a0b077f269d81e5039b1b5e666741beb2d22a297cf2c1285697950`。三个 bundle 统一使用
  `ds_dss_execution_plan` → `dss_execution` → `ds_execution_plan`，并由唯一 `para` 输入冻结
  Newton/exact 合同；内嵌
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
| IEEE123 SwapRef | `c912295fc9b42f357060ed723b1f51191a089981ad77469f84718738bc233e6e` | `d63d86ebfab97aee209a603d113f39b4e8e483e20a22d105cfca5b8f502c0b1f` | `0f8af0ea2f1673385ce549a39e08e0a83d48e5a33c4fd9ab1ef879f919c6baf9` | `50b4fe950a86491d05d8a1f91644f3d14dd39ccde4ae88520165283161b99242` | `93aac04c8afd379867404fa3809f79e35a99d0ac1d2e4c6f3b72bb49596ea06a` |
| AmpLimit SwapRef | `f4dd7e6b02e8989223335f5a22536e5fd281cc635bb6490e3cc9e0936eb8389b` | `88fa5dd8733807a28357c68fd4a12ceaa085770b154a5700dfffb34ee17e26be` | `8c6515a2d0752baf43a8d7039cac9ed7a621d3a85842151df3eeb149c23de8ea` | `dd23e0a28ba578844a5b67ecd947f178be7100b61df3ea5445181ca948d6df2d` | `93aac04c8afd379867404fa3809f79e35a99d0ac1d2e4c6f3b72bb49596ea06a` |
| AmpLimit Daily | `fa70afafae966b43e31291eb3dab78881350f94859da1265d9809d7acd796965` | `602c8426e0a5425ab45f001972774e1e5196c625155df0d0ddb283bcf366756f` | `087499b3374c799cbed5a2b90968960a84920a0743c77169168e4b797b8308d2` | `4a692a43fe3e94d990974b42d62f193da7e160ab97a72b9db3ecf61e67aa2837` | `726e655569f2a08875eaeffb1da8bba4f514b16ac13da82ba55acd490de64b00` |

三个 bundle 共用 `managed.rss` SHA-256
`00fa875a893eba2ed26109e944041b55b0d4e375c3adb433189c2aaaefc15de9`；共用
`inputs/para.parquet` SHA-256 为 `fd573beba2734d15a52873540b351ab1255364c219916bc3ea61683d1fcc1184`。候选目录原样安装后，
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
