# ISource 微电网 Snapshot + Daily 发布证据

日期：2026-11-12

## 固定来源与范围

- 官方入口：`Examples/Microgrid/ISource/Master.DSS`。
- 上游：`electricdss-tst@3b208397160213cae4a9e2d0a7d1aa3528ce26e1`，根目录
  `Version8/Distrib`。
- 入口 SHA-256：`5190965373a8b96877b327c27ff072da1f824dbd6c665225df4b7a3659d4c45d`。
- 发布位置：`Examples/Microgrid/ISource/Master.rssbundle/`。

入口保持原始执行顺序：一次 Snapshot 初始化，打开 `Line.671692` 的端口 2，声明
`Monitor.MVoltage`，切换到 Daily 后执行 24 步。三个单相 ISource 使用同一条
`pvloadshape`。本次不修改官方 DSS，也不声明 GFM、AmpLimit、Storage-target InvControl、
动态研究或 RegControl tap 等价。

## 构建、严格检查和 artifact

发布可执行文件使用以下显式 feature 边界构建：

```text
cargo build -p mems --release --no-default-features \
  --features tensoreval-cli,with_dsa_mumps --bin tensoreval
```

- release `tensoreval` SHA-256：
  `4aff54643307883db462fd73a975ec5f5b7414c057d966523bd491cc4a3f5c2d`。
- 严格检查识别 117 条 DSS 语句、2 个 solve epoch、`circuit_bootstrap=true`；运行时完成
  `[1, 24]` 共 25 次收敛 Newton 求解。
- bundle schema：v2；study：`execution_plan`；兼容性：`opendss-native`；求解器：
  `newton-raphson`。
- manifest 声明 17 个文件，source tree SHA-256：
  `a7dea85b9d92defb18d24296ddf413b0554fe2a0a031632d014d248735cfc3c5`。
- `manifest.json` SHA-256：
  `e8912f0a48c4586ffc74a29cfdad983772defd2b7d43030dee12a0b72a3befe9`。

原位置与复制到独立临时父目录并改名后的 bundle 均从各自根目录自执行成功，各输出
1,025×11 的 `voltage.csv`。两份文件均为 99,161 字节、1,026 行（含表头），SHA-256 均为
`4dea5dc7ecaf7e201a66388fce747189a6f9c8d40b7a2d25c9546b0f66076edf`。artifact 全量检索未发现
`/tmp`、工作树或上游采集根绝对路径。

## OpenDSS 原生数值门禁

锁定参考由 OpenDSSDirect.py 0.9.4 / DSS C-API 0.14.5 直接执行未修改官方入口生成；PSA 不参与
期望值生成。测试显式设置 `line_model="exact"`，并验证两个 epoch、端口 2 开路、三台 ISource、
最终时刻 86,400 秒以及 72 行有限 Monitor 投影。

| 指标 | 结果 |
|---|---:|
| 最终公共母线相键 | 41 |
| 带电母线相键 | 35 |
| 带电键最大归一化复电压误差 | `7.851799552545358e-5` |
| 源自由孤岛相位 | 6 |
| OpenDSS 孤岛最大数值残差 | `6.529222945397018 V` |
| ISource 终端功率键 | 3 |
| OpenDSS ISource 最大数值残差 | `3.878383456657604 VA` |

打开 `Line.671692` 的端口 2 后，`692/675` 成为没有电压源的孤岛。PSA 明确将其六个相电压和
三个 ISource 终端功率置零；OpenDSS 的稳定化导纳留下数伏、数 VA 的非物理残差。因此门禁对
35 个带电键使用相对复数误差，对全部孤岛键同时要求 PSA 为零并限制 OpenDSS 绝对残差；没有
把残差相对于自身归一化，也没有从比较集合删除这些键。

原生参考 SHA-256：

- `final_bus_phase.csv`：
  `5b7f720224ba0f3ad61c528b2624f36c413911ec6344f0c11a29236a63801a63`；
- `final_isource_power.csv`：
  `74858ea424a958a3a956ff431ca6cd15bf33271193657468ec4634c1cf13a26e`。

## 有界实现合同

- continuation epoch 的 `Open`/`Close` 是下一次 `Solve` 前的网络变更，不提前封闭回放窗口；
  真正的 `Solve` 后继续变更仍失败关闭。
- `Transformer.RdcOhms` 只有在每个写入绕组同时具备显式 AC `%R`（或完整等价 AC 电阻）且
  DC 值完整、有限、非负时才作为 provenance 保留；正常频率潮流仍只使用 AC 电阻。
- 负 ISource LoadShape 倍数通过幅值绝对值和相角加 180° 表达，正倍数恢复书写相角，避免负
  `i_magnitude` 进入 typed runtime。
- RegControl 仍保持书写 tap 并发出稳定 Warning；本发布不把调压控制声明为 OpenDSS 等价。

目录只把该入口一行从 `unsupported` 改为 `converted`。发布后为 189 `converted`、81
`unsupported`、578 `not_standalone_entry`。
