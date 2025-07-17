# MATLAB到RustScript格式转换工具

## 项目概述

本项目旨在实现一个将MATLAB格式的电力系统数据文件（如case14.m）转换为RustScript格式文件（如case14.txt）的转换工具。

## 输入输出格式对比

### 输入格式（MATLAB .m文件）
- **文件结构**：MATLAB函数定义，包含结构体字段
- **注释格式**：以 `%` 开头的注释
- **函数定义**：`function mpc = case14`
- **矩阵格式**：使用分号 `;` 分隔行，空格或制表符分隔列
- **数据组织**：通过结构体字段组织（如 `mpc.bus`, `mpc.gen`）

### 输出格式（RustScript .txt文件）
- **文件结构**：直接的数据定义，无函数包装
- **注释格式**：以 `//` 开头的注释
- **函数定义**：转换为 `fn` 关键字（如需要）
- **矩阵格式**：每行用方括号 `[]` 包围，元素间用逗号 `,` 分隔，**支持列对齐**
- **数据组织**：直接变量赋值（如 `bus = [...]`）

## 具体转换规则

### 1. 注释转换
- **输入**：`%CASE14    Power flow data for IEEE 14 bus test case.`
- **输出**：`//CASE14    Power flow data for IEEE 14 bus test case.`

### 2. 函数定义转换
- **输入**：`function mpc = case14`
- **输出**：`fn case14() -> MPC` 或直接省略（根据需求）

### 3. 结构体字段转换
- **输入**：`mpc.baseMVA = 100;`
- **输出**：`baseMVA = 100;`

### 4. 矩阵格式转换（支持列对齐）
**输入（MATLAB格式）：**
```matlab
mpc.bus = [
    1    3    0        0        0    0    1    1.06    0        0    1    1.06    0.94;
    2    2    21.7    12.7    0    0    1    1.045    -4.98    0    1    1.06    0.94;
    3    2    94.2    19        0    0    1    1.01    -12.72    0    1    1.06    0.94;
];
```

**输出（RustScript格式，列对齐）：**
```rust
bus = [
    [ 1,  3,    0,     0,      0,  0,  1,  1.06,      0,  0,  1,  1.06,  0.94],
    [ 2,  2, 21.7,  12.7,      0,  0,  1, 1.045,  -4.98,  0,  1,  1.06,  0.94],
    [ 3,  2, 94.2,    19,      0,  0,  1,  1.01, -12.72,  0,  1,  1.06,  0.94]
];
```

### 5. 特殊结构处理
**bus_name字段（字符串数组）：**
- **输入**：
```matlab
mpc.bus_name = {
    'Bus 1     HV';
    'Bus 2     HV';
};
```
- **输出**：
```rust
bus_name = [
    "Bus 1     HV",
    "Bus 2     HV"
];
```

## 实现架构

### 核心模块
1. **词法分析器**（lexer.rs）：将MATLAB代码分解为tokens
2. **语法分析器**（parser.rs）：解析MATLAB语法结构
3. **转换器模块**（converter.rs）：执行语法转换规则
4. **代码生成器**（codegen.rs）：生成格式化的RustScript代码

### 数据结构设计（保持代码顺序）

为了保持原有代码的顺序并支持后续扩展，我们使用基于AST（抽象语法树）的方法：

```rust
#[derive(Debug, Clone)]
pub enum MATLABNode {
    Comment(String),
    FunctionDef {
        name: String,
        output_var: String,
    },
    Assignment {
        target: String,
        value: MATLABValue,
    },
    BlankLine,
    SectionHeader(String),
}

#[derive(Debug, Clone)]
pub enum MATLABValue {
    Scalar(f64),
    String(String),
    Matrix(Vec<Vec<MATLABValue>>),
    CellArray(Vec<MATLABValue>),
    StructField {
        object: String,
        field: String,
        value: Box<MATLABValue>,
    },
}

#[derive(Debug)]
pub struct MATLABDocument {
    pub nodes: Vec<MATLABNode>,
}
```

### 转换流程

1. **词法分析**：将MATLAB代码分解为tokens
2. **语法分析**：构建AST（抽象语法树）
3. **转换处理**：遍历AST，应用转换规则
4. **代码生成**：输出格式化的RustScript代码（支持列对齐）

### 核心特性

- **保持顺序**：AST节点按原始代码顺序存储
- **易于扩展**：新的MATLAB表达式类型只需添加新的枚举变体
- **灵活转换**：可以对每个节点应用不同的转换规则
- **列对齐支持**：矩阵输出时自动计算列宽并对齐
- **调试友好**：可以追踪每个节点的源位置

## 使用方式

### 命令行接口
```bash
# 转换单个文件
cargo run -- input.m output.txt

# 使用默认case14文件测试
cargo run

# 示例：转换case14.m
cargo run -- ../rspower/data/case14.m ../rspower/data/case14_converted.txt
```

### 编程接口
```rust
use rustscript_transpiler::MATLABToRustConverter;

let converter = MATLABToRustConverter::new();
let result = converter.convert_file("case14.m", "case14.txt")?;
```

## 测试策略

### 单元测试
- 各个转换规则的独立测试
- 边界情况处理测试
- 错误处理测试

### 集成测试
- 完整文件转换测试
- 多种MATLAB格式支持测试
- 输出格式验证测试

### 验证测试
- 转换后数据的数值准确性验证
- 格式兼容性测试
- 列对齐功能测试
- 性能测试

## 错误处理

### 常见错误类型
1. **语法错误**：无法解析的MATLAB语法
2. **数据格式错误**：不支持的数据类型
3. **文件IO错误**：文件读写失败
4. **转换错误**：转换过程中的逻辑错误

### 错误恢复策略
1. 提供详细的错误信息和位置
2. 支持部分转换和警告输出
3. 提供手动干预选项

## 扩展性考虑

### 支持更多MATLAB特性
- 更复杂的数据结构
- 嵌套矩阵和单元格数组
- 更多的数据类型支持

### 配置化转换规则
- 允许用户自定义转换规则
- 支持不同的输出格式
- 插件式架构支持

## 项目文件结构

```
rustscript-transpiler/
├── Cargo.toml                    # 项目配置文件
├── LANGUAGE_SPEC.md              # 语言规范文档
├── MATLAB_TO_RUST_CONVERTER.md   # 项目文档（本文档）
├── src/
│   ├── lib.rs                    # 库入口，协调各模块
│   ├── main.rs                   # 主程序入口
│   ├── ast.rs                    # AST节点定义
│   ├── lexer.rs                  # 词法分析器
│   ├── parser.rs                 # 语法分析器
│   ├── converter.rs              # 转换器逻辑
│   └── codegen.rs                # 代码生成器（支持列对齐）
├── tests/
│   └── integration_tests.rs      # 集成测试
└── examples/
    └── convert_case.rs           # 转换case的示例
```

## 核心功能特性

### 1. 列对齐支持
- **自动计算列宽**：扫描整个矩阵，计算每列的最大宽度
- **右对齐数字**：数字采用右对齐格式，提高可读性
- **智能格式化**：自动选择合适的数字精度和格式

### 2. 数字格式化
- **整数处理**：整数显示为整数格式
- **小数处理**：自动去除尾部零
- **科学计数法**：极大或极小数字使用科学计数法

### 3. 代码顺序保持
- **AST结构**：按原始文件顺序存储节点
- **逐行处理**：保持原始代码的逻辑顺序
- **注释保持**：保留原始注释的位置

## 开发计划

### 第一阶段：基础转换功能 ✅
- [x] 项目文档完成
- [x] 词法分析器实现
- [x] 基础语法分析器实现
- [x] 核心转换逻辑实现
- [x] 基本测试用例
- [x] 列对齐功能实现

### 第二阶段：完善和优化
- [ ] 错误处理完善
- [ ] 性能优化
- [ ] 更多MATLAB语法支持
- [ ] 扩展功能支持
- [ ] 文档和示例完善

### 第三阶段：生产就绪
- [ ] 全面测试
- [ ] 用户界面优化
- [ ] 部署和发布准备
- [ ] 持续集成配置

### 后续计划
- [ ] 支持加减乘除等多种运算的读取
- [ ] 支持矩阵切片，赋值等操作（采用converter）
- [ ] 支持多种内置函数的解析和转换
- 写测试
