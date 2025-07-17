use crate::ast::{MATLABDocument, MATLABNode, MATLABValue};
use std::io::Result;

/// 转换器，负责将MATLAB AST转换为RustScript AST
pub struct Converter {
    // 可以在这里添加转换选项和配置
}

impl Converter {
    pub fn new() -> Self {
        Self {}
    }

    /// 转换整个MATLAB文档
    pub fn convert(&self, document: MATLABDocument) -> Result<MATLABDocument> {
        let mut converted_document = MATLABDocument::new();

        for node in document.nodes {
            if let Some(converted_node) = self.convert_node(node)? {
                converted_document.add_node(converted_node);
            }
        }

        Ok(converted_document)
    }

    /// 转换单个节点
    fn convert_node(&self, node: MATLABNode) -> Result<Option<MATLABNode>> {
        match node {
            MATLABNode::Comment(comment) => {
                // 将 % 注释转换为 // 注释
                Ok(Some(MATLABNode::Comment(comment)))
            }
            MATLABNode::FunctionDef { name: _, output_var: _ } => {
                // 函数定义暂时跳过（根据需求决定是否转换）
                Ok(None)
            }
            MATLABNode::Assignment { target, value } => {
                let converted_value = self.convert_value(value)?;
                let converted_target = self.convert_target(target)?;
                Ok(Some(MATLABNode::Assignment {
                    target: converted_target,
                    value: converted_value,
                }))
            }
            MATLABNode::BlankLine => {
                Ok(Some(MATLABNode::BlankLine))
            }
            MATLABNode::SectionHeader(header) => {
                Ok(Some(MATLABNode::SectionHeader(header)))
            }
        }
    }

    /// 转换赋值目标（移除结构体前缀）
    fn convert_target(&self, target: String) -> Result<String> {
        // 移除 mpc. 前缀
        if target.starts_with("mpc.") {
            Ok(target[4..].to_string())
        } else {
            Ok(target)
        }
    }

    /// 转换值
    fn convert_value(&self, value: MATLABValue) -> Result<MATLABValue> {
        match value {
            MATLABValue::Scalar(n) => Ok(MATLABValue::Scalar(n)),
            MATLABValue::String(s) => Ok(MATLABValue::String(s)),
            MATLABValue::Matrix(rows) => {
                let mut converted_rows = Vec::new();
                for row in rows {
                    let mut converted_row = Vec::new();
                    for element in row {
                        converted_row.push(self.convert_value(element)?);
                    }
                    converted_rows.push(converted_row);
                }
                Ok(MATLABValue::Matrix(converted_rows))
            }
            MATLABValue::CellArray(elements) => {
                let mut converted_elements = Vec::new();
                for element in elements {
                    converted_elements.push(self.convert_value(element)?);
                }
                Ok(MATLABValue::CellArray(converted_elements))
            }
            MATLABValue::StructField { object: _, field: _, value } => {
                // 如果是结构体字段赋值，转换为直接赋值
                let converted_value = self.convert_value(*value)?;
                Ok(converted_value)
            }
        }
    }
}

impl Default for Converter {
    fn default() -> Self {
        Self::new()
    }
}
