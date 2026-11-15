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
`647f5178c0dbbbcf2111f00fbad6a4a4a1479b4431572b702f37bad9a160d1dc`。

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
| source tree | `c322ea464ebd70b66093556924c223bce0f6e8f024c176c7529cc4b8af676c0d` | `d130bbdcd083e7531f6441dd6067892009f4c804e9043bb71528d3687ab2999c` |
| `manifest.json` | `9d4d4cf1c08747b374af3a61d2f4e62d880452cec88e657a9bd49a0a9dd2d80b` | `ba480eac9f79ccc2e5043c8c398ab47ac0bc949f49e4fdbd3331aeae67215601` |
| `main.rss` | `580e2a57b28065e146655c8d5d9a0aa4662f3d8380e25ef18a5f4feb2e22d32d` | `07b9dc0799de082a8ba9f174d8546281b7623c9fc458c8a8887ff101255a62c4` |
| `managed.rss` | `30f4b4492f42d0f629532b5823740786d5d723de4c378ff9d9e04ef6d1d863a4` | `30f4b4492f42d0f629532b5823740786d5d723de4c378ff9d9e04ef6d1d863a4` |
| `bus_phase.parquet` | `8593268dd93370f2b910855ecbb9ae061f141aa1ce2dbb2ef0cb1b6839572750` | `9051b4767ee9140985ad5f179407eafe2a7d7c6ef0704fd321b7b7487b8f26a5` |
| `island_status.parquet` | `cabe6d6bf1f1bdbaad765270b2e1ac1c91f4379cd3ccc9eac7befcf7f7bc1bdf` | `21cf9e38d05b0ff24eefd5bc598e7f4b65ca2b552bc80d3bc8ce042affd47503` |

候选只按字节复制到最终位置，没有重新生成。第二个独立 candidate root 的 bundle 树逐文件一致；
最终位置执行后，再复制到独立父目录并改名执行，四个 Parquet 输出仍与上表逐字节一致。两份
artifact 全量文本扫描均未发现 acquisition、临时目录或 worktree 绝对路径。

## Catalog 与范围边界

`conversion_report.csv` 只把上述两行从 `unsupported` 改为 `converted`，清空旧诊断并记录
bounded V2/native/搬移证据。全表仍为 848 行，计数为 193 `converted`、77 `unsupported`、
578 `not_standalone_entry`；其余 846 行的规范化 SHA-256 为
`cd5ad9a4a5a16c50565e12df746c3177bcb9465a60a4bcb96ae7273d04783294`。

本次明确不声明或实施：SwapRef、AmpLimit Daily、IEEE8500 GFM、Dynamic GFM、PVSystem GFM、
radial GFM、任意 Daily GFM，以及最终十五入口审计闭环。这两个有界官方 QSTS 入口的发布不会
扩大到上述能力。
