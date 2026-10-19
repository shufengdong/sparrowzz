# tensoreval DSS scripts

本目录保存经过回归测试验证的 expr_df DSS 脚本，统一使用 `.rss` 扩展名表示
RustScript 文件。
请在 eig-rs 工作区根目录运行命令，因为脚本中的 DSS 路径相对于进程工作目录解析。

从现有 OpenDSS 模型和 Python 仿真脚本迁移时，请参阅
[从 OpenDSS Python 脚本迁移到 RSS](docs/opendss_to_rss.md)。
完整语法、函数、schema、兼容范围和智能体编写指南见
[`docs/README.md`](docs/README.md)。

## 脚本

| 脚本 | 计算 |
| --- | --- |
| `bmopf_4bus_power_flow.rss` | BMOPF JSON 4Bus-DY-Bal 辐射状配电网潮流 |
| `bmopf_ieee13_power_flow.rss` | BMOPF JSON IEEE13 辐射状配电网潮流 |
| `bmopf_ieee34_power_flow.rss` | BMOPF JSON IEEE34 辐射状配电网潮流 |
| `ieee13_power_flow.rss` | IEEE13 辐射状配电网潮流 |
| `ieee34_power_flow.rss` | IEEE34 辐射状配电网潮流 |
| `ieee123_power_flow.rss` | IEEE123 辐射状配电网潮流 |
| `ieee13_harmonic.rss` | IEEE13 1/3/5/7 次谐波 |
| `ieee34_harmonic.rss` | IEEE34 1/3/5/7 次谐波 |
| `IEEE13Nodeckt_harmonic_run.rss` | 从完整 IEEE13 谐波 DSS 入口生成的链接脚本 |
| `ieee34Mod1_harmonic_run.rss` | 从完整 IEEE34 谐波 DSS 入口生成的链接脚本 |
| `ieee13_short_circuit.rss` | IEEE13 母线 652 单相接地短路 |
| `ieee34_short_circuit.rss` | IEEE34 母线 810 单相接地短路 |
| `4bus_daily_qsts.rss` | 4Bus-DY-Bal 三时点 Daily QSTS |
| `4bus_dy_run.rss` | 从完整 4Bus-DY-Bal Daily DSS 入口生成的链接脚本 |
| `ieee13_daily_qsts.rss` | IEEE13 全负荷三时点 Daily QSTS |
| `case14_4bus_master_slave_pf.rss` | IEEE case14 主网与 4Bus-DY-Bal 配网显式展开三轮的单 PCC 主从分裂案例 |

主从分裂案例的每轮主网计算都是独立 Prog，通过
[`runpf_bundle.txt`](runpf_bundle.txt) 递归装入相同的 RSPower 依赖，避免在案例中重复维护
函数清单。共享清单仍需写在每个 `{ ... }` 内，使对应 VM 获得完整函数定义。

`models/*_static`、`models/*_harmonic` 和 `models/4bus_daily_qsts` 保存测试所用的
OpenDSS 模型。基础模型保留网络定义，`*_run.dss` 是可由 OpenDSS 直接执行的完整入口，
包含计算模式、必要参数和 `Solve`。模型同时补齐 PSA 严格导入所需的线路额定值。

## OpenDSS 算例与转换结果

| OpenDSS 执行入口 | RSS 结果 | 说明 |
| --- | --- | --- |
| `models/ieee13_static/IEEE13Nodeckt_run.dss` | `ieee13_power_flow.rss` | IEEE13 Snapshot 潮流；已有脚本与生成结果等价，不重复保存 `_run.rss` |
| `models/ieee34_static/ieee34Mod1_run.dss` | `ieee34_power_flow.rss` | IEEE34 Snapshot 潮流；已有脚本与生成结果等价，不重复保存 `_run.rss` |
| `models/ieee123_static/IEEE123Master_run.dss` | `ieee123_power_flow.rss` | IEEE123 Snapshot 潮流；已有脚本与生成结果等价，不重复保存 `_run.rss` |
| `models/4bus_daily_qsts/4bus_dy_run.dss` | `4bus_dy_run.rss` | 三个一小时时步的 Daily 执行计划；RSS 调用 DSS 执行计划门面 |
| `models/ieee13_harmonic/IEEE13Nodeckt_harmonic_run.dss` | `IEEE13Nodeckt_harmonic_run.rss` | 完整内联 IEEE13 模型，先求解基频，再计算 1/3/5/7 次谐波 |
| `models/ieee34_harmonic/ieee34Mod1_harmonic_run.dss` | `ieee34Mod1_harmonic_run.rss` | 完整内联 IEEE34 模型，先求解基频，再计算 1/3/5/7 次谐波 |

三个生成的链接 RSS 保留 DSS 入口路径和源树摘要，运行时以表中 `.dss` 文件为权威输入。
`4bus_daily_qsts.rss`、`ieee13_harmonic.rss` 和 `ieee34_harmonic.rss` 则显式组装
DataFrame，便于查看和修改每张输入表。

回归测试会重新转换静态入口，并将静态潮流、Daily QSTS 和谐波结果逐节点、逐相、
逐时点或逐谐波与 OpenDSS 复电压基准比较，最大允许相对误差为 `1e-3`。

## 运行

完整 OpenDSS 执行文件可先严格转换为 RSS；详细边界见
[DSS 到 RSS 严格转换](docs/dss_to_rss.md)：

```bash
tensoreval --dss-to-rss \
  --output sparrowzz/dss/4bus_dy_run.rss \
  sparrowzz/dss/models/4bus_daily_qsts/4bus_dy_run.dss
```

潮流或短路脚本可直接运行：

```bash
tensoreval --df sparrowzz/dss/ieee13_power_flow.rss
```

BMOPF JSON 潮流脚本同样从工作区根目录运行：

```bash
tensoreval --df sparrowzz/dss/bmopf_ieee13_power_flow.rss
```

QSTS 脚本也可直接运行：

```bash
tensoreval --df sparrowzz/dss/4bus_daily_qsts.rss
```

该脚本用 0、3600 和 7200 秒的负荷曲线样本求解三个一小时时步；结果时间是每个时步
的结束时刻，即 3600、7200 和 10800 秒。

联合潮流案例需要主网数值求解器和配网后端：

```bash
tensoreval --df sparrowzz/dss/case14_4bus_master_slave_pf.rss
```

谐波计算应使用 release 构建：

```bash
tensoreval --df sparrowzz/dss/ieee13_harmonic.rss
```

通过 `cargo make tensoreval-native` 构建后，也可以直接运行：

```bash
target/release/tensoreval --df sparrowzz/dss/ieee13_power_flow.rss
```
