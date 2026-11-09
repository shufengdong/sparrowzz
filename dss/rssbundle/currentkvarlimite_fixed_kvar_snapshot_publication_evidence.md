# CurrentkvarLimite 固定 kvar 两入口 Snapshot 发布证据

## 身份与范围

- 固定上游提交：`3b208397160213cae4a9e2d0a7d1aa3528ce26e1`
- 正常文件名入口：
  `Examples/InverterModels/PVSystem/CurrentkvarLimite/PV_currentkvarLimit_kvar.dss`，
  SHA-256 `9d6ad8c4671f4662459a0d6f5945ae884448b05fe3d14079acce58bb78318099`
- `NEG` 文件名入口：
  `Examples/InverterModels/PVSystem/CurrentkvarLimite/PV_currentkvarLimit_kvarNEG.dss`，
  SHA-256 `7ee44094d1a4fcbe4f01d0c076f9b69e2a07059572cbfd98e17826f95bded7ff`
- 发布边界：显式 Snapshot、Radial、`line_model=exact`。

`NEG` 只存在于文件名和 `casename`；该官方文件的电气内容仍写入 `kvar=500`，与正常文件名入口
完全相同。因此本发布不把它解释成负 kvar 案例，也不复用
`NewFeatures/varCapability/PV_currentkvarLimit_kvarNEG.dss` 的负 kvar 参考。

## 独立原生数值证据

使用 OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5 分别从两个未修改官方文件生成参考。两次求解均
收敛，2 次迭代、1 次控制迭代；各有 1 个母线、3 个节点和 2 个电气元件。两个入口的全部七个
输出 CSV 逐字节相同：3 个复电压、10 个端口电流、10 个端口功率、2 个元件损耗、2 个对象、
0 个控制状态和 11 个汇总值。

原生三相 PV 输出均为每相约 `333333.333333 W + j66666.666667 var` 向网络注入，三相合计为
`1 MW + j200 kvar`，与源中的正向 `kvar=500` 受 `kvarMax=200` 限幅一致。

- 正常文件名 reference checksum manifest：
  `99456a6a846e83faff820ae791a069d2a20c15f97a6f0c6a865867d1c7a447e0`
- `NEG` 文件名 reference checksum manifest：
  `5e941a0cc0668950f42649f6d02eb02e6e63215bea064e20356eaf7c79c49f2d`
- Consumers：
  `pv_currentkvarlimite_fixed_kvar_snapshot_matches_exact_source_reference` 与
  `pv_currentkvarlimite_neg_filename_matches_its_positive_source_reference`
- 门限：全部 3 个节点复电压最大归一化误差 `1e-9`；全部 3 个 PV 相 P/Q 最大绝对误差
  `1e-6 W/var`。两个 consumer 均显式断言 `line_model=exact` 并通过。

## release 与 portable bundle 门禁

本阶段使用既有 release `tensoreval`，SHA-256：
`876ce8e214cec6ad56c541a658406648b3b5eca63cfb618e600459e0596dcc34`。

两个官方入口均严格验证为 12 条 DSS 语句、Snapshot、1 个 solve epoch 和
`circuit_bootstrap=true`。两个 schema-v2 manifest 均指定 `radial`、`ds_radial_pf` 和
`line_model=exact`，且各自三个文件的 SHA-256 自洽：

- 正常文件名 manifest：
  `6cbf15d0bb6e3ad1f9001a4406b68b4d88b514151d58bc1bbcf6c0f144294146`
- `NEG` 文件名 manifest：
  `8a28fecbb18be23fa27736b7e438120ce9b4fc707496fac8be8f2db8d4bef5b2`

两个 bundle 分别复制到独立临时父目录后自执行成功，`voltage.csv` 均为表头加 3 行结果。
closure 对固定上游、工作树和 `/tmp` 绝对路径的扫描无匹配。

发布位置：

- `Examples/InverterModels/PVSystem/CurrentkvarLimite/PV_currentkvarLimit_kvar.rssbundle/`
- `Examples/InverterModels/PVSystem/CurrentkvarLimite/PV_currentkvarLimit_kvarNEG.rssbundle/`

catalog 状态在五案例批次全部通过后统一更新。
