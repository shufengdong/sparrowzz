# IEEE8500 GFM Snapshot 与 Yearly bundle 发布证据

日期：2026-11-18

## 固定来源与发布范围

本批发布三个官方入口：

- `Examples/Microgrid/GridFormingInverter/GFM_IEEE8500/Run_8500Node_GFMSnap.dss`
- `Examples/Microgrid/GridFormingInverter/GFM_IEEE8500/Run_8500Node_GFMDaily.dss`
- `Examples/Microgrid/GridFormingInverter/GFM_IEEE8500/Run_8500Node_GFMDailySmallerPV.dss`

来源锁定为 `electricdss-tst@3b208397160213cae4a9e2d0a7d1aa3528ce26e1`。入口
入口 SHA-256 与完整依赖树 SHA-256 分别为：

| 入口 | entry SHA-256 | tree SHA-256 |
|---|---|---|
| Snapshot | `e8f9c801f15b329b6b57fed075d78dc3f839cd61fbcff7f513db8ab60de57d65` | `14e5e7f636be07569675b53ee024facbec6bdac53447f0617e44ab1608ffe210` |
| Daily | `1332b26e774629f4609ffebc6547fc71e1384e2598ca55b7633a8c60632d2e84` | `f53f352f73e849c7993a66f0e682a72c11ed6481bca22e50f1c293422f9d168f` |
| DailySmallerPV | `59b6358790dca58fab3d0e0abb0a90ebb5e29b8891a80c4e5a07633e54975ec0` | `154eef4d289b80e02c5cb3dbad2807051e0e1bb9ecb8a4b653d4fe7663a303f6` |

三个 bundle 均保留未修改 DSS 来源和各自依赖，使用 schema v2、`execution_plan`、
Newton-Raphson，并显式冻结
`line_model=exact` 与有序结果 `bus_phase,island_status,solve_attempt`。

manifest 的 GFM provenance 固定 `model_identity=psa_gfm_approx_v1`、
`numerical_parity=false`、上述 revision 和入口哈希。本发布不把 PSA GFM 稳态近似声明为
OpenDSS GFM 数值等价。

## 构建与执行

发布 CLI 按仓库约定关闭默认 features：

```text
cargo build -p mems --release --no-default-features \
  --features tensoreval-cli,with_dsa_mumps --bin tensoreval
```

Task 6 Snapshot release `tensoreval` SHA-256 为
`a4478ba45f498a5ace00ddc20a59522797a047d8a876f5069720b3ca1c87a561`；Task 8 Yearly
release SHA-256 为
`774063ba5c4f87cb7f925acf4feaa8916958813621beee39d212426d7009fc01`。源级
`--check` 验证 6,438 条 DSS 语句、3 个 solve epoch 和显式 circuit bootstrap。正式 bundle
在原位置执行成功，公开表形状如下：

| 结果 | 行数 | 列数 |
|---|---:|---:|
| `bus_phase` | 25,620 | 11 |
| `island_status` | 5 | 15 |
| `solve_attempt` | 3 | 13 |

`/usr/bin/time -l` 记录 43.34 秒 real、43.36 秒 user、1.59 秒 sys，最大常驻内存
741,720,064 字节，峰值内存 footprint 638,979,136 字节；低于 10 分钟和 8 GiB 门限。

Yearly 资源门禁在 `macOS 26.6.2 (25G83)`、`arm64`、
`rustc 1.99.1 (58cf8f9c3 2026-10-13)`、`cargo 1.99.1`、MUMPS 5.9.1 上用上述
Task 8 release 二进制独立执行：

| 入口 | bus_phase | island_status | solve_attempt | real/user/sys | 最大 RSS | peak footprint | 门限 |
|---|---:|---:|---:|---|---:|---:|---|
| Daily | 213,500 | 33 | 25 | 203.99/202.74/2.63 s | 1,587,314,688 B | 1,806,029,888 B | <30 min、<8 GiB |
| DailySmallerPV | 828,380 | 129 | 97 | 413.59/408.75/5.15 s | 3,086,827,520 B | 5,119,798,720 B | <45 min、<8 GiB |

两个入口的公开尝试均为 `SOLVED`，`electrical_rows=FULL`、
`state_commit=NUMERIC_AND_DECLARATIVE`，没有 generic failure。Daily 的 33 条 status 与
DailySmallerPV 的 129 条 status 全部为 `ENERGIZED_CONVERGED`；两者 owner 序列均为
`null → null → Storage.mystorage → null`，GFM owner 只出现在开关断开的 epoch 2。

## Failure-aware 三表审计

三次 source attempt 的发布结果为
`SOLVED → CLASSIFIED_TERMINAL → SOLVED`，均满足 `electrical_rows=FULL`、
`state_commit=NUMERIC_AND_DECLARATIVE`、空 attempt reason/code 和连续恢复来源约束。
审计使用 ledger 的稳定键桥接：

- `bus_phase`：`epoch,solve_event_sequence,time`；
- `island_status`：`epoch,step,time`；
- 所有 attempt 均保留同一完整 canonical bus/phase identity。

typed island outcome 合计为 4 条 `ENERGIZED_CONVERGED`、1 条
`DEENERGIZED_NO_REFERENCE`、0 条 `GFM_CAPACITY_INSUFFICIENT`。epoch 1 的第二个岛被明确分类为
无参考终态，epoch 2 由 `Storage.mystorage` 作为 GFM reference 恢复；owner 序列为
`null → null → Storage.mystorage`。中间 terminal attempt 没有被丢弃、替换为 last-known-good，
也没有伪装为普通收敛。

## Artifact 哈希、确定性与搬移

| 项目 | SHA-256 |
|---|---|
| `manifest.json` | `244f0de5108697efb4c6c4440dfd6d7773a0e6c06d82c59124045ac8c5892be7` |
| `main.rss` | `76f1765d655d7a62f40ecd3fdd78241551f80e67d17ddd679f37d52b2aa90fd4` |
| `managed.rss` | `1a4796abdc748f3092f70ddce30383c138668089e77b83867f38e960670b5ba5` |
| `inputs/para.parquet` | `df0775b0188904c5cf0a151500b9e24e9dde2b8badac9871c6bae86e5a547a50` |
| `bus_phase.parquet` | `1fa54bb56e0593e83b37de05d88042e8b618a8c64105cff1ab1f2392a9c79aff` |
| `island_status.parquet` | `2c8a2b011f7795401109eccea4e4bcf5e74756d4a608f5402f41e3ed5ed52b40` |
| `solve_attempt.parquet` | `aab30a8aea2ceb7abf64e12a49cb8780e7828838cf0088866a1d16d2afd0286d` |

Yearly bundle 与原位置执行结果哈希如下：

| 入口/项目 | SHA-256 |
|---|---|
| Daily `manifest.json` | `e454ae35d431b254df70adf21360c99179a5f689b88c046f7f8a9576109a4cc0` |
| Daily `main.rss` | `0f2510a789c5ab16f21611cda8758e9d1ffa8388c019f0745a4ba855baeb442c` |
| Daily `bus_phase.parquet` | `1e55df0f3df8e3f4e88ba3b6b41b2cdb95e2e6a34ac083e51d1716c94e0a4415` |
| Daily `island_status.parquet` | `e582cdf8e5abbea71ec0839091e7700af308b97429a958e04a4913b3ca9d94a3` |
| Daily `solve_attempt.parquet` | `8667c28feb6f246531be51ea91d43bb719b8b4a3602c9e5003568e709c05030b` |
| DailySmallerPV `manifest.json` | `3f49b972fe7ebb483d706cfef861df1c732457cc8c509c19d21a102e009dba9b` |
| DailySmallerPV `main.rss` | `b78341d28c6db0083684dc4a62794fbb53a8609d99a06f09bfee17a5e97cf52f` |
| DailySmallerPV `bus_phase.parquet` | `805561b9ab76f2e4971a7ab512e43a4c28bdbc8c7360596ddd0d936cb517d0c1` |
| DailySmallerPV `island_status.parquet` | `5458049147110320bfb9ac0eea3b82236ca684e30cb1b783bafa98cffc216f33` |
| DailySmallerPV `solve_attempt.parquet` | `bb8579700f098bc8be282ccf100d582019ab12b64a51f374dcf06a8f99ae608c` |
| 两者 `managed.rss` | `1a4796abdc748f3092f70ddce30383c138668089e77b83867f38e960670b5ba5` |
| 两者独立生成的 `inputs/para.parquet` | `df0775b0188904c5cf0a151500b9e24e9dde2b8badac9871c6bae86e5a547a50` |

同一锁定来源独立再生成到第二个临时目录后，19 个 bundle 文件经 `diff -rq` 逐文件一致。
完整 bundle 复制到不同父目录并改名后，在采集根之外仍执行成功，并返回相同三张表及形状，
因此 relocation Gate 9B 为绿色。

Daily 与 DailySmallerPV 也分别独立再生成，两个案例各自的 19 个文件均逐字节一致。两个 bundle
复制到不同父目录并分别改名后再次执行，三张 Parquet 与原位置结果逐字节一致；该比较同时覆盖
Parquet schema、列顺序与内容哈希。虽然两者的 `para.parquet` 内容相同，但文件均由各自生成过程
独立创建；manifest、entry、source tree 和 RSS 主程序保持不同，不共享案例输入闭包。

## 回归与 catalog

聚焦三表审计测试 4/4、failure-aware IEEE8500 生成器测试 3/3、manifest 三结果约束测试
1/1、完整 `dss2rss::` 回归 105/105 通过。完整回归同时锁定普通单结果 bundle 的公开
`voltage` 命名和既有 IEEE123 两结果路径，避免能力集合改造破坏旧 ABI。
Task 7 的两个原生/两个 PSA Yearly 生命周期测试 4/4 通过；Task 8 的 catalog validator 78/78
通过，并实际遍历全部 199 个已发布 bundle 的路径无关 DataFrame 投影。

`conversion_report.csv` 把两个 Yearly 入口从 `unsupported` 改为 `converted`。当前 848 个入口统计为
199 `converted`、71 `unsupported`、578 `not_standalone_entry`。`case_projections.json` 已由
Task 5 通用登记 Snapshot/Daily/Yearly 等 `SnapshotOrQsts` 模式的权威 `solve_attempt` 能力，
本阶段不添加 IEEE8500 案例特例。

## 非目标范围

本批不声明辐射状 GFM、动态 GFM、PVSystem GFM 或 PSA/OpenDSS GFM 数值等价，也不扩大
`numerical_parity=false` 的模型声明。已记录的性能门限仅适用于上述机器、二进制和三个锁定入口；
后续入口仍须分别通过完整
failure-aware ledger、资源门限、确定性和 relocation 门禁。
