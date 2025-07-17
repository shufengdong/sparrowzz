/// MATLAB抽象语法树节点定义
///
/// 这个设计的关键是保持原始代码的顺序，每个节点都按照在源文件中出现的顺序存储
#[derive(Debug, Clone)]
pub enum MATLABNode {
    /// 注释行
    Comment(String),

    /// 函数定义
    FunctionDef {
        name: String,
        output_var: String,
    },

    /// 赋值语句
    Assignment {
        target: String,
        value: MATLABValue,
    },

    /// 空行
    BlankLine,

    /// 章节分隔符（如 %%-----  Power Flow Data  -----%%）
    SectionHeader(String),
}

/// MATLAB值的类型
#[derive(Debug, Clone)]
pub enum MATLABValue {
    /// 标量值
    Scalar(f64),

    /// 字符串
    String(String),

    /// 矩阵（二维数组）
    Matrix(Vec<Vec<MATLABValue>>),

    /// 单元格数组
    CellArray(Vec<MATLABValue>),

    /// 结构体字段赋值
    StructField {
        object: String,
        field: String,
        value: Box<MATLABValue>,
    },
}

/// 完整的MATLAB文档AST
#[derive(Debug)]
pub struct MATLABDocument {
    /// 按原始顺序存储的节点列表
    pub nodes: Vec<MATLABNode>,
}

impl MATLABDocument {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: MATLABNode) {
        self.nodes.push(node);
    }

    /// 获取所有注释
    pub fn get_comments(&self) -> Vec<&String> {
        self.nodes.iter().filter_map(|node| {
            if let MATLABNode::Comment(comment) = node {
                Some(comment)
            } else {
                None
            }
        }).collect()
    }

    /// 获取函数定义
    pub fn get_function_def(&self) -> Option<(&String, &String)> {
        self.nodes.iter().find_map(|node| {
            if let MATLABNode::FunctionDef { name, output_var } = node {
                Some((name, output_var))
            } else {
                None
            }
        })
    }

    /// 获取所有赋值语句
    pub fn get_assignments(&self) -> Vec<(&String, &MATLABValue)> {
        self.nodes.iter().filter_map(|node| {
            if let MATLABNode::Assignment { target, value } = node {
                Some((target, value))
            } else {
                None
            }
        }).collect()
    }
}

impl Default for MATLABDocument {
    fn default() -> Self {
        Self::new()
    }
}

/// 源代码位置信息，用于错误报告和调试
#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

/// 带有位置信息的节点，用于调试
#[derive(Debug, Clone)]
pub struct LocatedNode {
    pub node: MATLABNode,
    pub location: SourceLocation,
}

