# EPRI K1 无 PV 发布证据

## 身份与范围

- 官方入口：`EPRITestCircuits/epri_dpv/K1/Master_NoPV.dss`
- 固定上游提交：`3b208397160213cae4a9e2d0a7d1aa3528ce26e1`
- 入口 SHA-256：`c48080699ae641b45dc009e540072e77f97bd7623aedcc617a7c0249186cebb8`
- 来源 closure SHA-256：`afec90c9bc34b08d22e2842dd7a09e526b9d896d901b7fe375a62380836bdbc5`
- 发布位置：`EPRITestCircuits/epri_dpv/K1/Master_NoPV.rssbundle/`

合同保留官方单次 Snapshot ExecutionPlan、8 个 Monitor 和 1 个 EnergyMeter。线路模型为
`exact`。可靠性报表、年度轨迹和当前未实现的 T2 电压调节不在等价声明内。

## 构建与 portable bundle

发布可执行文件使用以下命令构建：

```text
cargo build -p mems --release --no-default-features \
  --features tensoreval-cli,with_dsa_mumps --bin tensoreval
```

- `tensoreval` SHA-256：`65dbf8e28c929c8a7fb5ce73597eff92477e3913ca6db8ac721686a8d18b3e9c`
- schema：v2
- 严格检查：2,021 条 DSS 语句、1 个 solve epoch、`circuit_bootstrap=true`
- manifest 声明文件数：16
- `manifest.json` SHA-256：`6b3082da94877bb97566944eb680cbc69a1de4c61ddf3c2fac87dd9375528dee`

原目录与复制到独立临时父目录并改名后的 bundle 都执行成功，各返回 1,750×11 的 `voltage`
结果。两份 CSV 均为 186,353 字节，SHA-256 均为
`165443c6a5360d056c903cf4c56bec18b43b54029086ac938e4f1f96ec696c45`。

## OpenDSS 数值门禁

OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5 对固定官方来源收敛：9 次潮流迭代、3 次控制迭代，
1,282 个母线、1,751 个节点。PSA 的 1,750 个键全部存在于原生结果；唯一原生专有键
`K_LSB.4` 是 T2 星形侧显式中性点，经 `Reactor.T2_neutral` 接地，在 PSA 全导体方程中作为
接地回路消元。

| 指标 | 结果 |
|---|---:|
| 公共母线相键 | 1,750 |
| 最大缩放复电压误差 | `0.00825964194813738` |
| RMS 缩放复电压误差 | `0.008081250648405888` |
| 最大幅值缩放误差 | `0.004808093037910661` |
| 最大绝对误差 | `64.28735929963639 V` |
| 最差键 | `K_LSB.1` |
| 发布门限 | `0.01` |

原生检查点同时锁定总功率 `(-12749.43432982602 kW, -2264.227690110947 kvar)`、总损耗
`(199362.4838737801 W, 1188252.1844684011 var)` 和 T2 tap `0.95625`。当前 PSA 诊断明确报告
`RegControl.T2` 电压调节未实现并保持书写 tap，因此本次发布只声明上述电压门限通过，不声明
tap、总功率或损耗与 OpenDSS 等价；该边界没有被静默忽略。

## 通用实现范围

本次没有 K1 专用分支。改动包括：恢复全导体提升电容器的原始 `kv`/`kvar` 与默认值；保留禁用
设备审计行但不为 disabled-only 母线分配数值未知量；从多端口变压器绕组结果生成 Monitor；
显式接地端 `0` 不作为电容器相导纳槽。聚焦回归覆盖默认额定值、共享端点、禁用端点、变压器
Monitor 和接地电容器。
