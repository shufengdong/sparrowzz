# RSDss 函数参考

本文记录 `tensoreval --dataframe` 当前实际注册的 DataFrame 函数。顶层调用先由
`frame_function.rs` 或 `frame_ops.rs` 分派；嵌套表达式由 `translate.rs` 转为 Polars
表达式。函数名在顶层分派时转成小写。

## 1. 单表函数

单表函数的最后一个参数是输入 DataFrame，前面的参数是表达式。

| 函数 | 签名 | 行为 |
|---|---|---|
| `height` | `height(frame)` | 返回行数标量 |
| `width` | `width(frame)` | 返回列数标量 |
| `select` | `select(expr..., frame)` | 选择或计算列；至少一个表达式 |
| `filter` | `filter(select_expr..., predicate, frame)` | 按最后一个表达式过滤，再选择前面的表达式 |
| `with_column` | `with_column(expr, frame)` | 增加或替换一列 |
| `with_columns` | `with_columns(expr..., frame)` | 增加或替换多列 |
| `group_by` | `group_by(key..., frame)` | 按键分组并聚合全部列 |
| `agg` | `agg(group_by(key...), aggregate..., frame)` | 显式分组聚合 |
| `drop_nulls` | `drop_nulls(frame)` | 删除含 null 的行 |
| `drop_nans` | `drop_nans(frame)` | 删除含 NaN 的行 |
| `fill_null` | `fill_null(value, frame)` | 填充 null |
| `fill_nan` | `fill_nan(value, frame)` | 填充 NaN |
| `count` | `count(frame)` | 各列计数 |
| `null_count` | `null_count(frame)` | 各列 null 数 |
| `first` / `last` | `first(frame)` / `last(frame)` | 每列首值/末值 |
| `reverse` | `reverse(frame)` | 反转行顺序 |
| `cast_all` | `cast_all(type, frame)` | 全表转换类型 |
| `limit` | `limit(n, frame)` | 取前 `n` 行；`n` 必须是数值字面量 |
| `tail` | `tail(n, frame)` | 取后 `n` 行；`n` 必须是数值字面量 |
| `sum` / `min` / `max` | `sum(frame)` 等 | 按列聚合 |
| `mean` / `median` | `mean(frame)` 等 | 按列聚合 |
| `shift` | `shift(periods, frame)` | 整表位移 |
| `shift_and_fill` | `shift_and_fill(fill, periods, frame)` | 位移并填充值；实现按末两个表达式依次弹出 |

别名同时可用：`withcolumn`、`withcolumns`、`groupby`、`dropnulls`、`dropnans`、
`fillnull`、`fillnan`、`nullcount`、`castall`、`aggregate`、`shiftandfill`。

### `filter` 示例

```text
result = filter(
    col(bus),
    col(phase),
    col(v_mag),
    col(v_mag) > 1000,
    voltage
);
```

若只想保留全部原列，应显式使用 `all()` 作为选择表达式。

## 2. 整表函数

### `join`

```text
joined = join(left, right, left_columns, right_columns, how);
```

固定 5 个参数。列参数可以是单个名称、逗号分隔字符串或名称数组。当前实现只接受
`inner`、`left`、`right` 和 `full` 四种 `how`；其他值返回 `Unsupported join type`。

### `sort`

```text
sorted = sort(by, descending, maintain_order, frame);
```

- `by`：一个或多个列名；
- `descending`：与列对应的字符串/数组，`true/down/descending/default` 为降序，
  `false/up/ascending` 为升序；
- `maintain_order`：`true` 或 `maintain_order` 时保持稳定顺序。

还存在只有一个输入 DataFrame 时的三参数隐式形式。新脚本应使用四参数显式形式。

### `concat`

```text
combined = concat("vertical", frame1, frame2, frame3);
```

当前实现有两条显式分支：

| 模式 | 行为 |
|---|---|
| `vertical` | 相同 schema 纵向拼接 |
| `diagonal` | 按列名对角拼接，缺列填 null |
| `horizontal` | 横向拼接 |

除 `diagonal` 和 `horizontal` 外，当前实现走普通纵向拼接路径；它不会提供
`vertical_relaxed` 的类型放宽或 `align` 的公共列对齐语义。新脚本应只写上述三种明确
模式。第一个参数固定为模式槽。无参数形式会按变量名排序后拼接上下文中的所有
DataFrame，依赖范围过大，不应用于仿真脚本。

### `pivot`

```text
wide = pivot(on, index, values, aggregate, maintain_order, separator, frame);
```

固定 7 个参数。`on/index/values` 接受一个或多个列名，`aggregate` 是 Polars 聚合表达式，
`maintain_order` 是字符串布尔值，`separator` 用于生成列名。

### `unpivot`

```text
long = unpivot(on, index, frame);
```

显式形式固定 3 个参数。只有一个输入表时还支持隐式列名形式，新脚本应使用显式形式。

### `import_df_url`

```text
frame = import_df_url("https://example.test/input.csv");
```

固定一个 URL 参数，下载后由 DataFrame 文件读取器解析。远程失败返回结构化执行错误。

## 3. 列表达式函数

以下清单以 `translate` 的实际执行分支为准，不包含外层单表函数末尾的 DataFrame 参数。
同名函数可能同时存在于顶层和表达式层，例如 `sort(by,...,frame)` 排序整张表，
`sort(expr,direction)` 排序一个表达式。

### 选择器和特殊表达式

| 函数 | 行为 |
|---|---|
| `all()` | 选择全部列 |
| `pi()` | 产生 π 字面量 |
| `element()` | `pivot` 等列表/聚合上下文中的当前元素 |
| `col(name)` | 按字符串字面量读取一列 |
| `cols(name...)` | 按一个或多个字符串字面量选择多列，至少一个参数 |
| `group_by(key...)` | 仅作为顶层 `agg/aggregate` 的第一个参数标记分组键；不单独进入普通表达式求值 |

### 行、计数和唯一性

| 函数 | 行为 |
|---|---|
| `reverse(expr)` | 反转表达式的行顺序 |
| `count(expr)` | 非 null 值计数 |
| `null_count(expr)` | null 值计数 |
| `len(expr)` | 当前分组/上下文长度 |
| `first(expr)`, `last(expr)` | 首值、末值 |
| `n_unique(expr)` | 唯一值数量 |
| `unique(expr)` | 唯一值序列 |
| `arg_unique(expr)` | 唯一值首次出现位置 |
| `arg_min(expr)`, `arg_max(expr)` | 最小值、最大值的位置 |
| `is_unique(expr)` | 每行是否只出现一次 |
| `is_duplicated(expr)` | 每行是否为重复值 |
| `head(expr,n)`, `tail(expr,n)` | 取表达式头/尾 `n` 项；`n` 必须是数值字面量 |

### null、NaN 和布尔

| 函数 | 行为 |
|---|---|
| `not(expr)` | 布尔取反 |
| `not_null(expr)`, `is_null(expr)` | 非 null/null 判定 |
| `drop_nulls(expr)` | 从表达式中移除 null |
| `not_nan(expr)`, `is_nan(expr)` | 非 NaN/NaN 判定 |
| `drop_nans(expr)` | 从表达式中移除 NaN |
| `any_horizontal(expr)` | 对传入的 selector 表达式做横向 any |

### 比较、命名和类型

| 函数 | 行为 |
|---|---|
| `eq(left,right)`, `neq(left,right)` | 相等、不等比较 |
| `eq_missing(left,right)`, `neq_missing(left,right)` | 带 missing/null 等价规则的相等、不等比较 |
| `lt(left,right)`, `gt(left,right)` | 小于、大于 |
| `lt_eq(left,right)`, `gt_eq(left,right)` | 小于等于、大于等于 |
| `alias(expr,name)` | 把表达式命名为字符串字面量 `name` |
| `cast(expr,type)` | 转为第 4 节列出的类型 |
| `when_then_otherwise(predicate,yes,no)` | 条件表达式 |

### 聚合和统计

| 函数 | 行为 |
|---|---|
| `sum(expr)`, `min(expr)`, `max(expr)` | 求和、最小值、最大值；裸字符串参数按列名解释 |
| `mean(expr)`, `avg(expr)`, `median(expr)` | 均值、中位数；`avg` 是 `mean` 的别名 |
| `nan_min(expr)`, `nan_max(expr)` | 保留 NaN 规则的最小值、最大值 |
| `min_horizontal(expr)`, `max_horizontal(expr)` | 对 selector 展开的列做横向最小/最大值 |
| `sum_horizontal(expr)` | 对 selector 展开的列做横向求和，当前固定 `ignore_nulls=false` |
| `std(expr,ddof)`, `var(expr,ddof)` | 标准差、方差；`ddof` 必须是可转为 `u8` 的数值字面量 |
| `dot(left,right)` | 点积 |

### 排序、筛选、位移和累计

| 函数 | 行为 |
|---|---|
| `sort(expr,direction)` | 排序表达式；`true/down/descending` 降序，`false/up/ascending` 升序，其他值也按降序 |
| `filter(expr,predicate)` | 在表达式级按 predicate 过滤 `expr` |
| `shift(expr,periods)` | 位移；`periods` 必须是数值字面量 |
| `shift_and_fill(expr,periods,fill)` | 位移并用 `fill` 表达式填充；`periods` 必须是数值字面量 |
| `cum_count(expr,reverse)` | 累计计数 |
| `cum_sum(expr,reverse)` | 累计求和 |
| `cum_min(expr,reverse)`, `cum_max(expr,reverse)` | 累计最小值、最大值 |
| `cum_prod(expr,reverse)` | 累计乘积 |

累计函数的 `reverse` 是字符串/布尔字面量；仅值为 `true` 时反向累计。

### 数学函数

| 函数 | 行为 |
|---|---|
| `abs(expr)`, `sign(expr)` | 绝对值、符号 |
| `exp(expr)`, `log1p(expr)` | 指数、`ln(1+x)` |
| `log(expr,base)` | 以表达式 `base` 为底的对数 |
| `pow(expr,exponent)` | 幂；指数必须是数值字面量 |
| `sqrt(expr)`, `cbrt(expr)` | 平方根、立方根 |
| `sin`, `cos`, `tan`, `cot` | `fn(expr)`，输入按弧度 |
| `arcsin`, `arccos`, `arctan` | `fn(expr)`，结果按弧度 |
| `sinh`, `cosh`, `tanh` | 双曲函数 `fn(expr)` |
| `arcsinh`, `arccosh`, `arctanh` | 反双曲函数 `fn(expr)` |
| `degrees(expr)`, `radians(expr)` | 弧度转角度、角度转弧度 |
| `floor(expr)`, `ceil(expr)` | 向下、向上取整 |
| `round(expr,decimals)` | 按小数位数取整；`decimals` 必须是可转为 `u32` 的数值字面量 |
| `arange(start,end[,step[,dtype]])` | 生成范围；默认 `step=1`、`dtype=i32`，step 和 dtype 必须是字面量 |

### 字符串函数

| 函数 | 行为 |
|---|---|
| `split(expr,by)` | 按表达式 `by` 分割字符串 |
| `starts_with(expr,prefix)`, `ends_with(expr,suffix)` | 前缀、后缀判定 |
| `contains(expr,pattern[,literal])` | 包含匹配；`literal` 默认为 true，false 时按正则模式 |
| `replace(expr,pattern,value[,literal])` | 替换第一个匹配；`literal` 默认为 true，false 时按正则模式 |
| `find(expr,pattern[,strict])` | 返回匹配位置；`strict` 默认为 true |
| `slice(expr,offset,length)` | 字符串切片，不是 DataFrame 行切片 |
| `to_lowercase(expr)`, `to_uppercase(expr)` | 转为小写、大写 |

### 只在 arity 校验器中出现的名称

`sum_col`、`min_col`、`max_col`、`mean_col`、`avg_col`、`median_col` 当前会通过
`is_fun_arg_num_ok`，但 `translate` 没有对应执行分支，调用会返回
`Unimplemented function`。它们不属于可用函数，文档和机器 catalog 不把它们列入
supported 集合。

## 4. 类型名称

`cast`/`cast_all` 支持基础类型短名及时间类型，包括：

```text
i8 i16 i32 i64 i128
u8 u16 u32 u64 u128
f32 f64
str bool binary null unknown
date time
datetime[ms] datetime[μs] datetime[ns]
duration[ms] duration[μs] duration[ns]
```

`datetime[unit,timezone]` 还可携带 Chrono 能解析的时区。微秒单位使用字符 `μs`；写成
`us` 会回退到毫秒。未知类型会回退为 Polars `Unknown(Any)`，因此脚本生成器应只使用
参考中列出的类型。

## 5. 列名与 DataFrame 名的上下文

```text
col(bus)       // 读取 bus 列
alias(1, bus)  // 生成名为 bus 的列
select(..., bus) // 最后一个 bus 是 DataFrame 变量
```

相同裸名称在不同参数位置具有不同含义。生成脚本时应优先保留清楚的变量名，例如
`bus_table`、`voltage_result`，并用 `col(...)` 显式访问列。
