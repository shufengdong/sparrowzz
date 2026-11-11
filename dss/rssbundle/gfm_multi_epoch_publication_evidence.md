# GFM 多 epoch bundle 发布证据

本批从 OpenDSS 官方发行树提升并发布两个完整入口：

- `Examples/Microgrid/GridFormingInverter/GFM_IEEE123/Run_IEEE123Bus_GFMSnap.DSS`
- `Examples/Microgrid/GridFormingInverter/GFM_AmpsLimit_123/Run_IEEE123Bus_GFMSnap.DSS`

两者都保留 4 个原生 `Snapshot` epoch：GFL 初态、打开指定开关并切换为 GFM、保持
GFM 再求解，以及恢复 GFL/闭合开关或重复 GFM 求解。执行计划仅接受这个有界生命周期，
不会把任意 post-solve mutation 当作可发布序列。

## 严格转换和 bundle 自执行

发布构建遵循无默认 feature 口径：

```bash
cargo build -p mems --release --no-default-features \
  --features tensoreval-cli,with_dsa_mumps --bin tensoreval
```

本次 `tensoreval` SHA-256 为
`9ce8a62abe3b7b1935944eb1bc7884c0561459be455869d4285304c0b2941a5b`。
完整 GFM 入口严格验证 397 条展开语句和 4 个 epoch；AmpLimit 目录入口严格验证 392 条
展开语句和 4 个 epoch。两个 bundle 的原目录 `main.rss` 均独立执行成功，各产生 1,136
条 `voltage.csv` 数据记录。

为验证搬移性，bundle 分别复制并改名为 `renamed-full-gfm.rssbundle` 和
`renamed-amplimit-gfm.rssbundle` 后再次执行。搬移前后输出逐字节一致：

| 入口 | `voltage.csv` SHA-256 |
|---|---|
| GFM_IEEE123 | `c8ceb44ee04a55e9e76f56a113fe311619e2250f15b0d9c9ef5d860235ab2ee0` |
| GFM_AmpsLimit_123 | `260734cbe434f7a2d8ef84223e1679a1abc217e2cb6864b71a7d4c816fb8cbd0` |

运行时以 `.rssbundle` 后缀识别 acquisition root，因此搬移测试保留该后缀，只改变目录名和
上级路径。两个发布树均未发现 `/tmp`、本地工程目录或上游 checkout 的绝对路径泄漏。

## OpenDSS 数值门禁

参考由 OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5 从未修改的官方入口生成。PSA 门禁显式使用
`line_model="exact"`，逐 epoch 比较全部母线相电压和 Storage 复功率；开关打开的 epoch 还
验证 32 个失电导体。发布阈值为归一化电压 `1e-5`、Storage 复功率 `3 VA`。实测最大误差如下：

| 入口 | 最大归一化电压误差 | 最大 Storage 复功率误差 |
|---|---:|---:|
| GFM_IEEE123 | `4.112314026489773e-6` | `1.8875642641939643 VA` |
| GFM_AmpsLimit_123 | `4.112314026489773e-6` | `1.8875642641939643 VA` |

完整入口末 epoch 的官方源请求 `CHARGING`/GFL，但满电 Storage 在 OpenDSS 运行时归一化为
`IDLING`；比较器按 OpenDSS 的负载符号与 PSA 的注入符号进行显式换算。AmpLimit 目录入口的
重复 GFM epoch 在 OpenDSS 中也不是逐位相同，实测参考差异为 `7.23e-5 V` 和
`1.8872 VA`，仍在上述数值合同内。

## 来源和发布文件哈希

| 项目 | GFM_IEEE123 | GFM_AmpsLimit_123 |
|---|---|---|
| 官方入口 SHA-256 | `77934030e3010702d88ca381c2c63d5e82c5e8a718ba7dea0d53f9b2103fa1fd` | `505b6be8391ff8c62368524e8a365e915d5f934ad3ed5d7fe3a69c214a80c1ed` |
| `manifest.json` | `5013ee7013fc1244cc1e12a04d6e132ece2441c6d5d987bfe59e47f9959141b1` | `9fdb3ede83776eb7be3267a092b23b5f71089f00cd43d0d900dc646bea5fe8a5` |
| `main.rss` | `93f67c13107f1ce18c7b2aaf3f509913f738a4ddcb129f5b8729e3f19daae3f6` | `02c494bfdfba2fa4b810359efaf013524bbcac72505ab59f2706e2d75dd27db2` |
| `managed.rss` | `57a5f46d534568742cd79622f2dbc3cd4e8379605bb169883f15060644157a5b` | `57a5f46d534568742cd79622f2dbc3cd4e8379605bb169883f15060644157a5b` |

锁定参考包含 2,272 条电压数据和 8 条 Storage 状态，SHA-256 分别为
`d74d416127f1efa6311fc544e7bdef26dc42d68f0fc9f30f9db4e118a1c73356` 与
`705deefae94297b6b46852a77304df3bf85e78bb1989f2727b4f751624b332a2`。

## 范围边界

`GFM_AmpsLimit_123` 的这个 Snapshot 入口本身没有声明 `AmpLimit` 或 `AmpLimitGain`；目录名
不构成这两个属性已经受支持的证据，其 Daily 兄弟入口仍保持 unsupported。IEEE8500 GFM
候选也没有发布：原生 OpenDSS 在 `Maxiterations=20` 下 epoch 1 不收敛，而 epoch 0 和 2
收敛；PSA 对失败 epoch 立即终止符合当前合同，跨失败 epoch 继续执行属于另一项能力。
