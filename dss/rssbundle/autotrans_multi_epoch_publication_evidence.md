# Auto1bus / Auto3bus 八 epoch 发布证据

日期：2026-11-11

## 固定来源与 artifact

- 上游：`electricdss-tst@3b208397160213cae4a9e2d0a7d1aa3528ce26e1`，根目录
  `Version8/Distrib`。
- `Examples/AutoTrans/Auto1bus.dss` 源 SHA-256：
  `64e98e52196bae0309ebbe17e36acc59d8315d4fa4f76a4ca13b4196982f18c1`。
- `Examples/AutoTrans/Auto3bus.dss` 源 SHA-256：
  `6dd3e0bee54d57363f8ff942f90eacb929d995fd438df12c385c1a412c357ef5`。
- release `tensoreval` SHA-256：
  `ee3365237a9e183973c522edd7d122679c7cae68d1dbe5cf3207078c6d8fc36f`；构建命令显式使用
  `--no-default-features --features tensoreval-cli,with_dsa_mumps`。
- 两个 artifact 均为 schema-v2 `execution_plan`、Newton、`opendss-native`，各包含原始 DSS、
  `main.rss`、`managed.rss` 和 manifest。manifest SHA-256 分别为
  `018f7e432f23e0c0e93e189b668e2b9d811809b4cf8fc6bd1546fe978ce012b1`、
  `7aaff9bc9c9291f7073512a117017a3c3e8a5d8b1f547aca228f10210059ff36`。

## 执行与搬移

严格检查从未修改官方入口分别识别 52/50 条 DSS 语句和 8 个 Solve epoch。两个 bundle 在
仓库原位置和独立 `/tmp` 父目录均自执行成功：

| 入口 | DataFrame 行数 / 列数 | epoch_index | 原位置与搬移输出 SHA-256 |
|---|---:|---|---|
| Auto1bus | 120 / 11 | 0–7 | `125c8b79eb5ed6b8d9cb49d0f4257c9e06e300122ff7df8ba62c0f8604e49099` |
| Auto3bus | 96 / 11 | 0–7 | `31b11dcc9380056cd61ad70ce400d08a66c3eb20d0b46e646f5586a76e0c9fd6` |

两组原位置/搬移 CSV 均逐字节相同。artifact 文件未包含 `/tmp`、工作树或上游绝对路径；bundle
中的 DSS 文件与固定上游字节相同，manifest 声明哈希全部复算一致。

## OpenDSS 原生数值门禁

原生参考由 OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5 从两份固定 fixture 生成，CSV SHA-256：

- Auto1bus：`7b45f81d230229da03b9d747eb54edfc7c501f8e54fd1b0e3e1e06b600b45b04`；
- Auto3bus：`a9576f25414b729ee52bd452be27003a7a385c9713a2d0ec94edf7c3d9e70fdc`。

`psa/tests/autotrans_multi_epoch_execution.rs` 显式设置 `line_model="exact"`，并锁定：

- 8/8 epoch，每个 epoch 一个 Snapshot 结果；
- epoch 1/3/5 的 Fault.test 电流；
- epoch 0/2/4/6/7 的全部公共母线复电压、三台单相三绕组变压器的绕组复电压/复电流及各变压器复损耗。

Auto1bus 共比较 183 个量，其中 75 个母线复电压的最大/RMS 归一化误差为
`5.5131230e-5` / `1.1310687e-5`，最大绝对误差 `5.124474639 V`；全部量最大归一化误差
`0.002178116360`。Auto3bus 共比较 168 个量，其中 60 个母线复电压的最大/RMS 归一化误差为
`5.5876318e-5` / `1.2805735e-5`，最大绝对误差 `5.193730833 V`；全部量最大归一化误差
`0.002179617589`。两者均低于 `0.005` 门槛。近零绕组电流使用 10 A 归一化下限，即 0.05 A
绝对门槛；加载绕组仍使用 0.5% 门槛。

## 发布范围

本批确实新增两个 bundle 路径，而不是把既有 bundle 的 schema 升级计作新增。catalog 仅把这两个
入口从 `unsupported` 改为 `converted`，最终计数为 184 / 86 / 578。该合同不声称已支持带
Fuse/Recloser/Relay 的全导体故障支路量测、并发多故障或 `AutoAuto.dss` 的接法缩写边界。

## 2026-11-12 AutoAuto 后续发布

`Examples/AutoTrans/AutoAuto.dss` 随后以未修改的官方 CRLF 字节发布为
[`AutoAuto.rssbundle`](Examples/AutoTrans/AutoAuto.rssbundle)。源 SHA-256 为
`d0a21b0dccd34b29a295926ef15c9578c17e01cef792f693bb13e771f5874050`；本次仅把原生
`s/w/d` 接法缩写映射到既有 `series/wye/delta`，错误绕组顺序和不支持拓扑继续失败关闭。

release `tensoreval` 使用 `--no-default-features --features tensoreval-cli,with_dsa_mumps`
构建，SHA-256 为 `020abb0dfe94ef06f1e58ff087ddcd49290231748172419458c835d93569a04e`。
严格检查识别 46 条语句和 8 个 epoch；Newton bundle manifest SHA-256 为
`9a625e4e99b056745ccefb858fe9e131f2a369fbb9cfe5a2ec52540ef251933a`。
原地执行与采集根临时不可见时的搬移执行都产生 96×11 电压表，epoch 0–7 各 12 行，输出
SHA-256 均为 `b620ea10d157d973f40f9ecc0925b2b9b2b6358f129958d950f13fc3aaae5761`；
artifact 不含 `/tmp`、工作树或采集根绝对路径。

OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5 参考锁定 96 条电压、72 条三绕组端口功率和
3 条故障电流。PSA 显式设置 `line_model="exact"`；epoch 0/2/4/6/7 的 60 个复电压最大/RMS
归一化误差为 `5.028390533e-9` / `2.840536255e-9`，最大绝对误差 `0.000963288460 V`；
45 个绕组相复功率最大误差 `0.131 VA`；epoch 1/3/5 故障电流最大归一化误差
`1.026e-9`。参考 CSV SHA-256 分别为
`5705b2f91d29ebd6519585e1093c026e5c979766ccf88e90d56edc4a5071cb5e`、
`b59ec3b7fe47139dfb328f73e28c0305bf1081b5c99469abb4554cbc5332c716`、
`588336120ccdc48324ee62bfd4589bce0a0bdaf391a25ea4da94297ac74f0771`。

发布后 catalog 为 187 `converted`、83 `unsupported`、578 `not_standalone_entry`。历史扫描
JSON 保留当时的失败结果作为诊断来源，不回写成事后成功记录；当前状态以 conversion report、
数值表、bundle manifest 和本节发布证据为准。
