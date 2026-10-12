# RustScript 手册

本文按当前 `eig-expr` 解析器、表达式求值器、脚本 VM 和 `tensoreval` 实现编写。带 feature 条件的能力只有在相应构建中可用。

RustScript 面向数值计算，支持实数、复数、稠密张量、二维稀疏矩阵、控制流和用户函数。它借用了 Rust 与 MATLAB 的部分外观，但不兼容任一门语言。

RSDss 的 `{ ... }` 内嵌块只开放 RustScript 的受限计算子集，范围见 [RSDss 语言参考](../dss/docs/rss_language_reference.md#8-内嵌-rustscript-数值程序)。

## 1. 运行方式

```bash
tensoreval program.rss
tensoreval constants.rss calculation.rss
```

多个文件按命令行顺序拼接后解析。`--verbose` 打印 VM 统计信息。`--complex` 当前只是兼容参数；实数或复数域由输入和运算自动决定，该参数不改变执行路径。

`--df` 或 `--dataframe` 启用 RSDss DataFrame 模式，见 [RSDss 语言参考](../dss/docs/rss_language_reference.md)。

### 1.1 普通模式的两条执行路径

1. 顶层全部为单结果变量声明时，优先构建表达式依赖图。它允许前向引用，拒绝循环依赖，并输出未被其他节点消费的终端变量。
2. 出现函数、控制流、调用语句、`return`、多结果声明，或表达式图构建失败时，进入脚本 VM 并按语句顺序执行。

```rustscript
// 依赖图允许前向引用
result = intermediate * 2;
intermediate = 10 + 5;
```

需要顺序副作用或更新变量时应进入 VM：

```rustscript
x = 1;
x = x + 1;
println(x);
```

## 2. 词法和语句

### 2.1 标识符

脚本声明解析器接受 Unicode 字母、`_`、`$` 和 `\uXXXX`。数学表达式层的裸名称只接受 ASCII 字母、数字、`_`、`$`，且首字符不能是数字。为保证各层一致，新脚本应使用：

```text
[A-Za-z_$][A-Za-z0-9_]*
```

### 2.2 数值、复数和常量

数值统一为 `f64`，支持小数和科学计数法。负号是前缀运算符；应写 `0.5`，不能写 `.5`。

```rustscript
a = 42;
b = 1.23e-4;
z1 = 2 + 3i;
z2 = c(2, 3);
```

数字后缀 `i`、`j` 产生纯虚数；未被变量覆盖时，裸 `i`、`j` 是虚数单位。

| 名称 | 含义 |
|---|---|
| `pi`, `PI` | 圆周率 |
| `e` | 自然常数 |
| `eps` | `f64::EPSILON` |
| `NAN` | 非数 |
| `INF`, `NEG_INF` | 正、负无穷 |
| `i`, `j` | 虚数单位 |

语言没有 `const`。大写名称没有只读语义。

### 2.3 字符串

传给 VM 原生函数的字符串使用反引号：

```rustscript
println(`calculation started`);
data = read_tensor(`data/input.csv`);
```

双引号和单引号在数学表达式层会生成名称词元，不是可靠的 VM 字符串字面量。例如 `println("hello")` 会查找名为 `hello` 的变量。反引号主要用于原生函数参数；当前没有完整的字符串运算、插值或转义系统。

### 2.4 注释和分号

```rustscript
// 单行注释
x = 1; /* 块注释可以跨行 */
```

块注释不能嵌套。换行在表达式中是空白，不能可靠分隔语句；变量声明和调用应始终用分号结束。解析器也接受 `:` 赋值，新代码统一使用 `=`。

## 3. 值类型

当前运行时值包括：

- `f64` 实数标量和 `Complex64` 复数标量；
- 任意维度的稠密实数或复数张量；
- 二维稀疏实数或复数矩阵；
- 控制流使用的 VM 布尔值；
- 有限用途的字符串、函数、表达式和空值对象。

没有整数、结构体、字典、枚举或用户类型声明。位运算要求数值能无损表示为有限 `i64`。

### 3.1 张量

```rustscript
empty = [];
vector = [1, 2, 3];
row = [[1, 2, 3]];
column = [[1], [2], [3]];
matrix = [[1, 2], [3, 4]];
cube = [[[1, 2], [3, 4]], [[5, 6], [7, 8]]];
```

同层元素必须形成一致形状。出现复数元素时，整个值提升到复数域。

### 3.2 稀疏矩阵

`sparse(rows, columns, values, row_count, column_count)` 用零基坐标创建二维稀疏矩阵：

```rustscript
s = sparse([0, 1, 2], [1, 0, 1], [2, 3, 6], 3, 2);
dense = full(s);
```

坐标和值向量必须等长且不越界。`issparse(x)` 返回 `1` 或 `0`；`full(x)` 转为稠密张量。

## 4. 变量和函数

声明不使用 `let`、`mut` 或类型注解。在 VM 中再次赋值会更新绑定；纯声明图遇到重复名称会构图失败并回退到 VM。

```rustscript
x = 1;
x = x + 1;
matrix = ones(2, 3);
```

函数使用 `fn`，声明会提升，因此可以先调用后声明：

```rustscript
value = square(4);
fn square(x) {
    return x * x;
}
println(value);
```

函数可以返回多个值：

```rustscript
fn split_power(s) {
    return (s * 0.95, s * 0.20);
}
[active, reactive] = split_power(100);
```

多结果必须由 `[name1, name2, ...]` 接收。`return;` 表示无返回值。顶层 CLI 只展示一个可选结果，多返回值主要用于函数绑定。函数可读取父作用域变量；没有默认参数、命名参数、可变参数、方法或匿名函数。

## 5. 控制流

### 5.1 条件

```rustscript
if voltage > 1.05 {
    state = 1;
} else if voltage < 0.95 {
    state = -1;
} else {
    state = 0;
}
```

条件只把实数标量大于 `0` 视为真；零、负数、非标量和其他对象为假。复数条件使用实部。

### 5.2 循环

```rustscript
i = 0;
sum_value = 0;
while i < 4 {
    sum_value = sum_value + i;
    i = i + 1;
}

for i in 0..4 {
    println(i);
}
for i in 0..=3 {
    println(i);
}

loop {
    break;
}
```

范围循环以 `1` 为固定步长；`..` 为半开区间，`..=` 包含终点。降序循环应使用 `while`。

解析器接受 `for i = 0; i < 10; i + 1 { ... }`，但第三段只求值，不会把结果写回 `i`。当前 `continue` 直接跳到循环起点，会跳过 C 风格循环更新段，也会跳过范围循环尾部的自动 `+1`。应优先使用不含 `continue` 的范围循环，或用 `while` 明确更新变量。

## 6. 运算符

| 类别 | 运算符 | 语义 |
|---|---|---|
| 前缀 | `+x`, `-x` | 正号、负号 |
| 前缀 | `!x` | 标量阶乘；`x!` 不合法 |
| 前缀 | `~~x`, `~x` | 逻辑非、位取反 |
| 后缀 | `x'` | 转置，不共轭 |
| 幂 | `^`, `.^` | 矩阵/标量幂、逐元素幂 |
| 乘除 | `*`, `/`, `\`, `%` | 乘法、右除、左除、余数 |
| 逐元素 | `.*`, `./` | 逐元素乘、除 |
| 算术 | `+`, `-` | 加、减 |
| 移位 | `<<`, `>>` | 标量整数移位 |
| 比较 | `<`, `>`, `<=`, `>=`, `==`, `!=` | 逐元素比较 |
| 位运算 | `&`, `^^`, `|` | 标量整数位与、异或、位或 |
| 位读取 | `@` | 按从 `1` 开始的位序号读取一位 |
| 逻辑 | `&&`, `||` | 逐元素逻辑与、或 |

幂右结合，其他二元运算符左结合。前缀正负号优先级低于幂，所以 `-x^2` 是 `-(x^2)`。

`*` 在两个非标量值间执行矩阵乘法，标量参与时执行缩放。`.*`、`./`、`.^` 逐元素计算。`/` 在标量参与时为普通除法；矩阵间求解 `X * right = left`。`\` 求解 `left * X = right`。矩阵除法、矩阵负一次幂和线性代数函数需要 MKL 或 MUMPS/OpenBLAS 构建。矩阵求逆写成 `a ^ -1`，没有 `inv(a)`。

比较和逻辑运算返回实数结果；复数比较使用实部。逻辑规则是“大于零为真”。二元位运算只接受可表示为 `i64` 的标量，复数取实部；`~` 可逐元素处理张量和稀疏值。

## 7. 内置数值函数

以下名称来自 `eig-expr/src/expr_tensor.rs` 当前分发表。

### 7.1 数学与复数

| 类别 | 函数 |
|---|---|
| 基础一元 | `abs`, `sqrt`, `exp`, `ln`, `log10`, `floor`, `ceil`, `round`, `signum` |
| 三角 | `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2` |
| 双曲 | `sinh`, `cosh`, `tanh`, `asinh`, `acosh`, `atanh` |
| 角度 | `deg2rad`, `rad2deg` |
| 极值与幂 | `max`, `min`, `power` |
| 复数构造 | `c`, `c1`, `c2` |
| 复数分解 | `real`, `imag`, `conj`, `angle`, `rad` |
| 复数矩阵 | `ctranspose`, `max_im`, `min_im`, `max_abs`, `min_abs` |

`c(real, imag)` 用实部和虚部构造复数；`c1(magnitude, radians)` 使用弧度；`c2(magnitude, degrees)` 使用角度。`power(base, exponent)` 逐元素计算。

启用 `with_rand` 后提供 `rand()` 和 `rand2(lower, upper)`。默认 `tools` feature 会启用 `with_rand`，精简构建不一定包含。

### 7.2 创建、拼接与形状

| 函数 | 行为 |
|---|---|
| `zeros(d1, ...)`, `ones(d1, ...)` | 创建指定形状的全零、全一张量 |
| `eye(n)` | 创建单位矩阵 |
| `range(start, end[, step])` | 左闭右开序列，默认步长 `1` |
| `horzcat`, `vertcat` | 水平、垂直拼接兼容的一维或二维值 |
| `blockcat(block_rows, block_columns, blocks...)` | 按给定块网格的行列数拼接块矩阵 |
| `ndim(x)` | 标量为 `1`，张量为维数，稀疏矩阵为 `2` |
| `size(x)` / `size(x, dim)` | 形状向量或指定零基维度长度 |
| `length(x)` | 最大维度长度；稀疏值沿用稀疏实现 |

形状参数必须可转换为索引，`range` 步长不能为零。

### 7.3 索引和更新

所有索引从 `0` 开始。不支持 `a[0, 1]`，应使用：

| 函数 | 行为 |
|---|---|
| `get(x, i, ...)` | 按每维索引取单个元素 |
| `get_multi(x, indices)` | 按行主序扁平索引取多个元素 |
| `slice(x, spec1, ...)` | 连续切片 |
| `assign(x, value, spec1, ...)` | 返回替换切片后的新值 |
| `select(x, rows[, columns])` | 按显式行列集合选取二维矩阵 |
| `select_assign(x, value, rows, columns)` | 返回替换显式行列集合后的矩阵 |
| `set(x, indices, values)` | 按扁平索引覆盖，返回新值 |
| `set2(x, indices, values)` | 按扁平索引累加，返回新值 |

`slice` 描述规则：标量选一个索引并移除该维；`[]` 插入新维；`[start]` 从起点到末尾；`[start,end]` 左闭右开；`[start,end,step]` 增加步长。旧手册所说的 `[0]` 选整维，是因为它表示从 `0` 到末尾。

```rustscript
a = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
element = get(a, 1, 2);
column = slice(a, [0], 1);
submatrix = select(a, [0, 2], [0, 2]);
changed = assign(a, [-2, -5, -8], [0], 1);
picked = get_multi(a, [0, 4, 8]);
```

更新函数不原地修改输入。需要写 `a = assign(a, ...)`、`a = set(a, ...)` 或 `a = select_assign(a, ...)`。

### 7.4 归约、查找与稀疏转换

| 函数 | 行为 |
|---|---|
| `sum(x[, dim_or_dims])`, `sum_all(x)` | 按维或全部求和 |
| `max(x, ...)`, `min(x, ...)` | 极值或指定范围极值 |
| `any(x[, dim])` | 检查非零元素 |
| `find(x)` | 非零元素位置；稀疏输入保留稀疏表示 |
| `find_nearest`, `find_nearest_pos` | 最近值、最近值位置 |
| `is_empty`, `is_inf` | 检查空值、逐元素检查无穷 |
| `diag` | 提取对角或从向量创建对角矩阵 |
| `transpose`, `ctranspose` | 转置、共轭转置 |
| `sparse`, `full`, `issparse` | 创建稀疏、转稠密、检查稀疏存储 |

### 7.5 条件线性代数函数

MKL 或 MUMPS/OpenBLAS feature 启用时注册：

```text
trace norm_l1 norm_l2 norm_max det svd rank eig
linsolve linsolve_t linsolve_h
```

`linsolve(a,b)` 求解 `a*x=b`；`linsolve_t` 求解转置系统；`linsolve_h` 求解共轭转置系统。后端未启用时这些名称作为未知函数失败。

### 7.6 张量操作组合示例

下面的例子覆盖旧手册中分散在“创建张量”“切片”“赋值”“拼接”和“稀疏矩阵”各节的典型操作：

```rustscript
a = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];

// 创建与形状
z = zeros(2, 3);
identity = eye(3);
indices = range(0, 3);
shape = size(a);                 // [3, 3]

// 读取：全部使用零基索引
one_value = get(a, 1, 2);        // 6
one_column = slice(a, [0], 1);   // [2, 5, 8]
corners = select(a, [0, 2], [0, 2]);

// 更新函数返回新值
b = select_assign(a, -1, [0, 2], [1]);
cumulative = set2(zeros(3), [0, 0, 2], [1, 2, 4]); // [3, 0, 4]

// 拼接与归约
blocks = blockcat(2, 2, eye(2), zeros(2, 1), ones(1, 2), [[2]]);
column_sum = sum(a, 0);
total = sum_all(a);              // 45

// 稀疏值可显式转回稠密值
s = sparse([0, 1, 2], [0, 1, 2], [1, 5, 9], 3, 3);
dense = full(s);

assert_eq(shape, [3, 3]);
assert_eq(cumulative, [3, 0, 4]);
assert_eq(total, 45);
```

`select_assign` 中行、列索引的顺序决定待写入矩阵的对应位置，并支持标量扩展。`set2` 会将重复索引对应的值累加；上例索引 `0` 出现两次，所以结果首项为 `3`。

## 8. `tensoreval` 原生函数

这些函数由 `mems` 的普通 `tensoreval` VM 注册，不属于独立 `eig-expr` API。

### 8.1 输出、日志和计时

```text
println debug info warn error monotonic_ms
```

前五个函数逐个输出参数。`monotonic_ms()` 返回进程内相对毫秒值，适合测量耗时，不是 Unix 时间戳。

### 8.2 文件和网络 I/O

```text
read_tensor(path)
read_tensor_url(url)
write_tensor(tensor_variable, path)
```

路径和 URL 使用反引号。读取函数支持 `.xls`、`.xlsx`、`.parquet`、`.csv` 数值表；其他后缀按包含单个数学表达式的 UTF-8 文本读取。失败时记录警告并返回空值。

`write_tensor` 支持 `.parquet`、`.xlsx`、`.csv`、`.txt` 和无后缀路径。第一个参数应是已绑定的张量或稀疏变量，不应传复杂临时表达式。

```rustscript
input = read_tensor(`data/input.csv`);
scaled = input * 2;
write_tensor(scaled, `output.csv`);
```

RSDss 内嵌块不注册这些 I/O 函数。

### 8.3 断言

| 函数 | 行为 |
|---|---|
| `assert_eq(a,b)` | 比较数值域、容器、形状和精确值 |
| `assert_relative_eq(a,b[,epsilon])` | 相对与同值绝对容差，默认 `1e-8` |
| `assert_abs_diff_eq(a,b[,epsilon])` | 绝对容差，默认 `1e-6` |

失败会 panic，适合测试脚本。

### 8.4 求解器原生函数

启用 `mems/with_allsolvers` 后，编译器允许 `sp_linsolve`、`fsolve`、`fmincon`、`intlinprog`；实际注册仍取决于 `eig-aoe` 的 solver、NLP 和 MILP backend feature。它们接收尚未代入未知量的表达式，因此未知量名称通常用反引号传递。

| 函数 | 调用协议 | 返回值 |
|---|---|---|
| `sp_linsolve` | `sp_linsolve(equations, b, initial, names)` | `[x..., status]` |
| `fsolve` | `fsolve(equations, names)` 或 `fsolve(equations, initial, names)` | `[x..., status]` |
| `intlinprog` | `intlinprog(objective, constraints, types, names)`；可在 `types` 前加入 `lower, upper` | `[x..., objective, status]` |
| `fmincon` | `fmincon(objective, constraints, constraint_lower, constraint_upper, variable_lower, variable_upper, names[, initial])` | `[x..., objective, status]` |

`sp_linsolve` 的 `equations` 是等式左端，函数把同位置的 `b` 作为右端；`b=[]` 表示全零。`initial=[]` 表示全零初值。`fsolve` 直接以表达式等于零构造方程；省略初值时先读取上下文中同名标量，找不到时使用零。

```rustscript
linear = sp_linsolve(
    [`x1` + `x2`, `x1` - `x2`],
    [3, 1],
    [],
    [`x1`, `x2`]
);

nonlinear = fsolve(
    [`x1`^2 - 4, `x2` - `x1`],
    [1, 1],
    [`x1`, `x2`]
);
```

`intlinprog` 的变量类型为 `1`（0/1）、`2`（整数）、`3`（连续实数）。目标必须写成 `min(expression)` 或 `max(expression)`，约束表达式可以使用 `<=`、`>=` 或 `==`。例如：

```rustscript
result = intlinprog(
    max(5*`x1` + 3*`x2`),
    [2*`x1` + `x2` <= 4],
    [1, 3],
    [`x1`, `x2`]
);
```

`fmincon` 的每个约束表达式都必须有对应的下限和上限。变量名、变量上下限和可选初值的长度也必须一致。VM 桥接层还接受在末尾追加 `parameter_keys, parameter_values` 的 9 或 10 参数形式，供求解器后端传入字符串参数。

`sp_linsolve` 和 `fsolve` 成功时状态码为 `0`，求解失败时为 `1`。`intlinprog` 和 `fmincon` 的状态码来自所选后端；调用者应把末项当作状态码，而不是迭代次数。模型参数无法解析时，这些函数当前返回单元素 `[100]`。

这些专用模型函数不同于直接矩阵 `linsolve(a,b)`。前者从符号表达式建模并把状态附在结果末尾，后者直接计算矩阵方程。

当前 `solver_fn_names()` 也允许 `sp_linsolvex` 通过编译，但 `register_solver_fns()` 没有注册同名函数，因此它在 `tensoreval` 中不可用。

## 9. include 预处理

`#include` 和 `#include_url` 是 `tensoreval` 在解析前执行的文本预处理，不是模块语法：

```text
#include shared/constants.rss
#include_url https://example.test/common.rss
```

- `#include` 相对进程当前工作目录解析，不相对当前脚本；
- 内容在指令位置展开；
- 本地 `#include` 递归展开，嵌套文件中的相对路径仍以进程当前工作目录为基准；
- 本地展开拒绝包含循环、超过 32 层的嵌套和超过 16 MiB 的结果；
- `#include_url` 需要网络权限；
- URL 返回内容不继续扫描 include；
- 不支持 `mod`、`module`、`import`、命名空间或可见性语法。

## 10. 当前未实现的旧手册名称

以下名称曾出现在旧手册中，但当前内置分发表没有注册：

```text
all chol cond inv linspace mean median nnz numel qr
repmat reshape spy std tril triu var
```

当前也没有结构体字段访问、直接下标 `a[i]`、`const`、`let`、`mut`、类型声明、异常处理、模块系统或通用字符串计算。

## 11. MATLAB 迁移

| MATLAB | RustScript |
|---|---|
| 一基索引 | 零基索引 |
| `A(i,j)` | `get(A,i,j)` |
| `A(i,j)=v` | `assign`、`select_assign` 或 `set`，并接收返回值 |
| `A .* B`, `A ./ B`, `A .^ B` | 相同 |
| `A'` | 仅转置；共轭转置用 `ctranspose(A)` |
| `A\b` | 相同，需线性代数 backend |
| `inv(A)` | `A ^ -1`，需线性代数 backend |
| `1:n` | `range(1,n+1)` |
| 字符串 `'text'` | 原生函数参数使用反引号 `` `text` `` |

迁移时先确认一维张量、二维行向量和二维列向量的差异，再选择 `get`、`slice` 或 `select`。

## 12. 电力系统表格数据

RustScript 没有内置的 `bus`、`gen` 或 `branch` 结构体，但可以用二维张量保存 MATPOWER 风格的表格。列的含义由脚本约定，语言不会检查电力系统模式：

```rustscript
baseMVA = 100;

// bus_i, type, Pd, Qd, Gs, Bs, area, Vm, Va, baseKV, zone, Vmax, Vmin
bus = [
    [1, 3,  0.0,  0.0, 0, 0, 1, 1.060,   0.00, 0, 1, 1.06, 0.94],
    [2, 2, 21.7, 12.7, 0, 0, 1, 1.045,  -4.98, 0, 1, 1.06, 0.94],
    [3, 1, 94.2, 19.0, 0, 0, 1, 1.010, -12.72, 0, 1, 1.06, 0.94]
];

// bus, Pg, Qg, Qmax, Qmin, Vg, mBase, status, Pmax, Pmin
gen = [
    [1, 232.4, -16.9, 10,   0, 1.060, 100, 1, 332.4, 0],
    [2,  40.0,  42.4, 50, -40, 1.045, 100, 1, 140.0, 0]
];

// fbus, tbus, r, x, b, rateA, rateB, rateC, ratio, angle, status, angmin, angmax
branch = [
    [1, 2, 0.01938, 0.05917, 0.0528, 0, 0, 0, 0, 0, 1, -360, 360],
    [2, 3, 0.04699, 0.19797, 0.0438, 0, 0, 0, 0, 0, 1, -360, 360]
];

bus_count = size(bus, 0);
from_bus = slice(branch, [0], 0) - 1; // 表内母线号转为零基索引
to_bus = slice(branch, [0], 1) - 1;
```

读取列时直接使用该列的零基位置。若已有常量采用 MATPOWER 的一基列号，应在传给 `get`、`slice` 或 `select` 前减 `1`。表内保存的母线编号是否减 `1` 取决于后续算法；上例在生成索引向量时才转换。

## 13. 完整示例

```rustscript
fn row_energy(matrix, row_index) {
    row = slice(matrix, row_index, [0]);
    return sum_all(row .* row);
}

matrix = [[1, 2, 3], [4, 5, 6]];
energies = zeros(size(matrix, 0));
for row_index in 0..size(matrix, 0) {
    value = row_energy(matrix, row_index);
    energies = set(energies, [row_index], value);
}

expected = [14, 77];
assert_eq(energies, expected);
println(energies);
return energies;
```

运行 `tensoreval example.rss`，正常情况下先打印 `[14, 77]`，再报告成功及顶层返回值。

## 14. 旧版手册内容迁移表

当前手册按实现重组内容，没有按旧版的篇幅逐段保留。下表说明旧章节在当前手册中的去向：

| 旧版章节 | 当前位置 | 处理方式 |
|---|---|---|
| 1–4 概述、词法、类型、变量 | 1–4 | 保留并补充表达式图与 VM 的差异；把未实现的 `const` 明确列为不支持 |
| 5–6 创建和操作张量 | 3、7.2、7.3、7.6 | 保留真实接口，统一零基索引；删去无效的直接 `A[i]` 写法 |
| 7 运算符 | 6 | 合并重复的逐运算符示例，并补充优先级、矩阵/逐元素差异和后端条件 |
| 8–9 控制流和函数 | 4–5 | 保留并补充函数提升、多返回值、循环边界和 `continue` 行为 |
| 10 内置函数 | 7 | 所有当前注册名称均已列出；典型组合示例见 7.6 |
| 11 方程和优化 | 8.4 | 保留并按 VM 桥接代码修正参数协议、可选参数和返回状态 |
| 12 电力系统数据 | 12 | 保留表格组织方式，并说明它只是脚本约定，不是内置类型 |
| 13 模块系统 | 9、10 | 旧 `import`、`module`、`export` 是未实现设计，改为说明实际存在的 include 预处理 |
| 14 MATLAB 对比 | 11 | 保留迁移规则，修正索引、字符串、转置和逻辑运算的错误示例 |

旧版为每个数学函数重复编写的“输入/输出”段落改为第 7 节的完整名称表和统一语义；这部分是压缩，不是删除能力。旧版提到但实现中不存在的函数集中列在第 10 节，避免读者误以为可以调用。
