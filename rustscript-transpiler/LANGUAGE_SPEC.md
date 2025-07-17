# RustScript 语言规范

## 1. 语言概述

RustScript 是一种结合了 Rust 和 MATLAB 语法特征的脚本语言，专为科学计算和矩阵操作设计。它采用基于张量的数据结构，具有强类型推导和函数式编程特征。

## 2. 词法规则

### 2.1 标识符
```rustscript
// 有效标识符
variable_name
matrix1
bus_data
PQ_bus
```

### 2.2 数值字面量
```rustscript
// 整数
42
-17
0

// 浮点数
3.14
-2.718
1.23e-4
6.022e23

// 科学记数法
1.5e10
-2.3E-5
```

### 2.3 字符串字面量
```rustscript
// 字符串使用双引号
"Hello, World!"
"Power flow data for IEEE 14 bus"
"File path: /data/case14.txt"
```

### 2.4 注释
```rustscript
// 单行注释
/* 多行注释 */

/* 
 * 块注释
 * 支持多行
 */
```

## 3. 数据类型

### 3.1 基本数据类型
- `f64`: 64位浮点数（默认数值类型）
- `i64`: 64位整数
- `bool`: 布尔值 (`true`, `false`)
- `string`: 字符串类型
- `complex`: 复数类型

**注意：RustScript没有结构体类型，所有数据都基于基本类型和张量。**

### 3.2 复数类型
```rustscript
// 复数创建
z1 = c(3.0, 4.0);    // 3 + 4i
z2 = c(0, 1);        // i
z3 = c(5.0, 0);      // 实数

// 复数运算
result = z1 + z2 * 2.0;
magnitude = abs(z1);
phase = arg(z1);
```

### 3.3 张量类型
```rustscript
// 标量
scalar = 42.0;

// 1D 张量（一维张量）- shape为 [n]
vector = [1.0, 2.0, 3.0, 4.0];  // shape: [4]

// 2D 张量（二维张量）- shape为 [m, n]
matrix = [
    [1.0, 2.0, 3.0],
    [4.0, 5.0, 6.0],
    [7.0, 8.0, 9.0]
];  // shape: [3, 3]

// 行向量（二维张量）- shape为 [1, n]
row_vector = [[1.0, 2.0, 3.0]];  // shape: [1, 3]

// 列向量（二维张量）- shape为 [m, 1]
col_vector = [[1.0], [2.0], [3.0]];  // shape: [3, 1]

// 3D 张量
tensor3d = [
    [[1.0, 2.0], [3.0, 4.0]],
    [[5.0, 6.0], [7.0, 8.0]]
];  // shape: [2, 2, 2]
```

**重要区别：**
- **一维张量**：shape为 [n]，在运算时相当于行向量
- **二维张量**：包括矩阵、行向量 [1, n]、列向量 [m, 1]
- **行向量和列向量都属于二维张量**，但形状不同

### 3.4 数据组织方式
由于没有结构体，复杂数据通过张量和多个相关变量来组织：
```rustscript
// 电力系统数据组织（无结构体方式）
baseMVA = 100;          // 基准功率
bus = [...];            // 母线数据矩阵
gen = [...];            // 发电机数据矩阵
branch = [...];         // 支路数据矩阵

// 不是: mpc.baseMVA, mpc.bus 等结构体访问方式
```

## 4. 变量和赋值

### 4.1 变量声明
```rustscript
// 变量赋值（类型推导）
baseMVA = 100;
voltage = 1.06;
name = "RustScript";
is_valid = true;

// 张量赋值
vector = [1, 2, 3, 4];
matrix = [[1, 2], [3, 4]];
```

### 4.2 常量声明
```rustscript
// 常量定义（约定使用大写）
PI = 3.14159265359;
EULER = 2.71828;
MAX_ITERATIONS = 1000;

// 电力系统常量示例
PQ = 1;
PV = 2; 
REF = 3;
NONE = 4;

// 母线数据列索引
BUS_I = 0;
BUS_TYPE = 1;
PD = 2;
QD = 3;
```

## 5. 张量操作

### 5.1 张量索引
```rustscript
// 单个元素访问
element = get(matrix, row, col);  // 获取指定位置的单个元素

// 注意：不支持 A[0,1] 这样的直接索引方式
// 错误写法：element = matrix[0, 1];  // 不支持

// 负索引（从末尾开始）
last_element = get(vector, -1);
```

### 5.2 张量切片
```rustscript
// slice函数用于取矩阵的行列
// slice(matrix, row_spec, col_spec)
// [0] 代表所有行或所有列

// 重要：返回类型取决于参数格式

// 1. 使用范围参数[start, end] - 返回二维张量
col_2d = slice(a, [0], [1, 2]);       // 取第2列所有行，返回3×1列向量(二维张量)
row_2d = slice(a, [1, 2], [0]);       // 取第2行所有列，返回1×3行向量(二维张量)
sub_matrix = slice(matrix, [0,3], [1,4]); // 行0-2，列1-3的子矩阵(二维张量)

// 2. 使用单个索引 - 返回一维张量
col_1d = slice(a, [0], 1);            // 取第2列所有行，返回shape为[3]的一维张量
row_1d = slice(a, 1, [0]);            // 取第2行所有列，返回shape为[3]的一维张量

// 实际应用示例
// 假设a是3×3矩阵：
a = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];

// 二维张量切片（保持矩阵结构）
col_vector = slice(a, [0], [1, 2]);   // 返回[[2], [5], [8]] - 3×1列向量(二维张量)
row_vector = slice(a, [1, 2], [0]);   // 返回[[4, 5, 6]] - 1×3行向量(二维张量)

// 一维张量切片（扁平化）
col_flat = slice(a, [0], 1);          // 返回[2, 5, 8] - shape为[3]的一维张量
row_flat = slice(a, 1, [0]);          // 返回[4, 5, 6] - shape为[3]的一维张量

// 电力系统应用示例
voltage = slice(bus, [0], [VM-1, VM]);    // 所有行，VM列(二维张量)
angle = slice(bus, [0], VA-1);            // 所有行，VA列(一维张量)
```

### 5.3 张量赋值操作
```rustscript
// assign函数用于对对应位置赋值，要求源张量和目标位置的形状匹配
// assign(target, source, row_spec, col_spec)

// 重要：source的维度必须与目标位置的维度匹配

// 1. 赋值二维张量 - source必须是匹配的二维张量
b_2d = [[1.0], [2.0], [3.0]];        // 3×1列向量(二维张量)
assign(a, b_2d, [0], [1, 2]);        // 正确：b_2d是3×1列向量，匹配slice(a,[0],[1,2])

c_2d = [[1.0, 2.0, 3.0]];            // 1×3行向量(二维张量)  
assign(a, c_2d, [1, 2], [0]);        // 正确：c_2d是1×3行向量，匹配slice(a,[1,2],[0])

// 2. 赋值一维张量 - source必须是匹配的一维张量
b_1d = [1.0, 2.0, 3.0];              // shape为[3]的一维张量
assign(a, b_1d, [0], 1);             // 正确：b_1d是一维张量，匹配slice(a,[0],1)
assign(a, b_1d, 1, [0]);             // 正确：b_1d是一维张量，匹配slice(a,1,[0])

// 错误示例（形状不匹配）
// assign(a, b_1d, [0], [1, 2]);     // 错误：一维张量不能赋值给二维位置
// assign(a, b_2d, [0], 1);          // 错误：二维张量不能赋值给一维位置

// 实际应用示例
voltage = slice(bus, [0], [VM-1, VM]);      // 获取电压幅值列(二维张量)
new_voltage = [[1.05], [1.02], [1.01]];    // 新的电压值(二维张量)
assign(bus, new_voltage, [0], [VM-1, VM]); // 更新电压幅值

angle_flat = slice(bus, [0], VA-1);         // 获取电压角度(一维张量)
new_angles = [0.0, -2.5, 1.2];             // 新的角度值(一维张量)
assign(bus, new_angles, [0], VA-1);        // 更新电压角度
```

### 5.4 张量形状和维度说明
```rustscript
// 重要概念区分：
a = [1, 2, 3];                        // 一维张量, shape: [3]
b = [[1, 2, 3]];                      // 行向量(二维张量), shape: [1, 3]  
c = [[1], [2], [3]];                  // 列向量(二维张量), shape: [3, 1]

// 在运算中：
// - 一维张量在运算时相当于行向量
// - 行向量和列向量都属于二维张量，但形状不同
// - slice和assign函数的行为取决于参数格式和返回的张量维度

// 形状检查示例
ndims(a);    // 返回 1 (一维张量)
ndims(b);    // 返回 2 (二维张量)
ndims(c);    // 返回 2 (二维张量)

size(a, 0);  // 返回 3
size(b, 0);  // 返回 1 (行数)
size(b, 1);  // 返回 3 (列数)
size(c, 0);  // 返回 3 (行数)
size(c, 1);  // 返回 1 (列数)
```

## 6. 运算符

### 6.1 算术运算符
```rustscript
// 标量运算
a + b;  // 加法
a - b;  // 减法
a * b;  // 乘法
a / b;  // 除法
a ^ b;  // 幂运算

// 元素级运算（张量）
A .+ B;    // 元素级加法
A .- B;    // 元素级减法
A .* B;    // 元素级乘法
A ./ B;    // 元素级除法
A .^ 2;    // 元素级幂运算

// 矩阵运算
A * B;     // 矩阵乘法
A';        // 转置
inv(A);    // 逆矩阵
det(A);    // 行列式
```

### 6.2 比较运算符
```rustscript
// 标量比较
a == b;    // 等于
a != b;    // 不等于
a < b;     // 小于
a <= b;    // 小于等于
a > b;     // 大于
a >= b;    // 大于等于

// 元素级比较（张量）
A .== B;   // 元素级等于
A .< B;    // 元素级小于
```

### 6.3 逻辑运算符
```rustscript
// 标量逻辑
a && b;    // 逻辑与
a || b;    // 逻辑或
~~a;       // 逻辑非（注意：是两个波浪号）

// 元素级逻辑（张量）
A .&& B;   // 元素级逻辑与
A .|| B;   // 元素级逻辑或
.~~A;      // 元素级逻辑非（两个波浪号）

// 实际使用示例
bus_gen_status = ~~bus_gen_status;  // 逻辑非运算
pq = find(bus_type == PQ || ~~bus_gen_status);  // 组合逻辑运算
```

## 7. 函数定义

### 7.1 基本函数语法
```rustscript
fn function_name(param1, param2, ...) {
    // 函数体
    return result;
}

// 简化返回语法（最后一个表达式作为返回值）
fn add(a, b) {
    a + b
}
```

**重要限制：RustScript函数只能返回一个变量，不支持多返回值。**

### 7.2 函数示例
```rustscript
fn make_y_bus(baseMVA, bus, branch) {
    nb = size(bus, 0);
    nl = size(branch, 0);
    
    // 计算导纳矩阵
    stat = slice(branch, [0], [BR_STATUS-1, BR_STATUS]);
    Ys = stat ./ (slice(branch, [0], [BR_R-1, BR_R]) 
            + c(0,1) * slice(branch, [0], [BR_X-1, BR_X]));
    
    return Ys;  // 只能返回一个值
}
```

## 9. 内置函数

### 9.1 数学函数
```rustscript
// 基本数学函数
sqrt(x), exp(x), log(x), log10(x)
sin(x), cos(x), tan(x)
asin(x), acos(x), atan(x), atan2(y, x)
abs(x), sign(x), floor(x), ceil(x), round(x)

// 复数函数
real(z), imag(z), conj(z), abs(z), arg(z)
```

### 9.2 矩阵函数
```rustscript
// 线性代数
det(A), inv(A), rank(A), trace(A)
eig(A), svd(A), qr(A), chol(A)
norm(A), cond(A)

// 矩阵操作
transpose(A), diag(A), triu(A), tril(A)
reshape(A, m, n), repmat(A, m, n)
```

### 9.3 张量操作函数
```rustscript
// 形状和尺寸
size(tensor, dim)     // 获取指定维度大小
length(tensor)        // 获取元素总数
ndims(tensor)         // 获取维度数
numel(tensor)         // 元素个数

// 张量切片和操作
slice(tensor, row_range, col_range)  // 张量切片操作
set(tensor, indices, values)         // 设置张量元素值
set2(tensor, indices, values)        // 张量元素累加赋值 (相当于 +=)

// 张量拼接
horzcat(A, B, ...)    // 横向拼接张量
vertcat(A, B, ...)    // 纵向拼接张量

// 序列生成 
range(start, end)     // 生成序列 [start, start+1, ..., end-1] (前闭后开)
linspace(start, end, num)  // 在start和end间生成num个等间距点

// 统计函数
sum(tensor), mean(tensor), std(tensor), var(tensor)
max(tensor), min(tensor), median(tensor)

// 查找函数
find(condition)       // 查找满足条件的索引
any(tensor), all(tensor)

// 获取多个元素
get_multi(tensor, indices)  // 根据索引获取多个元素
```

### 9.4 稀疏矩阵
```rustscript
// 创建稀疏矩阵
sparse(row_indices, col_indices, values, m, n);  // 创建m×n稀疏矩阵

// 稀疏矩阵操作
full_mat = full(sparse_mat);  // 转为密集矩阵
nnz_count = nnz(sparse_mat);  // 非零元素个数
spy(sparse_mat);              // 显示稀疏模式
```

### 9.5 张量操作详细示例
```rustscript
// 实际使用示例（来自make_y_bus函数）
nb = size(bus, 0);                    // 获取母线数量
nl = size(branch, 0);                 // 获取支路数量

// 切片操作
stat = slice(branch, [0], [BR_STATUS-1, BR_STATUS]);  // 获取支路状态列
Ys = stat ./ (slice(branch, [0], [BR_R-1, BR_R]) 
        + c(0,1) * slice(branch, [0], [BR_X-1, BR_X]));

// 查找和设置操作
index = find(tap_col);                // 查找非零变比的索引
tap_init = set(ones(nl, 1), index, get_multi(tap_col, index));

// 序列生成和拼接
i = horzcat(range(0, nl), range(0, nl)) - 1;  // 创建行索引
j = range(0, nb);                             // 创建列索引

// 稀疏矩阵创建
Yf = sparse(i, horzcat(f, t), upper, nl, nb);
Ybus = sparse(horzcat(f,f,t,t), horzcat(f,t,f,t), 
              vertcat(Yff,Yft,Ytf,Ytt), nb, nb);
```

## 10. 数据结构示例

### 10.1 电力系统数据（基于 case14）
```rustscript
// 系统基准功率
baseMVA = 100;

// 母线数据 - 每行必须用[]包围，数字用逗号分隔
bus = [
    [1, 3, 0, 0, 0, 0, 1, 1.06, 0, 0, 1, 1.06, 0.94],
    [2, 2, 21.7, 12.7, 0, 0, 1, 1.045, -4.98, 0, 1, 1.06, 0.94],
    [3, 2, 94.2, 19, 0, 0, 1, 1.01, -12.72, 0, 1, 1.06, 0.94],
    [4, 1, 47.8, -3.9, 0, 0, 1, 1.019, -10.33, 0, 1, 1.06, 0.94]
];

// 发电机数据
gen = [
    [1, 232.4, -16.9, 10, 0, 1.06, 100, 1, 332.4, 0],
    [2, 40, 42.4, 50, -40, 1.045, 100, 1, 140, 0],
    [3, 0, 23.4, 40, 0, 1.01, 100, 1, 100, 0]
];

// 支路数据
branch = [
    [1, 2, 0.01938, 0.05917, 0.0528, 0, 0, 0, 0, 0, 1, -360, 360],
    [1, 5, 0.05403, 0.22304, 0.0492, 0, 0, 0, 0, 0, 1, -360, 360],
    [2, 3, 0.04699, 0.19797, 0.0438, 0, 0, 0, 0, 0, 1, -360, 360]
];
```

## 11. 模块系统

### 11.1 模块导入
```rustscript
// 导入整个模块
import power_flow;

// 导入特定函数
import {newton_pf, fast_decoupled_pf} from power_flow;

// 别名导入
import newton_pf as newton from power_flow;
```

### 11.2 模块定义
```rustscript
// 模块文件: power_flow.rs
export fn newton_pf(baseMVA, bus, gen, branch) {
    // 实现牛顿拉夫逊潮流计算
}

export fn fast_decoupled_pf(baseMVA, bus, gen, branch) {
    // 实现快速解耦潮流计算
}
```

## 12. 错误处理

### 12.1 Result 类型
```rustscript
fn divide(a, b) -> Result<f64, String> {
    if b == 0.0 {
        return Err("Division by zero");
    }
    Ok(a / b)
}

// 使用
result = divide(10.0, 2.0);
match result {
    Ok(value) => println("Result: {}", value),
    Err(error) => println("Error: {}", error),
}
```

### 12.2 异常处理
```rustscript
try {
    result = risky_operation();
} catch (error) {
    println("Caught error: {}", error);
    result = default_value;
}
```

## 13. 与MATLAB的对比

### 13.1 主要差异

| 特性   | MATLAB              | RustScript            | 说明 |
|------|---------------------|-----------------------|------|
| 索引基数 | 1基索引 `A(1,2)`       | 0基索引 `A[0,1]`         | **重要差异：索引从不同位置开始** |
| 矩阵定义 | `[1 2; 3 4]`        | `[[1, 2], [3, 4]]`    | 分号分行，逗号分元素 |
| 序列生成 | `0:n-1`             | `range(0, n)`         | MATLAB包含端点，RustScript前闭后开 |
| 注释   | `% 注释`              | `// 注释`               | 注释符号不同 |
| 字符串  | `'string'`          | `"string"`            | 引号类型不同 |
| 函数定义 | `function y = f(x)` | `fn f(x){ return y;}` | 函数语法不同 |
| 逻辑运算 | `&`, `\|`, `~`      | `&&`, `\|\|`, `~~`    | 逻辑运算符不同 |

### 13.2 索引转换详解

**最重要的差异：MATLAB使用1基索引，RustScript使用0基索引**

```matlab
% MATLAB (1基索引)
A(1, 1)      % 第一个元素
A(1:3, 2)    % 第1到3行，第2列
bus(:, i)    % 第i列所有行
0:n-1        % 生成 [0, 1, 2, ..., n-1]
```

```rustscript
// RustScript (0基索引)
A[0, 0]           // 第一个元素
slice(A, [0,3], [1])    // 第1到3行（0,1,2），第2列（索引1）
slice(bus, [0], [i-1,i])      // 第i列所有行（需要减1）
range(0, n)       // 生成 [0, 1, 2, ..., n-1] (前闭后开)
```

### 13.3 转换示例对比

**MATLAB到RustScript的典型转换：**

```matlab
% MATLAB版本
function Ybus = make_y_bus(baseMVA, bus, branch)
    nb = size(bus, 1);           % 获取行数
    f = branch(:, F_BUS);        % 从母线
    t = branch(:, T_BUS);        % 到母线
    
    % 创建索引（1基）
    i = [1:nl, 1:nl];
    j = 1:nb;
    
    % 稀疏矩阵（MATLAB自动处理1基索引）
    Ybus = sparse(i, j, values, nb, nb);
end
```

```rustscript
// RustScript版本
fn make_y_bus(baseMVA, bus, branch) {
    nb = size(bus, 0);                    // 获取行数
    f = slice(branch, [0], [F_BUS-1]) - 1;   // 从母线（转换为0基）
    t = slice(branch, [0], [T_BUS-1]) - 1;   // 到母线（转换为0基）
    
    // 创建索引（0基）
    i = horzcat(range(0, nl), range(0, nl));
    j = range(0, nb);
    
    // 稀疏矩阵（显式使用0基索引）
    Ybus = sparse(i, j, values, nb, nb);
    return Ybus;
}
```

### 13.4 转换器的索引处理策略

在MATLAB到RustScript转换过程中，转换器需要特别处理索引：

**Case文件转换（保持原始索引）：**
- Case文件通常包含数据定义，索引值保持不变
- 例如：母线编号仍然是1, 2, 3...

**函数转换（转换为0基索引）：**
- 函数中的索引操作需要转换：`A(i)` → `A[i-1]`
- 循环变量需要调整：`for i = 1:n` → `for i in range(0, n)`
- 序列生成需要转换：`1:n` → `range(1, n+1)` 或 `range(0, n)`
