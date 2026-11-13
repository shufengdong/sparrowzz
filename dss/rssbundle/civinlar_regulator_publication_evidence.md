# Civinlar 调压与十步电容 Snapshot 发布证据

## 发布范围

- 固定上游：`dss-extensions/electricdss-tst` 提交
  `3b208397160213cae4a9e2d0a7d1aa3528ce26e1`。
- 官方入口：`Examples/civinlar model/regulator.dss`，SHA-256
  `92e479ae6f4f4843a14431bed262117b54c88de80eae67272f1c7fa9ec894dea`。
- 数据资产 `ldc_2007.dat` SHA-256：
  `9994e93bf7024b05ec636ac03b460b61224b4841cfd72d999bf18cc191341690`。
- 发布物：[`Examples/civinlar model/regulator.rssbundle`](Examples/civinlar%20model/regulator.rssbundle)，
  schema v2、Radial execution plan、一个 Snapshot epoch。

本轮只补齐 OpenDSS 多步 Capacitor 在未显式书写 `states` 时的默认值：`numsteps=10` 会先形成
十个投入状态，再由 CapControl 按原生命令执行。显式 `states` 数量不足、内容非法或不连续的情况
继续使用既有稳定诊断失败关闭。未触碰或放宽任何 GFM 合同。

## 构建、严格检查和数值门禁

发布用 `tensoreval` 按仓库要求关闭默认 features 构建：

```bash
cargo build -p mems --release --no-default-features \
  --features tensoreval-cli,with_dsa_mumps --bin tensoreval
```

最终二进制 SHA-256 为
`29a1060e1a3761c8a11b08aaae52097a14da6d576a3aec5b22ca20c342cc4b7f`。严格检查展开 25 条 DSS
语句，识别 Circuit bootstrap 和一个 Solve epoch，并以 Radial 求解成功。

OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5 直接执行未修改的官方入口。PSA 门禁显式设置
`line_model="exact"`，比较全部 12 个母线相复电压、源端复功率、全网复损耗、调压器终态 tap
和十步电容器终态：

| 量 | 实测最大误差/终态 | 门限 |
|---|---:|---:|
| 归一化复电压 | `4.336207768984044e-7` | `1e-6` |
| 源端复功率 | `1.8861771742005504e-5` | `5e-5` |
| 全网复损耗 | `3.086740648099175e-5` | `5e-5` |
| `Transformer.sub` 二次侧 tap | `0.9925`，精确一致 | `1e-12` |
| `Capacitor.capbank` | `1/10` 步投入，精确一致 | 精确 |

锁定参考由 `psa/tests/bmopf/opendss_inputs/civinlar_regulator/generate_reference.py` 生成；metadata
绑定入口和生成器哈希，预期值不经过 PSA。

## Bundle 身份和可搬移性

- source tree SHA-256：`aed50fd5b1377ea037190cc623b879683c45a3e5f5289a839f0b76d352e420a5`
- `manifest.json`：`f830e2ec0be41e733917ec93beb52adabe49a9a0ac54037c83174b8c4bafd149`
- `main.rss`：`f278010917d17177651c5299a1d3a7b11abd9b748832949b4378608aca74a9f9`
- `managed.rss`：`a36d885a0c77198a0518b3e81955a336e04ff6b875a352b631eb2cce069c64f9`

原位置执行和复制到独立父目录并改名后的执行都输出 12 行、11 列的 `voltage.csv`；两份文件
逐字节一致，SHA-256 均为
`e38ce30b134f9a319c8fc30cd1a472537cfcded1d70ed9eb1c8e8baa04ab97d6`。bundle 中不含采集根、
`/tmp` 或工作树绝对路径。

## 同批 CIM 候选的边界

同批检查的 `Examples/CIM/IEEE13_CDPSM.dss` 已不再被两相 Storage 相数诊断拦截。导入器现在只对
GFL/PQ 两相 Wye Storage 开放，并以单元测试锁定两相 Delta 仍失败关闭；普通馈线上的两相 Wye
Storage 默认调度在 Radial 和 Newton 都通过。

官方 CIM 完整入口仍未发布：Newton 在 epoch 0 返回
`PSA.DSA.NUMERIC.NON_CONVERGED`。最小化探针分别定位到原文件的空载三次绕组、分相服务网络 PV
交互和该网络上的 Storage dispatch 控制闭环；它们不是本轮两相相数放行本身，继续处理会超过
“小修改可发布”的范围。因此 catalog 将该入口更新为当前真实的
`TENSOREVAL.DSS.TRANSLATE.SOLVE_FAILED`，没有修改官方源，也没有生成虚假 bundle。
