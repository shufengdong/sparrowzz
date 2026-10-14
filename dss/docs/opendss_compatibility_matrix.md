# OpenDSS 兼容矩阵

本矩阵描述“DSS 文本 → PSA DataFrame → `ds_*` 分析”的当前公开路径。它不表示完整的
OpenDSS 语言兼容。状态含义如下：

- **直接**：`import_dss` 的 `StaticPowerFlow` profile 能生成对应网络表；
- **有界**：只支持代码中明确校验和转换的属性/模式，超出范围会 fail-closed；
- **显式表**：RSS/调用方必须构造额外 DataFrame；
- **未公开**：PSA 内部可能已有结构，但 RSDss 当前没有对应导入或执行入口。

## 1. 对象族

| OpenDSS 对象 | RSS/PSA 表 | 状态 | 当前边界 |
|---|---|---|---|
| `Circuit` / 隐式 `Vsource.source` | `voltage_source`, `para` | 直接/有界 | 基频和受支持的源参数会转换；不可表示的源阻抗或属性报错 |
| `Vsource` | `voltage_source` | 有界 | 电压、相角、连接和已建模的源阻抗语义；不能假定与 OpenDSS 所有短路容量模型等价 |
| `Isource` | `current_source` | 有界 | 支持幅值、相角、频率、scan type 和 spectrum 的已声明子集 |
| `Bus` | `bus` | 直接 | 从设备连接生成端子和接地信息 |
| `LineCode` | `linecode` | 直接/有界 | 相矩阵、单位和额定值；无法解析的几何/引用 fail-closed |
| `Line` | `line` | 直接 | 显式线路、电缆参数和端子映射 |
| `Line switch=yes` | `switch` | 有界 | 导入器识别开关线路并恢复为显式开关身份 |
| `Transformer` | `transformer`, `transformer_winding`, `transformer_impedance` | 有界 | 两绕组和受支持的多绕组约束；绕组关系必须可无损表达 |
| `Load` | `load` | 有界 | 支持已实现的负荷模型、电压边界、曲线和 spectrum 绑定 |
| `Generator` | `generator` | 有界 | 已实现 operating model、Model 3 和 GenDispatcher 子集；组合限制会显式报错 |
| `PVSystem` | `ibr`, `control_profile`, `xy_curve` | 有界 | DC operating point、曲线和 InvControl/ExpControl 的已声明子集 |
| `Storage` | `storage` | 显式表 | PSA/QSTS 有储能契约；RSDss `import_dss` 的静态 21 返回值中该表当前固定为空 |
| `IndMach012` | `motor` | 有界 | 稳态和动态所需的已支持参数 |
| `Capacitor` | `capacitor` | 有界 | 固定/分级状态和已支持控制字段 |
| `Reactor` | `reactor`, `xy_curve` | 有界 | 包括按频率曲线采样的已支持形式 |
| `Spectrum` | `spectrum` | 直接/有界 | `h`, `mag_pct`, `angle_deg` 及设备引用 |
| `XYCurve` | `xy_curve` | 直接 | 每曲线多行点表 |
| `RegControl`, `CapControl` | 设备表、`control_profile` | 有界 | 仅运行时已有等价状态和控制律的组合 |
| `InvControl`, `ExpControl` | `control_profile`, `ibr` 扩展列 | 有界 | Volt-Var、Volt-Watt、DRC、AVR、Watt-PF/Watt-Var 等受控子集 |
| `StorageController` | QSTS `storage_controller` | 显式表/有界 | PSA 执行型导入可生成；RSDss 静态导入不返回该表 |
| `GenDispatcher` | `generator` 扩展列 | 有界 | 监视线路、成员和功率约束必须能保持身份及顺序 |
| `SwtControl` | QSTS `switch_control` | 显式表/有界 | 支持动作、延时、锁定、复位和稳定源顺序 |
| `TCC_Curve` | QSTS `tcc_curve` | 显式表/有界 | `npts/c_array/t_array` 必须一致 |
| `Fuse` | QSTS `fuse` | 显式表/有界 | 曲线、额定电流、开断状态和 reset |
| `Recloser` | QSTS `recloser` | 显式表/有界 | fast/delayed 曲线和 multi-shot 延续状态 |
| `Relay` | QSTS `relay` | 显式表/有界 | Current、Voltage、ReversePower、46、47、Generic、DOC、Distance、TD21 按各自 schema 校验 |
| `Monitor` | QSTS/harmonic `monitor` | 显式表/有界 | 目标类型必须与 mode 匹配 |
| `EnergyMeter` | QSTS `energy_meter` | 显式表/有界 | 区域、累计量和可靠性输出使用 PSA 的计量契约 |
| `Fault` | `fault` | 显式表/有界 | 静态短路或带起止时间的 QSTS 故障 |

“显式表”不表示该对象无法由 PSA 的 `ExecutionStudy` 导入器降低；它表示当前
`tensoreval` 的 `import_dss()` 只调用 `StaticPowerFlow` 并返回固定 21 项，因此 RSS 调用
QSTS 时必须显式传入第 21–32 位的研究表。

## 2. DSS 命令流

| DSS 语义 | 静态 `import_dss` | PSA 执行型导入 | RSS 等价方式 |
|---|---|---|---|
| `New` | 支持已知对象/属性 | 支持 | 保留在 `.dss` |
| `Edit` | 支持最终静态状态 | 支持 Solve 前有序变更 | 保留在 `.dss` |
| `More`, `~`, `m` | 支持当前活动对象 | 支持且记录顺序 | 保留在 `.dss` |
| 属性引用赋值 | 支持可解析形式 | 支持且记录顺序 | 保留在 `.dss` |
| `Redirect`, `Compile` | 本地入口支持相对 include 树 | 支持保留的 include | 使用本地 `import_dss` |
| `Set DefaultBaseFrequency` | 支持 | 支持 | 导入为 `para.base_frequency_hz` |
| 其他运行时 `Set` | fail-closed | 在执行契约内有界支持 | 在 `solution`/`para` 中显式设置 |
| `Solve` | fail-closed | 单 Solve profile 和多 Solve execution plan 均有 PSA API | `ds_radial_qsts` 等；RSDss 尚未公开 execution-plan adapter |
| `Open`, `Close` | 静态最终状态必须可表示 | 显式开关的整设备动作及 epoch mutation 有界支持 | 修改显式开关表或使用 QSTS 控制表 |
| `Enable`, `Disable` | 已支持对象的最终静态状态必须可表示 | 当前 execution profile 明确为 unsupported，遇到即 fail-closed | 在 RSS 中显式构造所需设备表 |
| `CalcVoltageBases`, `Export`, `Show` | fail-closed | 不作为网络 DataFrame 语义 | 求解后用 RSDss 选择和导出结果 |
| 未知命令/属性/对象 | fail-closed | fail-closed | 查看稳定 diagnostic code 后改写或补充 adapter |

静态导入器会把留存在上游 typed model 中的运行命令视为错误，不会静默跳过。`Clear` 可作为
仓库已验证模型的入口重置语句使用；它不产生一张网络表，也不能用来表达 RSS 运行期间的
状态重置。

## 3. 分析模式

| OpenDSS 工作 | RSS 函数 | 状态 | 说明 |
|---|---|---|---|
| Snapshot 潮流 | `ds_radial_pf` | 直接 | 适合辐射状/弱网状配电网，支持完整 20 表 |
| Snapshot Newton 潮流 | `ds_newton_raphson_pf` | 直接 | 需要 MUMPS 或 MKL feature |
| Daily/Yearly/Duty/PeakDay/LD1/LD2 | `ds_radial_qsts`, `ds_newton_raphson_qsts` | 显式表 | 固定 33 参数；时间、曲线、控制和计量独立成表 |
| Harmonic | `ds_harmonic_study` | 直接/有界 | 频谱和 `h_max` 显式；大型案例用 release |
| FaultStudy / 单故障求解 | `ds_fault_study` | 显式表 | 故障位置、类型和阻抗放在 `fault` |
| 状态估计 | `ds_wls_se`, `ds_mnmr_se`, `ds_mce_se` | 显式表 | 需要 measurement；需要稀疏后端 |
| Monte Carlo | `ds_monte_carlo` | 显式表 | 随机或 replay；当前拒绝非空独立 storage |
| 恢复重构 | `ds_restore` | 显式表 | 需要 switch 与 fault；完整形式对未消费对象 fail-closed |
| 保护配合审计 | `ds_protection_coordination` | 显式表 | 消费故障电流和动作证据，不执行网络故障求解 |
| 故障定位 | `ds_fault_location` | 显式表 | measurement 与候选表；当前拒绝非空 storage |
| 电机网络动态 | `ds_motor_dynamics` | 显式表 | timeline/event/保护/spec；当前拒绝非空 storage |
| 多个 `Solve` 连续命令流 | 无 RSDss 公开函数 | 未公开 | PSA 已有 execution plan 和 continuation state adapter，尚未接到 `tensoreval` |

QSTS 可直接选择 PSA 当前的 typed continuation state：`storage_state`,
`storage_controller_state`, `switch_control_state`, `fuse_state`, `recloser_state`,
`relay_state`, `fault_state`。这些结果用于观察一个 `ds_*_qsts` 调用内部的跨时点状态；
它们不等同于尚未公开的多 `Solve` execution-plan adapter。

## 4. 数值语义差异

兼容对象并不自动保证逐位相同。对照前至少统一：

1. 线路模型：不平衡网络优先 `line_model=exact`；
2. 电源模型：短路容量产生的内部阻抗与理想源假设；
3. 负荷模型及 `Vminpu/Vmaxpu` 外行为；
4. 变压器 tap、相移、绕组方向和额定基准；
5. 开关阻抗、控制器模式和初始状态；
6. 单位和角度：详细功率为 W/var，PF 相角通常为 rad，谐波 `voltage_angle` 为 degree；
7. 方向：按稳定键连接结果，再按端口定义处理电流/功率符号。

兼容性判断应以导入诊断和数值对照为准。对象名相同、脚本成功执行，只能证明 schema 和
拓扑被接受，不能证明 OpenDSS 的全部内部算法已被复现。
