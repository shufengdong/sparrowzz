use crate::ast::{MATLABDocument, MATLABNode, MATLABValue};
use crate::lexer::Token;
use std::io::{Result, Error, ErrorKind};

/// 语法分析器
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            current: 0,
        }
    }

    /// 解析tokens并构建AST
    pub fn parse(&mut self, tokens: Vec<Token>) -> Result<MATLABDocument> {
        self.tokens = tokens;
        self.current = 0;

        let mut document = MATLABDocument::new();

        while !self.is_at_end() {
            if let Some(node) = self.parse_node()? {
                document.add_node(node);
            }
        }

        Ok(document)
    }

    fn parse_node(&mut self) -> Result<Option<MATLABNode>> {
        match self.current_token() {
            Some(Token::Comment(comment)) => {
                let node = MATLABNode::Comment(comment.clone());
                self.advance();
                Ok(Some(node))
            }
            Some(Token::Function) => {
                Ok(Some(self.parse_function_def()?))
            }
            Some(Token::Identifier(name)) => {
                let name = name.clone();
                self.advance();

                // 检查是否是赋值语句
                if self.match_token(&Token::Assign) {
                    let value = self.parse_value()?;
                    Ok(Some(MATLABNode::Assignment { target: name, value }))
                } else {
                    // 如果不是赋值，可能是其他语句，暂时跳过
                    Ok(None)
                }
            }
            Some(Token::Newline) => {
                self.advance();
                Ok(Some(MATLABNode::BlankLine))
            }
            _ => {
                self.advance();
                Ok(None)
            }
        }
    }

    fn parse_function_def(&mut self) -> Result<MATLABNode> {
        // 跳过 'function'
        self.advance();

        // 读取输出变量
        let output_var = if let Some(Token::Identifier(name)) = self.current_token() {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(Error::new(ErrorKind::InvalidData, "Expected output variable after 'function'"));
        };

        // 跳过 '='
        if !self.match_token(&Token::Assign) {
            return Err(Error::new(ErrorKind::InvalidData, "Expected '=' after output variable"));
        }

        // 读取函数名
        let function_name = if let Some(Token::Identifier(name)) = self.current_token() {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(Error::new(ErrorKind::InvalidData, "Expected function name"));
        };

        Ok(MATLABNode::FunctionDef {
            name: function_name,
            output_var,
        })
    }

    fn parse_value(&mut self) -> Result<MATLABValue> {
        match self.current_token() {
            Some(Token::Number(n)) => {
                let value = *n;
                self.advance();
                Ok(MATLABValue::Scalar(value))
            }
            Some(Token::String(s)) => {
                let value = s.clone();
                self.advance();
                Ok(MATLABValue::String(value))
            }
            Some(Token::LeftBracket) => {
                self.parse_matrix()
            }
            Some(Token::LeftBrace) => {
                self.parse_cell_array()
            }
            Some(Token::Identifier(name)) => {
                let name = name.clone();
                self.advance();

                // 检查是否是结构体字段访问
                if self.match_token(&Token::Dot) {
                    if let Some(Token::Identifier(field)) = self.current_token() {
                        let field = field.clone();
                        self.advance();

                        if self.match_token(&Token::Assign) {
                            let value = self.parse_value()?;
                            Ok(MATLABValue::StructField {
                                object: name,
                                field,
                                value: Box::new(value),
                            })
                        } else {
                            Err(Error::new(ErrorKind::InvalidData, "Expected assignment after struct field"))
                        }
                    } else {
                        Err(Error::new(ErrorKind::InvalidData, "Expected field name after '.'"))
                    }
                } else {
                    // 普通标识符，暂时作为字符串处理
                    Ok(MATLABValue::String(name))
                }
            }
            _ => {
                Err(Error::new(ErrorKind::InvalidData, "Unexpected token in value"))
            }
        }
    }

    fn parse_matrix(&mut self) -> Result<MATLABValue> {
        // 跳过 '['
        self.advance();

        let mut rows = Vec::new();
        let mut current_row = Vec::new();

        while !self.check(&Token::RightBracket) && !self.is_at_end() {
            match self.current_token() {
                Some(Token::Number(n)) => {
                    current_row.push(MATLABValue::Scalar(*n));
                    self.advance();
                }
                Some(Token::String(s)) => {
                    current_row.push(MATLABValue::String(s.clone()));
                    self.advance();
                }
                Some(Token::Semicolon) => {
                    if !current_row.is_empty() {
                        rows.push(current_row);
                        current_row = Vec::new();
                    }
                    self.advance();
                }
                Some(Token::Newline) => {
                    self.advance();
                }
                _ => {
                    self.advance();
                }
            }
        }

        // 添加最后一行
        if !current_row.is_empty() {
            rows.push(current_row);
        }

        // 跳过 ']'
        if self.match_token(&Token::RightBracket) {
            Ok(MATLABValue::Matrix(rows))
        } else {
            Err(Error::new(ErrorKind::InvalidData, "Expected ']' to close matrix"))
        }
    }

    fn parse_cell_array(&mut self) -> Result<MATLABValue> {
        // 跳过 '{'
        self.advance();

        let mut elements = Vec::new();

        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            match self.current_token() {
                Some(Token::String(s)) => {
                    elements.push(MATLABValue::String(s.clone()));
                    self.advance();
                }
                Some(Token::Semicolon) | Some(Token::Newline) => {
                    self.advance();
                }
                _ => {
                    self.advance();
                }
            }
        }

        // 跳过 '}'
        if self.match_token(&Token::RightBrace) {
            Ok(MATLABValue::CellArray(elements))
        } else {
            Err(Error::new(ErrorKind::InvalidData, "Expected '}' to close cell array"))
        }
    }

    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.current += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() ||
        matches!(self.current_token(), Some(Token::EOF))
    }

    fn check(&self, token: &Token) -> bool {
        if let Some(current) = self.current_token() {
            std::mem::discriminant(current) == std::mem::discriminant(token)
        } else {
            false
        }
    }

    fn match_token(&mut self, token: &Token) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}
