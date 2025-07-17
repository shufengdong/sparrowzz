use crate::ast::{MATLABDocument, MATLABNode, MATLABValue};
use std::io::Result;

/// 代码生成器，负责将转换后的AST生成为RustScript代码
pub struct CodeGenerator {
    indent_level: usize,
    indent_size: usize,
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self {
            indent_level: 0,
            indent_size: 4,
        }
    }

    /// 生成RustScript代码
    pub fn generate(&self, document: MATLABDocument) -> Result<String> {
        let mut output = String::new();

        for node in document.nodes {
            let line = self.generate_node(&node)?;
            if !line.is_empty() {
                output.push_str(&line);
                output.push('\n');
            }
        }

        Ok(output)
    }

    /// 生成单个节点的代码
    fn generate_node(&self, node: &MATLABNode) -> Result<String> {
        match node {
            MATLABNode::Comment(comment) => {
                // 将 % 注释转换为 // 注释
                Ok(format!("//{}", comment))
            }
            MATLABNode::FunctionDef { name, output_var } => {
                // 函数定义转换为 fn（可选）
                Ok(format!("fn {}() -> {} {{", name, output_var))
            }
            MATLABNode::Assignment { target, value } => {
                let value_str = self.generate_value(value)?;
                Ok(format!("{} = {};", target, value_str))
            }
            MATLABNode::BlankLine => {
                Ok(String::new())
            }
            MATLABNode::SectionHeader(header) => {
                // 章节头转换为注释
                Ok(format!("// {}", header))
            }
        }
    }

    /// 生成值的代码
    fn generate_value(&self, value: &MATLABValue) -> Result<String> {
        match value {
            MATLABValue::Scalar(n) => {
                Ok(self.format_number(*n))
            }
            MATLABValue::String(s) => {
                Ok(format!("\"{}\"", s))
            }
            MATLABValue::Matrix(rows) => {
                self.generate_aligned_matrix(rows)
            }
            MATLABValue::CellArray(elements) => {
                let mut result = String::from("[\n");

                for (i, element) in elements.iter().enumerate() {
                    result.push_str(&self.get_indent());
                    result.push('\t');
                    result.push_str(&self.generate_value(element)?);

                    if i < elements.len() - 1 {
                        result.push(',');
                    }
                    result.push('\n');
                }

                result.push_str(&self.get_indent());
                result.push(']');
                Ok(result)
            }
            MATLABValue::StructField { object: _, field: _, value } => {
                // 结构体字段已经在转换阶段处理
                self.generate_value(value)
            }
        }
    }

    /// 生成对齐的矩阵代码
    fn generate_aligned_matrix(&self, rows: &[Vec<MATLABValue>]) -> Result<String> {
        if rows.is_empty() {
            return Ok("[]".to_string());
        }

        // 计算每列的最大宽度
        let column_widths = self.calculate_column_widths(rows)?;

        let mut result = String::from("[\n");

        for (i, row) in rows.iter().enumerate() {
            result.push_str(&self.get_indent());
            result.push('\t');
            result.push('[');

            // 格式化每个元素并对齐
            for (j, element) in row.iter().enumerate() {
                if j > 0 {
                    result.push_str(", ");
                }

                let formatted_element = self.format_matrix_element(element)?;
                let width = column_widths.get(j).unwrap_or(&0);

                // 右对齐数字
                result.push_str(&format!("{:>width$}", formatted_element, width = width));
            }

            result.push(']');

            if i < rows.len() - 1 {
                result.push(',');
            }
            result.push('\n');
        }

        result.push_str(&self.get_indent());
        result.push(']');
        Ok(result)
    }

    /// 计算每列的最大宽度
    fn calculate_column_widths(&self, rows: &[Vec<MATLABValue>]) -> Result<Vec<usize>> {
        if rows.is_empty() {
            return Ok(Vec::new());
        }

        let num_columns = rows[0].len();
        let mut column_widths = vec![0; num_columns];

        for row in rows {
            for (j, element) in row.iter().enumerate() {
                if j < num_columns {
                    let formatted = self.format_matrix_element(element)?;
                    column_widths[j] = column_widths[j].max(formatted.len());
                }
            }
        }

        Ok(column_widths)
    }

    /// 格式化矩阵元素
    fn format_matrix_element(&self, element: &MATLABValue) -> Result<String> {
        match element {
            MATLABValue::Scalar(n) => Ok(self.format_number(*n)),
            MATLABValue::String(s) => Ok(format!("\"{}\"", s)),
            _ => Ok("?".to_string()), // 其他类型暂不支持
        }
    }

    /// 格式化数字，保持合理的精度
    fn format_number(&self, n: f64) -> String {
        if n.fract() == 0.0 && n.abs() < 1e10 {
            // 整数
            format!("{}", n as i64)
        } else if n.abs() < 1e-10 {
            // 接近零的数
            "0".to_string()
        } else if n.abs() < 1e-3 || n.abs() >= 1e6 {
            // 科学计数法
            format!("{:.6e}", n)
        } else {
            // 普通小数，自动选择合适的精度
            let formatted = format!("{:.10}", n);
            // 移除尾部的零
            let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
            if trimmed.is_empty() {
                "0".to_string()
            } else {
                trimmed.to_string()
            }
        }
    }

    /// 获取当前缩进
    fn get_indent(&self) -> String {
        " ".repeat(self.indent_level * self.indent_size)
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}
