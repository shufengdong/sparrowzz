extern crate core;
extern crate nom;

use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use ndarray::{Array, Array2, IxDyn};
use num_complex::Complex64;

use serde::{Deserialize, Serialize};

pub mod expr;
pub mod expr_complex;
pub mod expr_tensor;
pub mod tokenizer;
pub mod shuntingyard;
pub mod tsfn_basic;

#[derive(Debug, Clone, PartialEq)]
pub enum MyF {
    F64(f64),
    Tensor(Array<f64, IxDyn>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MyCx {
    F64(Complex64),
    Tensor(Array<Complex64, IxDyn>),
}

/// An error reported by the parser.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// A token that is not allowed at the given location (contains the location of the offending
    /// character in the source string).
    UnexpectedToken(usize),
    /// Missing right parentheses at the end of the source string (contains the number of missing
    /// parens).
    MissingRParen(i32),
    /// Missing operator or function argument at the end of the expression.
    MissingArgument,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            ParseError::UnexpectedToken(i) => write!(f, "Unexpected token at byte {}.", i),
            ParseError::MissingRParen(i) => write!(
                f,
                "Missing {} right parenthes{}.",
                i,
                if i == 1 { "is" } else { "es" }
            ),
            ParseError::MissingArgument => write!(f, "Missing argument at the end of expression."),
        }
    }
}

impl std::error::Error for ParseError {
    fn description(&self) -> &str {
        match *self {
            ParseError::UnexpectedToken(_) => "unexpected token",
            ParseError::MissingRParen(_) => "missing right parenthesis",
            ParseError::MissingArgument => "missing argument",
        }
    }
}

/// Function evaluation error.
#[derive(Debug, Clone, PartialEq)]
pub enum FuncEvalError {
    TooFewArguments,
    TooManyArguments,
    NumberArgs(usize),
    UnknownFunction,
}

impl Display for FuncEvalError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            FuncEvalError::UnknownFunction => write!(f, "Unknown function"),
            FuncEvalError::NumberArgs(i) => write!(f, "Expected {} arguments", i),
            FuncEvalError::TooFewArguments => write!(f, "Too few arguments"),
            FuncEvalError::TooManyArguments => write!(f, "Too many arguments"),
        }
    }
}

impl std::error::Error for FuncEvalError {
    fn description(&self) -> &str {
        match *self {
            FuncEvalError::UnknownFunction => "unknown function",
            FuncEvalError::NumberArgs(_) => "wrong number of function arguments",
            FuncEvalError::TooFewArguments => "too few function arguments",
            FuncEvalError::TooManyArguments => "too many function arguments",
        }
    }
}

/// An error produced by the shunting-yard algorightm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RPNError {
    /// An extra left parenthesis was found.
    MismatchedLParen(usize),
    /// An extra left brackets was found.
    MismatchedLBracket(usize),
    /// An extra right parenthesis was found.
    MismatchedRParen(usize),
    /// An extra right bracket was found.
    MismatchedRBracket(usize),
    /// Comma that is not separating function arguments.
    UnexpectedComma(usize),
    /// Too few operands for some operator.
    NotEnoughOperands(usize),
    /// Too many operands reported.
    TooManyOperands,
}

impl std::error::Error for RPNError {
    fn description(&self) -> &str {
        match *self {
            RPNError::MismatchedLParen(_) => "mismatched left parenthesis",
            RPNError::MismatchedRParen(_) => "mismatched right parenthesis",
            RPNError::MismatchedLBracket(_) => "mismatched left blackets",
            RPNError::MismatchedRBracket(_) => "mismatched right blackets",
            RPNError::UnexpectedComma(_) => "unexpected comma",
            RPNError::NotEnoughOperands(_) => "missing operands",
            RPNError::TooManyOperands => "too many operands left at the end of expression",
        }
    }
}

impl Display for RPNError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            RPNError::MismatchedLParen(i) => {
                write!(f, "Mismatched left parenthesis at token {}.", i)
            }
            RPNError::MismatchedRParen(i) => {
                write!(f, "Mismatched right parenthesis at token {}.", i)
            }
            RPNError::MismatchedLBracket(i) => {
                write!(f, "Mismatched left blackets at token {}.", i)
            }
            RPNError::MismatchedRBracket(i) => {
                write!(f, "Mismatched right blackets at token {}.", i)
            }
            RPNError::UnexpectedComma(i) => write!(f, "Unexpected comma at token {}", i),
            RPNError::NotEnoughOperands(i) => write!(f, "Missing operands at token {}", i),
            RPNError::TooManyOperands => {
                write!(f, "Too many operands left at the end of expression.")
            }
        }
    }
}

// extern crate meval;
/// An error produced during parsing or evaluation.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    UnknownVariable(String),
    UnknownTensor(u64),
    Function(String, FuncEvalError),
    /// An error returned by the parser.
    ParseError(ParseError),
    /// The shunting-yard algorithm returned an error.
    RPNError(RPNError),
    // A catch all for all other errors during evaluation
    EvalError(String),
    EmptyExpression,
}

/**
 * @api {枚举_数学符号} /Operation Operation
 * @apiPrivate
 * @apiGroup A_Enum
 * @apiSuccess {String} Plus \+
 * @apiSuccess {String} Minus \-
 * @apiSuccess {String} Times \*
 * @apiSuccess {String} Div /
 * @apiSuccess {String} Rem %
 * @apiSuccess {String} Pow ^
 * @apiSuccess {String} Fact !
 * @apiSuccess {String} Equal \==，从这里开始往下是bool操作符
 * @apiSuccess {String} Unequal !=
 * @apiSuccess {String} LessThan \<
 * @apiSuccess {String} GreatThan \>
 * @apiSuccess {String} LtOrEqual \<=
 * @apiSuccess {String} GtOrEqual \>=
 * @apiSuccess {String} And &&
 * @apiSuccess {String} Or ||
 * @apiSuccess {String} Not ~~
 * @apiSuccess {String} BitAnd &，从这里开始往下是位操作
 * @apiSuccess {String} BitOr |
 * @apiSuccess {String} BitXor ^^
 * @apiSuccess {String} BitShl \<<
 * @apiSuccess {String} BitShr \>>
 * @apiSuccess {String} BitAt @
 * @apiSuccess {String} BitNot ~
 */
/// Mathematical operations.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum Operation {
    // +
    Plus,
    // -
    Minus,
    // *
    Times,
    // /
    Div,
    // %
    Rem,
    // ^
    Pow,
    // !
    Fact,

    // bool操作符
    // ==
    Equal,
    // !=
    Unequal,
    // <
    LessThan,
    // >
    GreatThan,
    // <=
    LtOrEqual,
    // >=
    GtOrEqual,
    // &&
    And,
    // ||
    Or,
    // ~~
    Not,
    // 下面是位操作
    // &
    BitAnd,
    // |
    BitOr,
    // ^^
    BitXor,
    // <<
    BitShl,
    // >>
    BitShr,
    // @
    BitAt,
    // ~
    BitNot,
}

/**
 * @api {枚举_Token} /Token Token
 * @apiPrivate
 * @apiGroup A_Enum
 * @apiSuccess {Object} Binary Binary operation，{"Binary": Operation}
 * @apiSuccess {Object} Unary Unary operation，{"Unary": Operation}
 * @apiSuccess {String} LParen Left parenthesis (
 * @apiSuccess {String} RParen Right parenthesis )
 * @apiSuccess {String} BigLParen Big Left parenthesis {
 * @apiSuccess {String} BigRParen Big Right parenthesis }
 * @apiSuccess {String} RBracket Right brackets ]
 * @apiSuccess {String} Comma function argument separator
 * @apiSuccess {Object} Number {"Number": f64}
 * @apiSuccess {Object} Tensor {"Tensor": usize}
 * @apiSuccess {Object} Var {"Var": String}
 * @apiSuccess {Object} Func {"Func": tuple(String, [usize])}
 */
/// Expression tokens.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Token {
    /// Binary operation.
    Binary(Operation),
    /// Unary operation.
    Unary(Operation),

    /// Left parenthesis.   (
    LParen,
    /// Right parenthesis.  )
    RParen,
    /// Big Left parenthesis.  {
    BigLParen,
    /// Big Right parenthesis. }
    BigRParen,
    /// Right brackets. ]
    RBracket,
    /// Comma: function argument separator
    Comma,

    /// A number.
    Number(f64),
    /// A tensor.
    Tensor(Option<usize>),
    /// A variable.
    Var(String),
    /// A function with name and number of arguments.
    Func(String, Option<usize>),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct Expr {
    pub rpn: Vec<Token>,
}

impl Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub trait ContextProvider {
    fn get_var(&self, _: &str) -> Option<f64> {
        None
    }
    fn get_var_cx(&self, _: &str) -> Option<Complex64> {
        None
    }
    fn get_tensor(&self, _: &str) -> Option<Array<f64, IxDyn>> {
        None
    }
    fn get_tensor_cx(&self, _: &str) -> Option<Array<Complex64, IxDyn>> {
        None
    }
    fn eval_func(&self, _: &str, _: &[f64]) -> Result<f64, FuncEvalError> {
        Err(FuncEvalError::UnknownFunction)
    }
    fn eval_func_cx(&self, _: &str, _: &[Complex64]) -> Result<Complex64, FuncEvalError> {
        Err(FuncEvalError::UnknownFunction)
    }
    fn eval_func_tensor(&self, _: &str, _: &[MyF]) -> Result<MyF, FuncEvalError> {
        Err(FuncEvalError::UnknownFunction)
    }
    fn eval_func_tensor_cx(&self, _: &str, _: &[MyCx]) -> Result<MyCx, FuncEvalError> {
        Err(FuncEvalError::UnknownFunction)
    }
    fn matrix_inv(&self, _: &Array2<f64>) -> Result<Array2<f64>, FuncEvalError> {
        Err(FuncEvalError::UnknownFunction)
    }
    fn matrix_inv_cx(&self, _: &Array2<Complex64>) -> Result<Array2<Complex64>, FuncEvalError> {
        Err(FuncEvalError::UnknownFunction)
    }
}

pub struct CtxProvider {
    var_values: HashMap<String, f64>,
    var_values_cx: HashMap<String, Complex64>,
    var_values_tensor: HashMap<String, Array<f64, IxDyn>>,
    var_values_tensor_cx: HashMap<String, Array<Complex64, IxDyn>>,
}

impl Default for CtxProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl CtxProvider {
    pub fn new() -> Self {
        CtxProvider {
            var_values: Default::default(),
            var_values_cx: Default::default(),
            var_values_tensor: Default::default(),
            var_values_tensor_cx: Default::default(),
        }
    }

    pub fn var<S: Into<String>>(&mut self, name: S, v: f64) {
        self.var_values.insert(name.into(), v);
    }

    pub fn var_cx<S: Into<String>>(&mut self, name: S, v: Complex64) {
        self.var_values_cx.insert(name.into(), v);
    }
    pub fn tensor<S: Into<String>>(&mut self, name: S, v: Array<f64, IxDyn>) {
        self.var_values_tensor.insert(name.into(), v);
    }
    pub fn tensor_cx<S: Into<String>>(&mut self, name: S, v: Array<Complex64, IxDyn>) {
        self.var_values_tensor_cx.insert(name.into(), v);
    }
}


fn factorial_unsafe(num: f64) -> f64 {
    if num == 0. || num == 1. {
        1.
    } else {
        num * factorial_unsafe(num - 1.)
    }
}

pub fn factorial(num: f64) -> Result<f64, &'static str> {
    if num.fract() != 0. || num < 0. {
        Err("Number must be non-negative with no fractional component!")
    } else if num > 170. {
        Ok(f64::INFINITY)
    } else {
        Ok(factorial_unsafe(num))
    }
}

pub fn parse_exprs(s: &str) -> Option<Vec<(String, Expr)>> {
    let lines: Vec<&str> = s.split(';').collect();
    let mut exprs = Vec::new();
    for p in lines {
        if p.trim().is_empty() {
            continue;
        }
        let id_to_value: Vec<&str> = if p.contains(':') {
            p.split(':').collect()
        } else if let Some(pos) = p.find('=') {
            let (first, second) = p.split_at(pos);
            vec![first, &second[1..]]
        } else {
            vec![]
        };
        if id_to_value.len() == 2 {
            let var_name = id_to_value[0].trim().to_string();
            // 检查是否重复变量定义
            let var_expr: Expr = id_to_value[1].parse().ok()?;
            exprs.push((var_name, var_expr));
        } else {
            return None;
        }
    }
    Some(exprs)
}
