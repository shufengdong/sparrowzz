# DSA 函数参考

本参考对应 `tensoreval --dataframe` 当前公开的 RSDss DSA 分派。所有函数都
需要 `with_dsa`；Newton 潮流、Newton QSTS 和状态估计还需要 MUMPS 或 Intel MKL 稀疏
后端。

## 阅读规则

- `arity` 是顶层调用的参数总数，包含末尾的 DataFrame 参数，不包含左值数量；
- `network[20]` 始终表示第 1 节的固定顺序，不能省略中间位置；
- 完整形式中，第 1–6 张网络表必须能通过各自解析器，位置 7–20 的零行表表示该设备族
  未配置；
- `para?` 表示整个参数可省略，`para` 表示必须传入一张表，但该表可以为零行；
- `returns` 中的顺序由 `para.result` 决定；DSS plan 的第二参数也可直接使用结果字符串；
  两者都没有时使用该函数的默认结果；
- 入口名按 ASCII 大小写不敏感规则匹配，结果名转为小写后精确匹配；
- 本参考给出函数级契约，所有输入列、类型、默认值和结果列以
  [DataFrame schema 参考](dataframe_schema_reference.md)及机器目录为准。

| 函数组 | arity | 默认结果 | 返回数量 |
|---|---:|---|---|
| `import_dss/import_dss_url/import_bmopf/import_bmopf_url` | 1 | 不适用 | 固定 21 |
| `import_dss_execution/import_dss_execution_url` | 1 | 不适用 | 固定 35 |
| `ds_radial_dss_plan* / ds_newton_raphson_dss_plan*` | 1 或 2 | `bus_voltage` | 由结果字符串或 `para.result` 决定 |
| `ds_dss_dynamic_plan` | 2 或 3 | `bus_voltage` | 由结果字符串或 `para.result` 决定 |
| `ds_radial_pf/ds_newton_raphson_pf` | 21；兼容 6–11 | `bus_voltage` | 由 `para.result` 决定 |
| `ds_radial_qsts/ds_newton_raphson_qsts` | 33 | `bus_voltage` | 由 `para.result` 决定 |
| `ds_harmonic_study` | 22；兼容 8/14 | `bus_voltage` | 由 `para.result` 决定 |
| `ds_fault_study` | 22；兼容 8/13 | `fault` | 由 `para.result` 决定 |
| `ds_wls_se/ds_mnmr_se/ds_mce_se` | 22；兼容 11/12 | 不适用 | 固定 1 |
| `ds_monte_carlo` | 23 | `voltage_statistics` | 由 `para.result` 决定 |
| `ds_restore` | 22；兼容 9 | `switches` | 由 `para.result` 决定 |
| `ds_protection_coordination` | 9 | `operations` | 由 `para.result` 决定 |
| `ds_fault_location` | 24 | 不适用 | 固定 1 |
| `ds_network_dynamics` | 29 | `bus_voltage` | 由 `para.result` 决定 |
| `ds_motor_dynamics` | 28 | `motor_state` | 由 `para.result` 决定 |

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

### `import_dss_execution` / `import_dss_execution_url`

```text
[20 network frames,
 solution, load_shape, shape_assignment, storage_controller,
 switch_control, tcc_curve, fuse, recloser, relay,
 monitor, energy_meter, fault, para,
 diagnostics, events] = import_dss_execution("run.dss");
```

固定一个参数并返回 35 张表。输入必须是 PSA ExecutionStudy 支持的单 `Solve` 命令流；
前 33 张表与 QSTS 的固定参数顺序完全相同，其中第 20 位 `storage` 来自执行研究表。
`diagnostics` 保留成功导入时的非致命诊断，`events` 保留 include 展开后的全局命令顺序。
前者列为 `category,severity,code,target,message`；后者列为 `sequence,source,line,kind,verb,
arguments,target_class,target_name,parameters`，其中 `parameters` 是 JSON 字符串。
URL 形式仍只下载一个 source，不能解析远程目录中的相对 include 文件。

## 3. 静态潮流

### 推荐完整形式

```text
[results...] = ds_radial_pf(network_1, ..., network_20, para);
[results...] = ds_newton_raphson_pf(network_1, ..., network_20, para);
```

固定 21 个参数。默认结果为 `bus_voltage`。`para.result` 支持：

```text
bus_voltage, bus_phase, load_state, generator_state, branch_flow, source_flow,
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
bus_voltage, bus_phase, load_state, generator_state, branch_flow, source_flow,
transformer_winding_flow, conductor_flow, regulator_state,
capacitor_state, reactor_state, ibr_state, gfm_state,
violations, summary, storage_state, storage_controller_state,
switch_control_state, fuse_state, recloser_state, relay_state,
fault_state, monitor, monitor_save, energy_meter, meter_loss_breakdown,
reliability, meter_zone, meter_phase_voltage, meter_save, load_duration,
execution_trace
```

逐时步结果自动追加 `time`、`load_multiplier` 和 `generation_multiplier` 三列。

潮流表、`summary` 以及七张 typed state 表都是逐时步结果。typed state 表直接对应 PSA
当前的 `TimeSeriesStepResult`：

| 结果名 | 内容 |
|---|---|
| `generator_state` | 发电机逐相 P/Q、电压、电流、无功上下限、控制模式及限幅状态 |
| `storage_state` | 储能 SOC、能量及受限后的实际 P/Q |
| `storage_controller_state` | 控制模式、目标、死区、群组请求/实际有功及锁定状态 |
| `switch_control_state` | 被控端子、逐相状态、量测条件和待执行动作 |
| `fuse_state` | 逐相熔断状态、电流、动作时间和到期时刻 |
| `recloser_state` | 操作序号、快/慢曲线、电流、锁定及待执行动作 |
| `relay_state` | Relay 类型对应的电流、电压、方向、距离、TD21 与多次重合闸状态 |
| `fault_state` | 各输入故障的投退、清除、相别及最近故障电流状态 |

保护状态表中的 `phases` 与 `phase_states` 是位置对应的 `list<str>`。没有配置某个
子系统时，相应结果是保留固定列结构的零行表。
显式 33 参数入口在 `fault` 非空时不接受 `execution_trace`；带故障轨迹应使用下面的
DSS execution-plan 入口。

### DSS execution plan

```text
voltage = ds_radial_dss_plan("run.dss");
voltage = ds_radial_dss_plan("run.dss", "bus_phase");
[results...] = ds_radial_dss_plan("run.dss", para);
[results...] = ds_newton_raphson_dss_plan("run.dss", para);
```

URL 形式为 `ds_radial_dss_plan_url` 和 `ds_newton_raphson_dss_plan_url`。入口导入并按顺序
执行一个或多个 `Solve`，在 epoch 之间延续 PSA 已声明的 Storage、控制、保护、计量和
设备状态。第二个参数可省略、传一张 `para` 表，或直接传一个逗号分隔的结果字符串。
省略时默认返回 `bus_voltage`；字符串形式直接选择结果且不改变求解参数。`para.result`
接受上面的全部 QSTS 结果，并额外接受
`diagnostics` 和 `events`。逐 epoch 结果带 `epoch_index`、`solve_event_sequence`；
`execution_trace` 是覆盖全部 `Solve` 的全局轨迹，也支持 fault-aware 执行计划。
计划入口的诊断表也带 epoch 标识；全局事件表把每条事件映射到对应 epoch，最后一个
`Solve` 之后尚未执行的 trailing events 的 epoch 列为空。

### DSS 原生 FaultStudy、Harmonic、AutoAdd、RelCalc 和 Dynamic

这些入口接收本地 `.dss` 路径和参数 DataFrame 或结果选择字符串；bundle 生成脚本可在两者
之间增加 `source_root`。
字符串形式使用 `PfParams` 默认值；DataFrame 形式由 `para.result` 选择结果，并允许设置
通用潮流参数。结果名按逗号顺序与左侧解构位置一一对应；未知名、重复名、错误参数数目
或无法执行的 DSS 生命周期都会失败，不返回部分表。

```text
fault_result = ds_dss_fault_study("run.dss", "fault_result");

[bus_voltage, branch_flow, conductor_flow, bus_sequence, branch_sequence, monitor]
    = ds_dss_harmonic_plan("run.dss", "bus_voltage,branch_flow,conductor_flow,bus_sequence,branch_sequence,monitor");

[ranking, selection, final_voltage, summary]
    = ds_dss_auto_add("run.dss", auto_add_para);

[meter, section, branch, bus]
    = ds_dss_relcalc("run.dss", "meter,section,branch,bus");

[bus_voltage, event, solver_trace, device_state]
    = ds_dss_dynamic_plan("run.dss", "bus_voltage,event,solver_trace,device_state");
```

| 函数 | DSS 生命周期 | 默认结果 | 全部可选结果 |
|---|---|---|---|
| `ds_dss_fault_study` | 有界 `FaultStudy` | `fault_result` | `fault_result` |
| `ds_dss_harmonic_plan` | `Harmonic` / `HarmonicT` | `bus_voltage` | `bus_voltage,branch_flow,conductor_flow,bus_sequence,branch_sequence,monitor,monitor_save` |
| `ds_dss_auto_add` | 有界 `AutoAdd` | `ranking` | `ranking,selection,final_voltage,summary` |
| `ds_dss_relcalc` | Snapshot 后的有界 `RelCalc` | `meter` | `meter,section,branch,bus` |
| `ds_dss_dynamic_plan` | 可选 Snapshot 初始化后的有界 `Dynamic/Dynamics` | `bus_voltage` | `bus_voltage,branch_current,source_flow,pvsystem_state,storage_state,motor_state,event,solver_trace,fault_state,protection_state,frt_state,control_state,device_state,generator_state,device_protection_connection` |

`ds_dss_dynamic_plan(path, para_or_results)` 读取本地入口；三参数形式
`ds_dss_dynamic_plan(path, source_root, para_or_results)` 为 bundle 限定依赖根。第一个参数必须是
本地字符串路径；最后一个参数必须是结果字符串或具名 DataFrame。直接运行
`tensoreval run.dss` 时，自动分类为 Dynamic 后请求上述全部 15 张表；在 RSS 中调用函数时只返回
结果字符串或 `para.result` 指定的表，缺省为 `bus_voltage`。命令行模式、落盘文件和退出码见
[tensoreval 命令行参考](tensoreval_cli_reference.md)。它把继承的
`h/step/stepsize`、`number`、`hour` 和 `sec/seconds` 降为一条严格等间隔 timeline，多个
Dynamic `Solve` 必须使用同一步长且时钟连续。`MaxIterations`、`MaxControlIter` 和
`Tolerance` 进入 PSA 求解参数；省略 `para` 时固定使用 `line_model=exact`。运行期对象修改、
Open/Close/Enable/Disable、未映射 `Set`、DynamicEq/DynOut、VCCS、用户 DLL 和 EMT 内容会
以稳定诊断失败，不能被静默忽略。

Harmonic/HarmonicT 的每张非空结果表都含 `step_index:UInt32,time_s:Float64`；含
`harmonic` 的表还含 `frequency_hz:Float64`。角度单位为度，电压为 V，电流为 A，功率
为 W/var，`thd` 为百分比。各结果主体列如下：

| 结果 | 主体列与顺序 |
|---|---|
| `bus_voltage` | `bus,phase,harmonic,voltage_mag,voltage_angle,thd` |
| `branch_flow` | `kind,element,terminal,bus,phase,harmonic,voltage_re,voltage_im,voltage_mag,voltage_angle,current_re,current_im,current_mag,current_angle,p,q` |
| `conductor_flow` | 与潮流 `conductor_flow` 相同，另含 `harmonic,frequency_hz`；内部屏蔽导体的端电压可为空 |
| `bus_sequence` | `bus,harmonic,sequence,voltage_re,voltage_im,voltage_mag,voltage_angle`，`sequence` 为 `zero/positive/negative` |
| `branch_sequence` | `kind,element,terminal,bus,harmonic,sequence` 加与 `branch_flow` 相同的数值列；序功率为 `3·Vseq·conj(Iseq)` |
| `monitor` | `monitor,frequency_hz,harmonic,mode,element,terminal,phase,channel,value`；未配置 Monitor 时返回固定 schema 零行表 |

AutoAdd 只接受 PSA 已限定的单次、径向、固定额定值候选搜索。结果按 `score` 升序，分数
相同按源候选顺序排列：

| 结果 | 列与语义 |
|---|---|
| `ranking` | `rank,candidate,kind,bus,converged,p_loss_w,voltage_penalty,unserved_power_w,score,improvement,selected` |
| `selection` | `selected_bus:String,added_device:String`；选中时一行，否则固定 schema 零行 |
| `final_voltage` | `bus,v_mag,v_ang`；V 和弧度；为加入所选设备后的径向潮流，未选中时为原网络 |
| `summary` | 单行 `baseline_score,selected,final_score`；未选中时 `final_score=baseline_score` |

RelCalc 只接受 PSA 已限定的互不重叠径向 EnergyMeter 区域、显式客户数据和受支持保护
分段。`meter` 每表计一行；`section` 按表计及源顺序分段；`branch` 按区域径向顺序；
`bus` 按表计和区域顺序。`*_h` 为小时：

| 结果 | 列与顺序 |
|---|---|
| `meter` | `meter,restore,total_customers,saifi,saifi_kw,saidi_h,caidi_h,customer_interruptions` |
| `section` | `meter,section,ocp_device_type,customers,branches,total_customers,sequence_index,average_repair_time_h,fault_rate_x_repair_h,sum_branch_fault_rates` |
| `branch` | `meter,branch_class,branch,from_terminal,section,lambda,accumulated_lambda,repair_hours,customers,total_customers` |
| `bus` | `meter,bus,section,interruptions,interruption_duration_h,total_customers,customer_duration_h` |

这三个入口只读取本地文件，不提供 URL 变体。它们不会把 Harmonic、AutoAdd 或 RelCalc
与普通 QSTS 自动拆分或串联；混合生命周期应由上层显式编排。

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

这里的 `measurement` 是状态估计量测表，必需列和按 `type` 变化的位置列见
[状态估计 measurement](dataframe_schema_reference.md#状态估计-measurement)。它不是第 11 节
故障定位使用的相量列表表。返回列固定为 `bus,v_mag,v_ang`。

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

`spec` 必须恰好一行；四种 mode 对 `multipliers` 的空表/列要求不同，详见
[Monte Carlo spec 与 multipliers](dataframe_schema_reference.md#monte-carlo-spec-与-multipliers)。
四张结果的固定列见 [Monte Carlo 结果 schema](dataframe_schema_reference.md#monte-carlo)。

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
四张结果的固定列见[恢复重构结果 schema](dataframe_schema_reference.md#恢复重构)。

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

`fault_case/current_sample/pair/observed_action/para` 的列、引用关系和默认值见
[保护配合输入](dataframe_schema_reference.md#保护配合输入)；六张结果的固定列见
[保护配合结果 schema](dataframe_schema_reference.md#保护配合)。零行 `observed_action` 表示没有
现场动作证据；零行 `pair` 允许只计算设备动作表。

## 11. 故障定位

```text
result = ds_fault_location(
    network_1, ..., network_20,
    measurement, hypothesis_switch, hypothesis_line, para
);
```

固定 24 个参数，返回单张结果表。当前不接受非空独立 `storage` 表。候选表可为空，表示
不启用对应候选类型。

这里的 `measurement` 使用逐相复电流/电压列表，和状态估计量测表完全不同。精确列、协方差
顺序、假设表约束及输出列见[故障定位输入](dataframe_schema_reference.md#故障定位输入)与
[故障定位结果](dataframe_schema_reference.md#故障定位)。

## 12. 网络动态与兼容入口

选择规则如下：已有 DSS Dynamic/Dynamics 命令流时用 `ds_dss_dynamic_plan`；已经有类型化网络、
timeline、事件和保护表时用 `ds_network_dynamics`；`ds_motor_dynamics` 只用于兼容旧脚本。三个
入口均由 `tensoreval --df` 执行，直接 `.dss/.rssbundle` 的命令行行为见
[tensoreval 暂态参考](tensoreval_cli_reference.md#5-暂态仿真的三种入口)。

完整 V12 DataFrame 门面为：

```text
[results...] = ds_network_dynamics(
    network_1, ..., network_20,
    timeline, event, tcc_curve, fuse, recloser, relay, fault, spec, para
);
```

固定 29 个参数。`timeline` 必需且不能是零行；`event`、四张保护表、`fault`、`spec`
和 `para` 可使用对应 schema 的零行表。第 20 位 `storage` 可非空。`event` 描述明确时刻的操作，
`fault` 描述带投退生命周期的保护故障对象，两者位置和 schema 不可交换。默认返回
`bus_voltage`，完整结果集合与 `ds_dss_dynamic_plan` 相同，统一采用 PSA V12 的
`time_s,sequence,...` 长表契约。内部事件边界可能在外部 timeline 之间产生额外时点；同刻多行
用 `sequence` 确定动作前后顺序。输入列、事件种类、默认值和 15 张结果表见
[网络动态输入与 V12 结果](dataframe_schema_reference.md#网络动态输入与-v12-结果)。

原有电机兼容入口继续可用：

```text
[results...] = ds_motor_dynamics(
    network_1, ..., network_20,
    timeline, event, tcc_curve, fuse, recloser, relay, spec, para
);
```

固定 28 个参数，当前不接受非空独立 `storage` 表。默认结果为 `motor_state`；支持
`bus_voltage, motor_state, event`。新脚本需要 Storage、Generator、PV、保护连接审计或
完整动态结果时应使用 `ds_network_dynamics`。

`timeline/event/spec` 的条件列和默认值见
[电机网络动态输入](dataframe_schema_reference.md#电机网络动态)。三张结果的固定列见
[电机网络动态结果](dataframe_schema_reference.md#电机网络动态)。`event` 输入可为空；
`timeline` 必须至少能形成有效且严格递增的求解时间序列。

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

## 14. 空表、错误和后端边界

- 固定位置必须始终传入 DataFrame 变量。可选设备没有数据时传零行表，不能删除参数；
- 解析失败、未知结果、返回数量不匹配、求解失败和不可用后端都通过 RSDss
  `OperationFailed` 或更具体的表达式错误返回，`tensoreval --df` 以非零状态退出；
- 完整网络中的非空可选表会被实际解析。若某分析明确拒绝该设备族，必须把对应位置改为
  零行表；不能传任意一行占位数据；
- Newton、状态估计和恢复重构在没有相应稀疏/优化后端时不会自动退化为 Radial；
- URL 导入遵守下载大小和超时边界，且 DSS URL 入口不递归获取远程相对 include；
- `import_dss*` 和 `import_dss_execution*` 的返回数量固定，左值必须完整绑定；分析函数的
  多结果左值数量必须与 `para.result` 解析后的名称数量完全一致。
