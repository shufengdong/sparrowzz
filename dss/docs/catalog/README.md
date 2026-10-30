# DSS 机器目录格式

本目录是 `tensoreval --df` DataFrame 研究门面的发布合同。调用方必须从同一版本同时读取：

- `functions.json`（schema version 2）：函数分派名、arity、规范参数组和结果名；
- `study_facades.json`（schema version 1）：逐门面的参数位置、`inputKind`、`schemaRef`、结果映射和结果选择约束；
- `dataframes.json`（schema version 4）：输入、Struct 和结果 DataFrame 的完整执行 schema；version 4 正式发布机器规则联合类型，并规定只有 Struct 列可以声明 `card`。
- `case_projections.json`（schema version 2）：DSS 来源函数到 DataFrame 门面的投影 profile、projector 完整输出、门面输入/结果合同、bundle 结果映射与稳定诊断。

`study_facades.json` 是研究门面的入口索引，`dataframes.json` 是它引用的类型库。调用方禁止从函数名、参数名、Rust 源码、样例或说明段落推断执行事实。

## rssbundle 投影合同

`case_projections.json` 是案例导入的权威机器合同。启用 profile 的 `inputs` 和 `results`
必须逐字段等于目标 `study_facades.json` 门面；`resultMap` 描述来源函数的结果选择名，
`bundleResultMap` 描述 manifest 中旧脚本绑定名到目标结果名的映射。`managedEntry` 固定为
`managed.rss`。`compatibility.failClosed=true` 且必须提供 `stableDiagnostic`。

每个启用 profile 的 `projectorOutputs` 是 projector 返回 DataFrame 的完整有序合同，位置从 1
连续编号。每项必须包含 `id`、`alias`、`inputKind`、`schemaRef`、`required`、`position` 和
`disposition`：

- `facade_input` 必须另带 `targetInput`，并与 `inputs` 中一个输入唯一、同 schema、同角色地映射；
- `diagnostic` 是导入诊断资源，不声明 `targetInput`，也禁止注入目标研究门面。

`projectorOutputs` 必须覆盖 projector 的每一张返回表，包括零行表和尾部诊断表。宿主禁止按
固定数量截断、根据列名猜测，或把诊断表伪装成研究输入。例如 fault projector 的第 23 张表为：

```json
{
  "id": "import_diagnostic",
  "alias": "import_diagnostic",
  "inputKind": "data",
  "schemaRef": "execution_inputs.import_diagnostic",
  "required": true,
  "position": 23,
  "disposition": "diagnostic"
}
```

bundle manifest schema v2 的 `projection` 包含 `source_function`、`profile`、`projector`、
`target_function`、`managed_entry`、`status`、`stable_diagnostic` 和有序 `results`。每个结果项
明确给出 `source`、`target` 和无冲突的 `binding`。`managed.rss` 只引用已注入 DataFrame
别名；DSS 文件与 `main.rss` 是可审计导入来源，不是受管研究的运行依赖。

```json
{
  "projection": {
    "source_function": "ds_radial_dss_plan",
    "profile": "dss_execution_radial_v1",
    "projector": "project_dss_execution_plan",
    "target_function": "ds_radial_execution_plan",
    "managed_entry": "managed.rss",
    "status": "enabled",
    "stable_diagnostic": "PSA.DSS.PROJECTION.INVALID",
    "results": [
      {"source": "voltage", "target": "bus_phase", "binding": "result_voltage"}
    ]
  }
}
```

宿主必须先以 `projector` 对 `source.entry` 做一次导入，严格按 `projectorOutputs[].position`
解释返回值；将 `inputKind=model` 的表写入模型版本，将 `data|struct` 写入可替换本地资源，保存
`disposition=diagnostic` 的导入诊断，再按 `targetInput` 和 `inputs[].position` 注入默认 alias。
结果卡使用 manifest 的 `results[].target` 写入 profile 声明的 selector；最终输出资源名使用
`results[].binding`。宿主禁止把 `source.entry` 或 `source.root` 传给受管脚本。

当前 144 个发布案例均为 `enabled`：121 个 radial execution、10 个 Newton execution、
3 个 fault、3 个 harmonic、6 个 dynamic 和 1 个 Newton static。AutoAdd/RelCalc 的新 bundle
会发布 `disabled` profile 与 `TENSOREVAL.RSS.BUNDLE.PROJECTION_UNAVAILABLE`，不会生成近似研究。

## 执行合同与展示字段

下列字段属于执行合同，修改后必须视为 schema 变化：

- `dtype_grammar`；
- `contract.kind`、`rowCount`、`shape` 和 `columnPolicy`；
- `columns` 中的顺序、`name`、`dtype`、`required`、`nullable` 和 `unit`；
- `stableKeys`、`sortKeys`、`columnAliases`、`allowedConversions` 和 `rules`；
- Struct 列的 `card`；
- `discriminator`、`variants`、`base_schema` 和 `baseMerge`。

`title`、`description`、`note`、`context`、`differences` 以及列说明用于展示。展示文字可以调整，不能覆盖或补充缺失的执行合同。`contract.source` 是可审计的源码或测试依据，也必须指向仓库内真实路径或已登记的虚拟源码 URI。

## dtype grammar version 1

`dataframes.json.dtype_grammar` 是唯一合法类型语法。语法为：

```text
dtype := primitive | semantic | constructor "<" dtype ">"
```

原子类型包括 `str`、`bool`、`i32`、`u32`、`u64` 和 `f64`。一元构造器包括：

- `list<T>`：Arrow `List<T>`、Parquet `LIST<T>`；元素不可为 null，整格是否可为 null 由列的 `nullable` 决定；
- `matrix<T>`：物理表示仍为 Arrow/Parquet 一维 List，逻辑上按行主序扁平编码；矩阵维度由字段规则、相数或配套表决定。

语义类型包括：

- `u32_semantics`：上传时可接受可无损转成 `u32` 的 `u32/u64/i32/f64`，数值必须有限、非负、为整数且不超过 `u32::MAX`；规范落盘类型是 Arrow `UInt32`；
- `u32_or_list_bool`：电容器 `states` 可使用已投入步数 `UInt32`，或使用按物理步顺序排列的 `List<Boolean>`；一个上传列只能选择其中一种物理形态；
- `f64_or_list`：同一上传列选择标量 `Float64` 或 `List<Float64>` 物理形态，具体广播规则由字段合同决定；
- `f64_or_matrix`：标量 `Float64` 或行主序扁平 `List<Float64>`；
- `str_or_list`：标量 Utf8 或 `List<Utf8>`，标量逻辑上是一项列表。

一个 Arrow/Parquet 列必须选择一种物理形态；语义联合类型不允许同一列逐行混用物理 dtype。`upload_conversions` 给出唯一允许的受控转换。未登记类型、非法泛型和隐式有损转换必须拒绝。

## 通用 DataFrame contract

每个被门面直接或通过 `base_schema` 间接引用的 schema 都有 `contract`：

```json
{
  "kind": "model|data|struct|result",
  "rowCount": "zero|one|zero_or_one|zero_or_more|one_or_more",
  "shape": "uniform|discriminated",
  "columnPolicy": {"mode": "closed|open|dynamic", "unknownColumns": "forbidden|ignored"},
  "columns": [],
  "stableKeys": [],
  "sortKeys": [],
  "columnAliases": {},
  "allowedConversions": [],
  "rules": [],
  "source": []
}
```

`columns` 是规范列顺序。每列必须显式给出：

- `required`：该列是否必须存在；
- `nullable`：列存在时单元格是否允许 null；
- `dtype`：dtype grammar 中的合法类型；
- `unit`：物理单位，或字面值 `not_applicable`。

`required=false` 与 `nullable=true` 含义不同：前者允许整列缺失，后者允许已有列中的单元格为 null。缺字段既不表示“尚未记录”，也不表示“不适用”。不适用必须显式写为 `not_applicable` 或由 variant/规则禁用。

当前输入解析器普遍使用 open column policy：未知列被明确标记为 `ignored`。结果表使用 closed policy：未列出的结果列不属于合同。若以后出现动态列，`mode=dynamic` 必须同时发布可解析的生成规则。

`stableKeys` 表示行身份键，`sortKeys` 表示规范排序键；空列表表示该 schema 没有发布稳定键或排序承诺。`columnAliases` 从允许的上传别名映射到规范列名。别名不会成为额外结果列。

## Struct 参数卡

`kind=struct` 的 `rowCount` 只能是 `zero`、`one` 或 `zero_or_one`。每个 Struct 列都必须带有完整
`card`。`kind=model|data|result` 的列禁止声明 `card`；这些表由模型版本或普通数据资源提供，
不能被参数卡片解释。

```json
{
  "default": {"state": "value|required|omitted", "value": 100},
  "unit": {"state": "value|not_applicable", "value": "s"},
  "range": {
    "state": "bounded|not_applicable",
    "minimum": 0,
    "maximum": null,
    "minimumInclusive": false,
    "maximumInclusive": false
  },
  "enum": {"state": "values|not_applicable", "values": []},
  "visibleWhen": {"state": "always|condition", "when": {}},
  "enabledWhen": {"state": "always|condition", "when": {}}
}
```

默认值状态不可省略：

- `value`：省略字段时解析器采用给定值；
- `required`：用户必须填写，没有默认值；
- `omitted`：省略表示 `None` 或交由外部上下文，不存在隐含值。

`rules` 保存跨字段约束。version 4 的规则联合类型包括 `positive`、`nonnegative`、`range`、
`compare`、`conditional_compare`、`conditional_required`、`conditional_enabled`、
`conditional_ignored`、`conditional_empty_input`、`mutually_exclusive`、
`mutually_exclusive_groups`、`length`、`length_equal`、`equal_length`、`paired_presence`、
`unique`、`unique_items`、`unique_csv_items`、`finite`、`requires`、`forbidden`、`enum`、
`strictly_increasing`、`positive_contiguous`、`contiguous_zero_based`、`exactly_one_variant` 和
`cross_table_dense_matrix`。未登记的规则类型必须拒绝。规则引用的本 schema 字段必须存在；
涉及另一个门面的输入时使用 `input` 明确标识，禁止仅靠文字说明。

`contiguous_zero_based` 的结构固定为：

```json
{"type": "contiguous_zero_based", "field": "snapshot_index"}
```

它禁止其他字段。`field` 必须是 schema 中真实存在的无符号整数列（`u32`、`u64` 或
`u32_semantics`）。非空表中该列必须从 0 开始、严格递增且相邻值差恰好为 1；空表是否允许只由
该 schema 的 `rowCount` 决定，规则本身不改变行数约束。

`dataframes.json` 从 version 3 升到 4，是因为规则联合类型和非 Struct `card` 禁令会改变消费者
是否接受合同；`case_projections.json` 从 version 1 升到 2，是因为 `projectorOutputs` 成为启用
profile 的必需执行字段。消费者必须按版本 fail closed，不能把新字段静默塞回旧版本语义。

## 多形态表

`shape=uniform` 禁止声明 discriminator 或 variants。`shape=discriminated` 必须声明两者：

- `discriminator.source=column`：由真实列值选择；
- `source=presence`：由一组字段是否出现选择，并以 `synthetic=true` 表明判别字段不是上传列；
- `source=external_struct`：由同一门面的另一个 Struct 字段选择。

每个 variant 有稳定 `id`、机器 `when` 条件，以及 `required`、`optional`、`enabled` 或 `sideSchemas`。这些字段和侧表引用都必须存在。`relay`、`reactor`、`transformer`、动态事件、状态估计量测和 Monte Carlo replay 表均按此格式发布，不需要调用方从自然语言或名称猜测行形态。

## base_schema 合并

`base_schema` 只能指向同一 `dataframes.json` 中 `<catalogSection>.<schemaName>` 两段式顶层 schema。禁止指向 `columns`、`required` 等嵌套对象或数组。

继承 schema 必须声明：

```json
"baseMerge": {
  "order": "base_then_local|local_then_base",
  "override": "forbidden"
}
```

解析时递归展开整个继承闭包。`base_then_local` 先放基类列再放本地列；`local_then_base` 用于保存前缀列。任何重名列均为冲突，不能覆盖 dtype、nullable、单位或顺序。未知引用和循环也必须拒绝。QSTS 结果通过该规则继承静态或状态结果的完整列合同，而非只继承列名。

## 完整性与发布检查

Model、Data、Struct 和 Result 合同都必须包含非空的解析后列集、合法 dtype、显式单位和 source。Struct 还必须能独立生成参数卡；Result 必须能在不读 Rust 源码的情况下生成 Arrow/Parquet schema。

运行：

```bash
python3 dss/docs/catalog/validate_catalog.py
python3 dss/docs/catalog/test_validate_catalog.py
```

校验器遍历 `study_facades.json` 的全部门面及传递 `base_schema` 引用，还扫描全部 rssbundle
manifest、`managed.rss`、结果映射和文件哈希。目录发布要求两个命令均通过且完整性诊断为零。
