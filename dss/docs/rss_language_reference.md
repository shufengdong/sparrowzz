# RSDss（`.rss`）语言参考

RSDss 是面向 DataFrame 和配电系统分析的 RustScript 方言。`.rss` 是本目录采用的脚本
扩展名；`tensoreval --dataframe`（简写 `--df`）先用 `eig-expr` RustScript 解析器读取
程序，再把顶层声明编译为 RSDss 依赖图。RSDss 文件不是 Rust 源码，也不是 OpenDSS
命令文件。

本文说明 RSDss 的词法、赋值、DataFrame 函数和列表达式语义。每个函数的精确参数和行为见
[RSDss 函数参考](rsdss_function_reference.md)，DSA 入口见
[DSA 函数参考](dsa_function_reference.md)。

## 1. 执行模式与语法边界

同一份 RustScript 语法会被三个入口以不同的值类型和函数集合执行。智能体必须先确定入口，
不能仅根据 `.rss` 扩展名推断运行模式：

| 入口 | 顶层程序 | 外部输入 | 结果 |
|---|---|---|---|
| `tensoreval --df FILE...` | RSDss DataFrame 声明图，可含 `{...}` 数值程序 | `--input NAME=FILE`、脚本内导入函数 | 最深依赖层产生的具名 DataFrame |
| `tensoreval FILE...` | 普通 Tensor 表达式图或整段 RustScript VM 程序 | 脚本字面量；VM 程序还可调用 `read_tensor*` | Tensor/标量的调试文本，不形成 RSDss DataFrame 输出 |
| mems 流程中的 `TensorEval` 节点 | 由 `para` 参数位选择 DataFrame 图、Tensor 图或整段 VM 程序 | 入边别名对应的 DataFrame，以及流程变量和量测上下文 | 一张或多张 DataFrame，交给后续流程边 |

本文把第一个入口和 `TensorEval` 节点设置 `TE_PARA_DF` 后使用的 DataFrame 语言称为
RSDss。第二个入口的完整数值语法见[《RustScript 手册》](../../rustscript/RustScript手册.md)。

RSDss 命令行模式通过以下命令启用：

```bash
tensoreval --df case.rss
```

顶层只接受具名声明；每个声明产生 DataFrame、多张 DataFrame 或供后续节点使用的标量。
`if`、`for`、`while`、`loop`、`fn`、`break`、`continue` 和 `return` 属于普通 RustScript
数值程序语法，不能作为当前 `tensoreval --df` 的顶层语句。

普通 tensoreval 模式执行 Tensor/VM 程序，可以使用 RustScript 控制流，但不能调用
`import_dss`、`ds_*` 等 RSDss DataFrame 入口。两种命令行模式共用文件拼接、include
预处理和 RustScript 解析器，不共用函数集合、输入协议、运行时值类型或输出协议。

## 2. 词法规则

### 2.1 空白和注释

空格、制表符和换行可出现在词元之间。支持单行注释和块注释：

```text
// 单行注释

/* 多行注释
   可以跨行 */
```

块注释在遇到第一个 `*/` 时结束，不应嵌套。

### 2.2 标识符

顶层 RustScript 解析器支持 Unicode 字母、`_`、`$` 和 Unicode 转义，但 RSDss 表达式
层的裸标识符约束更窄。为保证声明、依赖分析和列表达式都能一致解析，脚本应使用：

```text
[A-Za-z_][A-Za-z0-9_]*
```

有效示例：

```text
variable_name
matrix1
bus_data
PQ_bus
```

关键字和函数名不应用作输出变量名。多结果中的 `_` 是丢弃占位符，不会声明变量。

### 2.3 数值字面量

数值统一先解析为 `f64`，支持整数外观、小数和科学计数法：

```text
42
0
3.14
1.23e-4
6.022E23
-17
-2.718
```

负号是前缀一元运算符，不属于数字本身。数字必须以十进制数字开头，因此使用 `0.5`，
不要写 `.5`。RSDss 在构造 DataFrame 时由 Polars 决定最终列类型；需要整数列时使用
`cast(expr, i32)` 等显式转换。

### 2.4 字符串、裸名称和列名

双引号字符串是 RSDss 文档和案例的标准写法：

```text
"DAILY"
"Power flow data for IEEE 14 bus"
"path/to/model.dss"
```

表达式解析器还接受单引号名称和反引号字符串，但新脚本应统一使用双引号。当前引号词元
不提供通用字符串转义协议，应避免在内容中嵌入与外层相同的引号。

RSDss 的裸名称具有位置语义：

```text
alias(3600.0, step_size_s)
alias("DAILY", mode)
alias(exact, line_model)
col(v_mag)
select(col(v_mag), voltage)
```

- 在列表达式中，`col(v_mag)` 才读取 `v_mag` 列；
- `alias(..., step_size_s)` 的裸名称表示输出列名；
- 模式、枚举和结果列表优先写成双引号字符串；
- 单表函数最后位置的 `voltage` 解析为已绑定的 DataFrame。

### 2.5 常量

`pi`、`e`、`NAN`、`INF` 和 `NEG_INF` 是普通 RustScript 数值运行时的
上下文常量。RSDss DataFrame 表达式不会把这些裸名称自动转换成相同的数值常量。

| 需求 | RSDss 写法 |
|---|---|
| π | `pi()` |
| 自然常数 e | `exp(1)` |
| NaN、正负无穷 | 没有通用裸字面量；从输入数据保留，或使用明确支持它的 DataFrame 运算产生 |

因此不要在 RSDss 脚本中把裸 `pi`、`e`、`NAN`、`INF` 或 `NEG_INF` 当作数值。

### 2.6 类型名称

RSDss 没有变量类型声明语法。类型名称是 `cast`、`cast_all` 和 `arange` 等函数的参数。
当前支持：

```text
null
i8 i16 i32 i64 i128
u8 u16 u32 u64 u128
f32 f64
bool binary str
date time
datetime[ms] datetime[μs] datetime[ns]
duration[ms] duration[μs] duration[ns]
unknown
```

`datetime[unit,timezone]` 可附带 Chrono 能解析的时区。微秒单位必须写成 `μs`。包含
方括号的类型名作为参数时应加双引号，例如 `cast(col(time), "datetime[ms]")`。

### 2.7 include 预处理

tensoreval 在解析 RustScript 前处理以下行：

```text
#include path/to/common.rss
#include_url https://example.test/common.rss
```

内容按出现顺序拼接到当前程序。`#include` 会递归展开本地包含文件，所有相对路径都以
进程当前工作目录为基准；展开过程拒绝包含循环、超过 32 层的嵌套和超过 16 MiB 的结果。
`#include_url` 只接受运行环境允许访问的 HTTP(S) 地址，请求超时为 30 秒，响应上限为
64 MiB，其返回内容不继续扫描 include。DSS 文件中的 `Redirect`/`Compile` 由 DSS 导入器
处理，与 RSDss include 是两套机制。

expr_df 中每个 `{ ... }` 都会编译为独立 Prog。某个代码块调用的函数必须在该代码块内
定义或包含，不能依赖另一个代码块已经加载的函数。多个代码块使用同一组依赖时，可把
包含清单放在独立文件中：

```text
result = {
    #include sparrowzz/dss/runpf_bundle.txt
    return runpf_case(base_mva, bus, gen, branch);
};
```

共享清单只集中维护依赖列表；预处理仍会在每个指令位置展开完整内容，因此各 Prog 的 VM
上下文彼此独立。

## 3. 声明、赋值和依赖图

### 3.1 单结果声明

标准声明使用 `=`，并以分号结束：

```text
rows = height(input);
output = select(col(value), input);
```

解析器也接受 `:` 作为声明分隔符，但仓库脚本统一使用 `=`。分号应始终保留。

声明会编译为依赖图，而不是按文本顺序立即执行。编译器先注册所有左值，因此允许前向
引用；循环依赖、重复声明和找不到的 DataFrame 会失败。为了便于阅读，仍建议先声明输入
转换，再声明求解和结果处理。

`height(frame)` 和 `width(frame)` 返回标量，可供后续表达式使用：

```text
rows = height(input);
output = with_columns(
    alias(cast(arange(0, rows) + 1, u32), index),
    input
);
```

终端输出必须是 DataFrame；未被后续节点消费的纯标量不能作为 tensoreval 的最终结果。

### 3.2 多结果声明

返回多张 DataFrame 的函数按位置解构：

```text
[bus, branch, summary] = ds_radial_pf(...);
```

`_` 保留返回位置但不注册变量：

```text
[voltage, _, summary] = ds_radial_pf(...);
```

实际返回数量必须等于左值槽位数量，否则产生 `ResultCountMismatch`。多结果调用不能直接
嵌入只接受单张 DataFrame 的参数，必须先解构再使用。

### 3.3 顶层右值

RSDss 顶层右值必须最终形成一个受支持的函数调用，例如 `select(...)`、`height(...)`、
`import_dss(...)` 或 `ds_radial_pf(...)`。单独写算术表达式不能作为 DataFrame 图节点：

```text
// 不作为 RSDss 顶层节点使用
x = 1 + 2;

// 把计算放进产生 DataFrame 的函数
output = select(alias(1 + 2, value), input);
```

## 4. 表达式语法

### 4.1 函数调用和嵌套

函数使用圆括号，参数用逗号分隔，允许嵌套：

```text
alias(cast(arange(0, rows) + 1, u32), index)
when_then_otherwise(col(v) > 1.0, col(v), 1.0)
```

零参数函数仍需圆括号，例如 `all()`、`pi()`、`element()`。参数个数在执行表达式前校验。

### 4.2 运算符

RSDss 列表达式当前执行以下运算符：

| 类别 | 运算符 | 含义 |
|---|---|---|
| 前缀 | `+x`, `-x` | 正号、负号 |
| 算术 | `+`, `-`, `*`, `/`, `%`, `^` | 加、减、乘、除、余数、幂 |
| 比较 | `==`, `!=`, `<`, `>`, `<=`, `>=` | 逐元素比较 |
| 逻辑 | `&&`, `||` | 逐元素与、或 |
| 异或 | `^^` | 逐元素 xor |

使用圆括号明确组合顺序：

数值列表列支持与标量列之间的逐元素加、减、乘、除。例如主配网案例用
`col(v_magnitude) * col(vm_pu)` 缩放电压源的各端子幅值；两列必须具有 Polars 可广播的
形状。

```text
(col(p) + col(q)) / 1000
(col(v_mag) >= 0.95) && (col(v_mag) <= 1.05)
```

底层 RustScript 解析器还能识别部分 Tensor、位运算、阶乘和转置运算符，但 RSDss 的
Polars 翻译器没有为它们全部提供执行分支。RSDss 脚本只应使用上表运算符；逻辑取反使用
`not(expr)`。

### 4.3 名称数组

方括号可以表达名称列表，供接受多个列名的整表函数使用；`cols(...)` 则产生 Polars
列选择器：

```text
sorted = sort([bus, phase], [false, false], true, voltage);
selected = select(cols(bus, phase, v_re, v_im), voltage);
```

多结果声明左侧同样使用方括号，但语义是有序绑定槽位，不是普通数组值。

## 5. DataFrame 函数调用规则

### 5.1 标量结果函数

| 函数 | 行为 |
|---|---|
| `height(frame)` | DataFrame 行数 |
| `width(frame)` | DataFrame 列数 |

`size(frame)` 当前没有注册，不能调用。需要元素总数时分别取得 `height` 和
`width`，再在后续产生 DataFrame 的表达式或普通数值程序中相乘。

### 5.2 单表函数

单表函数把最后一个参数解释为输入 DataFrame，其余参数是列表达式：

```text
selected = select(col(bus), col(phase), voltage);
filtered = filter(all(), col(v_mag) > 1000, selected);
extended = with_columns(alias(col(v_mag) / 1000, v_kv), filtered);
```

当前单表函数完整分类如下：

| 类别 | 函数 |
|---|---|
| 选择与过滤 | `select`, `filter` |
| 添加或替换列 | `with_column`, `with_columns` |
| 分组与聚合 | `group_by`, `agg` / `aggregate` |
| null/NaN 行处理 | `drop_nulls`, `drop_nans`, `fill_null`, `fill_nan` |
| 全表统计 | `count`, `null_count`, `first`, `last`, `sum`, `min`, `max`, `mean`, `median` |
| 行顺序和截取 | `reverse`, `limit`, `tail` |
| 类型 | `cast_all` |
| 位移 | `shift`, `shift_and_fill` |

`filter` 的最后一个表达式是 predicate，predicate 之前的表达式是输出选择。保留全部原列
时显式传入 `all()`：

```text
filter(all(), col(enabled), input)
```

### 5.3 整表和多表函数

以下函数自行解释所有参数，不采用统一的“最后一项必为输入表”规则：

| 函数 | 用途 |
|---|---|
| `join` | 按键连接两张表 |
| `sort` | 按一列或多列排序 |
| `concat` | 纵向、对角或横向拼接多张表 |
| `pivot` | 长表转宽表 |
| `unpivot` | 宽表转长表 |
| `import_df_url` | 从 URL 读取 DataFrame |

推荐显式写 `concat` 模式和所有输入表：

```text
combined = concat("vertical", first, second, third);
```

新脚本只使用 `vertical`、`diagonal` 和 `horizontal`。兼容形式 `concat()` 或
`concat(mode)` 会隐式使用上下文中的全部 DataFrame，不适合仿真脚本。

### 5.4 列表达式函数

当前可用的 RSDss 列表达式函数按用途归类如下：

| 类别 | 函数 |
|---|---|
| 选择器 | `all`, `col`, `cols`, `element` |
| 命名与类型 | `alias`, `cast` |
| 条件与布尔 | `when_then_otherwise`, `not`, `not_null`, `is_null`, `not_nan`, `is_nan` |
| 删除缺失值 | `drop_nulls`, `drop_nans` |
| 比较 | `eq`, `eq_missing`, `neq`, `neq_missing`, `lt`, `gt`, `lt_eq`, `gt_eq` |
| 唯一性 | `n_unique`, `unique`, `arg_unique`, `is_unique`, `is_duplicated` |
| 位置与顺序 | `arg_min`, `arg_max`, `reverse`, `first`, `last`, `head`, `tail`, `sort`, `filter` |
| 计数与聚合 | `count`, `null_count`, `len`, `sum`, `min`, `max`, `mean`, `avg`, `median`, `nan_min`, `nan_max` |
| 横向聚合 | `min_horizontal`, `max_horizontal`, `sum_horizontal`, `any_horizontal` |
| 统计 | `std`, `var`, `dot` |
| 累计 | `cum_count`, `cum_sum`, `cum_min`, `cum_max`, `cum_prod` |
| 位移 | `shift`, `shift_and_fill` |
| 基础数学 | `pi`, `abs`, `sign`, `pow`, `exp`, `log1p`, `log`, `sqrt`, `cbrt`, `floor`, `ceil`, `round`, `arange` |
| 三角函数 | `sin`, `cos`, `tan`, `cot`, `arcsin`, `arccos`, `arctan`, `degrees`, `radians` |
| 双曲函数 | `sinh`, `cosh`, `tanh`, `arcsinh`, `arccosh`, `arctanh` |
| 字符串 | `split`, `starts_with`, `ends_with`, `contains`, `replace`, `find`, `slice`, `to_lowercase`, `to_uppercase` |

签名、字面量要求、默认值和特殊限制见
[RSDss 函数参考的列表达式章节](rsdss_function_reference.md#3-列表达式函数)。

### 5.5 DSA 函数

启用 `with_dsa` 或相应求解器 feature 后，RSDss 还提供：

```text
import_dss import_dss_url import_bmopf import_bmopf_url
import_dss_execution import_dss_execution_url
ds_radial_pf ds_newton_raphson_pf
ds_radial_qsts ds_newton_raphson_qsts
ds_radial_dss_plan ds_newton_raphson_dss_plan
ds_radial_dss_plan_url ds_newton_raphson_dss_plan_url
ds_harmonic_study ds_fault_study
ds_wls_se ds_mnmr_se ds_mce_se
ds_monte_carlo ds_restore ds_protection_coordination
ds_fault_location ds_motor_dynamics
```

这些函数的位置参数、返回表和 feature 要求见 [DSA 函数参考](dsa_function_reference.md)。

## 6. DataFrame 名称与表达式上下文

同一个裸名称会按所在位置解释：

```text
col(bus)                  // 读取 bus 列
alias("source", kind)    // kind 是新列名
select(col(bus), voltage) // voltage 是 DataFrame 变量
```

RSDss 图会把已声明的标量放入后续表达式的 literal 上下文，把 DataFrame 放入 frame
上下文。函数参数的位置决定它读取哪一种上下文。输出变量名应避免与随后仍作为枚举字面量
使用的名称相同。

## 7. RSDss 输入契约

### 7.1 程序文本

`tensoreval --df` 至少需要一个位置参数 `FILE`。传入多个文件时，工具按命令行顺序读取
并以换行连接，然后对合并后的文本执行第 2.7 节的 include 预处理，最后只解析和执行一次：

```bash
tensoreval --df declarations.rss study.rss
```

文件边界不创建模块、命名空间或作用域。所有顶层声明处于同一张图中，重复变量名会失败。
本地脚本、`#include` 和 `--input` 的相对路径都以进程当前工作目录为基准，不以入口文件
所在目录为基准。

### 7.2 命令行 DataFrame 输入

外部表通过可重复的 `--input NAME=FILE` 注入：

```bash
tensoreval --df \
  --input load_profile=profiles.csv \
  --input settings=settings.parquet \
  case.rss
```

每个 `NAME` 成为求值开始前已经存在的 DataFrame 变量。约束如下：

| 项目 | 契约 |
|---|---|
| 名称 | `NAME` 和 `FILE` 都不能为空；同一命令中 `NAME` 必须唯一；要在脚本中引用，`NAME` 应满足第 2.2 节标识符规则；脚本不应再次声明同名变量 |
| CSV | 除小写 `.xls`、`.xlsx`、`.parquet` 后缀外的路径都按带表头 CSV 读取，并扫描全部行推断类型 |
| Excel | 支持 `.xls`、`.xlsx`；工作簿必须恰好产生一个工作表，多个工作表会使加载失败 |
| Parquet | `.parquet` 按 Parquet 读取 |
| 失败 | 任一输入无法读取或解析时，整个命令在求值前失败 |

`--input` 只在 `--df` 模式可用。它不会把路径字符串传入脚本，也不会自动把文件名或列名
变成变量。DataFrame 保留文件读取器得到的列名、行顺序和 dtype，直到脚本显式变换它。

### 7.3 脚本内数据源

RSDss 不提供通用 DataFrame 表字面量。没有 `--input` 时，第一层必须使用能够自行产生
DataFrame 的导入或分析节点：

- `import_df_url(URL)` 从 HTTP(S) 读取远程 CSV、Excel 或 Parquet DataFrame，请求超时
  30 秒且响应上限为 64 MiB；
- `import_dss*`、`import_bmopf*` 从 DSS 或 BMOPF source 产生固定位置的网络表；
- `ds_*_dss_plan*` 直接读取 DSS source 并运行受支持的执行计划。

获得至少一张表后，数值和字符串字面量才可通过 `select`、`with_columns` 等列表达式生成
或修改列。Tensor 字面量只能在第 9 节的 `{...}` 数值程序中使用，不能替代 RSDss 表输入。

这些都是 RSDss 表达式节点。`#include`/`#include_url` 只导入程序文本，不产生
DataFrame；DSS 文件内部的 `Redirect`/`Compile` 只由相应 DSS 导入入口处理。

### 7.4 名称解析和依赖

声明右值中的 DataFrame 名称按以下顺序理解：

1. 脚本中任意位置声明的变量；
2. `--input` 注入的同名 DataFrame；
3. 在 mems 流程中，入边 `DfActionEdge.name` 对应的 DataFrame；
4. 内嵌数值程序还可以在运行时从流程变量或量测上下文解析未由图内变量满足的自由变量。

声明图允许前向引用，但不允许重复声明或环。DataFrame 函数只接收明确出现在参数位置的表；
除兼容形式 `concat()` 等少数旧入口外，不会隐式消费当前上下文中的所有输入。为使依赖图
确定，智能体应始终显式列出所有输入表，并给每条流程入边使用唯一别名。

## 8. RSDss 输出契约

### 8.1 哪些声明成为输出

执行器为每个声明计算依赖层：不依赖其他脚本声明的节点位于第 0 层，消费其结果的节点位于
更深层。只有全图最大依赖层中的声明会作为终端输出。输出按声明节点 ID，也就是源程序中的
声明顺序稳定排列；同一个多结果声明内部再按左值顺序排列。

这条规则不等价于“输出所有没有消费者的变量”。一个与主链无关、但层数较浅的叶节点不会
返回。智能体应生成一张连通依赖图，并把所有需要返回的表放在同一个最终声明中，或保证它们
处于相同的最大层。mems 流程中的 `TE_PARA_DF` 计划校验更严格：整张图必须只有一个连通
分量，并且最大层只能有一个声明节点；该节点仍可通过多结果左值返回多张表。

终端结果必须是 DataFrame。`height`、`width` 等标量可以作为中间变量，但终端标量会产生
`InvalidExpression`。`_` 会占据并校验一个返回槽位，但不会形成输出。

### 8.2 输出名称和多结果

命令行 `--df` 模式使用终端声明的左值作为输出名：

```text
result = select(cols(bus, v_mag), voltage);
[bus_result, branch_result] = ds_radial_pf(...);
[summary, _] = some_two_result_function(...);
```

- 单表右值必须绑定一个名称；
- 多表右值返回数必须与方括号中的槽位数完全一致；
- 多结果不能直接作为另一个函数的单个参数，必须先解构；
- DataFrame 变量名只标识整张表，不会自动改写表内列名。

### 8.3 命令行呈现和进程状态

成功时先打印 `Run expr_df successfully`，随后对每张结果打印 `result NAME:` 和 Polars
表格文本。`--verbose` 在每张表后再打印 `schema NAME:`。该标准输出是供人查看的文本，
不是 CSV、JSON 或稳定的机器交换格式；业务程序需要结构化结果时应调用 Rust API，或把
脚本放入 mems 流程并由后续节点持久化。

自动化必须以退出码判断成功：`0` 表示全部输入、编译和求值成功，非零表示失败。诊断可能
写到标准错误、标准输出或日志，不能把某段提示文本当作成功条件。`--df` 当前没有输出文件
参数，也不向终端输出中间 DataFrame。

### 8.4 完整的命令行输入输出示例

给定 `profiles.csv`：

```csv
bus,p_kw,enabled
1,10,true
2,20,false
3,30,true
```

脚本 `case.rss`：

```text
selected = filter(
    col(bus),
    alias(col(p_kw) * 1000, p_w),
    col(enabled),
    profile
);
rows = height(selected);
result = with_columns(alias(rows, selected_rows), selected);
```

运行：

```bash
tensoreval --df --input profile=profiles.csv --verbose case.rss
```

`profile` 是外部输入，`selected` 和 `rows` 是中间值，`result` 是唯一最大层 DataFrame，
因此唯一终端结果名为 `result`，列为 `bus,p_w,selected_rows`，行数为 2。

## 9. 内嵌 RustScript 数值程序

完整的 RustScript 词法、数据类型、张量运算、控制流和函数定义语法见
[《RustScript 手册》](../../rustscript/RustScript手册.md)。该手册是语言语法参考；RSDss
内嵌代码块只开放下述计算子集，并不提供手册所处完整运行环境中的全部原生函数。

### 9.1 声明与返回值

RustScript 解析器支持把代码块作为声明右值：

```text
result = {
    value = input * 2;
    return value;
};
```

也可以把元组返回值绑定到多个名称：

```text
[active, reactive] = {
    p = input * 0.95;
    q = input * 0.20;
    return (p, q);
};
```

返回值数量必须与左侧名称数量一致；左侧可以用 `_` 丢弃对应返回值。代码块可以使用局部
声明、`if/else`、`for`、`while`、`loop`、`break`、`continue`、局部函数和 `return`。

### 9.2 DataFrame 和标量输入

编译器分析 `{...}` 中的自由变量并建立依赖。局部声明和函数参数不是自由变量；日志函数、
允许的求解器函数等已注册原生函数也不是输入。每个实际引用的图内值按名称注入独立 VM：

- DataFrame 转成实数或复数的稠密 Tensor 或稀疏 Tensor；一行一列仍是形状 `[1,1]` 的 Tensor；
- RSDss 表达式产生的数值或布尔标量转成 VM 实数标量；
- 字符串、null 或其他不能转成 `f64` 的 RSDss 标量产生 `LiteralToTensor`；
- 只在 mems 流程中，仍未满足的自由变量可继续由流程变量和量测上下文解析。

DataFrame 转换遵循下表。转换会丢弃普通表的列名和 dtype，脚本不能按列名访问 Tensor：

| DataFrame 形态 | VM 数值和形状 |
|---|---|
| 普通 `m` 行、`n` 列表 | 稠密 Tensor `[m,n]`，保持行序和列序 |
| 单列名 `mat_D1_D2_...` | 按列名指定的形状恢复稠密 Tensor；维度乘积必须等于行数 |
| 单列名 `mul_D1_D2_...` | 每个单元格是一段同形 Tensor 文本，合并为 `[D1,D2,...,行数]` |
| 三列且首列名以 `sp_` 开头 | 稀疏三元组；首行给出 `[总行数,总列数,0]`，其余行为 `[row,column,value]` |

普通数值列、布尔列和可解析为数字的字符串优先转换为 `f64`，布尔值映射为 `1/0`。只要
普通字符串列包含不能解析为 `f64` 的文本，转换器就尝试把整张表提升到复数域，此时每个
字符串值都必须能解析为复数，否则转换失败。普通表中的 null 按零写入；需要区分缺失值时，
必须在进入代码块前用 RSDss 的 `filter`、`fill_null`、`is_null` 等函数显式处理。日期、
时间、列表、结构体或其他不支持的 dtype 会产生 `DataFrameToTensor`。

### 9.3 返回值到 DataFrame

代码块必须显式返回与左值槽位数相同的数值结果：

```text
one = { return tensor_value; };
[first, second] = { return (value1, value2); };
[kept, _] = { return (value1, value2); };
```

没有数值返回值、返回数不匹配或返回值不能转换时分别产生 `ScriptRuntime`、
`ResultCountMismatch` 或 `TensorToDataFrame`。每个结果独立转换：

| VM 返回值 | RSDss DataFrame |
|---|---|
| 实数标量 | 一行一列的 `f64` 表，列名是实现默认的空名称 |
| 复数标量 | 一行一列的字符串表，列名是实现默认的空名称 |
| 二维实数 Tensor | `m` 行、`n` 列，列名 `col_0` 到 `col_{n-1}` |
| 二维复数 Tensor | 同样展开为 `col_*`，单元格为复数字符串 |
| 非二维稠密 Tensor | 单列 `mat_D1_D2_...`，按内部迭代顺序展平 |
| 稀疏 Tensor | 三列 `sp__r,sp__c,sp__v` 坐标表，首行保存 `[总行数,总列数,0]` |

在 `TE_PARA_DF` 混合图和独立 `tensoreval --df` 中，二维结果总是优先展开为 `col_*`；
这一路径不读取 `TE_PARA_MATRIX`。转换不会恢复进入代码块前的列名。后续 RSDss 节点应使用
`select`/`alias` 显式建立业务列名。

每个代码块使用新 VM。脚本修改其全局或局部变量不会写回 RSDss 上下文；只有 `return`
绑定的结果会成为后续图变量。同一依赖层中的代码块也看不到彼此在该层产生的结果。

### 9.4 可用函数范围

在支持混合图的运行时中，自由 DataFrame 变量会转换为 Tensor，数值或布尔标量进入独立
VM，返回值再转换回 DataFrame。内嵌代码块可以使用：

- 不访问外部状态的数值、布尔、复数和 Tensor 表达式；
- 《RustScript 手册》中相应的控制流、局部变量和局部函数语法；
- `println`、`debug`、`info`、`warn`、`error` 和 `monotonic_ms`；
- 启用 `with_allsolvers` feature 且相应 eig-aoe backend 注册后提供的 `sp_linsolve`、
  `fsolve`、`fmincon` 和 `intlinprog`。

内嵌代码块不提供文件读写、目录访问、URL 下载、网络请求、数据库或其他运行时外部 I/O
函数。解析前的 `#include`/`#include_url` 仍可把函数定义文本展开到代码块中；执行期间不能
再动态加载模块。未列入上述范围的原生函数会在编译脚本节点时被拒绝。

当前内部名称表还包含 `sp_linsolvex`，但运行时没有注册同名函数，因此它不能实际调用，
不属于可用范围。

`import_dss`、`import_bmopf`、`import_df_url` 和 `ds_*` 等 RSDss DataFrame/DSA 函数应写在
代码块外部，作为普通 RSDss 声明节点调用。这样可以保留列名、类型、依赖关系和结构化错误。

### 9.5 运行入口和 I/O 限制

内嵌代码块可由独立 `tensoreval --df` 和 mems 的 `TE_PARA_DF` 节点执行。两者都只注册
日志函数和构建中启用的求解器函数，不注册 `read_tensor`、`read_tensor_url`、
`write_tensor` 或断言函数。数据和文件 I/O 必须放在代码块外部的 RSDss 节点或流程节点。

## 10. mems `TensorEval` 节点契约

### 10.1 参数位和执行分派

节点的 `para` 是位集合，可用按位或组合：

| 常量 | 值 | 作用范围 | 行为 |
|---|---:|---|---|
| `TE_PARA_MATRIX` | `0x02` | 非 DataFrame Tensor 图或整段 VM 程序 | 要求二维 Tensor，并展开为 `col_0...`；未设置时保存为单列 `mat_D1_D2_...` |
| `TE_PARA_DF` | `0x04` | 整个节点 | 按 RSDss 表达式/脚本混合图执行；二维脚本返回值始终展开 |
| `TE_PARA_MULTI_RETURN` | `0x08` | 整段 VM 程序及节点输出绑定 | 按配置的输出名称数量从 VM 读取多个返回值，并把多张表分别命名 |

当前只识别这三个参数位；`0x01` 和其他位没有公开语义，智能体不得设置。只需要 RSDss
DataFrame 图时 `para=4`；RSDss 节点发布多张结果表时通常使用 `para=12`。

分派顺序固定：设置 `TE_PARA_DF` 时进入 RSDss 混合图；否则，若程序全部是简单表达式声明，
进入 Tensor 依赖图；其余程序作为一个完整 `Prog` 在 VM 中执行。`TE_PARA_MATRIX` 和
`TE_PARA_MULTI_RETURN` 不会改变语言模式。

### 10.2 节点输入

每条入边的 `DfActionEdge.name` 是脚本看到的 DataFrame 变量名。节点收集所有入边结果为
名称映射，因此名称必须唯一。非 DataFrame 模式会尝试把所有入表转换成 VM 数值；
`TE_PARA_DF` 只在表达式或代码块实际引用某张表时消费它。

Tensor 表达式和 VM 执行还可通过运行时上下文读取流程变量和量测。它们是数值上下文，
不是带 schema 的 DataFrame。混合图代码块先使用图内 DataFrame/标量依赖，再由同一运行时
上下文解析其余自由变量。

### 10.3 节点输出名称

持久化模型中的 `names` 与脚本内 RSDss 左值是两层名称。RSDss 左值负责图内依赖；节点
完成后，流程运行时按以下规则发布结果：

| 配置 | 发布规则 |
|---|---|
| 未设置 `TE_PARA_MULTI_RETURN`，`names` 为空 | 必须恰好返回一张表，以空结果别名发布 |
| 未设置 `TE_PARA_MULTI_RETURN`，`names` 非空 | 必须返回一张表且列数等于名称数；运行时尝试把 `col_i` 重命名为对应名称 |
| 设置 `TE_PARA_MULTI_RETURN` | `names` 必填；返回表数必须等于名称数，每张表按对应名称发布 |

因此“给一张结果表命名”和“给一张矩阵的各列命名”不是同一操作。多表输出要设置
`TE_PARA_MULTI_RETURN`；单表列名要在 RSDss 中优先用 `alias` 明确建立，只有兼容旧 Tensor
节点时才依赖 `names` 的 `col_i` 重命名。

### 10.4 各运行时的原生函数差异

| 运行时 | 日志函数 | 文件/URL Tensor I/O | 断言 | 可选数值求解器 |
|---|---|---|---|---|
| `tensoreval --df` 的 `{...}` | 是 | 否 | 否 | `with_allsolvers` 时可用 |
| `TensorEval` + `TE_PARA_DF` 的 `{...}` | 是 | 否 | 否 | `with_allsolvers` 时可用 |
| `TensorEval` 整段 VM 程序 | 是 | 是 | 否 | `with_allsolvers` 时可用 |
| `tensoreval` 普通 VM 模式 | 是 | 是 | 是 | `with_allsolvers` 时可用 |

日志函数是 `println`、`debug`、`info`、`warn`、`error`、`monotonic_ms`；Tensor I/O 是
`read_tensor`、`read_tensor_url`、`write_tensor`。求解器集合是 `sp_linsolve`、`fsolve`、
`fmincon` 和 `intlinprog`。`sp_linsolvex` 当前出现在允许名称表中但没有同名运行时注册，
不能作为可用能力。

## 11. 普通 `tensoreval` Tensor/VM 模式

不带 `--df` 时，位置文件仍按顺序拼接并展开 include，但 `--input` 不可用。全为简单表达式
声明的程序编译成 Tensor 依赖图，输出最大层数值的调试表示；含控制流、函数或其他脚本
语句的程序由一个 VM 执行，`return` 的单个可选数值以调试表示打印。

普通 VM 模式可以调用上表的日志、Tensor I/O、断言和可选求解器函数。`read_tensor` 按文件
扩展名读取 CSV、Excel、Parquet，或把其他文本解析为 Tensor 表达式；`read_tensor_url`
提供对应 URL 行为；`write_tensor` 支持 `.xlsx`、`.parquet`、`.csv`、`.txt` 或无扩展名。
这些函数处理 Tensor 编码表，不提供 RSDss 的列语义或 DSA DataFrame 函数。

`--verbose` 在普通模式打印 VM 统计，而不是 DataFrame schema。`--complex` 当前不会选择另一
套执行器；实数或复数域由表达式值自行决定。`--dir` 和 `--url` 当前是保留参数，不参与程序
加载或求值。智能体不应生成依赖这三个参数改变计算语义的命令。

普通模式保留一个旧兼容行为：VM 的 `execute` 返回运行错误时，工具打印
`Failed to run prog`，但当前不会显式设置非零退出码。因此只有 `--df` 模式可以把退出码
作为完整的执行成功判据；普通模式的自动化调用还必须检查该失败提示或改用直接返回
`Result` 的 Rust API。

## 12. 错误边界

RSDss 保留以下错误类别：

- `InvalidExpression`：声明、依赖或表达式结构不合法；
- `UnsupportedFunction`：当前 feature 没有该函数；
- `InvalidArgumentCount`：参数不足或数量不匹配；
- `ResultCountMismatch`：多返回值绑定数量错误；
- `MissingDataFrame`：引用的表不存在；
- `DataFrameToTensor`：代码块引用的 DataFrame 不能转换成数值 Tensor；
- `LiteralToTensor`：代码块引用的中间标量不能转换成 VM 实数标量；
- `ScriptRuntime`：内嵌代码块编译后执行失败或没有返回所需数值；
- `TensorToDataFrame`：代码块的某个返回值不能转换回 DataFrame；
- `OperationFailed`：参数语义或底层分析失败；
- `Polars`：列、类型或 DataFrame 操作失败。

`tensoreval --df` 在输入、解析、编译或执行失败时返回非零退出码。部分诊断来自日志或旧
兼容路径，可能出现在标准输出或标准错误；自动化调用必须检查退出码，并把两条输出流都
保留用于诊断。普通 Tensor/VM 模式的运行错误退出码例外见第 11 节。
