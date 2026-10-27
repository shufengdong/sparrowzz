# DataFrame schema 参考

本文记录 RSS 中直接构造或传给 `ds_*` 的 DataFrame 契约。列名区分大小写，表内对象名的
解析通常不区分 ASCII 大小写。完整类型语法、Arrow/Parquet 映射、nullable、受控上传转换、
Struct 参数卡和结果继承规则以 [机器目录格式](catalog/README.md)及
`catalog/dataframes.json.dtype_grammar` 为准。自然语言表格用于阅读，不得覆盖机器合同。

除非单独说明，电气量使用 SI：电压 V、电流 A、有功 W、无功 var、视在功率 VA、阻抗
ohm、时间 s、频率 Hz、角度 rad。DSS 导入器负责把 OpenDSS 常用的 kV、kW、kvar、kVA
和角度制换算到这些契约。

## 机器可读门面与输入分类

[`catalog/study_facades.json`](catalog/study_facades.json) 是全部已注入 DataFrame `ds_*`
研究门面的权威参数映射。它逐函数、逐位置给出 `inputKind` 和指向本文件同一 snapshot 中
[`catalog/dataframes.json`](catalog/dataframes.json) 的 `schemaRef`；函数名、arity 和结果名仍由
[`catalog/functions.json`](catalog/functions.json) 定义。目录格式见
[`catalog/README.md`](catalog/README.md)。调用方必须读取这三份合同，禁止根据函数名或参数名
猜测输入类别、列类型、默认值或结果结构。

分类语义固定为：

- `model`：网络拓扑、设备、保护装置或持久控制对象；身份和跨表引用关系属于模型；
- `data`：量测、曲线采样值、场景、事件、故障工况、观测、假设等可替换的行式数据；
- `struct`：逻辑上的一个配置对象，使用零行或单行参数 DataFrame，具体行数由 schema 约束。

变化频率不能单独决定类别。解析器如何消费每行、该表是否声明持久身份、是否被其他模型表
引用，才是分类依据。当前源码审计矩阵如下；固定网络在 JSON 中展开为 20 个独立输入：

| 门面/输入 | `inputKind` | `schemaRef` | 源码语义与判断依据 |
|---|---|---|---|
| 全部完整门面的 `bus`…`storage` 20 张固定网络表 | `model` | `frames.bus`…`frames.storage` | 构成拓扑、设备和持久网络控制模型；位置顺序固定 |
| 通用 `para` | `struct` | `frames.para` | 表示零个或一个配置对象；底层 `parse_params` 在无 `result` 时容忍多行但只读取第 0 行，调用方禁止依赖被忽略的额外行；有 `result` 时强制恰好一行 |
| QSTS `solution` | `struct` | `frames.solution` | `parse_solution` 强制恰好一行并生成整个求解日程 |
| QSTS `load_shape` | `data` | `frames.load_shape` | 每行是命名曲线的一个时间采样点 |
| QSTS `shape_assignment` | `model` | `frames.shape_assignment` | 保存设备身份到命名曲线的持久引用关系 |
| QSTS `storage_controller`、`switch_control` | `model` | `frames.storage_controller`、`frames.switch_control` | 有稳定控制器身份、受控设备引用和跨时点状态 |
| QSTS/保护/动态 `tcc_curve` | `data` | `frames.tcc_curve` | 保存电流—时间曲线采样值；保护设备按名称引用它 |
| QSTS/保护/动态 `fuse`、`recloser`、`relay` | `model` | `frames.fuse`、`frames.recloser`、`frames.relay` | 有保护装置身份、整定、曲线引用和动作目标 |
| QSTS/谐波 `monitor`、QSTS `energy_meter` | `model` | `frames.monitor`、`frames.energy_meter` | 有持久表计身份、被测元件或区域引用及寄存器配置 |
| 短路/QSTS `fault` | `data` | `frames.fault` | 每行是可替换的故障位置、类型、阻抗或投退工况 |
| 恢复重构 `fault` | `data` | `analysis_inputs.restore_fault` | 每行只声明必须保持断开的开关身份，不使用短路 fault schema |
| 网络动态 `fault` | `data` | `analysis_inputs.dynamic_fault` | 在基础故障电气数据上增加稳定事件名和投退生命周期 |
| 状态估计 `measurement` | `data` | `analysis_inputs.state_estimation_measurement` | 标量量测、位置、数值和权重行 |
| Monte Carlo `spec`、`multipliers` | `struct`、`data` | `analysis_inputs.monte_carlo_spec`、`analysis_inputs.monte_carlo_multipliers` | `spec` 恰好一行；`multipliers` 是 replay 场景样本行 |
| 保护配合 `fault_case`、`current_sample`、`pair`、`observed_action` | `data` | 对应 `analysis_inputs.protection_*` | 本次审计的故障工况、电流样本、主后备组合和观测证据 |
| 保护配合 `para` | `struct` | `analysis_inputs.protection_para` | 零行或单行容差与结果选择配置 |
| 故障定位 `measurement`、`hypothesis_switch`、`hypothesis_line` | `data` | 对应 `analysis_inputs.fault_location_*` | 复相量观测及可替换的候选网络假设行 |
| 故障定位 `para` | `struct` | `analysis_inputs.fault_location_para` | 零行或单行搜索与损失函数配置 |
| 动态 `timeline`、`event` | `data` | `analysis_inputs.dynamic_timeline`、`analysis_inputs.dynamic_event` | 外部采样要求和按时刻排列的运行事件 |
| 动态 `spec` | `struct` | `analysis_inputs.dynamic_spec` | 零行或单行动态模型与积分配置 |

同名 alias 不建立 schema 等价关系。状态估计和故障定位的 `measurement`、短路/QSTS、恢复
重构和网络动态的 `fault` 都必须按所选 facade 的 `schemaRef` 分别处理。

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
PSA owned DataFrame 支持的混合相数子集要求完整三相主绕组，其他非空相子集只能使用
grounded-Wye，并由各行 `terminal_map` 保留物理相身份。mixed Delta、floating-Wye、
两绕组混合宽度和 AutoTrans 混合宽度会 fail closed；该扩展不能由 DSS Transformer
自动导入，因为 OpenDSS 没有逐绕组相数属性。

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

### `storage`

除额定能量、SOC 和充放电效率外，Storage 可选使用成对的
`inverter_efficiency_curve_x/y:list<f64>` 表示 OpenDSS `EffCurve`。两列必须同时出现、
长度相等且至少含两个点；X 必须非负并严格递增，Y 必须在 `(0,1]`，同时必须提供
`s_max`。X 是交流端有功绝对值与设备额定 VA 的比值，Y 是逆变器效率，运行时线性插值并
按端点斜率外推。放电能量顺序为“交流输出除以曲线效率、加空载损耗、再除以放电效率”；
充电顺序为“交流输入乘曲线效率、扣空载损耗、再乘充电效率”。机器调用方必须以
[`catalog/dataframes.json`](catalog/dataframes.json) 的 `frames.storage.contract` 为准。

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
`monitored_bus/monitored_phases/monitored_v_base`。OpenDSS 多节点 `MonBus` 使用
`monitored_nodes/monitored_v_bases` 两个等长列表；节点写作 `bus.1`（相地）或
`bus.1.2`（相间），额定电压单位为 kV。

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

通用 `para` 的公开合同是零行或单行配置表；零行选择该分析默认值。底层 `parse_params`
在没有 `result` 列时会容忍多行但只读取第 0 行；调用方禁止依赖被忽略的额外行。
`result` 列存在时，表达式分派会强制表恰好一行。`import_dss` 至少生成：

| 列 | 类型 | 规则 |
|---|---|---|
| `base_frequency_hz` | `f64` | 有限且大于 0 |
| `max_iter` | 正整数 | 非线性或优化迭代上限 |
| `max_control_iter` | 正整数 | 离散控制外层迭代上限 |
| `tol` | `f64` | 有限且大于 0 |
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
`enabled`。DSS ExecutionStudy 的有界 Fault 导入要求显式提供 `ontime`、`temporary`、
`minamps` 和 `enabled`，不从 OpenDSS 运行期默认值猜测。
相间故障使用 `kind=ll` 和两个不同的非接地 `terminal_map` 节点。ExecutionStudy 可把
`Phases=1 Bus1=bus.1 Bus2=bus.2` 的同母线 DSS Fault 无损降低为该形式；跨母线、同节点、
显式 `GMatrix` 或多节点非接地 Bus2 仍会 fail closed。

## 8. 分析专用输入表

本节中的逻辑表名用于区分同一个 RSDss 参数槽在不同函数中的 schema。例如状态估计和
故障定位都把参数写成 `measurement`，但它们不是同一种表，不能互换。机器目录使用
`state_estimation_measurement` 和 `fault_location_measurement` 消除这种歧义。

### 谐波 `para`

谐波 `para` 使用 `analysis_inputs.harmonic_para`，必须恰好一行并包含正整数语义的
`h_max`。可选 `buses:list<str>` 只能含大小写不敏感的唯一非空母线名；`skin_effect` 和
`neglect_load_y` 默认为 `false`。公共求解、基频和 `result` 列遵循通用 `para` 规则。

### 状态估计 `measurement`

每行必需 `type:str, phase:str, value:f64`；`weight:f64` 可省略，默认 `1.0`，且必须
有限并大于零。其余必需列由 `type` 决定：

| `type` | 位置列 | 其他列 |
|---|---|---|
| `voltage_magnitude` | `bus` | 无 |
| `power_injection` | `bus` | `kind=P/Q` |
| `power_flow` | `bus_from,bus_to` | `kind=P/Q` |
| `current_magnitude` | `bus_from,bus_to` | 无 |

未知 `type`、非有限 `value`、无效 `kind` 或缺少对应位置列都会使整个量测表解析失败。
状态估计结果采用 `bus_voltage` 列表 schema：`bus,v_mag,v_ang`，电压单位 V、角度单位 rad。

### Monte Carlo `spec` 与 `multipliers`

`spec` 必须恰好一行。可选列和默认值为：

| 列 | 类型 | 默认值/范围 |
|---|---|---|
| `mode` | `str` | `m1`；还可取 `global,m1_replay,global_replay` |
| `solver` | `str` | `radial`；还可取 `newton,newton_raphson` |
| `n_scenarios` | 无符号整数 | `100`，随机模式使用 |
| `seed` | 无符号整数 | PSA 固定默认 seed |
| `mean` | `f64` | `1.0` |
| `std` | `f64` | `0.1`，必须非负 |
| `v_min` / `v_max` | `f64` | `0.95` / `1.05`，满足 `0 <= v_min < v_max` |

随机模式要求 `multipliers` 是零行表。Replay 模式要求
`scenario:整数,multiplier:f64`；场景编号从 1 连续递增。`m1_replay` 还要求 `load:str`，
并为每个场景和每个网络负荷恰好提供一行；`global_replay` 每个场景恰好一行。

### 恢复重构 `fault`

恢复重构的 `fault` 使用 `analysis_inputs.restore_fault`，每行只需要 `name:str`。`name`
必须引用完整网络 `switch` 表中的开关，表示故障隔离后该开关必须保持断开。该表不接受
短路 `frames.fault` 的 `bus,terminal_map,kind` 语义；两者参数 alias 相同，但 schema 不同。

### 保护配合输入

| 参数表 | 必需列 | 可选列与默认值 |
|---|---|---|
| `fault_case` | `name,location` | `start_time_seconds=0` |
| `current_sample` | `fault,device,phase_currents_amp:list<f64>` | `ground_current_amp=0` |
| `pair` | `primary,backup` | `minimum_margin_seconds=0` |
| `observed_action` | `fault,device,action,time_seconds` | 整张表可为空 |
| `para` | 无 | 三个非负容差列及 `result` |

`fault_case.name` 按 ASCII 大小写不敏感规则唯一。电流列表必须非空，所有电流有限且非负。
`current_sample.fault` 和 `observed_action.fault` 必须引用已有工况；主保护和后备保护不能是
同一个设备。`para` 的容差列为 `pickup_tolerance_seconds`、
`operation_tolerance_seconds`、`reclose_tolerance_seconds`。

### 故障定位输入

`measurement` 至少一行，必需列为：

```text
bus:str, element:str,
current_re:list<f64>, current_im:list<f64>
```

可选列为 `terminal=1`、成对的 `voltage_re/voltage_im`、
`current_standard_deviation`、`voltage_standard_deviation`、`covariance` 和 `weight=1`。
实虚部列表必须非空、等长且有限；标准差必须为正并与相量等长。`covariance` 使用
电流实/虚部后接电压实/虚部的残差顺序，长度必须为矩阵阶数的平方且矩阵正定；它与两列
标准差不能同时设置。

`hypothesis_switch` 使用 `hypothesis,name,open`，`hypothesis_line` 使用
`hypothesis,name,scale`。后者的 `scale` 必须有限且大于零。同一假设中不能重复同名设备。
两张表都可为零行表。

故障定位 `para` 可为零行或一行。可选列是 `candidate_lines,step_percent,top_n,
resistance_min_ohm,resistance_max_ohm,reactance_min_ohm,reactance_max_ohm,loss,
loss_parameter`，以及公共潮流列 `max_iter,max_control_iter,tol,line_model`。`loss` 仅接受
`least_squares,huber,cauchy`。

### 网络动态输入与 V12 结果

`ds_network_dynamics` 的前 20 个参数是固定网络表，后 9 个位置为：

| 位置 | 表 | 行数要求 |
|---:|---|---|
| 21 | `timeline` | 至少一行 |
| 22 | `event` | 零行或多行 |
| 23 | `tcc_curve` | 零行或多行 |
| 24 | `fuse` | 零行或多行 |
| 25 | `recloser` | 零行或多行 |
| 26 | `relay` | 零行或多行 |
| 27 | `fault` | 零行或多行 |
| 28 | `spec` | 零行或一行 |
| 29 | `para` | 零行或一行；有 `result` 时必须恰好一行 |

位置不可省略或交换。零行表只表示对应子系统未配置；任何非空表都会按该位置的 schema 解析。

`timeline` 必需 `time_s:f64`，可选 `source_scale_re:f64=1,source_scale_im:f64=0`。所有值必须
有限，时刻必须严格递增。该表同时用于 `ds_network_dynamics` 和兼容入口
`ds_motor_dynamics`。

`event` 每行必需 `time_s,kind`，`kind` 决定其他列：

| `kind` | 必需列 | 可选列 |
|---|---|---|
| `fault_on` | `event_id,bus,phases,fault_kind` | `bus2,percent,zf_re=0,zf_im=0,g_matrix` |
| `fault_off` | `event_id` | 无 |
| `topology` | `element,action` | 无 |
| `conductor_topology` | `element,terminal,conductor,action` | 无 |
| `protection` | `device,element,action` | 无 |
| `conductor_protection` | `device,element,terminal,conductor,action` | 无 |
| `storage_state` | `device,action` | `action=IDLING/CHARGING/DISCHARGING` |
| `device_availability` | `device_class,device,enabled` | `device_class=Load/PVSystem/Generator` |

拓扑/保护 `action` 接受 `open/trip` 或 `close/reclose`。`fault_on.g_matrix` 若非空，
必须是按行展平、有限、对称的 `len(phases) × len(phases)` 电导矩阵。`spec` 可为零行或一行；支持
`frequency_hz,initial_slip,shaft_power_w,mechanical_load,torque_exponent,
voltage_exponent,speed_exponent,locked_rotor,stall_slip_threshold,
max_corrector_iterations,tolerance,max_network_iterations,voltage_tolerance,relaxation,
max_same_time_action_rounds=16`。
`mechanical_load` 接受 `constant_power/constant`、`speed_dependent_torque/torque` 或
`power_law`；省略的数值取 PSA `MotorNetworkDynamicSpec` 默认值。

`para` 使用公共潮流参数 `max_iter,max_control_iter,tol,line_model,base_frequency_hz`，并用
`result:str` 选择输出。`base_frequency_hz` 必须有限且大于零；`result` 的逗号项去除首尾空白后
按顺序返回，空项、未知名称和重复名称失败。零行 `para` 使用默认求解参数和默认结果
`bus_voltage`。

`ds_network_dynamics` 在上述输入之后还接收 `fault`。它使用保护型动态 fault schema：
声明中至少提供故障名称、母线、故障类型和相别，并可用 `on_time/off_time` 定义生命周期；
精确列与 DSS `Fault` 导入表一致。`tcc_curve/fuse/recloser/relay` 使用 QSTS 同名输入
schema。四张表和 `fault` 都可为零行表。

V12 固定返回以下 15 种可选结果。逐采样电气表均以 `time_s,sequence` 开头；空设备族仍
返回带固定列的零行表。

| 结果 | 固定列（省略换行） |
|---|---|
| `bus_voltage` | `time_s,sequence,bus,phase,v_re,v_im,v_mag,v_angle_deg` |
| `branch_current` | `time_s,sequence,element,kind,terminal,bus,phase,i_re,i_im,i_mag,i_angle_deg` |
| `source_flow` | `time_s,sequence,source,terminal,bus,phase,v_re,v_im,i_re,i_im,p_w,q_var` |
| `pvsystem_state` | `time_s,sequence,pvsystem,bus,phase,voltage_re,voltage_im,current_re,current_im,p_w,q_var,modulation,control_error,safe_mode` |
| `storage_state` | `time_s,sequence,storage,bus,phase,voltage_re,voltage_im,current_re,current_im,p_w,q_var,modulation,control_error,command_state,safe_mode,stored_kwh,soc_pu` |
| `motor_state` | `time_s,sequence,motor,phase,voltage_re,voltage_im,current_re,current_im,current_mag,current_angle_deg,slip,speed_pu,relative_speed_rad_s,rotor_angle_deg,p_w,q_var,electromagnetic_torque_nm,shaft_power_w,stalled` |
| `event` | `time_s,event_id,kind,target,action,applied,sequence,source,reason` |
| `solver_trace` | `time_s,sequence,predictor_iterations,corrector_iterations,voltage_residual,current_residual,kcl_residual,converged` |
| `fault_state` | `time_s,sequence,name,bus,kind,enabled,active,cleared,current_a,transition_reason` |
| `protection_state` | `time_s,sequence,name,device_type,monitored_obj,monitored_term,channel,current_a,pickup,timer_s,operation,state,pending_action,next_due_s,relay_type,measurement_kind,min_voltage_pu,max_voltage_pu,trip_source,locked_out,doc_control_power_w,doc_current_a,doc_current_angle_deg,directional_blocked,doc_region,negative_sequence_current_a,negative_sequence_voltage_v,reverse_power_w,monitored_conductor,pickup_phase,switched_obj,switched_term,switched_conductor` |
| `frt_state` | `time_s,sequence,name,bus,voltage_pu,frequency_hz,voltage_zone,frequency_zone,violation_time_s,connection_state,recovery_fraction,p_w,q_var,stored_kwh,min_voltage_pu,max_voltage_pu,governing_voltage_side,operating_mode,zone_response,policy,connection_kind,measurement_phase_from,measurement_phase_to,device_voltage_pu,port_available` |
| `control_state` | `time_s,sequence,name,control_type,monitored_obj,monitored_term,controlled_obj,mode,measurement_kind,measurement_value,setpoint_value,deadband_value,state,pending_action,next_due_s,source_ordinal,tap_position,tap_ratio,regulator_direction,capacitor_step,capacitor_steps,last_open_s` |
| `device_state` | `time_s,sequence,class,name,bus,enabled,p_w,q_var,protection_connected,protection_owner,frt_connected,effective_connected,disconnect_reason,protection_owner_count,protection_connected_phase_count,protection_required_phase_count` |
| `generator_state` | `time_s,sequence,generator,bus,phase,enabled,energized,voltage_re,voltage_im,current_re,current_im,current_mag,current_angle_deg,p_w,q_var,e1_mag_v,rotor_angle_deg,relative_speed_rad_s,speed_pu,shaft_power_w,electrical_power_w` |
| `device_protection_connection` | `time_s,sequence,class,name,terminal,conductor,phase,owner_type,owner,owner_connected,effective_conductor_connected,aggregate_device_connected` |

电压、电流、有功、无功和角度分别使用 V、A、W、var 和 degree。`branch_current` 的电流方向为
从端口母线流入支路；PV/Storage 的 P/Q 正值表示向网络注入，Storage 充电 P 为负；电机 P/Q
正值表示流入电机。未配置设备族时仍返回上述固定列的零行表。

外部 `timeline` 是采样要求，不是输出行数承诺。故障切换、保护到期、普通控制和 FRT 状态变化
会插入精确内部边界；同一个 `time_s` 可以有多个 `sequence`。跨表连接使用
`time_s,sequence` 加结果自身的设备/母线/端子/相身份列，不能只按时间或当前行号连接。

## 9. 固定结果 schema

同一个结果名可能随分析不同而改变 schema。例如静态潮流的 `bus_voltage` 与谐波的
`bus_voltage` 不同。下列上下文是契约的一部分。

### 潮流与状态估计

- `bus_voltage`：`bus,v_mag,v_ang`，其中后两列是按母线相序排列的列表，单位 V/rad；
- `bus_phase`：`bus,phase,v_re,v_im,v_mag,v_ang`，单位 V/rad；
- `generator_state`：`generator,bus,phase,p,q,v_mag,i_mag,q_min,q_max,control_mode,
  q_limited`；
- `summary`：`method,converged,control_iterations,control_actions,control_time,solve_count,
  plan_builds,last_iterations,total_iterations,p_loss,q_loss,violation_count,
  max_violation_ratio`；
- `violations`：`kind,element,bus,phase,value,limit,ratio`。

支路、电源、绕组、导体和设备状态表的固定列定义在 PSA
`src/dsa/output/power_flow.rs`、`device_state.rs` 和 `line_conductor.rs`。它们的精确列顺序
已登记在 [`catalog/dataframes.json`](catalog/dataframes.json) 的 `result_frames`，并由目录
校验器直接与这些 DataFrame 构造器核对。不能从 OpenDSS 导出列猜测名称。

QSTS 的 `meter_phase_voltage` 固定为
`meter,time,voltage_base_v,phase,sample_count,v_min_pu,v_max_pu,v_avg_pu`。

### 谐波和短路

谐波 `bus_voltage` 固定为
`bus,phase,harmonic,voltage_mag,voltage_angle,thd`，角度单位 deg，THD 单位 `%`。
谐波 `bus_sequence` 为
`bus,harmonic,sequence,voltage_re,voltage_im,voltage_mag,voltage_angle`；
`sequence` 取 `zero/positive/negative`。

谐波 `branch_flow` 为 `kind,element,terminal,bus,phase,harmonic,voltage_re,
voltage_im,voltage_mag,voltage_angle,current_re,current_im,current_mag,current_angle,p,q`。
`branch_sequence` 在相同电气量之前使用
`kind,element,terminal,bus,harmonic,sequence` 标识行。`monitor` 的固定 23 列及
`monitor_save` 的两个前缀列已登记在机器目录；后者是在相同 monitor schema 前增加
`save_harmonic,save_frequency_hz`。

短路 `fault` 每个独立故障一行，固定列为：

```text
bus,kind,i_f_mag,i_f_ang,voc_mag,voc_ang,v_f_mag,v_f_ang,
i_seq_mag,i_seq_ang,i_residual_mag,i_residual_ang,z_sc_mag,z_sc_ang
```

逐相故障电流、故障前/故障后电压和序量使用 `list<f64>`；残流幅值/相角是标量。电压、
电流和角度分别使用 V、A 和 rad。

短路 `branch_flow` 固定为：

```text
fault_bus,kind,element,bus_from,bus_to,phase_from,phase_to,
v_from_re,v_from_im,v_to_re,v_to_im,
i_from_re,i_from_im,i_from_mag,i_to_re,i_to_im,i_to_mag,
p_from,q_from,p_to,q_to,i_fault_max
```

### Monte Carlo

| 结果 | 固定列 |
|---|---|
| `voltage_statistics` | `bus,phase,v_mean,v_std,v_min,v_max,v_violation_pct,successful_scenarios,non_converged_scenarios` |
| `scenario_voltage` | `scenario,bus,phase,v_mag,violated` |
| `scenario_load_power` | `scenario,load,multiplier,p_total,q_total,converged` |
| `summary` | `requested_scenarios,successful_scenarios,non_converged_scenarios,seed,solver` |

### 恢复重构

- `switches`：`switch,closed`；
- `restored_loads`：`bus,phase,demand_p,demand_q,restored_p,restored_q,fraction`；
- `bus_voltage`：潮流列表 schema；
- `objective`：单行单列 `objective`，单位 W。

### 保护配合

- `operations`：`fault,location,device,kind,shot,curve_mode,phase,source,
  phase_current_amp,ground_current_amp,pickup_time_seconds,trip_time_seconds,
  operating_time_seconds`；
- `selectivity`：`fault,location,primary,backup,shot,phase,primary_source,backup_source,
  primary_time_seconds,backup_time_seconds,margin_seconds,required_margin_seconds,selective,status`；
- `minimum_margins`：`primary,backup,minimum_margin_seconds,required_margin_seconds,
  worst_fault,worst_location,worst_shot,worst_phase,insufficient_margin_count,overtrip_count,
  uncovered_count`；
- `misoperations` 和 `uncovered` 与 `selectivity` 同 schema；
- `action_diagnostics`：`event_id,device,action_family,observed_action,predicted_action,
  observed_time_seconds,predicted_time_seconds,time_error_seconds,tolerance_seconds,status`。

### 故障定位

单张结果固定为：

```text
model_hypothesis,relative_likelihood,line,distance_percent,fault_kind,fault_phases,
fault_resistance_ohm,fault_reactance_ohm,current_match_error,objective,refined,
identifiable,identifiability_condition,distance_stddev_percent,
distance_ci95_low_percent,distance_ci95_high_percent,
fault_resistance_stddev_ohm,fault_reactance_stddev_ohm,weak_parameter_direction,
measurement_count,residual_count,measurement_element,measurement_terminal,
predicted_current_re,predicted_current_im,predicted_voltage_re,predicted_voltage_im
```

### 电机网络动态

- `bus_voltage`：`time_s,bus,phase,v_re,v_im,v_mag,v_angle_deg,source_scale_re,
  source_scale_im`；
- `motor_state`：`time_s,motor,phase,voltage_re,voltage_im,current_re,current_im,
  current_mag,current_angle_deg,slip,speed_pu,relative_speed_rad_s,rotor_angle_deg,p_w,
  q_var,electromagnetic_torque_nm,shaft_power_w,stalled`；
- `event`：`time_s,kind,event_id,device,element,terminal,conductor,action,bus,bus2,percent`。

## 10. 时序和执行计划附加列

QSTS 对每张逐时步结果追加 `time,load_multiplier,generation_multiplier`。执行计划入口再
追加 `epoch_index,solve_event_sequence`；`diagnostics` 和 `events` 使用各自的执行审计
schema。各状态结果在三列时序元数据之前的固定列为：

机器目录的 `qsts_outputs` 为每个公开 QSTS 结果保存上下文完整的 schema；逐时步结果的
`columns` 已包含上述三列，Monitor、EnergyMeter、可靠性、区域、保存快照和负荷持续时间
结果则引用各自真实构造器合同。调用方不得把静态 `result_frames.bus_voltage` 直接当成
QSTS `bus_voltage`，否则会遗漏时间元数据。

| 结果 | 固定列 |
|---|---|
| `storage_state` | `name,soc,energy,p_total,q_total` |
| `storage_controller_state` | `name,mode,state,monitored,target,deadband,requested_p,fleet_p,inhibited,locked` |
| `switch_control_state` | `name,switched_obj,switched_term,switched_conductor,state,normal,locked,condition_active,measurement,pending_action,due_time,phases,phase_states` |
| `fuse_state` | `name,monitored_obj,monitored_term,switched_obj,switched_term,state,blown,current,trip_time,due_time,phases,phase_states` |
| `recloser_state` | `name,monitored_obj,monitored_term,switched_obj,switched_term,switched_conductor,state,locked_out,operation,curve_mode,phase_current,ground_current,trip_source,trip_time,pending_action,due_time,phases,phase_states` |
| `fault_state` | `index,bus,kind,active,cleared,on_time,off_time,temporary,min_amps,current,phases` |

`relay_state` 列较多，完整顺序登记在机器目录的 `result_frames.relay_state`。其中包含一般
电流/电压状态、DOC、Distance、TD21、动作时刻，以及末尾的 `phases,phase_states`。

`execution_trace` 不是逐时步电气结果，不追加 `time/load_multiplier/generation_multiplier`；
它的固定 30 列登记在机器目录的 `result_frames.execution_trace`。其中 `causes,state_before,
state_after,profile_causes,controller_solve_causes` 是 JSON 字符串。执行计划为该表追加
`epoch_index,solve_event_sequence`。

RSDss 当前没有运行期“查询 schema”的函数。`cols(...)` 是选择器，不会返回列名；脚本
生成器必须在执行前依据本参考和机器目录选择列。结果对照时，`bus_phase` 的稳定键为
`lower(bus),phase`，支路表使用元件、terminal/端点和 phase 作为稳定键。
