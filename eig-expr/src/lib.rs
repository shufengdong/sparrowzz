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

#[cfg(any(feature = "test", feature = "enable_ndarray_blas"))]
extern crate ndarray_linalg;

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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::expr::{eval_str, Context};
    use crate::Expr;

    #[test]
    fn it_works() {
        let r = eval_str("1 + 2").unwrap();
        assert_eq!(r, 3.0);
    }

    #[test]
    fn test1() {
        assert_eq!(eval_str("3+5*(6-1)-18/3^2").unwrap(), 26.0);
        assert_eq!(eval_str("3+5*(6-1)-16%3").unwrap(), 27.0);
        assert_eq!(eval_str("1/(2*2+3*3)^0.5").unwrap(), 0.2773500981126146);
        let r = eval_str(
            "(0 - 1.5625859498977661)/((0 - 1.5625859498977661)*(0 - 1.5625859498977661)+(0 - 0.444685697555542)*(0 - 0.444685697555542))^0.5");
        assert!(r.is_ok());
        let r = eval_str("(0 - 1E-2)/((0 - 2e-1)*(0 - 1)+ 2E-2^0.5)");
        assert!(r.is_ok());
        let r = eval_str("1.0 - 3.0 * sin(10 - 3) / 2.0").unwrap();
        assert_eq!(r, 1.0 - 3.0 * (10.0_f64 - 3.0_f64).sin() / 2.0);
        let r = eval_str("1.0 - 3.0^2 * cos(10 - 3) / 2.0 - sqrt(4)").unwrap();
        assert_eq!(
            r,
            1.0 - 9.0 * (10.0_f64 - 3.0_f64).cos() / 2.0 - 4.0_f64.sqrt()
        );
        let r = eval_str("1.0 - exp(2.2)^2 * tan(10 - 3) / 2.0 - sqrt(4)").unwrap();
        assert!(
            (r - (1.0
                - 2.2_f64.exp() * 2.2_f64.exp() * (10.0_f64 - 3.0_f64).tan() / 2.0
                - 4.0_f64.sqrt()))
            .abs()
                < 1e-5
        );
    }

    #[test]
    fn test2() {
        let r: f64 = eval_str("3+5.5").unwrap();
        assert_eq!(r, 8.5);
        let r: f64 = eval_str("10.0/(10.0*10.0 + 2.0 * 2.0)^0.5").unwrap();
        assert!((r - 0.98).abs() < 0.01);
    }

    #[test]
    fn test3() {
        let expr: Expr = "10.0/(x*10.0 + 2.0 * 2.0)^0.5".parse().unwrap();
        let func = expr.bind("x").unwrap();
        let mut vm = HashMap::new();
        vm.insert("var1", 10.0);
        let ans = func((vm.get("var1").copied()).unwrap());
        assert!((ans - 0.98).abs() < 0.01);
    }

    #[test]
    fn test4() {
        assert_eq!(eval_str("12-5*2>10").unwrap(), 0.0);
        assert_eq!(eval_str("(12-5)*2>10").unwrap(), 1.0);
        assert_eq!(eval_str("(12-5)*2>10").unwrap(), 1.0);
        assert_eq!(eval_str("30<40/8*5.5").unwrap(), 0.0);
        assert_eq!(eval_str("20/3-5<2").unwrap(), 1.0);
        assert_eq!(eval_str("25.5 == (6.5-1.5)*5").unwrap(), 0.0);
        assert_eq!(eval_str("25 == (6.5-1.5)*5").unwrap(), 1.0);
        let expr: Expr = "var1 == (6.5-1.5)*5 ".parse().unwrap();
        let func = expr.bind("var1").unwrap();
        assert_eq!(func(25.5), 0.0);
        let expr: Expr = "var1 + var2>40/8*5.5 ".parse().unwrap();
        let mut context = Context::new();
        context.var("var1", 20.0);
        context.var("var2", 10.0);
        assert_eq!(expr.eval_with_context(context).unwrap(), 1.0);
        assert_eq!(eval_str("100 - 20 > 2.77 || 20 - 100 > 2.77").unwrap(), 1.0);
    }

    /*abs绝对值运算测试*/
    #[test]
    fn test5() {
        assert_eq!(eval_str("5.5*abs(-4.2)").unwrap(), 23.1);
        assert_eq!(eval_str("3+5*abs(5-7)").unwrap(), 13.);
    }

    /* 向下取整函数"floor()"测试*/
    #[test]
    fn test7() {
        assert_eq!(eval_str("floor(2.5)*6").unwrap(), 12.0);
        assert_eq!(eval_str("5+floor(-1.1)*5").unwrap(), -5.0);
        assert_eq!(eval_str("2.5*floor((2.5-1)+3.02)").unwrap(), 10.0);
    }

    /*提取数据中最大值max，最小值min测试*/
    #[test]
    fn test8() {
        assert_eq!(eval_str("max(7.5,14.8,9.8,2.0,5.5)").unwrap(), 14.8);
        assert_eq!(
            eval_str("max(47,14,4,70,49,13,35,86,90,71)").unwrap(),
            90_f64
        );
        assert_eq!(
            eval_str("max(2.4,9.2,8.6,5.5,8.5,6.2,2.7,0.01,2.5,5.4)").unwrap(),
            9.2
        );
        let mut vm = HashMap::new();
        vm.insert("x1", 10.0);
        let expr4: Expr = "max(15.0,x1,1)".parse().unwrap();
        let func = expr4.bind("x1").unwrap();
        assert_eq!(func((vm.get("x1").copied()).unwrap()), 15.0);
        assert_eq!(eval_str("min(7.5, 14.8, 9.8, 2.0, 5.5)").unwrap(), 2.0);
        assert_eq!(
            eval_str("min(47,14,4,70,49,13,35,86,90,71)").unwrap(),
            4_f64
        );
        assert_eq!(
            eval_str("min(2.4,9.2,8.6,5.5,8.5,6.2,2.7,0.01,2.5,5.4)").unwrap(),
            0.01
        );
        let expr: Expr = "min(15.0,x,1)".parse().unwrap();
        let func = expr.bind("x").unwrap();
        assert_eq!(func((vm.get("x1").copied()).unwrap()), 1_f64);
        assert_eq!(eval_str("2*3+max(0,1,0.5)").unwrap(), 7_f64);
        assert_eq!(eval_str("2*3+min(0,1,0.5)").unwrap(), 6_f64);
        assert_eq!(eval_str("2+3*max(1, 2.5, 0.5)").unwrap(), 9.5);
        assert_eq!(eval_str("2+3*min(1, 2.5, 0.5)").unwrap(), 3.5);
        assert_eq!(eval_str("max(0,max(1,2))").unwrap(), 2_f64);
        assert_eq!(eval_str("max(0,1+max(1,2))").unwrap(), 3_f64);
        assert_eq!(eval_str("max(2,min(3,4)*max(1,2))").unwrap(), 6_f64);
        assert_eq!(eval_str("max(max(1,0), min(1,0))").unwrap(), 1_f64);
        assert_eq!(
            eval_str("max(max(1,0) + min(1,0), min(1,0) + max(3,5))").unwrap(),
            5_f64
        );
        assert_eq!(
            eval_str("min(max(8,2) - min(5,4), min(1,3) - max(3,5))").unwrap(),
            -4.0
        );
        assert_eq!(
            eval_str("max(max(6,2) * min(5,4), min(1,0) * max(3,5))").unwrap(),
            24_f64
        );
        assert_eq!(
            eval_str("min(max(8,2) / min(5,4), min(1,3) * max(3,5))").unwrap(),
            2_f64
        );
        vm.insert("x2", 4_f64);
        let expr21: Expr = "min(max(8,2) / x, min(1,3) * max(3,5))".parse().unwrap();
        let func = expr21.bind("x").unwrap();
        assert_eq!(func((vm.get("x2").copied()).unwrap()), 2_f64);
    }

    // bool运算符测试">="和"<="
    #[test]
    fn test9() {
        assert_eq!(eval_str("4 >= 5").unwrap(), 0.0);
        assert_eq!(eval_str("7 >= 3").unwrap(), 1.0);
        assert_eq!(eval_str("4.5 <= 7.5").unwrap(), 1.0);
        let expr: Expr = "x1 >= 1".parse().unwrap();
        assert_eq!(expr.bind("x1").unwrap()(10.0), 1.0);
        let expr: Expr = "x1 >= 100".parse().unwrap();
        assert_eq!(expr.bind("x1").unwrap()(10.0), 0.0);
        let expr: Expr = "x1 <= 100".parse().unwrap();
        assert_eq!(expr.bind("x1").unwrap()(10.0), 1.0);
        let expr: Expr = "x1 <= 1".parse().unwrap();
        assert_eq!(expr.bind("x1").unwrap()(10.0), 0.0);

        assert_eq!(eval_str("7.5 <= 7.5").unwrap(), 1.0);
        assert_eq!(eval_str("5.6 >= 5.6").unwrap(), 1.0);
        let expr: Expr = "10 <= x1".parse().unwrap();
        assert_eq!(expr.bind("x1").unwrap()(10.0), 1.0);
        // test not
        let expr: Expr = "~~x1".parse().unwrap();
        assert_eq!(expr.bind("x1").unwrap()(10.0), 0.0);
        let expr: Expr = "~~x1".parse().unwrap();
        assert_eq!(expr.bind("x1").unwrap()(-10.0), 1.0);
    }

    // bool算符测试"!="
    #[test]
    fn test10() {
        assert_eq!(eval_str("1!=1").unwrap(), 0.0);
        assert_eq!(eval_str("0.5 != 1").unwrap(), 1.0);
        assert_eq!(eval_str("1.1 != 1").unwrap(), 1.0);
        let expr: Expr = "x1 != 9.9".parse().unwrap();
        assert_eq!(expr.bind("x1").unwrap()(10.0), 1.0);
        let expr: Expr = "x1 != 10.1".parse().unwrap();
        assert_eq!(expr.bind("x1").unwrap()(10.0), 1.0);
        let expr: Expr = "x1 != 10".parse().unwrap();
        assert_eq!(expr.bind("x1").unwrap()(10.0), 0.0);
    }

    // 负号
    #[test]
    fn test11() {
        assert_eq!(eval_str("2 + 5 - 4").unwrap(), 3.0);
        let expr: Expr = "x1-5".parse().unwrap();
        assert_eq!(expr.bind("x1").unwrap()(10.0), 5.0);
    }

    // 四舍五入算符round
    #[test]
    fn test12() {
        assert_eq!(3_f64, eval_str("round(3.2)").unwrap());
        assert_eq!(4_f64, eval_str("round(3.6)").unwrap());
        assert_eq!(7.5, eval_str("round(5.2) + 2.5").unwrap());
        assert_eq!(7.5, eval_str("2.5 + round(4.85)").unwrap());
        assert_eq!(12.5, eval_str("2.5 * round(4.85)").unwrap());
        assert_eq!(10_f64, eval_str("round(4.85 + 5)").unwrap());
        let mut r8 = HashMap::new();
        r8.insert("x1", 5.3);
        let expr: Expr = "round(x)".parse().unwrap();
        assert_eq!(expr.clone().bind("x").unwrap()(5.3), 5_f64);
        assert_eq!(expr.bind("x").unwrap()(5.3 * 3.0), 16_f64);
    }

    // 位运算测试与Not
    // 与&；或|；反~；异或^^；左移<<；右移>>
    #[test]
    fn test13() {
        assert_eq!(eval_str("!1").unwrap(), 1.0);
        assert_eq!(eval_str("!2").unwrap(), 2.0);
        assert_eq!(eval_str("!2!=1").unwrap(), 1.0);
        assert_eq!(eval_str("~~!2==0").unwrap(), 1.0);
        assert_eq!(eval_str("!~~2==1").unwrap(), 1.0);
        assert_eq!(eval_str("~~0").unwrap(), 1.0);
        assert_eq!(eval_str("~~0.").unwrap(), 1.0);
        assert_eq!(eval_str("~~1").unwrap(), 0.0);
        assert_eq!(eval_str("~~1.").unwrap(), 0.0);
        assert_eq!(eval_str("~~5").unwrap(), 0.0);
        assert_eq!(eval_str("~5").unwrap(), -6.0);
        assert_eq!(eval_str("3&5").unwrap(), 1.0);
        assert_eq!(eval_str("3|5").unwrap(), 7.0);
        assert_eq!(eval_str("3^^5").unwrap(), 6.0);
        assert_eq!(eval_str("7>>2").unwrap(), 1.0);
        assert_eq!(eval_str("11<<2").unwrap(), 44.0);
        let expr: Expr = "~x12".parse().unwrap();
        assert_eq!(expr.bind("x12").unwrap()(5.0), -6_f64);
        let expr: Expr = "($123+2)&3".parse().unwrap();
        assert_eq!(expr.bind("$123").unwrap()(5.0), 3_f64);
        let expr: Expr = "x123<<2".parse().unwrap();
        assert_eq!(expr.bind("x123").unwrap()(5.0), 20_f64);
        let expr: Expr = "(x123+1)>>1".parse().unwrap();
        assert_eq!(expr.bind("x123").unwrap()(5.0), 3_f64);
    }

    // 位运算测试：a@b，返回a的二进制第b位
    #[test]
    fn test14() {
        let expr: Expr = "_123@1".parse().unwrap();
        assert_eq!(expr.bind("_123").unwrap()(169.0), 1_f64);
        let expr: Expr = "_123@2".parse().unwrap();
        assert_eq!(expr.bind("_123").unwrap()(169.0), 0_f64);
        let expr: Expr = "_123@3".parse().unwrap();
        assert_eq!(expr.bind("_123").unwrap()(169.0), 0_f64);
        let expr: Expr = "_123@4".parse().unwrap();
        assert_eq!(expr.bind("_123").unwrap()(169.0), 1_f64);
        let expr: Expr = "_123@5".parse().unwrap();
        assert_eq!(expr.bind("_123").unwrap()(169.0), 0_f64);
        let expr: Expr = "_123@6".parse().unwrap();
        assert_eq!(expr.bind("_123").unwrap()(169.0), 1_f64);
        let expr: Expr = "_123@7".parse().unwrap();
        assert_eq!(expr.bind("_123").unwrap()(169.0), 0_f64);
        let expr: Expr = "_123@8".parse().unwrap();
        assert_eq!(expr.bind("_123").unwrap()(169.0), 1_f64);

        let expr: Expr = "$123@1".parse().unwrap();
        assert_eq!(expr.bind("$123").unwrap()(29.0), 1_f64);
        let expr: Expr = "_123@2".parse().unwrap();
        assert_eq!(expr.bind("_123").unwrap()(29.0), 0_f64);
        let expr: Expr = "_123@3".parse().unwrap();
        assert_eq!(expr.bind("_123").unwrap()(29.0), 1_f64);
        let expr: Expr = "_123@4".parse().unwrap();
        assert_eq!(expr.bind("_123").unwrap()(29.0), 1_f64);
        let expr: Expr = "_123@5".parse().unwrap();
        assert_eq!(expr.bind("_123").unwrap()(29.0), 1_f64);
        let expr: Expr = "_123@6".parse().unwrap();
        assert_eq!(expr.bind("_123").unwrap()(29.0), 0_f64);
        let expr: Expr = "_123@7".parse().unwrap();
        assert_eq!(expr.bind("_123").unwrap()(29.0), 0_f64);
        let expr: Expr = "_123@8".parse().unwrap();
        assert_eq!(expr.bind("_123").unwrap()(29.0), 0_f64);

        let expr: Expr = "_123@1".parse().unwrap();
        assert_eq!(expr.bind("_123").unwrap()(90.0), 0_f64);
        let expr: Expr = "_123@2".parse().unwrap();
        assert_eq!(expr.bind("_123").unwrap()(90.0), 1_f64);
        let expr: Expr = "_123@3".parse().unwrap();
        assert_eq!(expr.bind("_123").unwrap()(90.0), 0_f64);
        let expr: Expr = "_123@4".parse().unwrap();
        assert_eq!(expr.bind("_123").unwrap()(90.0), 1_f64);
        let expr: Expr = "_123@5".parse().unwrap();
        assert_eq!(expr.bind("_123").unwrap()(90.0), 1_f64);
        let expr: Expr = "_123@6".parse().unwrap();
        assert_eq!(expr.bind("_123").unwrap()(90.0), 0_f64);
        let expr: Expr = "_123@7".parse().unwrap();
        assert_eq!(expr.bind("_123").unwrap()(90.0), 1_f64);
        let expr: Expr = "_123@8".parse().unwrap();
        assert_eq!(expr.bind("_123").unwrap()(90.0), 0_f64);
    }

    // now()获取系统毫秒数测试
    #[test]
    fn test15() {
        let expr: Expr = "now(0)".parse().unwrap();
        let mut context = Context::new();
        context.func1("now", get_time_stamp1);
        assert!(expr.eval_with_context(context).unwrap() > 0.0);

        let expr: Expr = "-now(0)".parse().unwrap();
        let mut context = Context::new();
        context.func1("now", get_time_stamp1);
        assert!(expr.eval_with_context(context).unwrap() < 0.0);

        let expr: Expr = "now()".parse().unwrap();
        let mut context = Context::new();
        context.func0("now", get_time_stamp0);
        assert!(expr.eval_with_context(context).unwrap() > 0.0);
    }

    // test random
    #[test]
    fn test16() {
        let expr: Expr = "rand()".parse().unwrap();
        let context = Context::new();
        let r = expr.eval_with_context(context).unwrap();
        assert!(r >= 0.0);
        assert!(r < 1.0);

        let expr: Expr = "rand2(1,2)".parse().unwrap();
        let context = Context::new();
        let r = expr.eval_with_context(context).unwrap();
        assert!(r >= 1.0);
        assert!(r < 2.0);
    }

    #[test]
    fn test17() {
        let expr = "f(a, b, f2(), f3(), c)".parse::<Expr>();
        println!("{:?}", expr);
        assert!(expr.is_ok());
    }

    fn get_time_stamp1(_: f64) -> f64 {
        let now = std::time::SystemTime::now();
        now.duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as f64
    }

    fn get_time_stamp0() -> f64 {
        let now = std::time::SystemTime::now();
        now.duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as f64
    }
}
