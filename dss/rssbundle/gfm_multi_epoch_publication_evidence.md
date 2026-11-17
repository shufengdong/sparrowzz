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
`c49f525e34a0b077f269d81e5039b1b5e666741beb2d22a297cf2c1285697950`。两个 bundle 均使用
`ds_dss_execution_plan`、profile `dss_execution` 和 `ds_execution_plan`，唯一 `para` 输入冻结
Newton/exact 合同。
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
| source tree | `a95930b9e48c5143596425de20f42c11b2b0d3a50a661eee7449b8a7605685e9` | `d24c0e728c35c83f0d534c68d23f9a13a3e566900369e74024a140cd62885117` |
| `manifest.json` | `2ae87bf9433ef83c77291141625551346e78de7762f763cb33204de51aeaf497` | `c73fc755919a977931f9df68c1d5a46041250933a054c32e233bce9a20ba4e7e` |
| `main.rss` | `c4be3ed98c52b5358d906db694e3cd62adb7a83699b6b39531c9e01b6fd31ccb` | `4170dd83fe8736bce44b0c6c34a02b36988d0eced46be4929fae99388fb0b1d0` |
| `managed.rss` | `00fa875a893eba2ed26109e944041b55b0d4e375c3adb433189c2aaaefc15de9` | `00fa875a893eba2ed26109e944041b55b0d4e375c3adb433189c2aaaefc15de9` |
| `inputs/para.parquet` | `fd573beba2734d15a52873540b351ab1255364c219916bc3ea61683d1fcc1184` | `fd573beba2734d15a52873540b351ab1255364c219916bc3ea61683d1fcc1184` |

锁定参考包含 2,272 条电压数据和 8 条 Storage 状态，SHA-256 分别为
`d74d416127f1efa6311fc544e7bdef26dc42d68f0fc9f30f9db4e118a1c73356` 与
`705deefae94297b6b46852a77304df3bf85e78bb1989f2727b4f751624b332a2`。

## 范围边界

`GFM_AmpsLimit_123` 的这个 Snapshot 入口本身没有声明 `AmpLimit` 或 `AmpLimitGain`；目录名
不构成这两个属性已经受支持的证据，其 Daily 兄弟入口仍保持 unsupported。IEEE8500 GFM
候选也没有发布：原生 OpenDSS 在 `Maxiterations=20` 下 epoch 1 不收敛，而 epoch 0 和 2
收敛；PSA 对失败 epoch 立即终止符合当前合同，跨失败 epoch 继续执行属于另一项能力。
