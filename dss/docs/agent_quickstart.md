# 智能体编写 RSS 仿真脚本快速指南

本文给出从用户需求到可运行 `.rss` 的固定步骤。函数签名和表结构分别以
[DSA 函数参考](dsa_function_reference.md) 和
[DataFrame schema 参考](dataframe_schema_reference.md) 为准。

## 1. 先把需求归入分析类型

| 用户目标 | 首选函数 | 选择条件 |
|---|---|---|
| 单电源、连通、无环配电网静态潮流 | `ds_radial_pf` | 前推回代，无稀疏后端要求 |
| 环网、并联支路、多电源、PV/GFM 或通用静态潮流 | `ds_newton_raphson_pf` | 需要 MUMPS 或 MKL feature |
| Daily/Yearly/Duty/PeakDay/LoadDuration 时序 | `ds_radial_qsts` | 网络满足 Radial 边界 |
| 通用网络时序 | `ds_newton_raphson_qsts` | 需要 MUMPS 或 MKL feature |
| 保留 DSS 单/多 `Solve` 命令流 | `ds_radial_dss_plan` / `ds_newton_raphson_dss_plan` | 输入满足 PSA ExecutionStudy 有界合同 |
| 谐波扫频 | `ds_harmonic_study` | 需要 Spectrum 和 `h_max` |
| 故障/短路 | `ds_fault_study` | 显式构造 `fault` 表 |
| WLS、MNMR、MCE 状态估计 | `ds_wls_se`、`ds_mnmr_se`、`ds_mce_se` | 需要 measurement 表和稀疏后端 |
| Monte Carlo | `ds_monte_carlo` | 需要 spec、multipliers、para |
| 供电恢复重构 | `ds_restore` | 需要显式 switch 和 fault 表 |
| 保护配合审计 | `ds_protection_coordination` | 消费故障和量测样本，不自行求故障电流 |
| 故障定位 | `ds_fault_location` | 需要量测和候选开关/线路表 |
| 完整 RMS/phasor 网络动态 | `ds_network_dynamics` | 29 参数；需要 timeline，可接 Storage、Generator、故障和保护表 |
| 保留受支持的 DSS Dynamic 命令流 | `ds_dss_dynamic_plan` | 本地 `.dss`，固定步长且满足 PSA V12 有界合同 |
| 兼容电机网络动态 | `ds_motor_dynamics` | 28 参数；只返回三种旧结果且拒绝非空 Storage |

不确定网络是否满足 Radial 边界时，先检查电源数量、连通性和环路。已明确存在环网、并联
支路或多个固定电压源时直接选择 Newton。

## 2. 优先复用 DSS 静态模型

标准入口返回固定的 20 张网络表和一张参数表：

```text
[
    bus, line, linecode, load, transformer, voltage_source,
    motor, transformer_winding, transformer_impedance, current_source,
    generator, ibr, switch, shunt, capacitor, reactor, spectrum,
    control_profile, xy_curve, storage, para
] = import_dss("path/to/model.dss");
```

必须保留位置。空对象表也占据自己的槽位。当前 `import_dss` 使用静态导入 profile；
`Solve`、运行时 `Set`、`Export` 等执行命令不能作为静态模型的一部分。

需要保留受支持的单 `Solve` 时，可用 `import_dss_execution` 得到完整 33 项 QSTS bundle
及 `diagnostics/events`；需要直接连续执行多个 `Solve` 时，使用 `ds_*_dss_plan`，不要先走
静态 `import_dss`。

暂态不要走静态 `import_dss` 后再猜测 DSS 命令语义。已有 Dynamic/Dynamics 命令流时使用
`ds_dss_dynamic_plan`，或直接执行 `.dss/.rssbundle`；上游已经提供 20 张网络表、timeline 和
保护表时才使用 `ds_network_dynamics`。三种命令行路径和落盘规则见
[tensoreval 命令行与暂态仿真参考](tensoreval_cli_reference.md)。

## 3. 显式构造分析配置

静态潮流的最小参数表：

```text
pf_para = with_columns(
    alias(exact, line_model),
    alias(bus_phase, result),
    para
);
```

多结果使用逗号分隔字符串，并按同一顺序绑定：

```text
pf_para = with_columns(
    alias("bus_phase,branch_flow,summary", result),
    para
);

[voltage, branch, summary] = ds_radial_pf(
    bus, line, linecode, load, transformer, voltage_source,
    motor, transformer_winding, transformer_impedance, current_source,
    generator, ibr, switch, shunt, capacitor, reactor, spectrum,
    control_profile, xy_curve, storage,
    pf_para
);
```

`result` 的数量、顺序和左值数量必须一致。单个结果可使用 `value = ...` 或
`[value] = ...`；多个结果必须用方括号。

## 4. QSTS 使用完整的 33 参数入口

完整网络的 20 个参数之后依次为：

```text
solution, load_shape, shape_assignment, storage_controller,
switch_control, tcc_curve, fuse, recloser, relay,
monitor, energy_meter, fault, para
```

没有配置的子系统仍需传入一张零行 DataFrame。导入所得的空 `spectrum` 可在示例中作为
通用空表占位，但有真实 Spectrum 时应选择另一张确定为空的表，避免把频谱误当成控制表。

参考 [`../4bus_daily_qsts.rss`](../4bus_daily_qsts.rss)。该案例的曲线在时步起点
`0/3600/7200 s` 采样，Solution 结果在时步结束时刻 `3600/7200/10800 s` 输出。

## 5. 只请求任务所需结果

常用结果选择：

| 需求 | `result` |
|---|---|
| OpenDSS 母线相电压对照 | `bus_phase` |
| 支路电流、功率和损耗 | `branch_flow` |
| 电源端口功率 | `source_flow` |
| 收敛、迭代和总损耗 | `summary` |
| QSTS Monitor | `monitor` 或 `monitor_save` |
| QSTS 电量计 | `energy_meter`、`meter_loss_breakdown` |
| QSTS 跨步设备状态 | `storage_state`、`storage_controller_state`、`switch_control_state`、`fuse_state`、`recloser_state`、`relay_state`、`fault_state` |
| DSS 执行计划证据 | `execution_trace`、`diagnostics`、`events` |
| 谐波母线结果 | `bus_voltage` |
| 故障和故障支路电流 | `fault,branch_flow` |
| 暂态电压和收敛 | `bus_voltage,solver_trace` |
| 暂态故障与保护动作 | `branch_current,event,fault_state,protection_state,device_protection_connection` |
| 暂态 DER/电机/Generator | `pvsystem_state,storage_state,motor_state,generator_state,frt_state,device_state` |

输出变量名不能与随后仍作为裸标识符使用的枚举值同名。例如参数中使用
`alias(bus_phase, result)` 时，把输出命名为 `voltage` 或 `qsts_voltage`，避免形成意外
依赖或循环。

## 6. 暂态输入和输出检查

显式 V12 门面固定为 29 个参数：20 张网络表，随后依次是
`timeline,event,tcc_curve,fuse,recloser,relay,fault,spec,para`。`timeline` 必须非空且
`time_s` 严格递增；其他研究表没有配置时仍传零行 DataFrame。不要交换 `event` 与 `fault`：
前者描述明确时刻的操作，后者描述带 on/off 生命周期的保护故障对象。

所有暂态结果都是带 `time_s,sequence` 的长表；内部故障、保护、控制和 FRT 边界可能在外部
时间轴之间插入行。同一时间的动作前后状态靠 `sequence` 区分。验收至少同时检查
`solver_trace.converged`、`event.applied/reason`、保护终态和 `device_protection_connection`，
不能只看最终母线电压。

已有 DSS Dynamic 入口时优先运行已验证 bundle：

```bash
tensoreval --result-dir /tmp/dynamic-results \
  sparrowzz/dss/rssbundle/Examples/DistanceRelays/DistanceRelayTest.rssbundle
```

完整输入、15 张输出和 fail-closed 条件见
[tensoreval 暂态参考](tensoreval_cli_reference.md#5-暂态仿真的三种入口)。

## 7. 运行并检查

```bash
tensoreval --df path/to/case.rss
tensoreval --df --verbose --result-dir results path/to/case.rss
```

验收至少检查：

1. 进程退出码为 0；
2. 输出变量名和 DataFrame 数量与脚本声明一致；
3. 结果行数覆盖全部预期时点、设备、母线和相；
4. `summary` 显示收敛；
5. 电压单位为 V、电流为 A、功率为 W/var；
6. 与 OpenDSS 对照时按稳定键连接，不依赖行顺序；
7. 比较复数实部/虚部，并记录每类量的容差。

## 8. 生成脚本时的禁止推断

- 不从 OpenDSS 属性名直接猜测 DataFrame 列名；查 schema 参考。
- 不交换 `import_dss` 返回位置或完整网络参数位置。
- 不省略 QSTS 的空子系统参数。
- 不把 `import_dss` 当成 OpenDSS 命令解释器。
- 不假设所有网络都能使用 Radial。
- 不假设角度单位统一；静态 `bus_phase.v_ang` 为 rad，谐波角度为 degree。
- 不使用结果表的当前行顺序作为业务身份。
- 不把测试的 `1e-3` 容差推广为所有网络的通用验收标准。
- 不把 RMS/phasor V12 暂态称为 EMT 或完整 OpenDSS Dynamic Mode。
- 不按 `time_s` 单列去重暂态输出；同刻的不同 `sequence` 表示确定的动作前后边界。
