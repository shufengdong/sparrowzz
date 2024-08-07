use std;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::from_utf8;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{digit0, digit1, multispace0};
use nom::combinator::{complete, map, map_res, opt};
use nom::error::{Error, ErrorKind};
use nom::sequence::{delimited, preceded, terminated};
use nom::IResult;
use crate::{ParseError, Token, Operation};

#[derive(Debug, Clone, Copy)]
enum TokenizerState {
    // accept any token that is an expression from the left: var, num, (, negpos
    LExpr,
    // accept any token that needs an expression on the left: fact, binop, ), comma
    AfterRExpr,
}

#[derive(Debug, Clone, Copy)]
enum ParenState {
    Subexpr,
    Func,
    Tensor,
}

/// Continuing the trend of starting from the simplest piece and building up,
/// we start by creating a parser for the built-in operator functions.
fn binop(i: &[u8]) -> IResult<&[u8], Token, Error<&[u8]>> {
    alt((
        // bool操作符
        map(tag(">="), |_| Token::Binary(Operation::GtOrEqual)),
        map(tag("<="), |_| Token::Binary(Operation::LtOrEqual)),
        map(tag("=="), |_| Token::Binary(Operation::Equal)),
        map(tag("!="), |_| Token::Binary(Operation::Unequal)),
        map(tag("&&"), |_| Token::Binary(Operation::And)),
        map(tag("||"), |_| Token::Binary(Operation::Or)),
        // 位运算
        map(tag("^^"), |_| Token::Binary(Operation::BitXor)),
        map(tag("<<"), |_| Token::Binary(Operation::BitShl)),
        map(tag(">>"), |_| Token::Binary(Operation::BitShr)),
        map(tag(">>"), |_| Token::Binary(Operation::BitShr)),
        map(tag("&"), |_| Token::Binary(Operation::BitAnd)),
        map(tag("|"), |_| Token::Binary(Operation::BitOr)),
        map(tag("@"), |_| Token::Binary(Operation::BitAt)),
        // 四则混合运算
        map(tag("+"), |_| Token::Binary(Operation::Plus)),
        map(tag("-"), |_| Token::Binary(Operation::Minus)),
        map(tag("*"), |_| Token::Binary(Operation::Times)),
        map(tag("/"), |_| Token::Binary(Operation::Div)),
        map(tag("%"), |_| Token::Binary(Operation::Rem)),
        map(tag("^"), |_| Token::Binary(Operation::Pow)),
        // alt有21个parser的限制，可以通过嵌套alt方法突破
        alt((
            // bool运算
            map(tag(">"), |_| Token::Binary(Operation::GreatThan)),
            map(tag("<"), |_| Token::Binary(Operation::LessThan)),
        )),
    ))(i)
}

//
fn lparen(i: &[u8]) -> IResult<&[u8], Token, Error<&[u8]>> {
    map(tag("("), |_| Token::LParen)(i)
}

fn tensor(i: &[u8]) -> IResult<&[u8], Token, Error<&[u8]>> {
    map(tag("["), |_| Token::Tensor(None))(i)
}

fn rparen(i: &[u8]) -> IResult<&[u8], Token, Error<&[u8]>> {
    map(tag(")"), |_| Token::RParen)(i)
}

fn rbracket(i: &[u8]) -> IResult<&[u8], Token, Error<&[u8]>> {
    map(tag("]"), |_| Token::RBracket)(i)
}

fn fact(i: &[u8]) -> IResult<&[u8], Token, Error<&[u8]>> {
    if !i.starts_with(&b"!="[..]) {
        map(tag("!"), |_| Token::Unary(Operation::Fact))(i)
    } else {
        Err(nom::Err::Error(Error {
            input: i,
            code: ErrorKind::Tag,
        }))
    }
}

fn comma(i: &[u8]) -> IResult<&[u8], Token, Error<&[u8]>> {
    map(tag(","), |_| Token::Comma)(i)
}

fn negpos(i: &[u8]) -> IResult<&[u8], Token, Error<&[u8]>> {
    alt((
        map(tag("+"), |_| Token::Unary(Operation::Plus)),
        map(tag("-"), |_| Token::Unary(Operation::Minus)),
    ))(i)
}

fn not(i: &[u8]) -> IResult<&[u8], Token, Error<&[u8]>> {
    map(tag("~~"), |_| Token::Unary(Operation::Not))(i)
}

fn bitnot(i: &[u8]) -> IResult<&[u8], Token, Error<&[u8]>> {
    map(tag("~"), |_| Token::Unary(Operation::BitNot))(i)
}

fn number(i: &[u8]) -> IResult<&[u8], Token, Error<&[u8]>> {
    let (left, mut len) = map(digit1, |s: &[u8]| s.len())(i)?;
    let (left, s) = opt(preceded(tag("."), map(digit0, |s: &[u8]| s.len() + 1)))(left)?;
    len += s.unwrap_or(0);
    let (mut left, op) = opt(alt((tag("e"), tag("E"))))(left)?;
    if op.is_some() {
        let (l, s) = alt((
            preceded(
                alt((tag("+"), tag("-"))),
                map(digit1, |s: &[u8]| s.len() + 2),
            ),
            map(digit1, |s: &[u8]| s.len() + 1),
        ))(left)?;
        len += s;
        left = l;
    }
    let f_bytes = &i[0..len];
    let f = from_utf8(f_bytes).unwrap().parse::<f64>().unwrap();
    Ok((left, Token::Number(f)))
}

fn ident(input: &[u8]) -> IResult<&[u8], &[u8], Error<&[u8]>> {
    // first character must be 'a'...'z' | 'A'...'Z' | '_'
    match input.first().cloned() {
        Some(b'a'..=b'z') | Some(b'A'..=b'Z') | Some(b'_') | Some(b'$') => {
            let n = input
                .iter()
                .skip(1)
                .take_while(|&&c| matches!(c, b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'0'..=b'9'))
                .count();
            let (parsed, rest) = input.split_at(n + 1);
            Ok((rest, parsed))
        }
        Some(b'\'') | Some(b'\"')=> {
            let start = *input.first().unwrap();
            let n = input
                .iter()
                .skip(1)
                .take_while(|&&c| c != start)
                .count();
            let (parsed, rest) = input.split_at(n + 2);
            if parsed.len() == 2 {
                Err(nom::Err::Error(Error {
                    input,
                    code: ErrorKind::Alpha,
                }))
            } else {
                Ok((rest, &parsed[1..parsed.len()-1]))
            }
        }
        _ => Err(nom::Err::Error(Error {
            input,
            code: ErrorKind::Alpha,
        })),
    }
}

fn var(i: &[u8]) -> IResult<&[u8], Token, Error<&[u8]>> {
    map(map_res(ident, from_utf8), |s: &str| Token::Var(s.into()))(i)
}

fn func(i: &[u8]) -> IResult<&[u8], Token, Error<&[u8]>> {
    map(
        map_res(
            terminated(ident, preceded(multispace0, complete(tag("(")))),
            from_utf8,
        ),
        |s: &str| Token::Func(s.into(), None),
    )(i)
}

fn lexpr(i: &[u8]) -> IResult<&[u8], Token, Error<&[u8]>> {
    delimited(
        multispace0,
        alt((number, func, tensor, var, negpos, lparen, not, bitnot, fact)),
        multispace0,
    )(i)
}

fn after_rexpr(i: &[u8]) -> IResult<&[u8], Token, Error<&[u8]>> {
    delimited(
        multispace0,
        alt((binop, rparen, rbracket)),
        multispace0,
    )(i)
}

fn after_rexpr_no_paren(i: &[u8]) -> IResult<&[u8], Token, Error<&[u8]>> {
    delimited(multispace0, binop, multispace0)(i)
}

fn after_rexpr_comma(i: &[u8]) -> IResult<&[u8], Token, Error<&[u8]>> {
    delimited(
        multispace0,
        alt((binop, rparen, rbracket, comma)),
        multispace0,
    )(i)
}

pub fn tokenize<S: AsRef<str>>(input: S) -> Result<Vec<Token>, ParseError> {
    let mut state = TokenizerState::LExpr;
    // number of function arguments left
    let mut paren_stack = vec![];

    let mut res = vec![];

    let input = input.as_ref().as_bytes();
    let mut s = input;

    while !s.is_empty() {
        let r = match (state, paren_stack.last()) {
            (TokenizerState::LExpr, _) => lexpr(s),
            (TokenizerState::AfterRExpr, None) => after_rexpr_no_paren(s),
            (TokenizerState::AfterRExpr, Some(&ParenState::Subexpr)) => after_rexpr(s),
            (TokenizerState::AfterRExpr, Some(&ParenState::Func)) => after_rexpr_comma(s),
            (TokenizerState::AfterRExpr, Some(&ParenState::Tensor)) => after_rexpr_comma(s),
        };

        match r {
            Ok((rest, t)) => {
                match &t {
                    Token::LParen => {
                        paren_stack.push(ParenState::Subexpr);
                    }
                    Token::Tensor(_) => {
                        paren_stack.push(ParenState::Tensor);
                        if let Ok((rest2, _)) = delimited(multispace0, rbracket, multispace0)(rest) {
                            res.push(Token::Tensor(Some(0)));
                            s = rest2;
                            paren_stack.pop().expect("The paren_stack is empty!");
                            state = TokenizerState::AfterRExpr;
                            continue;
                        }
                    }
                    Token::Func(name, _) => {
                        paren_stack.push(ParenState::Func);
                        if let Ok((rest2, _)) = delimited(multispace0, rparen, multispace0)(rest) {
                            res.push(Token::Func(name.clone(), Some(0)));
                            s = rest2;
                            paren_stack.pop().expect("The paren_stack is empty!");
                            state = TokenizerState::AfterRExpr;
                            continue;
                        }
                    }
                    Token::RParen => {
                        paren_stack.pop().expect("The paren_stack is empty!");
                    }
                    Token::RBracket => {
                        paren_stack.pop().expect("The bracket_stack is empty!");
                    }
                    Token::Var(_) | Token::Number(_) => {
                        state = TokenizerState::AfterRExpr;
                    }
                    Token::Binary(_) | Token::Comma => {
                        state = TokenizerState::LExpr;
                    }
                    _ => {}
                }
                res.push(t);
                s = rest;
            }
            _ => {
                println!(
                    "Unexpected parse result when parsing `{}` at `{}`: {:?}",
                    String::from_utf8_lossy(input),
                    String::from_utf8_lossy(s),
                    r
                );
                return Err(ParseError::UnexpectedToken(s.len()));
            }
        }
    }

    match state {
        TokenizerState::LExpr => Err(ParseError::MissingArgument),
        _ if !paren_stack.is_empty() => Err(ParseError::MissingRParen(paren_stack.len() as i32)),
        _ => Ok(res),
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Token::Binary(op) => match op {
                Operation::Plus => write!(f, "+"),
                Operation::Minus => write!(f, "-"),
                Operation::Times => write!(f, "\\times "),
                Operation::Div => write!(f, "\\div "),
                Operation::Rem => write!(f, "\\mid "),
                Operation::Pow => write!(f, "^"),
                Operation::Fact => write!(f, "!"),
                Operation::Equal => write!(f, "=="),
                Operation::Unequal => write!(f, "\\neq "),
                Operation::LessThan => write!(f, "<"),
                Operation::GreatThan => write!(f, ">"),
                Operation::LtOrEqual => write!(f, "\\leqslant "),
                Operation::GtOrEqual => write!(f, "\\geqslant "),
                Operation::And => write!(f, "\\&\\&"),
                Operation::Or => write!(f, "\\parallel "),
                Operation::BitAnd => write!(f, "\\And "),
                Operation::BitOr => write!(f, "|"),
                Operation::BitXor => write!(f, "\\oplus "),
                Operation::BitShl => write!(f, "<<"),
                Operation::BitShr => write!(f, ">>"),
                Operation::BitAt => write!(f, "@"),
                _ => write!(f, "Unsupported"),
            },
            Token::Unary(op) => match op {
                Operation::Not => write!(f, "!"),
                Operation::BitNot => write!(f, "\\sim "),
                Operation::Fact => write!(f, "!"),
                Operation::Plus => write!(f, "+"),
                Operation::Minus => write!(f, "-"),
                _ => write!(f, "Unsupported"),
            },
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::RBracket => write!(f, "]"),
            Token::Comma => write!(f, ","),
            Token::BigLParen => write!(f, "{{"),
            Token::BigRParen => write!(f, "}}"),
            Token::Number(n) => write!(f, "{}", n),
            Token::Var(v) => write!(f, "{}", v),
            Token::Func(func, _) => write!(f, "{}(", func),
            Token::Tensor(size) => write!(f, "Tensor({:?})", size),
        }
    }
}