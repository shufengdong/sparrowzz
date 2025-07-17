pub mod ast;
pub mod lexer;
pub mod parser;
pub mod converter;
pub mod codegen;

use std::fs;
use std::io::Result;

pub use ast::{MATLABDocument, MATLABNode, MATLABValue};
pub use lexer::Lexer;
pub use parser::Parser;
pub use converter::Converter;
pub use codegen::CodeGenerator;

/// 主要的转换器，负责协调整个转换过程
pub struct MATLABToRustConverter {
    // 移除字段，改为在方法中创建实例
}

impl MATLABToRustConverter {
    pub fn new() -> Self {
        Self {}
    }

    /// 转换MATLAB文件到RustScript文件
    pub fn convert_file(&self, input_path: &str, output_path: &str) -> Result<()> {
        // 1. 读取MATLAB文件
        let matlab_content = fs::read_to_string(input_path)?;

        // 2. 转换内容
        let rustscript_content = self.convert_string(&matlab_content)?;

        // 3. 写入输出文件
        fs::write(output_path, rustscript_content)?;

        Ok(())
    }

    /// 转换MATLAB字符串到RustScript字符串
    pub fn convert_string(&self, matlab_content: &str) -> Result<String> {
        // 1. 词法分析
        let mut lexer = Lexer::new();
        let tokens = lexer.tokenize(matlab_content)?;

        // 2. 语法分析，构建AST
        let mut parser = Parser::new();
        let ast = parser.parse(tokens)?;

        // 3. 转换AST
        let converter = Converter::new();
        // let converted_ast = converter.convert(ast)?;
        let converted_ast = ast;

        // 4. 生成RustScript代码
        let codegen = CodeGenerator::new();
        let rustscript_code = codegen.generate(converted_ast)?;

        Ok(rustscript_code)
    }
}

impl Default for MATLABToRustConverter {
    fn default() -> Self {
        Self::new()
    }
}
