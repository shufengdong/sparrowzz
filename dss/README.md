# tensoreval DSS scripts

本目录保存由 `mems/tests/dff/dff_script.rs` 验证的 expr_df DSS 脚本，统一使用
`.rss` 扩展名表示 RustScript 文件。
请在 eig-rs 工作区根目录运行命令，因为脚本中的 DSS 路径相对于进程工作目录解析。

从现有 OpenDSS 模型和 Python 仿真脚本迁移时，请参阅
[从 OpenDSS Python 脚本迁移到 RSS](docs/opendss_to_rss.md)。
完整语法、函数、schema、兼容范围和智能体编写指南见
[`docs/README.md`](docs/README.md)。

## 脚本

| 脚本 | 计算 |
| --- | --- |
| `ieee13_power_flow.rss` | IEEE13 辐射状配电网潮流 |
| `ieee34_power_flow.rss` | IEEE34 辐射状配电网潮流 |
| `ieee123_power_flow.rss` | IEEE123 辐射状配电网潮流 |
| `ieee13_harmonic.rss` | IEEE13 1/3/5/7 次谐波 |
| `ieee34_harmonic.rss` | IEEE34 1/3/5/7 次谐波 |
| `ieee13_short_circuit.rss` | IEEE13 母线 652 单相接地短路 |
| `ieee34_short_circuit.rss` | IEEE34 母线 810 单相接地短路 |
| `4bus_daily_qsts.rss` | 4Bus-DY-Bal 三时点 Daily QSTS |
| `ieee13_daily_qsts.rss` | IEEE13 全负荷三时点 Daily QSTS |
| `case14_4bus_master_slave_pf.rss` | IEEE case14 主网与 4Bus-DY-Bal 配网显式展开三轮的单 PCC 主从分裂案例 |

主从分裂案例的每轮主网计算都是独立 Prog，通过
`runpf_bundle.txt` 递归装入相同的 RSPower 依赖，避免在案例中重复维护
函数清单。

`models/*_static`、`models/*_harmonic` 和 `models/4bus_daily_qsts` 是测试所用
OpenDSS 模型的静态投影。它们移除了求解命令和调压控制，并补齐 PSA 严格导入所需的
线路额定值。谐波脚本在 expr_df 中建立电源与负荷频谱，谐波模型保留相应的高短路
容量电源参数。QSTS 脚本在 expr_df 中建立三点时间轴、负荷曲线和曲线绑定。

## 运行

潮流或短路脚本可直接运行：

```bash
cargo run -p mems \
  --features tensoreval-cli,with_dsa \
  --bin tensoreval -- \
  --df sparrowzz/dss/ieee13_power_flow.rss
```

QSTS 脚本也可直接运行：

```bash
cargo run -p mems \
  --features tensoreval-cli,with_dsa \
  --bin tensoreval -- \
  --df sparrowzz/dss/4bus_daily_qsts.rss
```

该脚本用 0、3600 和 7200 秒的负荷曲线样本求解三个一小时时步；结果时间是每个时步
的结束时刻，即 3600、7200 和 10800 秒。

联合潮流案例需要主网数值求解器和配网后端：

```bash
cargo run -p mems \
  --features tensoreval-cli,with_dsa_mumps,full_mumps-static \
  --bin tensoreval -- \
  --df sparrowzz/dss/case14_4bus_master_slave_pf.rss
```

谐波计算应使用 release 构建：

```bash
cargo run --release -p mems \
  --features tensoreval-cli,with_dsa \
  --bin tensoreval -- \
  --df sparrowzz/dss/ieee13_harmonic.rss
```

通过 `cargo make tensoreval-native` 构建后，也可以直接运行：

```bash
target/release/tensoreval --df sparrowzz/dss/ieee13_power_flow.rss
```
