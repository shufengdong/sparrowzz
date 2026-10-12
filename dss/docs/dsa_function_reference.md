# DSA 函数参考

本参考对应 `mems/src/ml` 中当前公开的 RSDss DSA 分派。所有函数都
需要 `with_dsa`；Newton 潮流、Newton QSTS 和状态估计还需要 MUMPS 或 Intel MKL 稀疏
后端。

## 1. 完整网络参数顺序

所有“完整网络形式”固定使用以下 20 个位置：

| 位置 | 名称 | 空表是否允许 |
|---:|---|---|
| 1 | `bus` | 否 |
| 2 | `line` | 允许无线路的小模型 |
| 3 | `linecode` | 与线路引用一致 |
| 4 | `load` | 是 |
| 5 | `transformer` | 是 |
| 6 | `voltage_source` | 取决于分析 |
| 7 | `motor` | 是 |
| 8 | `transformer_winding` | 是 |
| 9 | `transformer_impedance` | 是 |
| 10 | `current_source` | 是 |
| 11 | `generator` | 是 |
| 12 | `ibr` | 是 |
| 13 | `switch` | 是 |
| 14 | `shunt` | 是 |
| 15 | `capacitor` | 是 |
| 16 | `reactor` | 是 |
| 17 | `spectrum` | 是 |
| 18 | `control_profile` | 是 |
| 19 | `xy_curve` | 是 |
| 20 | `storage` | 是 |

位置 1–6 始终按必需表解析；位置 7–20 的零行表转为 `None`。非空表会进入对应 PSA
解析器，不能用任意非空表充当占位符。

## 2. 导入函数

### `import_dss` / `import_dss_url`

```text
[20 network frames, para] = import_dss("model.dss");
[20 network frames, para] = import_dss_url("https://.../model.dss");
```

固定一个参数并返回 21 张表，顺序为上节 20 张网络表加 `para`。文件/URL 后缀必须是
`.dss`。使用 `DssImportProfile::StaticPowerFlow`；本地 DSS source 可以解析相对 include
文件树，URL 形式只下载单个 source。当前导入结果的 `storage` 固定为空表，`para` 至少
包含 `base_frequency_hz`。

### `import_bmopf` / `import_bmopf_url`

调用与返回顺序相同，后缀必须是 `.json`。

## 3. 静态潮流

### 推荐完整形式

```text
[results...] = ds_radial_pf(network_1, ..., network_20, para);
[results...] = ds_newton_raphson_pf(network_1, ..., network_20, para);
```

固定 21 个参数。默认结果为 `bus_voltage`。`para.result` 支持：

```text
bus_voltage, bus_phase, load_state, branch_flow, source_flow,
transformer_winding_flow, conductor_flow, regulator_state,
capacitor_state, reactor_state, ibr_state, gfm_state,
violations, summary
```

兼容短形式接受 6–11 个参数：

```text
bus, line, linecode, load, transformer, voltage_source,
[generator, shunt, capacitor, storage, para]
```

短形式无法表达完整设备集合，新脚本应使用完整形式。

## 4. QSTS

```text
[results...] = ds_radial_qsts(
    network_1, ..., network_20,
    solution, load_shape, shape_assignment, storage_controller,
    switch_control, tcc_curve, fuse, recloser, relay,
    monitor, energy_meter, fault, para
);
```

`ds_newton_raphson_qsts` 使用相同的 33 个参数。位置 21–33 固定为：

| 位置 | 表 |
|---:|---|
| 21 | `solution` |
| 22 | `load_shape` |
| 23 | `shape_assignment` |
| 24 | `storage_controller` |
| 25 | `switch_control` |
| 26 | `tcc_curve` |
| 27 | `fuse` |
| 28 | `recloser` |
| 29 | `relay` |
| 30 | `monitor` |
| 31 | `energy_meter` |
| 32 | `fault` |
| 33 | `para` |

除 `solution` 和 `para` 外，零行表表示未配置该子系统。`fault` 非空时自动选择带故障和
保护量测的 QSTS 入口。默认结果为 `bus_voltage`。支持结果：

```text
bus_voltage, bus_phase, load_state, branch_flow, source_flow,
transformer_winding_flow, conductor_flow, regulator_state,
capacitor_state, reactor_state, ibr_state, gfm_state,
violations, summary, monitor, monitor_save, energy_meter,
meter_loss_breakdown, reliability, meter_zone, meter_save,
load_duration
```

逐时步结果自动追加 `time`、`load_multiplier` 和 `generation_multiplier` 三列。

## 5. 谐波

### 完整形式

```text
[results...] = ds_harmonic_study(
    network_1, ..., network_20,
    monitor, para
);
```

固定 22 个参数。默认结果为 `bus_voltage`。支持结果：

```text
bus_voltage, branch_flow, conductor_flow,
bus_sequence, branch_sequence, monitor, monitor_save
```

兼容形式：

- 8 参数：六张基础网络表、`spectrum`、`para`；
- 14 参数：六张基础网络表、`generator, ibr, shunt, capacitor, storage, spectrum, monitor, para`。

`para` 必须满足谐波 spec，常用列为 `h_max`、`line_model` 和 `result`。

## 6. 故障计算

### 完整形式

```text
[results...] = ds_fault_study(
    network_1, ..., network_20,
    fault, para
);
```

固定 22 个参数。默认结果为 `fault`；可请求 `fault,branch_flow`。

兼容形式：

- 8 参数：六张基础网络表、`fault`、`para`；
- 13 参数：六张基础网络表、`generator, ibr, shunt, capacitor, storage, fault, para`。

## 7. 状态估计

```text
result = ds_wls_se(network_1, ..., network_20, measurement, para);
result = ds_mnmr_se(network_1, ..., network_20, measurement, para);
result = ds_mce_se(network_1, ..., network_20, measurement, para);
```

完整形式固定 22 个参数，并返回单张结果表。兼容形式接受：

- 11 参数：六张基础网络表、`generator, shunt, capacitor, storage, measurement`；
- 12 参数：上述参数再加 `para`。

## 8. Monte Carlo

```text
[results...] = ds_monte_carlo(
    network_1, ..., network_20,
    spec, multipliers, para
);
```

固定 23 个参数，不接受非空独立 `storage` 表。`spec` 必须恰好一行：

- `mode`：`m1`、`global`、`m1_replay` 或 `global_replay`，默认 `m1`；
- `solver`：`radial`、`newton` 或 `newton_raphson`，默认 `radial`；
- 随机模式可配置 `n_scenarios, seed, mean, std, v_min, v_max`；
- replay 模式从 `multipliers` 读取确定性样本；随机模式要求 `multipliers` 为空。

默认结果为 `voltage_statistics`。支持：

```text
voltage_statistics, scenario_voltage, scenario_load_power, summary
```

## 9. 恢复重构

```text
[results...] = ds_restore(
    network_1, ..., network_20,
    fault, para
);
```

完整形式固定 22 个参数，要求 `switch` 非空，并拒绝当前恢复算法不消费的其他非空可选
设备表。兼容 9 参数形式为：

```text
bus, line, linecode, load, transformer, voltage_source,
switch, fault, para
```

默认结果为 `switches`。支持 `switches, restored_loads, bus_voltage, objective`。

## 10. 保护配合审计

```text
[results...] = ds_protection_coordination(
    tcc_curve, fuse, recloser, relay,
    fault_case, current_sample, pair, observed_action, para
);
```

固定 9 个参数。该入口消费给定的故障电流/动作证据，不执行网络故障求解。默认结果为
`operations`。支持：

```text
operations, selectivity, minimum_margins,
misoperations, uncovered, action_diagnostics
```

## 11. 故障定位

```text
result = ds_fault_location(
    network_1, ..., network_20,
    measurement, hypothesis_switch, hypothesis_line, para
);
```

固定 24 个参数，返回单张结果表。当前不接受非空独立 `storage` 表。候选表可为空，表示
不启用对应候选类型。

## 12. 电机网络动态

```text
[results...] = ds_motor_dynamics(
    network_1, ..., network_20,
    timeline, event, tcc_curve, fuse, recloser, relay, spec, para
);
```

固定 28 个参数，当前不接受非空独立 `storage` 表。默认结果为 `motor_state`；支持
`bus_voltage, motor_state, event`。

## 13. `para.result` 契约

带结果选择的入口统一遵循：

- `para` 有 `result` 列时必须恰好一行；
- `result` 必须是非 null 字符串；
- 多个结果以逗号分隔，按给定顺序返回；
- 名称匹配转为小写；
- 空项、未知名称和重复名称均报错；
- 左值数量必须与实际返回结果数量一致。

```text
para2 = with_columns(
    alias("bus_phase,summary", result),
    para
);
[voltage, summary] = ds_radial_pf(..., para2);
```

`para.base_frequency_hz` 若存在，必须可转换为 `Float64`、非 null、有限且大于零。
