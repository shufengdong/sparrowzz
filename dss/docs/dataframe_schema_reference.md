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
| 18 | `control_profile` | InvControl、ExpControl 和 GFM 控制配置 | 由 `ibr.control_profile` 引用 |
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

`generator` 还支持当前 PSA 的 typed execution contract：`execution_mode` 可为
`CONSTANT_PQ` 或 `CONSTANT_PV`，显式 PV 必须提供逐相 `v_ref:list<f64>`，PQ 禁止提供。
`kvar_limit` 与 `v_ref` 互斥；`v_min_pu/v_max_pu` 默认 0.95/1.05。逐相
`p_min/p_max/q_min/q_max`、`s_max/i_max` 约束工作点。短路等值使用 `z_r/z_x`，或
`z1_r/z1_x/z0_r/z0_x`，也可由 `xdpp`（优先）/`xdp`、`s_rating`、`v_nom` 和 `x_r`
折算。`dispatcher` 相同的发电机组成 GenDispatcher；同组还使用
`dispatch_bus_from`, `dispatch_bus_to`, `dispatch_terminal`, `dispatch_kw_limit`,
`dispatch_kvar_limit`, `dispatch_kw_band`, `dispatch_weight`。

### `voltage_source` 与 `current_source`

电压源使用逐端子的 `v_magnitude:list<f64>` 和 `v_angle:list<f64>`；电流源使用逐相的
`i_magnitude:list<f64>` 和 `i_angle:list<f64>`。电压源还可用 `bus2/terminal_map2`
表达 Newton 双端口，并通过短路容量或序阻抗字段表示源阻抗。
`current_source.frequency_hz` 默认网络基频，`scan_type` 和 `spectrum` 用于谐波行为。

### `ibr`

`topology` 当前使用 `SINGLE_PHASE`、`THREE_LEG` 或 `FOUR_LEG`；`prime_mover` 使用
`PV`、`BATTERY`、`GENERIC`、`STATCOM` 或 `DSTATCOM`。除必需的 `s_max` 外，可选
`i_max`、`p_avail` 和控制/限值列决定实际注入。OpenDSS `PVSystem` 映射到这张表。
`port_model` 可为 `PQ` 或 `GFM`；GFM 还使用 `v_nom`, `e_mag_pu`, `e_angle_deg`,
`gfm_angle_source`, `r_ohm`, `x_ohm`, `droop_p`, `droop_q`。PV 可用
`pmpp`, `irradiance`, `pct_pmpp`, 温度字段、`pv_pt_curve_x/y`、`pv_eff_curve_x/y`、
`inverter_efficiency`, `cut_in/cut_out`, `var_follow_inverter` 描述阵列和逆变器。
`q_setpoint` 优先于控制配置中的定功率因数。多个设备共享 bounded InvControl 或
ExpControl 时，分别用从 0 连续递增的 `invcontrol_member_order` 或
`expcontrol_member_order` 固定 OpenDSS DERList/PVSystemList 的逐设备顺序；这些序号
不表示 fleet、权重或优先级。

### `control_profile`

`name` 必需，`enabled` 默认 true。`mode` 支持 `VOLTVAR`, `VOLTWATT`,
`DYNAMICREACCURR`, `WATTPF`, `WATTVAR`, `AVR`, `EXPCONTROL`, `GFM`；`combi_mode`
支持 `VV_VW`, `VV_DRC` 以及兼容的 GFM 值，且不能与 `mode` 同时给出。

曲线与公共控制字段包括 `volt_var_breakpoints/volt_var_q_limits`,
`volt_watt_breakpoints/volt_watt_p_limits`,
`volt_watt_charge_breakpoints/volt_watt_charge_p_limits`,
`watt_pf_breakpoints/watt_pf_pf`, `watt_var_breakpoints/watt_var_q_limits`,
`voltage_reference`, `voltage_curve_x_ref`, `reactive_power_reference`,
`delta_p_factor/delta_q_factor`, 三个 change tolerance、`rate_of_change_mode`,
`lpf_tau`, `rise_fall_limit`, `control_model`, `mon_voltage_calc`，以及远方量测的
`monitored_bus/monitored_phases/monitored_v_base`。

AVR 使用 `v_setpoint/avr_gain`。ExpControl 使用 `exp_vreg`, `exp_slope`,
`exp_vreg_tau`, `exp_q_bias`, `exp_vreg_min/max`, `exp_q_max_lead/lag`,
`exp_t_response`, `prefer_q`。`prefer_q=true` 允许当前 bounded 契约中的三相或单相
PVSystem 在无功能力不足时削减本机有功释放 kVA，且与 `ibr.watt_priority` 独立。
GFM 使用 `frequency_deviation_pu`。这些字段的单位、缺省值与组合约束以当前
`psa/src/dsa/parse.rs` 的 `control_profile` 表为准。

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

逐相 `p_rated`, `p_min/p_max/q_min/q_max`, `s_max/i_max`，以及 `v_nom`,
`sc_r_pct/sc_x_pct`, `spectrum`, `watt_priority`, `idling_loss`, `cut_in/cut_out`,
`var_follow_inverter`, `voltage_model`, `v_min_pu/v_max_pu`, `limit_current` 均为可选。
调度字段包括 `dispatch_mode`（`DEFAULT`, `FOLLOW`, `EXTERNAL`, `LOADLEVEL`, `PRICE`）、
`dispatch_state`, `charge_rate/discharge_rate`, `charge_trigger/discharge_trigger` 和
`time_charge_trigger`。QSTS 按潮流实际有功推进 SOC，并把 dispatch 状态跨时点保留。
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

核心列是 `name`, `element`, `terminal`, `mon_phase`, `mode_discharge`,
`mode_charge`, `kw_target`, `kw_target_low`。放电模式支持 `PeakShave`, `Follow`,
`Support`, `Loadshape`, `Time`, `Schedule`, `I-PeakShave`；充电模式支持 `Loadshape`,
`Time`, `PeakShaveLow`, `I-PeakShaveLow`。绝对死区 `kw_band/kw_band_low` 优先于百分比
死区 `pct_kw_band/pct_kw_band_low`。

`element_list:list<str>` 与正 `weights:list<f64>` 按位置对应。其他当前字段为
`pct_rate_kw`, `pct_rate_charge`, `pct_reserve`, `reset_level`,
`time_discharge_trigger`, `time_charge_trigger`, `inhibit_time`, `t_up/t_flat/t_down`,
`kw_threshold`, `disp_factor`, `daily/yearly/duty`, `season_shape`,
`season_targets/season_targets_low`, `enabled`。一个储能只能属于一个启用控制器；省略
`element_list` 时按表顺序接管尚未分配的储能。

### `switch_control`

核心列为 `name`, `switched_obj`, `switched_term`, `action`；`switched_conductor=0`
表示整个端子，正数表示从 1 开始的单导体。状态列包括 `delay`, `normal`, `state`,
`lock`, `reset`, `enabled`。声明式 QSTS 扩展还支持 `action_time`, `repeat_daily`,
`condition`, `monitored_element`, `monitored_terminal`, `monitored_phase`, `threshold`,
`reset_threshold`。`condition` 可为 `Always/Time` 或 Voltage/Current/ActivePower/
ReactivePower 的 Above/Below。DSS 导入器另保留 `source_order`,
`construction_action_pending`, `initial_locked`，以重放 New/Edit/More 的全局稳定顺序。

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
`time`, `load_multiplier`, `generation_multiplier`。

PSA 的 typed continuation state 在 RSDss 中对应七张可选结果表：

| 表 | PSA 状态类型 | 主要列 |
|---|---|---|
| `storage_state` | `TimeSeriesStorageState` | `name,soc,energy,p_total,q_total` |
| `storage_controller_state` | `TimeSeriesStorageControllerState` | `name,mode,state,monitored,target,deadband,requested_p,fleet_p,inhibited,locked` |
| `switch_control_state` | `TimeSeriesSwitchControlState` | 被控对象/端子/导体、`state,normal,locked,condition_active,measurement,pending_action,due_time` |
| `fuse_state` | `TimeSeriesFuseState` | 被测/被控对象、`state,blown,current,trip_time,due_time` |
| `recloser_state` | `TimeSeriesRecloserState` | 被测/被控对象、`state,locked_out,operation,curve_mode`、电流和队列状态 |
| `relay_state` | `TimeSeriesRelayState` | 公共状态以及 Voltage、ReversePower、46、47、Generic、DOC、Distance、TD21 的 typed 量测字段 |
| `fault_state` | `TimeSeriesFaultState` | `index,bus,phases,kind,active,cleared,on_time,off_time,temporary,min_amps,current` |

四张保护/开关状态表中的 `phases` 与 `phase_states` 都是按位置对应的 `list<str>`。
`operation`, `index`, `td21_history_points`, `td21_quiet_samples` 输出为 `u64`；端子和导体
编号为 `u32`。具体输出列由分析实现决定，生成脚本时先请求最小结果集，再用 `cols()`
检查实际 schema，避免猜测不存在的输出列。
