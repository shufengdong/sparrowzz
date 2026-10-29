# tensoreval RSS 文档索引

本目录描述 `tensoreval --dataframe` 执行的 RustScriptSimulation（RSS）脚本，以及
`with_dsa` 提供的配电系统分析函数。文档以当前仓库生产代码为准，目标是让用户和
代码智能体能够从仿真需求生成可运行、可检查的 `.rss` 脚本。

## 推荐阅读顺序

1. [tensoreval 命令行与暂态仿真参考](tensoreval_cli_reference.md)：选择执行模式、准备输入、落盘输出并运行 V12 暂态。
2. [智能体快速指南](agent_quickstart.md)：从需求选择求解器、输入和输出。
3. [RSS 语言参考](rss_language_reference.md)：语法、求值图和多结果规则。
4. [RSDss 函数参考](rsdss_function_reference.md)：DataFrame 变换函数。
5. [DSA 函数参考](dsa_function_reference.md)：`import_*` 与全部 `ds_*` 入口。
6. [DataFrame schema 参考](dataframe_schema_reference.md)：输入表的列、类型和单位。
7. [OpenDSS 兼容矩阵](opendss_compatibility_matrix.md)：对象、命令和分析模式的对应关系。
8. [仿真配方](simulation_recipes.md)：按任务组织的可执行脚本模板。
9. [验证与故障排查](validation_and_troubleshooting.md)：结果核验、容差和错误处理。
10. [从 OpenDSS 迁移到 RSS](opendss_to_rss.md)：完整迁移流程和数值对照方法。
11. [DSS 到 RSS 严格转换](dss_to_rss.md)：`tensoreval --dss-to-rss`、bundle、成功条件和不支持诊断。

机器可读合同的格式见 [`catalog/README.md`](catalog/README.md)，由三个互补目录组成：

- [`catalog/functions.json`](catalog/functions.json) 固定函数分派名称、规范 arity、参数组和结果名；
- [`catalog/dataframes.json`](catalog/dataframes.json) 固定各输入、Struct 参数卡和输出 DataFrame 的完整执行 schema、dtype grammar 与继承规则；
- [`catalog/study_facades.json`](catalog/study_facades.json) 是 DataFrame 研究门面的权威机器合同，
  把每个函数的每个位置参数映射到精确 `schemaRef`，并用 `inputKind=model|data|struct`
  声明输入类别。

生成器必须联合读取三者。禁止根据函数名、参数名或对 PSA 源码的猜测推断输入类别；同名
参数在不同门面可以引用不同 schema，例如两类 `measurement` 和恢复重构、短路研究使用的
两类 `fault`。

## 可信来源

文档冲突时按下列优先级判断实际行为：

1. `tensoreval` 当前发布版本中的 RSDss 整表和 DSA 函数分派实现；
2. `tensoreval` 当前发布版本中的 DataFrame 函数和表达式翻译实现；
3. `psa/src/dsa/parse/` 及各分析模块的表解析器：schema、默认值和校验；
4. 随发布版本执行的集成测试与 `psa/tests/`：可执行行为和数值边界；
5. 本目录文档。

新增或修改函数时，应在同一变更中更新函数参考、相关 schema、机器可读 catalog 和至少
一个可运行案例。不能根据 OpenDSS 属性名推测 RSS 列名，也不能根据 PSA 内部结构推测
RSDss 已公开相同入口。

文档发布流程会运行目录一致性检查，核对三个 JSON 的版本和内部引用、完整网络顺序、全部
DataFrame 研究门面、Rust `*_RESULT_NAMES` 常量与 catalog 结果列表、QSTS 状态、计量和轨迹
构造器的固定列，以及 Markdown 本地链接；潮流的 11 张详细结果表也直接与 PSA DataFrame
构造器核对。它不会替代 Rust 编译和仿真案例测试。

## 已验证案例

| 案例 | 分析 | 主要输出 | 验证 |
|---|---|---|---|
| [`../bmopf_4bus_power_flow.rss`](../bmopf_4bus_power_flow.rss) | BMOPF 4Bus Radial 静态潮流 | `bus_phase` | OpenDSS 复电压 |
| [`../bmopf_ieee13_power_flow.rss`](../bmopf_ieee13_power_flow.rss) | BMOPF IEEE13 Radial 静态潮流 | `bus_phase` | OpenDSS 复电压 |
| [`../bmopf_ieee34_power_flow.rss`](../bmopf_ieee34_power_flow.rss) | BMOPF IEEE34 Radial 静态潮流 | `bus_phase` | OpenDSS 复电压 |
| [`../ieee13_power_flow.rss`](../ieee13_power_flow.rss) | Radial 静态潮流 | `bus_phase` | OpenDSS 复电压 |
| [`../ieee34_power_flow.rss`](../ieee34_power_flow.rss) | Radial 静态潮流 | `bus_phase` | OpenDSS 复电压 |
| [`../ieee123_power_flow.rss`](../ieee123_power_flow.rss) | Radial 静态潮流 | `bus_phase` | OpenDSS 复电压 |
| [`../ieee13_harmonic.rss`](../ieee13_harmonic.rss) | 谐波 | `bus_voltage` | OpenDSS 复电压 |
| [`../ieee34_harmonic.rss`](../ieee34_harmonic.rss) | 谐波 | `bus_voltage` | OpenDSS 复电压 |
| [`../ieee13_short_circuit.rss`](../ieee13_short_circuit.rss) | 单相接地短路 | `fault` | OpenDSS 复相量 |
| [`../ieee34_short_circuit.rss`](../ieee34_short_circuit.rss) | 单相接地短路 | `fault` | OpenDSS 复相量 |
| [`../4bus_daily_qsts.rss`](../4bus_daily_qsts.rss) | 三时点 Daily QSTS | `bus_phase` | 36 行 OpenDSS 复电压 |
| [`../ieee13_daily_qsts.rss`](../ieee13_daily_qsts.rss) | IEEE13 全负荷三时点 Daily QSTS | `bus_phase`, `load_state`, `summary` | 可执行行为与完整行覆盖 |
| [`../case14_4bus_master_slave_pf.rss`](../case14_4bus_master_slave_pf.rss) | case14 正序主网 + 4Bus 三相配网主从分裂 | `bus0..3` 等各轮 DataFrame | 显式执行三轮、PCC 功率与电压交换 |
| [`../rssbundle/Examples/DistanceRelays/DistanceRelayTest.rssbundle`](../rssbundle/Examples/DistanceRelays/DistanceRelayTest.rssbundle) | 10 ms Distance Relay 网络暂态 V12 | 15 张 V12 长表 | OpenDSS 数值、29 个事件、9 组 Relay 终态 |
| [`../rssbundle/Examples/DistanceRelays/TD21RelayTest.rssbundle`](../rssbundle/Examples/DistanceRelays/TD21RelayTest.rssbundle) | 1 ms TD21 网络暂态 V12 | 15 张 V12 长表 | OpenDSS 数值、动作顺序/次数和 Relay 终态 |

## 运行入口

```bash
tensoreval --df sparrowzz/dss/4bus_daily_qsts.rss
```

暂态 bundle 直接运行并把 15 张结果完整写入目录：

```bash
tensoreval --result-dir /tmp/dynamic-results \
  sparrowzz/dss/rssbundle/Examples/DistanceRelays/DistanceRelayTest.rssbundle
```

`--dataframe` 和 `--df` 等价。Newton 潮流、Newton QSTS 和状态估计还需要选择
包含 MUMPS 或 MKL 后端的发布版本。谐波整网案例应使用发布版二进制。
