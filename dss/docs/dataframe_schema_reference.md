# DataFrame schema 参考

本文记录 RSS 中直接构造或传给 `ds_*` 的 DataFrame 契约。列名区分大小写，表内对象名的
解析通常不区分 ASCII 大小写。类型写作 `str`、`bool`、`u32`、`f64`、`list<str>` 和
`list<f64>`；脚本中的整数常可由 Polars 转为解析器需要的数值类型，但新脚本应尽量按表中
类型构造。

除非单独说明，电气量使用 SI：电压 V、电流 A、有功 W、无功 var、视在功率 VA、阻抗
ohm、时间 s、频率 Hz、角度 rad。DSS 导入器负责把 OpenDSS 常用的 kV、kW、kvar、kVA
和角度制换算到这些契约。

## 1. 完整网络的 20 张表

完整网络参数必须按下列顺序传入。`import_dss` 和 `import_bmopf` 也按此顺序返回。

| 位置 | 表 | 每行表示 | 最小必需列 |
|---:|---|---|---|
| 1 | `bus` | 母线 | `id:str`, `terminals:list<str>` |
| 2 | `line` | 线路 | `name`, `bus_from`, `bus_to`, `terminal_map_from`, `terminal_map_to`, `length`，以及 `linecode` 或 `geometry` |
| 3 | `linecode` | 单位长度线路参数 | `name`, `n_conductors`, `r_series`, `x_series` |
| 4 | `load` | 负荷 | `name`, `bus`, `terminal_map`, `configuration`, `p_nom`, `q_nom` |
| 5 | `transformer` | 变压器本体 | 简单两绕组或 `multi_winding` 契约，见下文 |
| 6 | `voltage_source` | 理想电压源 | `name`, `bus`, `terminal_map`, `v_magnitude`, `v_angle` |
| 7 | `motor` | 感应电机 | `name`, `bus`, `terminal_map`, `kv`, `kw` |
| 8 | `transformer_winding` | 多绕组的物理绕组 | 与 `transformer` 的 `multi_winding` 行配套 |
| 9 | `transformer_impedance` | 多绕组两两短路阻抗 | 与物理绕组编号配套 |
| 10 | `current_source` | 电流源 | `name`, `bus`, `terminal_map`, `i_magnitude`, `i_angle` |
| 11 | `generator` | 同步/等值发电机 | `name`, `bus`, `terminal_map`, `configuration`, `p_nom`, `q_nom` |
| 12 | `ibr` | 逆变器资源 | `name`, `bus`, `terminal_map`, `topology`, `prime_mover`, `s_max` |
| 13 | `switch` | 显式线路开关 | `name`, `bus_from`, `bus_to`, `terminal_map_from`, `terminal_map_to` |
| 14 | `shunt` | 并联导纳 | `name`, `bus`, `terminal_map`, `g`, `b` |
| 15 | `capacitor` | 电容器组 | `name`, `bus`, `terminal_map`, `configuration`, `q_rated`, `v_nom` |
| 16 | `reactor` | 电抗器 | 由 `reactor` 解析器规定；优先使用导入器生成 |
| 17 | `spectrum` | 谐波频谱 | `name`, `h`, `mag_pct`, `angle_deg`，三列可为等长列表 |
| 18 | `control_profile` | 调压/电容等控制配置 | 与目标设备和控制类型配套 |
| 19 | `xy_curve` | XY 曲线点 | `name`, `x`, `y`；一个曲线占多行 |
| 20 | `storage` | 储能设备及状态 | 见“可注入设备” |

零行 DataFrame 可用于没有该对象族的可选位置。它只作为空值占位，不会绕过对非空表的
schema 检查。不要交换位置，也不要用一张有行的无关表占位。

## 2. 网络基础表

### `bus`

| 列 | 类型 | 必需 | 含义/默认值 |
|---|---|---:|---|
| `id` | `str` | 是 | 唯一母线名 |
| `terminals` | `list<str>` | 是 | 节点/相标签，如 `["1","2","3"]` |
| `grounded` | `list<str>` | 否 | 接地端子集合 |
| `v_min`, `v_max` | `f64` | 否 | 母线总限值 |
| `v_min_phase`, `v_max_phase` | `f64` 或列表 | 否 | 分相限值 |

端子标签来自实际连接，不能假定所有母线都是三相 `1,2,3`。

### `linecode`

`r_series`、`x_series` 是按导体顺序展开的方阵，`n_conductors` 必须与矩阵维度一致。
可选 `g_from`、`b_from`、`g_to`、`b_to` 描述两端并联导纳；可选 `i_max`、`s_max` 及
normal/emergency rating 列用于越限检查。矩阵值是单位长度参数，`line.length` 与之相乘。

### `line`

| 列 | 类型 | 必需 | 含义 |
|---|---|---:|---|
| `name` | `str` | 是 | 唯一线路名 |
| `bus_from`, `bus_to` | `str` | 是 | 首、末端母线 |
| `terminal_map_from`, `terminal_map_to` | `list<str>` | 是 | 两端导体到母线端子的映射 |
| `linecode` | `str` | 条件必需 | 与 `geometry` 二选一 |
| `geometry` | `str` | 条件必需 | 与 `linecode` 二选一 |
| `length` | `f64` | 是 | 与线路参数的单位长度基准一致 |
| `open_from`, `open_to` | `list<bool>` | 否 | 分导体开断状态 |

### `transformer`

简单两绕组形式使用 `name, subtype, bus_from, bus_to, terminal_map_from,
terminal_map_to, v_nom_from, v_nom_to, s_rating`，并提供相应串联/励磁阻抗列。
`subtype=multi_winding` 时，绕组连接和两两阻抗放入 `transformer_winding` 与
`transformer_impedance`。该契约包含绕组编号、tap、连接方式和阻抗基准等关联约束，手写
前应查阅 `psa/src/dsa/parse/transformer*.rs`；普通 DSS 变压器优先交给 `import_dss`。

### `switch`

基础列与一条无 `linecode` 的连接支路相同：`name, bus_from, bus_to,
terminal_map_from, terminal_map_to`。可选 `open` 给出开断状态，`i_max` 和额定值列用于
越限。显式 `switch` 表保留开关身份和控制状态，不应改写成普通线路来代替。

## 3. 可注入设备

### `load` 与 `generator`

共同必需列为 `name, bus, terminal_map, configuration, p_nom, q_nom`。
`configuration` 使用解析器支持的连接名。`load.model` 默认为恒功率；负荷还可以提供
电压模型边界和 `spectrum`。有功/无功的符号由设备契约决定，不能通过观察某个结果表的
功率方向反推输入符号。

### `voltage_source` 与 `current_source`

电压源使用 `v_magnitude: f64` 和 `v_angle: f64/list<f64>`；电流源使用
`i_magnitude` 和 `i_angle`。`current_source.frequency_hz` 默认网络基频，`scan_type` 和
`spectrum` 用于谐波行为。

### `ibr`

`topology` 当前使用 `SINGLE_PHASE`、`THREE_LEG` 或 `FOUR_LEG`；`prime_mover` 使用
`PV`、`BATTERY`、`GENERIC`、`STATCOM` 或 `DSTATCOM`。除必需的 `s_max` 外，可选
`i_max`、`p_avail` 和控制/限值列决定实际注入。OpenDSS `PVSystem` 映射到这张表。

### `motor`

必需 `name, bus, terminal_map, kv, kw`。`configuration` 默认 `DELTA`，`kva` 默认
`1.2 * abs(kw)`；启动、滑差和动态参数由具体分析解析。

### `storage`

必需列为：

```text
name, bus, terminal_map, configuration,
p_nom, q_nom, energy_rated, soc_initial, soc_min, soc_max,
charge_efficiency, discharge_efficiency
```

功率、kVA、电流、充放电阈值、待机损耗、时间状态和 dispatch 相关列均为可选扩展。
`import_dss` 的 RSDss 静态返回值当前不会填充这张表；需要储能时，应在 RSS 中构造或从
外部数据读取后放在完整网络的第 20 位。

## 4. `para`

`para` 是一行表。`import_dss` 至少生成：

| 列 | 类型 | 规则 |
|---|---|---|
| `base_frequency_hz` | `f64` | 有限且大于 0 |
| `line_model` | `str` | 常用 `exact`；各分析可有自己的默认值 |
| `result` | `str` | 逗号分隔、忽略大小写；不得为空、未知或重复 |

每个 `ds_*` 支持的 `result` 名称见 [DSA 函数参考](dsa_function_reference.md)。返回表顺序
严格跟随 `result` 中的顺序。

## 5. QSTS 表

### `solution`

必须恰好表达一组时间设置：

| 列 | 类型 | 必需 | 含义 |
|---|---|---:|---|
| `mode` | `str` | 是 | `DAILY`, `YEARLY`, `DUTY`/`DUTYCYCLE`, `PEAKDAY`, `LOADDURATION`, `LD1`, `LD2` |
| `number` | 数值 | 是 | 求解步数，正整数语义 |
| `step_size_s` | `f64` | 是 | 步长，秒 |
| `hour`, `seconds` | 数值 | 否 | 起始时刻，默认 0 |
| `load_multiplier` | `f64` | 否 | 全局负荷倍率 |
| `generation_multiplier` | `f64` | 否 | 全局发电倍率 |
| `control_mode` | `str` | 否 | 控制模式 |
| `trapezoidal` | `bool` | 否 | 累计量积分方式 |
| `sample_energy_meters` | `bool` | 否 | 是否采样电能表 |
| `default_daily`, `default_yearly`, `default_duty` | `str` | 否 | 全局默认曲线名 |
| `price_curve` | `str` | 否 | 价格曲线名 |
| `price_signal` | `f64` | 否 | 默认价格信号 |
| `peak_day` | `u32` | 否 | 仅 PeakDay 模式 |

运行时先在 `hour*3600 + seconds` 采样曲线，然后求解首个 `start + step_size_s` 结果；共返回
`number` 个时点。

### `load_shape`

长表形式，每个曲线点一行：

| 列 | 类型 | 必需 | 默认/约束 |
|---|---|---:|---|
| `name` | `str` | 是 | 曲线名 |
| `time_s` | `f64` | 是 | 曲线内时间，秒 |
| `p_multiplier` | `f64` | 是 | 有功倍率或 actual 值 |
| `q_multiplier` | `f64` | 否 | 默认等于 `p_multiplier` |
| `period_s` | `f64` | 否 | 同一曲线各行必须一致 |
| `interpolation` | `str` | 否 | 同一曲线各行必须一致 |
| `use_actual` | `bool` | 否 | 同一曲线各行必须一致 |

### `shape_assignment`

每行把曲线绑定到设备：`device_kind:str`, `device:str` 必需；`daily`, `yearly`, `duty`
为可选曲线名。`device_kind` 应使用运行时支持的对象族，如 `LOAD`。

### `storage_controller`

导入器生成的列为：

```text
name, element, terminal, mon_phase, mode_discharge, mode_charge,
kw_target, kw_target_low, kw_band, kw_band_low,
element_list, weights, pct_reserve, disp_factor, enabled
```

`element_list:list<str>` 与 `weights:list<f64>` 按位置对应。运行时对多控制器、功率/电流
限制和成员仲裁有明确规则；不要通过重复行模拟优先级。

### `switch_control`

```text
source_order:u32, name:str, switched_obj:str, switched_term:u32,
action:str, delay:f64, normal:str, state:str,
construction_action_pending:bool, initial_locked:bool,
lock:bool, reset:bool, enabled:bool
```

`source_order` 用于保留 OpenDSS 命令流中的稳定顺序。

## 6. 保护表

### `tcc_curve`

列顺序和名称固定为 `source_order, name, npts, c_array, t_array`。
`c_array:list<f64>` 和 `t_array:list<f64>` 长度都必须等于 `npts`，并按同一位置组成
电流/动作时间点。

### `fuse`

```text
source_order, name, monitored_obj, monitored_term,
switched_obj, switched_term, fuse_curve, rated_current, delay,
normal:list<str>, state:list<str>, reset:bool, enabled:bool
```

### `recloser`

```text
source_order, name, monitored_obj, monitored_term,
switched_obj, switched_term, num_fast,
phase_fast, phase_delayed, [ground_fast], [ground_delayed],
phase_trip, ground_trip, phase_inst, ground_inst,
reset, shots, reclose_intervals:list<f64>, delay,
td_phase_fast, td_ground_fast, td_phase_delayed, td_ground_delayed,
normal, state, enabled
```

方括号列是可选列。`reclose_intervals` 与 `shots` 的关系由解析器校验，不应在 RSS 中自行
截断。多次重合闸状态会跨 QSTS 时步继续。

### `relay`

公共前缀为：

```text
source_order, name, monitored_obj, monitored_term,
switched_obj, switched_term, type,
phase_trip, ground_trip, td_phase, td_ground,
phase_inst, ground_inst, kv_base,
pct_pickup_46, base_amps_46, isqt_46, pct_pickup_47,
over_trip, under_trip
```

后续列随 relay 类型扩展，包括方向过流、电压、频率、距离和跳闸/重合闸状态。手写非过流
Relay 时应直接核对 `psa/src/dsa/parse/dss_import/controls.rs::relay_frame` 与保护解析器，
不能只填公共前缀。

## 7. 监视、计量和故障

### `monitor`

必需 `name, element`；常用可选列为 `terminal, mode, enabled, v_base, residual,
vi_polar, p_polar, action, on_time`。DSS 导入器生成的核心列是：

```text
source_order, name, element, terminal, mode,
enabled, residual, vi_polar, p_polar
```

### `energy_meter`

必需 `name, element`；可选 `terminal, enabled, zone_elements, local_only, customers,
voltage_level, aux1..aux30, action, on_time, action_param, param, reduce_pct`。导入器还可生成
`source_order, option, kva_normal, kva_emerg, peak_current`。区域成员名称和电气拓扑必须能
一致解析。

### `fault`

静态故障的核心列为：

| 列 | 类型 | 必需 | 含义 |
|---|---|---:|---|
| `bus` | `str` | 是 | 故障母线 |
| `terminal_map` | `list<str>` | 是 | 参与故障的端子 |
| `kind` | `str` | 是 | `slg`, `ll`, `llg`, `2pg`, `3p`, `3pg` |
| `bus2` | `str` | 否 | 第二端母线 |
| `percent` | `f64` | 否 | 线路中点百分比等定位信息 |
| `zf`, `xf` | `f64` | 否 | 故障阻抗分量 |

QSTS 故障还可使用 `on_time`/`ontime`, `off_time`, `temporary`, `min_amps`/`minamps`,
`enabled`。

## 8. 输出表选择原则

`bus_phase` 最适合逐相对照，稳定键为 `lower(bus), phase`；`bus_voltage` 把相量放在列表
列中。支路比较应使用元件、端点/terminal 和 phase 作为键。QSTS 为每张逐时步结果追加
`time`, `load_multiplier`, `generation_multiplier`。具体输出列由分析实现决定，生成脚本时
先请求最小结果集，再用 `cols()` 检查实际 schema，避免猜测不存在的输出列。

