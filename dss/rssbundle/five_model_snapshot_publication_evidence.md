# 五个模型入口 Snapshot 批量发布证据

## 固定输入与合同

- 上游：`dss-extensions/electricdss-tst` 提交
  `3b208397160213cae4a9e2d0a7d1aa3528ce26e1` 的 `Version8/Distrib`。
- 五个入口都以显式 Snapshot、Radial 和 `line_model=exact` 发布。
- 前四个 bundle 使用发布二进制
  `876ce8e214cec6ad56c541a658406648b3b5eca63cfb618e600459e0596dcc34`；
  GFM balanced 在补齐 converter 的显式 static-projection 路由后使用
  `d3370dd39f3ecefeabbbea8682f46377bbe35d5779c24d35d924901e169c4fe7`。
- 旧二进制对 GFM balanced 的严格检查只报
  `loadshape.pvshape` 和 `loadshape.LoadShape1` 两个
  `PSA.DSS.OBJECT.UNSUPPORTED`。新边界只允许显式 static projection 丢弃无消费者的
  LoadShape 计算语义；普通 `import_dss` 和任何被 `Daily`、`Yearly`、`Duty`、
  `CVRCurve`、`ControlSignal` 或全局默认曲线引用的声明仍然 fail closed。

## 入口身份与 bundle 门禁

| 官方入口 | 源 SHA-256 | 严格检查 | manifest 文件数 | manifest SHA-256 | 搬移结果 |
|---|---|---:|---:|---|---:|
| `Examples/InverterModels/PVSystem/CurrentkvarLimite/PV_currentkvarLimit_kvar.dss` | `9d6ad8c4671f4662459a0d6f5945ae884448b05fe3d14079acce58bb78318099` | 12 条语句，1 epoch | 3 | `6cbf15d0bb6e3ad1f9001a4406b68b4d88b514151d58bc1bbcf6c0f144294146` | 3 行电压 |
| `Examples/InverterModels/PVSystem/CurrentkvarLimite/PV_currentkvarLimit_kvarNEG.dss` | `7ee44094d1a4fcbe4f01d0c076f9b69e2a07059572cbfd98e17826f95bded7ff` | 12 条语句，1 epoch | 3 | `8a28fecbb18be23fa27736b7e438120ce9b4fc707496fac8be8f2db8d4bef5b2` | 3 行电压 |
| `Examples/Microgrid/GridFormingInverter/GFM_IEEE8500/Master-unbal.dss` | `303089cddf567f758594f54730963a416e5caaa6004bfb04e7bbe73bd7c2f364` | 7,590 条语句，1 epoch | 14 | `9673701081b345ca420667da9e86a294303ef1a2df74a5ada46c3d2a23da3c14` | 11,038 行电压 |
| `IEEETestCases/8500-Node/Master.dss` | `9bd0e17f33e9ec7e0baa46693abeec069b148cbee8477301d77450f95d601ad8` | 6,413 条语句，1 epoch | 14 | `604ba246bbb132e1bd7e9272e3c7b8e0b13cd586801f8a1655066b8853c1ee8b` | 11,038 行电压 |
| `Examples/Microgrid/GridFormingInverter/GFM_IEEE8500/Master.dss` | `901c7cdf24abf5a05e00643cfec2a6d60b669468c7e35c14f43bb268df223425` | 6,416 条语句，1 epoch | 16 | `cee45815d13e77c585bd165f5393cff6982a2b6bd53fd96ac23be62d5a90b73c` | 11,038 行电压 |

每个 bundle 的 schema-v2 manifest、`radial` solver、`ds_radial_pf` source function 和全部
`.files[]` SHA-256 都已重新校验。五个 bundle 均复制到独立 `/tmp` 父目录后直接执行成功；closure
中没有固定上游临时目录或工作树绝对路径。GFM balanced 的 16 个 manifest 文件明确包含
`LoadShapes.dss` 与 `LoadShape1.CSV`，因此保留原始资产闭包而不把两条曲线应用到 Snapshot 网络。

## 数值等价证据

两个 CurrentkvarLimite 文件分别从未修改官方源生成 OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5
参考，并由两个独立 PSA consumer 检查 3 个复电压和 3 个 PV 相 P/Q。电压最大归一化误差门限为
`1e-9`，相功率绝对误差门限为 `1e-6 W/var`。`kvarNEG` 只体现在文件名和 `casename`；官方内容仍是
`kvar=500`，两个入口的七张原生输出 CSV 逐字节相同。本发布不把它描述为负 kvar 案例。

GFM unbalanced Master 与既有原生对标入口只有尾部空行差异。balanced 官方 Master 则直接使用
既有 IEEE8500 balanced reference。`ieee8500_original_decks_lower_to_single_step_static_control_execution_profiles`
显式传入 `line_model="exact"`，对 balanced/unbalanced 各比较 8,531 个原生节点、15,882 个端口
电流、15,882 个端口功率、元件功率/损耗、源端汇总和控制终态；本批两次独立重跑均通过。

GFM balanced 与官方 balanced Master 的唯一源差异是 `Redirect LoadShapes.dss`。实际 Master
重定向闭包中没有曲线消费者；两个搬移结果的 11,038 个 `bus + phase` 键完全相同，复电压最大/RMS
归一化误差分别为 `1.2399434592283917e-16` / `1.1045306762394304e-17`，最大绝对误差为
`9.094947017729282e-13 V`（一 ULP 量级）。由此同时闭合了源差异、投影网络维度与最终数值结果
三个门禁。

## 目录结论

`conversion_report.csv` 只把上述五行从 `not_standalone_entry` 改为 `converted`：计数由
`156/108/584` 更新为 `161/108/579`，独立入口分母由 264 增至 269，尚未发布的独立入口仍为
108。`numerical_comparison.csv` 为五个入口分别记录对应 consumer、门限与搬移结果。
