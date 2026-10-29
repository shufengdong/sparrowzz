# 仿真配方

配方以当前 `tensoreval --df` 和完整网络参数为准。先复制最接近需求的已验证案例，再替换
模型路径、研究表和结果选择。完整位置参数见 [DSA 函数参考](dsa_function_reference.md)，
列契约见 [DataFrame schema 参考](dataframe_schema_reference.md)。

## 1. 通用静态潮流

```text
[
    bus, line, linecode, load, transformer, voltage_source,
    motor, transformer_winding, transformer_impedance, current_source,
    generator, ibr, switch, shunt, capacitor, reactor, spectrum,
    control_profile, xy_curve, storage, para
] = import_dss("case/model.dss");

pf_para = with_columns(
    alias(exact, line_model),
    alias("bus_phase,branch_flow,summary", result),
    para
);

[voltage, branch, summary] = ds_radial_pf(
    bus, line, linecode, load, transformer, voltage_source,
    motor, transformer_winding, transformer_impedance, current_source,
    generator, ibr, switch, shunt, capacitor, reactor, spectrum,
    control_profile, xy_curve, storage, pf_para
);
```

使用 `bus_phase` 做 OpenDSS 对照；使用 `branch_flow` 查线路、变压器功率与损耗；只关心
收敛和系统总量时请求 `summary`。可运行版本见
[`../ieee13_power_flow.rss`](../ieee13_power_flow.rss)、
[`../ieee34_power_flow.rss`](../ieee34_power_flow.rss) 和
[`../ieee123_power_flow.rss`](../ieee123_power_flow.rss)。

## 2. Newton 潮流

保持上面的 21 个参数，只替换函数名：

```text
[voltage, branch, summary] = ds_newton_raphson_pf(
    bus, line, linecode, load, transformer, voltage_source,
    motor, transformer_winding, transformer_impedance, current_source,
    generator, ibr, switch, shunt, capacitor, reactor, spectrum,
    control_profile, xy_curve, storage, pf_para
);
```

构建时启用 `with_dsa_mumps` 或 `with_dsa_mkl`。输入表和结果契约与 radial 版本相同，
因此同一份 RSS 数据准备可以复用。两种算法的迭代路径和收敛域不同，不应要求迭代次数完全
一致。

## 3. Daily QSTS

最小 QSTS 需要 `solution`、`load_shape` 和 `shape_assignment`；没有配置的其余研究表可用
任意已有零行表占位。以下模式取自可执行的
[`../4bus_daily_qsts.rss`](../4bus_daily_qsts.rss)：

```text
solution = select(
    alias("DAILY", mode),
    alias(3.0, number),
    alias(3600.0, step_size_s),
    alias("OFF", control_mode),
    para
);

s0 = select(alias("daily_load", name), alias(0.0, time_s),
            alias(0.5, p_multiplier), alias(0.5, q_multiplier), para);
s1 = select(alias("daily_load", name), alias(3600.0, time_s),
            alias(1.0, p_multiplier), alias(1.0, q_multiplier), para);
s2 = select(alias("daily_load", name), alias(7200.0, time_s),
            alias(1.5, p_multiplier), alias(1.5, q_multiplier), para);
load_shape = concat("vertical", s0, s1, s2);

shape_assignment = select(
    alias("LOAD", device_kind),
    alias("load1", device),
    alias("daily_load", daily),
    para
);

qsts_para = with_columns(alias(exact, line_model), alias(bus_phase, result), para);

[voltage] = ds_radial_qsts(
    bus, line, linecode, load, transformer, voltage_source,
    motor, transformer_winding, transformer_impedance, current_source,
    generator, ibr, switch, shunt, capacitor, reactor, spectrum,
    control_profile, xy_curve, storage,
    solution, load_shape, shape_assignment,
    spectrum, spectrum, spectrum, spectrum, spectrum, spectrum,
    spectrum, spectrum, spectrum, qsts_para
);
```

末尾九个 `spectrum` 都是零行占位，依次表示 `storage_controller, switch_control,
tcc_curve, fuse, recloser, relay, monitor, energy_meter, fault`。只在对应子系统缺省时复用空表；
一旦有配置，应把准确表放回其固定位置。QSTS 输出自动带 `time`, `load_multiplier`,
`generation_multiplier`。

将一条曲线绑定到模型中的所有负荷时，可以由 `load.name` 生成 assignment，而无需手工列出
对象名。完整案例见 [`../ieee13_daily_qsts.rss`](../ieee13_daily_qsts.rss)：

```text
shape_assignment = select(
    alias("LOAD", device_kind),
    alias(col(name), device),
    alias("daily_load", daily),
    load
);
```

## 4. 谐波

模型中没有可直接复用的频谱时，可以在 RSS 中建立长表，再聚合为列表：

```text
h1 = select(alias("src", name), alias(1, h), alias(100, mag_pct), alias(0, angle_deg), para);
h3 = select(alias("src", name), alias(3, h), alias(5, mag_pct), alias(30, angle_deg), para);
h5 = select(alias("src", name), alias(5, h), alias(3, mag_pct), alias(-20, angle_deg), para);
rows = concat("vertical", h1, h3, h5);
spectrum = aggregate(group_by(col(name)), col(h), col(mag_pct), col(angle_deg), rows);

voltage_source = with_columns(alias("src", spectrum), voltage_source);
harmonic_para = with_columns(
    alias(5, h_max),
    alias(exact, line_model),
    alias(bus_voltage, result),
    para
);

[harmonic_voltage] = ds_harmonic_study(
    bus, line, linecode, load, transformer, voltage_source,
    motor, transformer_winding, transformer_impedance, current_source,
    generator, ibr, switch, shunt, capacitor, reactor, spectrum,
    control_profile, xy_curve, storage,
    storage, harmonic_para
);
```

倒数第二个 `storage` 是空 `monitor` 占位。完整例子见
[`../ieee13_harmonic.rss`](../ieee13_harmonic.rss) 和
[`../ieee34_harmonic.rss`](../ieee34_harmonic.rss)。整网谐波应使用 release 构建。

## 5. 单相接地短路

从网络表取真实端子映射，避免硬编码不存在的相：

```text
selected = filter(
    cols(bus_to, terminal_map_to),
    cast(col(bus_to), i64) == 652,
    line
);

fault = select(
    alias("652", bus),
    alias(head(col(terminal_map_to), 1), terminal_map),
    alias(slg, kind),
    alias(0.1, zf),
    selected
);

fault_para = with_columns(alias(exact, line_model), alias("fault,branch_flow", result), para);
[fault_result, branch] = ds_fault_study(
    bus, line, linecode, load, transformer, voltage_source,
    motor, transformer_winding, transformer_impedance, current_source,
    generator, ibr, switch, shunt, capacitor, reactor, spectrum,
    control_profile, xy_curve, storage, fault, fault_para
);
```

可运行版本见 [`../ieee13_short_circuit.rss`](../ieee13_short_circuit.rss) 和
[`../ieee34_short_circuit.rss`](../ieee34_short_circuit.rss)。两相、两相接地和三相故障只
能在端子映射与 `kind` 一致时改写。

## 6. 储能与 StorageController QSTS

静态 `import_dss` 返回的第 20 位 storage 为空。储能研究的步骤是：

1. 从 CSV/数据库/上游 DataFrame 获得满足 storage schema 的表，或在 RSS 中构造；
2. 用该表替换完整网络第 20 位；
3. 构造 `storage_controller`，保持 `element_list` 和 `weights` 同长；
4. 在 QSTS 的第 24 位传入 controller；
5. 请求 `load_state`, `branch_flow`, `summary` 或相关状态结果，检查 SOC 与功率限制。

StorageController 不能靠修改 `shape_assignment` 模拟。控制器会读取监视支路、目标带宽、
reserve 和 dispatch factor，并与储能状态共同决定实际功率。

## 7. 开关控制、保护与计量 QSTS

33 参数中的研究表必须保持以下相对顺序：

```text
..., solution, load_shape, shape_assignment,
storage_controller, switch_control,
tcc_curve, fuse, recloser, relay,
monitor, energy_meter, fault, para
```

带保护的研究建议同时请求：

```text
result = "bus_phase,branch_flow,monitor,energy_meter,reliability,summary"
```

保护曲线先独立检查 `npts/c_array/t_array`，再检查设备引用和 terminal，最后加入故障的
`on_time/off_time`。Recloser 的 `shots` 和 `reclose_intervals` 共同决定多次动作；其计数器
在连续 QSTS 时步间延续。

## 8. Monte Carlo replay

确定性回放用于复现实验和缩小随机故障：

```text
spec = select(
    alias(global_replay, mode),
    alias(radial, solver),
    para
);

mc_para = with_columns(alias("voltage_statistics,scenario_voltage,summary", result), para);
[stats, scenarios, summary] = ds_monte_carlo(
    bus, line, linecode, load, transformer, voltage_source,
    motor, transformer_winding, transformer_impedance, current_source,
    generator, ibr, switch, shunt, capacitor, reactor, spectrum,
    control_profile, xy_curve, storage,
    spec, multipliers, mc_para
);
```

`multipliers` 的精确 schema 取决于 `m1_replay` 或 `global_replay`。先从 PSA 对应测试复制
一组最小表，不要把 QSTS `load_shape` 直接当作 replay 输入。当前此入口拒绝非空独立
storage。

## 9. 网络暂态 V12

已有受支持的 OpenDSS Dynamic/Dynamics 入口时，先用严格转换检查，随后生成并运行 bundle：

```bash
tensoreval --dss-to-rss --check path/to/case.dss
tensoreval --dss-to-rss --mode bundle --output path/to/case.rssbundle path/to/case.dss
tensoreval --result-dir path/to/results path/to/case.rssbundle
```

只有模型定义、没有研究命令时，在前两条转换命令中增加 `--study dynamic`。`--check` 不写
输出；第三条命令运行后，目录中每个终端结果对应一个 CSV。转换器生成的 Dynamic bundle 默认
请求全部 15 张 V12 结果。

显式 DataFrame 入口固定写成：

```text
dynamic_para = with_columns(
    alias("bus_voltage,solver_trace,event,protection_state,device_protection_connection", result),
    para
);

[voltage, trace, events, protection, connection] = ds_network_dynamics(
    bus, line, linecode, load, transformer, voltage_source,
    motor, transformer_winding, transformer_impedance, current_source,
    generator, ibr, switch, shunt, capacitor, reactor, spectrum,
    control_profile, xy_curve, storage,
    timeline, event, tcc_curve, fuse, recloser, relay, fault, spec, dynamic_para
);
```

前 20 张网络表和后 9 张研究表都不能省略。`timeline.time_s` 必须有限并严格递增；没有配置的
`event/tcc_curve/fuse/recloser/relay/fault/spec` 传零行 DataFrame。故障或保护动作时同一
`time_s` 可能输出多个 `sequence`，结果连接键必须包含二者和设备身份列。

常用选择：电气稳定性用 `bus_voltage,branch_current,solver_trace`；保护动作加入
`event,fault_state,protection_state,device_protection_connection`；DER 和电机响应加入
`pvsystem_state,storage_state,motor_state,generator_state,frt_state,device_state`。

可运行案例：

```bash
tensoreval --result-dir /tmp/distance-dynamic \
  sparrowzz/dss/rssbundle/Examples/DistanceRelays/DistanceRelayTest.rssbundle
```

该案例是 120 个 10 ms 外部时点的 Distance Relay 动态研究；TD21 案例使用 1,200 个 1 ms
外部时点。两者均按 OpenDSS 数值、事件顺序、动作次数和终态验证。完整输入、输出、单位和失败
边界见 [tensoreval 命令行与暂态仿真参考](tensoreval_cli_reference.md)。

## 10. 外部数据与结果后处理

通过 `#include` 或 `#include_url` 读取脚本文本，通过 RSDss 的输入函数读取数据，再使用
`select/filter/join/aggregate/sort` 处理。组合多张表时显式写方向：

```text
all_rows = concat("vertical", first, second);
joined = join(voltage, limits, bus, bus, "left");
```

裸标识符在表达式位置通常是字符串字面量；要引用列使用 `col(name)`，要一次选多列使用
`cols(a, b)`。多返回函数必须用 `[a, b]` 解构，返回数量必须与 `para.result` 匹配。

## 11. 运行命令

```bash
tensoreval --df path/to/case.rss
tensoreval --df --result-dir path/to/results path/to/case.rss
tensoreval --result-dir path/to/results path/to/case.rssbundle
```

Newton 和状态估计需要安装包含 MUMPS 或 MKL 后端的版本。谐波和大模型使用发布版
`tensoreval`。
