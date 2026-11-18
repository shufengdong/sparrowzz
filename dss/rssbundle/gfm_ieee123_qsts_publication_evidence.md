# IEEE123 GFM QSTS bundle 发布证据

日期：2026-11-15

## 固定来源与发布范围

本批只发布两个 OpenDSS 官方入口：

- `Examples/Microgrid/GridFormingInverter/GFM_IEEE123/Run_IEEE123Bus_GFMDaily.DSS`
- `Examples/Microgrid/GridFormingInverter/GFM_IEEE123/Run_IEEE123Bus_GFMWholeDaily.DSS`

来源锁定为 `electricdss-tst@3b208397160213cae4a9e2d0a7d1aa3528ce26e1`，入口 SHA-256
分别为 `17b993e4cc9193d84b05f89102e431798994a3357f096d7eeca4f4fc5965335b` 和
`ab1f52cb6b83dd4e23ab81847f139cb13d474b685471179d2665fa44e9f64728`。发布 bundle
保留未修改 DSS 来源和依赖，使用 schema v2、`execution_plan`、Newton-Raphson，以及有序输出
`bus_phase,island_status`。

## 构建、执行与 typed outcome

发布 CLI 按仓库约定关闭默认 features：

```text
cargo build -p mems --release --no-default-features \
  --features tensoreval-cli,with_dsa_mumps --bin tensoreval
```

release `tensoreval` SHA-256 为
`c49f525e34a0b077f269d81e5039b1b5e666741beb2d22a297cf2c1285697950`。
两个 bundle 均使用 `ds_dss_execution_plan`、profile `dss_execution` 和
`ds_execution_plan`；唯一 `para` 输入显式冻结 `solver=newton` 与 `line_model=exact`。

| 入口 | 求解尝试 | `island_status` 行 | `ENERGIZED_CONVERGED` | `DEENERGIZED_NO_REFERENCE` | `GFM_CAPACITY_INSUFFICIENT` |
|---|---:|---:|---:|---:|---:|
| GFMDaily | 25 | 31 | 31 | 0 | 0 |
| GFMWholeDaily | 25 | 49 | 49 | 0 | 0 |

三个 outcome 分别计数；本次没有 capacity 或无参考终态，因此没有把 classified terminal row
重新标记成普通收敛。两案均保留 `psa_gfm_approx_v1`、`numerical_parity=false` 和固定上游
revision/入口哈希 provenance。

## Native 门禁与模型边界

原生参考由 OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5 生成。所有 PSA/native 数值断言显式
使用 `line_model="exact"`。严格电压窗口为两案初始 Snapshot、GFMDaily 小时 1-10、17、21-24；
GFM-active 电气量只作探索证据。

GFMDaily 小时 18-20 是显式 post-GFM 过渡排除：OpenDSS 的 GFM SOC 轨迹使它比 PSA 早一个
区间充满，状态化网络轨迹到小时 21 才重新进入 `1e-5` 电压门限。保留窗口的最大归一化复电压
误差为 `5.5234035082016954e-6`。小时 17 的 OpenDSS Storage 端口 P/Q 包含 `%R=50`
内部阻抗项，而 PSA 稳态合同不重复该项；该单点误差为 `29.937693242669997 VA`，使用窄化
`35 VA` 门限，其余对标点仍使用 `3 VA`。GFMWholeDaily 初始 Snapshot 最大电压误差为
`9.424552263460009e-7`，Storage 复功率误差为 `0.016574570877822384 VA`。

## Artifact 哈希、确定性与搬移

| 项目 | GFMDaily | GFMWholeDaily |
|---|---|---|
| source tree | `b0a9d69d8975b4efe02177d200a7fe4aa50a3208cb039645acfd2ffae98c8f76` | `88a38148d32ea72883a7463306674a8388d2961cd0da29ad2918fd10e2ac238f` |
| `manifest.json` | `4e7600f89ccd230169f7e643507e8c8a86587331c17cf83a0dd01969c70c6bb7` | `774087dc8dd4ab7042935c4b71a082403bba63716a4fe7f96121583fa687f2fe` |
| `main.rss` | `cc661d48873fad055eb38ae7594e6cd7a16b88f3047e66d2fbde71095bee77f8` | `e20f5b96b3407dc1e1ab39fec632d17c25be47e9b43d7548f6f8e9fe4bb4dd62` |
| `managed.rss` | `00fa875a893eba2ed26109e944041b55b0d4e375c3adb433189c2aaaefc15de9` | `00fa875a893eba2ed26109e944041b55b0d4e375c3adb433189c2aaaefc15de9` |
| `inputs/para.parquet` | `fd573beba2734d15a52873540b351ab1255364c219916bc3ea61683d1fcc1184` | `fd573beba2734d15a52873540b351ab1255364c219916bc3ea61683d1fcc1184` |
| `bus_phase.parquet` | `8593268dd93370f2b910855ecbb9ae061f141aa1ce2dbb2ef0cb1b6839572750` | `9051b4767ee9140985ad5f179407eafe2a7d7c6ef0704fd321b7b7487b8f26a5` |
| `island_status.parquet` | `cabe6d6bf1f1bdbaad765270b2e1ac1c91f4379cd3ccc9eac7befcf7f7bc1bdf` | `21cf9e38d05b0ff24eefd5bc598e7f4b65ca2b552bc80d3bc8ce042affd47503` |

候选只按字节复制到最终位置，没有重新生成。第二个独立 candidate root 的 bundle 树逐文件一致；
最终位置执行后，再复制到独立父目录并改名执行，四个 Parquet 输出仍与上表逐字节一致。两份
artifact 全量文本扫描均未发现 acquisition、临时目录或 worktree 绝对路径。

## Catalog 与范围边界

`conversion_report.csv` 只把上述两行从 `unsupported` 改为 `converted`，清空旧诊断并记录
bounded V2/native/搬移证据。全表仍为 848 行，当前计数为 196 `converted`、74 `unsupported`、
578 `not_standalone_entry`；其余 846 行的规范化 SHA-256 为
`f131615c9914dea4ca3c49ea2b7fe20a368c8cb9f336cb64e9d90b073d5b5900`。

本次明确不声明或实施：SwapRef、AmpLimit Daily、IEEE8500 GFM、Dynamic GFM、PVSystem GFM、
radial GFM、任意 Daily GFM，以及最终十五入口审计闭环。这两个有界官方 QSTS 入口的发布不会
扩大到上述能力。
