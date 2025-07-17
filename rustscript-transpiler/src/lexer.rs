use std::io::{Result, Error, ErrorKind};

/// 词法分析器的Token类型
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // 关键字
    Function,
    End,

    // 标识符和字面量
    Identifier(String),
    Number(f64),
    String(String),

    // 运算符和分隔符
    Assign,           // =
    Semicolon,        // ;
    Comma,            // ,
    LeftParen,        // (
    RightParen,       // )
    LeftBracket,      // [
    RightBracket,     // ]
    LeftBrace,        // {
    RightBrace,       // }
    Dot,              // .

    // 注释和空白
    Comment(String),
    Newline,

    // 文件结束
    EOF,
}

/// 词法分析器
pub struct Lexer {
    input: String,
    position: usize,
    current_char: Option<char>,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            position: 0,
            current_char: None,
            line: 1,
            column: 1,
        }
    }

    /// 对输入字符串进行词法分析
    pub fn tokenize(&mut self, input: &str) -> Result<Vec<Token>> {
        self.input = input.to_string();
        self.position = 0;
        self.line = 1;
        self.column = 1;
        self.current_char = self.input.chars().next();

        let mut tokens = Vec::new();

        while let Some(ch) = self.current_char {
            match ch {
                ' ' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    tokens.push(Token::Newline);
                    self.advance();
                    self.line += 1;
                    self.column = 1;
                }
                '\r' => {
                    self.advance();
                    if self.current_char == Some('\n') {
                        tokens.push(Token::Newline);
                        self.advance();
                        self.line += 1;
                        self.column = 1;
                    }
                }
                '%' => {
                    tokens.push(self.read_comment()?);
                }
                '=' => {
                    tokens.push(Token::Assign);
                    self.advance();
                }
                ';' => {
                    tokens.push(Token::Semicolon);
                    self.advance();
                }
                ',' => {
                    tokens.push(Token::Comma);
                    self.advance();
                }
                '(' => {
                    tokens.push(Token::LeftParen);
                    self.advance();
                }
                ')' => {
                    tokens.push(Token::RightParen);
                    self.advance();
                }
                '[' => {
                    tokens.push(Token::LeftBracket);
                    self.advance();
                }
                ']' => {
                    tokens.push(Token::RightBracket);
                    self.advance();
                }
                '{' => {
                    tokens.push(Token::LeftBrace);
                    self.advance();
                }
                '}' => {
                    tokens.push(Token::RightBrace);
                    self.advance();
                }
                '.' => {
                    tokens.push(Token::Dot);
                    self.advance();
                }
                '\'' => {
                    tokens.push(self.read_string()?);
                }
                c if c.is_alphabetic() || c == '_' => {
                    tokens.push(self.read_identifier()?);
                }
                c if c.is_ascii_digit() || c == '-' => {
                    tokens.push(self.read_number()?);
                }
                _ => {
                    self.advance();
                }
            }
        }

        tokens.push(Token::EOF);
        Ok(tokens)
    }

    fn advance(&mut self) {
        self.position += 1;
        self.column += 1;

        if self.position >= self.input.len() {
            self.current_char = None;
        } else {
            self.current_char = self.input.chars().nth(self.position);
        }
    }

    fn read_comment(&mut self) -> Result<Token> {
        let mut comment = String::new();

        // 跳过 %
        self.advance();

        // 读取到行尾
        while let Some(ch) = self.current_char {
            if ch == '\n' || ch == '\r' {
                break;
            }
            comment.push(ch);
            self.advance();
        }

        Ok(Token::Comment(comment))
    }

    fn read_string(&mut self) -> Result<Token> {
        let mut string_value = String::new();

        // 跳过开始的单引号
        self.advance();

        while let Some(ch) = self.current_char {
            if ch == '\'' {
                self.advance();
                break;
            }
            string_value.push(ch);
            self.advance();
        }

        Ok(Token::String(string_value))
    }

    fn read_identifier(&mut self) -> Result<Token> {
        let mut identifier = String::new();

        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' {
                identifier.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // 检查是否是关键字
        let token = match identifier.as_str() {
            "function" => Token::Function,
            "end" => Token::End,
            _ => Token::Identifier(identifier),
        };

        Ok(token)
    }

    fn read_number(&mut self) -> Result<Token> {
        let mut number_str = String::new();

        // 处理负号
        if self.current_char == Some('-') {
            number_str.push('-');
            self.advance();
        }

        // 读取数字部分
        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() || ch == '.' {
                number_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let number = number_str.parse::<f64>().map_err(|_| {
            Error::new(ErrorKind::InvalidData, format!("Invalid number: {}", number_str))
        })?;

        Ok(Token::Number(number))
    }
}

impl Default for Lexer {
    fn default() -> Self {
        Self::new()
    }
}
