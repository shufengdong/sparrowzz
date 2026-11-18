# IEEE8500 GFM Snapshot bundle 发布证据

日期：2026-11-18

## 固定来源与发布范围

本批只发布官方入口：

- `Examples/Microgrid/GridFormingInverter/GFM_IEEE8500/Run_8500Node_GFMSnap.dss`

来源锁定为 `electricdss-tst@3b208397160213cae4a9e2d0a7d1aa3528ce26e1`。入口
SHA-256 为 `e8f9c801f15b329b6b57fed075d78dc3f839cd61fbcff7f513db8ab60de57d65`，
完整依赖树 SHA-256 为
`14e5e7f636be07569675b53ee024facbec6bdac53447f0617e44ab1608ffe210`。bundle 保留未修改
DSS 来源和依赖，使用 schema v2、`execution_plan`、Newton-Raphson，并显式冻结
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

最终 release `tensoreval` SHA-256 为
`a4478ba45f498a5ace00ddc20a59522797a047d8a876f5069720b3ca1c87a561`。源级
`--check` 验证 6,438 条 DSS 语句、3 个 solve epoch 和显式 circuit bootstrap。正式 bundle
在原位置执行成功，公开表形状如下：

| 结果 | 行数 | 列数 |
|---|---:|---:|
| `bus_phase` | 25,620 | 11 |
| `island_status` | 5 | 15 |
| `solve_attempt` | 3 | 13 |

`/usr/bin/time -l` 记录 43.34 秒 real、43.36 秒 user、1.59 秒 sys，最大常驻内存
741,720,064 字节，峰值内存 footprint 638,979,136 字节；低于 10 分钟和 8 GiB 门限。

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

同一锁定来源独立再生成到第二个临时目录后，19 个 bundle 文件经 `diff -rq` 逐文件一致。
完整 bundle 复制到不同父目录并改名后，在采集根之外仍执行成功，并返回相同三张表及形状，
因此 relocation Gate 9B 为绿色。

## 回归与 catalog

聚焦三表审计测试 4/4、failure-aware IEEE8500 生成器测试 3/3、manifest 三结果约束测试
1/1、完整 `dss2rss::` 回归 105/105 通过。完整回归同时锁定普通单结果 bundle 的公开
`voltage` 命名和既有 IEEE123 两结果路径，避免能力集合改造破坏旧 ABI。

`conversion_report.csv` 只把本入口从 `unsupported` 改为 `converted`。当前 848 个入口统计为
197 `converted`、73 `unsupported`、578 `not_standalone_entry`。

## 非目标范围

本批不发布 `Run_8500Node_GFMDaily.dss` 或
`Run_8500Node_GFMDailySmallerPV.dss`，不声明 radial GFM、动态 GFM、PVSystem GFM、任意 Daily
性能/数值边界，也不扩大 `numerical_parity=false` 的模型声明。后续入口仍须分别通过完整
failure-aware ledger、资源门限、确定性和 relocation 门禁。
