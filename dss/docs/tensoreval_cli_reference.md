# tensoreval 命令行与暂态仿真参考

本文描述发布给用户的唯一命令行工具 `tensoreval`。命令格式统一写为：

```text
tensoreval [参数] 输入
```

本文负责选择执行模式、准备命令行输入和解释输出。RSS 语法、函数签名和逐列 schema 分别见
[RSS 语言参考](rss_language_reference.md)、[DSA 函数参考](dsa_function_reference.md)和
[DataFrame schema 参考](dataframe_schema_reference.md)。

## 1. 构建能力

配电网分析、DSS 直接执行、DSS→RSS 转换和 `.rssbundle` 运行要求发布版包含 `with_dsa`。
Newton 潮流、Newton QSTS 和状态估计还要求 MUMPS 或 Intel MKL 稀疏后端。没有所需 feature
时工具明确失败，不会改用另一种算法。

`--dir` 和 `--url` 当前不参与 RSDss 执行；本地脚本使用位置参数和 `#include`，远程脚本片段
使用 `#include_url`。`--complex` 是普通 Tensor/VM 模式的兼容参数，当前不改变实数、复数或
RSDss DataFrame 的执行路径。

### 1.1 配电仿真功能

| 能力 | RSDss 入口 | 输入核心 |
|---|---|---|
| DSS/BMOPF 导入 | `import_dss*`、`import_bmopf*` | 本地文件或单 URL source |
| 辐射状/通用静态潮流 | `ds_radial_pf`、`ds_newton_raphson_pf` | 20 张网络表和 para |
| Radial/Newton QSTS | `ds_radial_qsts`、`ds_newton_raphson_qsts` | 网络、solution、曲线、控制、保护、计量和 para |
| DSS 单/多 Solve 执行计划 | `ds_radial_dss_plan*`、`ds_newton_raphson_dss_plan*` | 本地或 URL DSS 执行入口 |
| 谐波与 DSS Harmonic/HarmonicT | `ds_harmonic_study`、`ds_dss_harmonic_plan` | Spectrum、Monitor 和研究参数 |
| 短路与 DSS FaultStudy | `ds_fault_study`、`ds_dss_fault_study` | 网络、故障表或 DSS 生命周期 |
| WLS/MNMR/MCE 状态估计 | `ds_wls_se`、`ds_mnmr_se`、`ds_mce_se` | 网络、量测和参数 |
| Monte Carlo | `ds_monte_carlo` | 网络、场景 spec、multipliers 和参数 |
| 供电恢复重构 | `ds_restore` | 网络、开关、故障和优化参数 |
| 保护配合审计 | `ds_protection_coordination` | 保护设备、故障/电流样本、配合对和现场动作 |
| 故障定位 | `ds_fault_location` | 网络、量测和拓扑/线路假设 |
| 网络暂态 V12 | `ds_network_dynamics`、`ds_dss_dynamic_plan` | 网络/timeline/事件/保护表，或 DSS Dynamic 命令流 |
| DSS 专项研究 | `ds_dss_auto_add`、`ds_dss_relcalc` | 有界 AutoAdd 或 RelCalc 生命周期 |

所有函数的参数数量、固定位置、默认结果和完整结果集合见
[DSA 函数参考](dsa_function_reference.md)。DataFrame 的 `select/filter/join/aggregate/sort`、
列表达式和脚本内数值 Prog 见 [RSDss 函数参考](rsdss_function_reference.md)。

## 2. 输入模式

`tensoreval` 根据参数和输入后缀选择模式。不要在一次命令中混用下表的输入类型。

| 目标 | 命令 | 输入要求 | 结果 |
|---|---|---|---|
| 运行 RSDss DataFrame 脚本 | `tensoreval --df FILE.rss` | 一至多个 UTF-8 RSS 文件 | 输出依赖图最深层的具名 DataFrame |
| 运行普通 Tensor/VM 程序 | `tensoreval FILE.rss` | 一至多个普通 RustScript 文件 | Tensor/标量调试文本 |
| 直接执行 DSS 入口 | `tensoreval FILE.dss` | 恰好一个本地 `.dss` 入口 | 自动分类研究并输出该研究的具名 DataFrame |
| 运行便携 bundle | `tensoreval FILE.rssbundle` | 恰好一个小写 `.rssbundle` 目录 | 校验 manifest 和资产后执行 `main.rss` |
| 只转换 DSS | `tensoreval --dss-to-rss FILE.dss` | 恰好一个本地 `.dss` 入口 | 生成 `.rss` 或 `.rssbundle`，不执行 |

`--dataframe` 与 `--df` 等价。多个 RSS 文件按命令行顺序拼接后解析；局部 `#include` 在解析前
展开。`.dss` 和 `.rssbundle` 必须独占位置参数，不能和其他脚本一起执行。

`--df` 的顶层程序必须只包含以分号结束的具名声明。声明绑定本身参与依赖图和输出选择，禁止
追加裸变量表达式；完整规则和正反例见
[RSDss 顶层程序约束](rss_language_reference.md#11-顶层程序约束must)。

### 2.1 命令行 DataFrame 输入

```text
tensoreval --df --input NAME=FILE [--input NAME2=FILE2] SCRIPT.rss
```

每个 `NAME` 在本次命令中必须唯一且非空，并成为 RSS 求值开始前已有的 DataFrame 变量；
脚本直接引用该 `NAME`，不需要且禁止在脚本末尾追加裸变量表达式。`.xls/.xlsx` 必须只有一个
工作表，`.parquet` 按 Parquet 读取，其他后缀按带表头 CSV 读取；所有行参与 schema 推断。
任一文件读取或解析失败时，在脚本求值前终止整个命令。

模型输入有两种不同场景：独立 RSS 脚本自行读取 DSS 时使用 `import_dss(...)`；宿主通过
`--input NAME=FILE` 注入模型表时，脚本直接引用这些 `NAME`，禁止再次调用 `import_dss`
读取同一模型。每个 `--input` 只产生一张 DataFrame，不会自动展开 DSS 文件。

`--input` 只适用于 `--df`；处理 `.dss` 时仅可与 `--dss-to-rss --mode bundle` 一起使用，
此时输入文件及 schema 被写入 bundle 清单。

## 3. 输出协议

不指定落盘目录时，工具在标准输出中依次打印结果名和供人查看的 DataFrame 文本；表格显示
可能因终端宽度而截断，不能用它做完整数值对照。完整结果使用：

```text
tensoreval --result-dir DIR [--output-format csv|parquet] 输入
```

- `csv` 为缺省格式，写表头；`parquet` 使用 Snappy 压缩；
- `--df` 的结果名称来自最大依赖层 DataFrame 声明的左侧绑定；绑定本身就是输出，禁止在
  脚本末尾追加 `NAME;`；直接 DSS 和 bundle 使用其执行脚本声明的结果名；
- `--result-dir` 使用这些绑定名称，将每个结果写为 `DIR/NAME.csv` 或 `DIR/NAME.parquet`；
- 结果名只能包含字母、数字、`_`、`-`、`.`，且不能是 `.` 或 `..`；
- `--result-dir` 适用于 `--df`、直接 `.dss` 和 `.rssbundle`，不能与 `--dss-to-rss` 同用；
- `--output-format` 必须与 `--result-dir` 同时出现；
- `--verbose` 为 DataFrame、DSS 和 bundle 结果额外打印 schema；普通 VM 模式下打印 VM 统计。

RSDss、直接 DSS、bundle 和转换模式成功退出码为 0；输入、解析、schema、结果选择、求解或
写文件失败返回非零退出码，且不返回可冒充成功结果的部分表。普通 Tensor/VM 模式的 VM 运行期
错误当前只打印 `Failed to run prog`，调用方不能把它的进程退出码当作 DSA 成功门禁。诊断文本
可能写入标准错误、标准输出或日志；配电网自动化调用应同时检查退出码、预期结果名和结果文件。

## 4. DSS→RSS 转换

```text
tensoreval --dss-to-rss [选项] INPUT.dss
```

`--dss2rss`、`--dsstorss` 和 `--dss_to_rss` 是等价别名。常用参数：

| 参数 | 约束与效果 |
|---|---|
| `--mode linked|bundle` | 缺省 `linked`；bundle 冻结 DSS 文件树、外部输入和 manifest |
| `--output PATH` | 缺省为同名 `.rss` 或 `.rssbundle` |
| `--source-root DIR` | DSS 允许访问的依赖根；缺省为入口父目录 |
| `--solver auto|radial|newton-raphson` | 生成脚本引用的静态/QSTS 求解器 |
| `--study snapshot|execution|fault-study|harmonic|auto-add|relcalc|dynamic` | 为只有模型定义、没有研究命令的入口指定研究 |
| `--check` | 完成导入和求解校验，但不写输出 |
| `--force` | 覆盖已有转换输出 |

转换器先解析完整 DSS 命令流再分类。混合 QSTS、Harmonic、AutoAdd、RelCalc 或 Dynamic 生命周期
会失败；模型入口也不会被猜成某种研究，必须显式给出 `--study`。详细诊断码、bundle 清单和
成功条件见 [DSS 到 RSS 严格转换](dss_to_rss.md)。

## 5. 暂态仿真的三种入口

`tensoreval` 当前公开的是有界 RMS/phasor 网络动态 V12，不是 EMT，也不表示完整兼容
OpenDSS Dynamic Mode。三种入口使用同一 PSA V12 结果合同：

| 场景 | 入口 | 何时使用 |
|---|---|---|
| 已有可执行 DSS Dynamic/Dynamics 命令流 | `tensoreval CASE.dss` 或 RSS 中的 `ds_dss_dynamic_plan` | 保留 DSS 时间步、故障和保护配置 |
| 需要便携交付或先审查转换 | `--dss-to-rss --study dynamic` 后运行 `.rss/.rssbundle` | 冻结依赖、先 `--check`、部署到其他目录 |
| 上游已经产生类型化 DataFrame | RSS 中的 `ds_network_dynamics` | 显式控制网络、timeline、事件、保护和结果 |

旧的 `ds_motor_dynamics` 仅保留兼容：它固定 28 个参数，只返回 `bus_voltage,motor_state,event`，
并拒绝非空 Storage。新脚本需要 Storage、PVSystem、Generator、故障、完整保护状态或设备连接
审计时使用 `ds_network_dynamics`。

## 6. DSS Dynamic/Dynamics 输入要求

直接执行：

```text
tensoreval --result-dir results CASE.dss
```

转换并执行 bundle：

```text
tensoreval --dss-to-rss --mode bundle --study dynamic --output CASE.rssbundle CASE.dss
tensoreval --result-dir results CASE.rssbundle
```

如果 DSS 已包含 Dynamic/Dynamics `Solve`，通常省略 `--study dynamic`；该参数主要用于没有研究
命令的模型入口。`--dss-to-rss` 只转换，第二条命令才运行。

DSS 动态入口必须满足：

- 可选 Snapshot 初始化之后只能进入一个有界 Dynamic/Dynamics 生命周期；
- `h/step/stepsize` 必须形成有限、严格为正且固定的外部步长；
- `number`、`hour`、`sec/seconds` 必须形成连续、不能回拨的时钟；多个 Dynamic `Solve` 可累计，
  但不能在中途改变步长；
- `MaxIterations`、`MaxControlIter` 和 `Tolerance` 映射到 PSA 求解参数；
- `ControlMode=Time` 和零时钟 `Time=(0,0)` 可导入；其他控制模式或非零时钟重置失败；
- Fault、TCC_Curve、Fuse、Recloser、受支持 Relay 和只读 `Monitor Mode=32` 按有界合同导入；
- 运行期对象修改、未映射 `Set`、Open/Close/Enable/Disable、DynamicEq/DynOut、VCCS、用户 DLL、
  EMT、逐导体保护动作和未覆盖对象失败关闭。

Distance/TD21 当前只覆盖完整三相端口、整端动作的已验证子集。对象和属性的详细边界见
[OpenDSS 兼容矩阵](opendss_compatibility_matrix.md)。

RSS 函数形式为：

```text
[results...] = ds_dss_dynamic_plan("CASE.dss", "result1,result2");
[results...] = ds_dss_dynamic_plan("CASE.dss", para);
[results...] = ds_dss_dynamic_plan("CASE.dss", "SOURCE_ROOT", para_or_results);
```

第一个参数必须是本地路径。三参数形式只用于限定依赖根。第二或第三参数必须是逗号分隔的结果
字符串，或一张由 `para.result` 选择结果的 DataFrame；省略 `para` 的字符串形式固定采用
`line_model=exact`。未知、重复或空结果名，以及左值数量不匹配都会失败。

## 7. 显式 DataFrame 暂态输入

```text
[results...] = ds_network_dynamics(
    network_1, ..., network_20,
    timeline, event, tcc_curve, fuse, recloser, relay, fault, spec, para
);
```

固定 29 个参数，任何位置都不能省略：

1. 前 20 张表按完整网络顺序传入；`storage` 是第 20 张，可非空；
2. `timeline` 必须非空，必需 `time_s:f64`，可选
   `source_scale_re:f64=1`、`source_scale_im:f64=0`；所有值有限，`time_s` 严格递增；
3. `event`、`tcc_curve`、`fuse`、`recloser`、`relay`、`fault` 和 `spec` 没有配置时仍传零行
   DataFrame；非空时必须满足各自 schema；
4. `event` 用于明确时刻的 fault/topology/protection/storage/availability 操作；`fault` 用于带
   `on_time/off_time` 生命周期的受保护故障对象，不能互换；
5. `spec` 只能为零行或一行，控制电机机械负载、网络校正迭代、容差、松弛和同刻动作轮数；
6. `para` 必须占据最后位置，可为零行；`result` 选择输出，`base_frequency_hz` 若存在必须是
   有限正数，公共潮流参数按 `PfParams` 解析。

逐列类型、事件 `kind` 对应的条件必需列、spec 默认值和完整网络顺序见
[网络动态输入与 V12 结果](dataframe_schema_reference.md#网络动态输入与-v12-结果)。零行表只表示
子系统未配置，不会绕过对非空表的校验。

## 8. V12 暂态输出

默认只返回 `bus_voltage`。直接执行 Dynamic `.dss` 以及转换器生成的 Dynamic RSS/bundle
会请求全部 15 张表：

| 类别 | 结果名 | 主要内容 |
|---|---|---|
| 网络电气量 | `bus_voltage` | 母线逐相复电压、幅值和角度 |
| 网络电气量 | `branch_current` | 元件各端子逐相复电流、幅值和角度 |
| 网络电气量 | `source_flow` | 电源端电压、电流和 P/Q |
| 动态设备 | `pvsystem_state` | PV 电流、P/Q、调制、控制误差和 SafeMode |
| 动态设备 | `storage_state` | Storage 电流、P/Q、命令、能量和 SOC |
| 动态设备 | `motor_state` | 电机滑差、转速、转角、P/Q 和电磁转矩 |
| 动态设备 | `generator_state` | Generator 电气量、内部电势、转角、转速和功率 |
| 求解审计 | `solver_trace` | 预测/校正迭代、残差和收敛标志 |
| 事件与保护 | `event` | 输入、故障、控制和保护事件是否应用及原因 |
| 事件与保护 | `fault_state` | 故障投退、清除、电流和状态变化原因 |
| 事件与保护 | `protection_state` | Fuse/Recloser/Relay 量测、计时、动作和锁定状态 |
| 事件与保护 | `frt_state` | 电压/频率穿越区、连接状态和功率状态 |
| 控制 | `control_state` | 调压器、电容器等普通控制的量测、到期动作和终态 |
| 设备汇总 | `device_state` | Load/PV/Storage/Generator 的 enabled、P/Q 和连接原因 |
| 连接审计 | `device_protection_connection` | owner、端子/导体连接及设备聚合连接状态 |

全部 V12 表均包含 `time_s` 和 `sequence`。除 `event` 的 `sequence` 位于事件身份列之后外，
其余结果以二者开头。引擎会在外部样本之间插入故障切换、保护到期、普通控制和 FRT 边界；同一个
`time_s` 可以有多个动作前后状态，必须用 `(time_s,sequence,设备身份列)` 连接和排序，不能只按
时间去重。

电压、电流、有功、无功和角度单位分别为 V、A、W、var 和 degree。PV/Storage 的 P/Q 正值表示
向网络注入，Storage 充电有功为负；电机 P/Q 正值表示流入电机。未配置的设备族返回带固定 schema
的零行表。15 张表的精确列顺序见
[DataFrame schema 参考](dataframe_schema_reference.md#网络动态输入与-v12-结果)，不能从
OpenDSS Monitor 列名推断。

## 9. 暂态结果选择与验收

建议按问题选择最小结果集合：

| 目标 | 建议结果 |
|---|---|
| 电压和收敛 | `bus_voltage,solver_trace` |
| 故障电流和开断顺序 | `branch_current,event,fault_state,protection_state` |
| Distance/TD21 动作 | `branch_current,event,protection_state,device_protection_connection` |
| PV/Storage/FRT | `pvsystem_state,storage_state,frt_state,device_state` |
| Generator/电机机电响应 | `generator_state,motor_state,bus_voltage,solver_trace` |
| 调压器和电容器动作 | `control_state,event,bus_voltage` |

至少检查：退出码为 0；`solver_trace.converged` 全部为 true；残差有限且满足研究容差；时间和
sequence 非递减；输入事件在 `event` 中有明确的 applied/reason；保护终态同时核对
`protection_state`、`device_state` 和 `device_protection_connection`。与 OpenDSS 比较时排除双方
离散事件边界的约定窗口，再按稳定键比较复数电压和电流。详见
[验证与故障排查](validation_and_troubleshooting.md)。

仓库内可直接执行的暂态保护案例：

```text
tensoreval --result-dir /tmp/dynamic-results \
  sparrowzz/dss/rssbundle/Examples/DistanceRelays/DistanceRelayTest.rssbundle
```

同目录的 `ReverseDistanceRelayTest`、`TD21RelayTest` 和 `ReverseTD21RelayTest` bundle 也已完成
OpenDSS 数值、动作顺序、动作次数、终态和独立目录自执行门禁。
