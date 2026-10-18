# tensoreval RSS 文档索引

本目录描述 `tensoreval --dataframe` 执行的 RSS（RustScript/RSDss）脚本，以及
`with_dsa` 提供的配电系统分析函数。文档以当前仓库生产代码为准，目标是让用户和
代码智能体能够从仿真需求生成可运行、可检查的 `.rss` 脚本。

## 推荐阅读顺序

1. [智能体快速指南](agent_quickstart.md)：从需求选择求解器、输入和输出。
2. [RSS 语言参考](rss_language_reference.md)：语法、求值图和多结果规则。
3. [RSDss 函数参考](rsdss_function_reference.md)：DataFrame 变换函数。
4. [DSA 函数参考](dsa_function_reference.md)：`import_*` 与全部 `ds_*` 入口。
5. [DataFrame schema 参考](dataframe_schema_reference.md)：输入表的列、类型和单位。
6. [OpenDSS 兼容矩阵](opendss_compatibility_matrix.md)：对象、命令和分析模式的对应关系。
7. [仿真配方](simulation_recipes.md)：按任务组织的可执行脚本模板。
8. [验证与故障排查](validation_and_troubleshooting.md)：结果核验、容差和错误处理。
9. [从 OpenDSS 迁移到 RSS](opendss_to_rss.md)：完整迁移流程和数值对照方法。
10. [DSS 到 RSS 严格转换](dss_to_rss.md)：`dss2rss` 命令、成功条件和不支持诊断。
11. [RSS bundle 合同](../../../mems/docs/dss_to_rss_bundle.md)：便携目录结构、manifest schema、校验顺序和稳定错误码。

机器可读索引位于 [`catalog/functions.json`](catalog/functions.json) 和
[`catalog/dataframes.json`](catalog/dataframes.json)。Markdown 解释语义，JSON 固定名称、
参数位置和 schema，生成脚本时应同时参考两者。

## 可信来源

文档冲突时按下列优先级判断实际行为：

1. `mems/src/ml` 中的 RSDss 整表和 DSA 函数分派实现；
2. `mems/src/ml` 中的 RSDss DataFrame 函数和表达式翻译实现；
3. `psa/src/dsa/parse/` 及各分析模块的表解析器：schema、默认值和校验；
4. `mems/tests/dff/dff_script.rs` 与 `psa/tests/`：可执行行为和数值边界；
5. 本目录文档。

新增或修改函数时，应在同一变更中更新函数参考、相关 schema、机器可读 catalog 和至少
一个可运行案例。不能根据 OpenDSS 属性名推测 RSS 列名，也不能根据 PSA 内部结构推测
RSDss 已公开相同入口。

修改目录后从仓库根目录运行一致性检查：

```bash
python3 sparrowzz/dss/docs/catalog/validate_catalog.py
```

该检查会核对两个 JSON 的版本和内部引用、完整网络顺序、全部公开 DSA 名称、Rust
`*_RESULT_NAMES` 常量与 catalog 结果列表、QSTS 状态/轨迹构造器的固定列，以及 Markdown
本地链接；潮流的 11 张详细结果表也直接与 PSA DataFrame 构造器核对。它不会替代 Rust
编译和仿真案例测试。

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

## 运行入口

```bash
cargo run -p mems \
  --features tensoreval-cli,with_dsa \
  --bin tensoreval -- \
  --df sparrowzz/dss/4bus_daily_qsts.rss
```

`--dataframe` 和 `--df` 等价。Newton 潮流、Newton QSTS 和状态估计还需要选择
`with_dsa_mumps` 或 `with_dsa_mkl`。谐波整网案例应使用 release 构建。
