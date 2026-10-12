# RSDss（`.rss`）语言参考

RSDss 是面向 DataFrame 和配电系统分析的 RustScript 方言。`.rss` 是本目录采用的脚本
扩展名；`tensoreval --dataframe`（简写 `--df`）先用 `eig-expr` RustScript 解析器读取
程序，再把顶层声明编译为 RSDss 依赖图。RSDss 文件不是 Rust 源码，也不是 OpenDSS
命令文件。

本文说明 RSDss 的词法、赋值、DataFrame 函数和列表达式语义。每个函数的精确参数和行为见
[RSDss 函数参考](rsdss_function_reference.md)，DSA 入口见
[DSA 函数参考](dsa_function_reference.md)。

## 1. 执行模式与语法边界

RSDss DataFrame 模式通过以下命令启用：

```bash
tensoreval --df case.rss
```

顶层只接受具名声明；每个声明产生 DataFrame、多张 DataFrame 或供后续节点使用的标量。
`if`、`for`、`while`、`loop`、`fn`、`break`、`continue` 和 `return` 属于普通 RustScript
数值程序语法，不能作为当前 `tensoreval --df` 的顶层语句。

普通 tensoreval 模式执行 Tensor/VM 程序，可以使用 RustScript 控制流，但不能调用
`import_dss`、`ds_*` 等 RSDss DataFrame 入口。两种模式共用表达式解析器，不共用函数
集合和运行时值类型。

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
`#include_url` 要求运行环境允许网络访问，其返回内容不继续扫描 include。DSS 文件中的
`Redirect`/`Compile` 由 DSS 导入器处理，与 RSDss include 是两套机制。

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
ds_radial_pf ds_newton_raphson_pf
ds_radial_qsts ds_newton_raphson_qsts
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

## 7. 外部输入和终端输出

外部 CSV、Excel 或 Parquet 可通过重复的 `--input NAME=FILE` 注入：

```bash
tensoreval --df \
  --input load_profile=profiles.csv \
  --input settings=settings.parquet \
  case.rss
```

输入名称必须唯一，并作为已存在的 DataFrame 变量参与依赖解析。未被其他声明消费的末层
DataFrame 是具名终端输出；tensoreval 按稳定节点顺序打印名称和内容。`--verbose` 额外
打印 schema。

## 8. 内嵌 RustScript 数值程序

完整的 RustScript 词法、数据类型、张量运算、控制流和函数定义语法见
[《RustScript 手册》](../../rustscript/RustScript手册.md)。该手册是语言语法参考；RSDss
内嵌代码块只开放下述计算子集，并不提供手册所处完整运行环境中的全部原生函数。

### 8.1 声明与返回值

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

### 8.2 可用范围

在支持混合图的运行时中，自由 DataFrame 变量会转换为 Tensor，数值或布尔标量进入独立
VM，返回值再转换回 DataFrame。内嵌代码块可以使用：

- 不访问外部状态的数值、布尔、复数和 Tensor 表达式；
- 《RustScript 手册》中相应的控制流、局部变量和局部函数语法；
- `println`、`debug`、`info`、`warn`、`error` 和 `monotonic_ms`；
- 启用 `with_allsolvers` feature 且相应 eig-aoe backend 注册后提供的 `sp_linsolve`、
  `fsolve`、`fmincon` 和 `intlinprog`。

内嵌代码块不提供文件读写、目录访问、URL 下载、网络请求、数据库或其他外部 I/O 函数，
也不能依赖通过文件或网络加载的模块。未列入上述范围的原生函数会在编译脚本节点时被拒绝。

当前内部名称表还包含 `sp_linsolvex`，但运行时没有注册同名函数，因此它不能实际调用，
不属于可用范围。

`import_dss`、`import_bmopf`、`import_df_url` 和 `ds_*` 等 RSDss DataFrame/DSA 函数应写在
代码块外部，作为普通 RSDss 声明节点调用。这样可以保留列名、类型、依赖关系和结构化错误。

### 8.3 运行入口限制

内嵌代码块属于 mems 的 DataFrame/数值混合运行时能力。当前独立 `tensoreval --df` 使用
纯表达式图入口，只接受普通变量声明，不能执行这种代码块。

## 9. 错误边界

RSDss 保留以下错误类别：

- `InvalidExpression`：声明、依赖或表达式结构不合法；
- `UnsupportedFunction`：当前 feature 没有该函数；
- `InvalidArgumentCount`：参数不足或数量不匹配；
- `ResultCountMismatch`：多返回值绑定数量错误；
- `MissingDataFrame`：引用的表不存在；
- `OperationFailed`：参数语义或底层分析失败；
- `Polars`：列、类型或 DataFrame 操作失败。

tensoreval 把错误写到标准错误并返回非零退出码。自动化调用必须检查退出码，不能只搜索
标准输出中的结果文本。
