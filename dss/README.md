# tensoreval DSS scripts

本目录保存由 `mems/tests/dff/dff_script.rs` 验证的 expr_df DSS 脚本，统一使用
`.rss` 扩展名表示 RustScript 文件。
请在 eig-rs 工作区根目录运行命令，因为脚本中的 DSS 路径相对于进程工作目录解析。

## 脚本

| 脚本 | 计算 |
| --- | --- |
| `ieee13_power_flow.rss` | IEEE13 径向潮流 |
| `ieee34_power_flow.rss` | IEEE34 径向潮流 |
| `ieee123_power_flow.rss` | IEEE123 径向潮流 |
| `ieee13_harmonic.rss` | IEEE13 1/3/5/7 次谐波 |
| `ieee34_harmonic.rss` | IEEE34 1/3/5/7 次谐波 |
| `ieee13_short_circuit.rss` | IEEE13 母线 652 单相接地短路 |
| `ieee34_short_circuit.rss` | IEEE34 母线 810 单相接地短路 |

`models/*_static` 和 `models/*_harmonic` 是测试所用 OpenDSS 模型的静态投影。
它们移除了求解命令和调压控制，并补齐 PSA 严格导入所需的线路额定值。谐波脚本在
expr_df 中建立电源与负荷频谱，谐波模型保留相应的高短路容量电源参数。

## 运行

潮流或短路脚本可直接运行：

```bash
cargo run -p mems \
  --features tensoreval-cli,with_dsa \
  --bin tensoreval -- \
  --df sparrowzz/dss/ieee13_power_flow.rss
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
