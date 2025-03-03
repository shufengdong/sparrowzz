//! Tokenizer that converts a mathematical expression in a string form into a series of `Token`s.
//!
//! The underlying parser is build using the [nom] parser combinator crate.
//!
//! The parser should tokenize only well-formed expressions.
//!
//! [nom]: https://crates.io/crates/nom
//!
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
use nom::{IResult, Parser};
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
    )).parse(i)
}

//
fn lparen(i: &[u8]) -> IResult<&[u8], Token, Error<&[u8]>> {
    map(tag("("), |_| Token::LParen).parse(i)
}

fn tensor(i: &[u8]) -> IResult<&[u8], Token, Error<&[u8]>> {
    map(tag("["), |_| Token::Tensor(None)).parse(i)
}

fn rparen(i: &[u8]) -> IResult<&[u8], Token, Error<&[u8]>> {
    map(tag(")"), |_| Token::RParen).parse(i)
}

fn rbracket(i: &[u8]) -> IResult<&[u8], Token, Error<&[u8]>> {
    map(tag("]"), |_| Token::RBracket).parse(i)
}

fn fact(i: &[u8]) -> IResult<&[u8], Token, Error<&[u8]>> {
    if !i.starts_with(&b"!="[..]) {
        map(tag("!"), |_| Token::Unary(Operation::Fact)).parse(i)
    } else {
        Err(nom::Err::Error(Error {
            input: i,
            code: ErrorKind::Tag,
        }))
    }
}

fn comma(i: &[u8]) -> IResult<&[u8], Token, Error<&[u8]>> {
    map(tag(","), |_| Token::Comma).parse(i)
}

fn negpos(i: &[u8]) -> IResult<&[u8], Token, Error<&[u8]>> {
    alt((
        map(tag("+"), |_| Token::Unary(Operation::Plus)),
        map(tag("-"), |_| Token::Unary(Operation::Minus)),
    )).parse(i)
}

fn not(i: &[u8]) -> IResult<&[u8], Token, Error<&[u8]>> {
    map(tag("~~"), |_| Token::Unary(Operation::Not)).parse(i)
}

fn bitnot(i: &[u8]) -> IResult<&[u8], Token, Error<&[u8]>> {
    map(tag("~"), |_| Token::Unary(Operation::BitNot)).parse(i)
}

fn number(i: &[u8]) -> IResult<&[u8], Token, Error<&[u8]>> {
    let (left, mut len) = map(digit1, |s: &[u8]| s.len()).parse(i)?;
    let (left, s) = opt(preceded(tag("."), map(digit0, |s: &[u8]| s.len() + 1))).parse(left)?;
    len += s.unwrap_or(0);
    let (mut left, op) = opt(alt((tag("e"), tag("E")))).parse(left)?;
    if op.is_some() {
        let (l, s) = alt((
            preceded(
                alt((tag("+"), tag("-"))),
                map(digit1, |s: &[u8]| s.len() + 2),
            ),
            map(digit1, |s: &[u8]| s.len() + 1),
        )).parse(left)?;
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
        // support chinese variable name
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
    map(map_res(ident, from_utf8), |s: &str| Token::Var(s.into())).parse(i)
}

fn func(i: &[u8]) -> IResult<&[u8], Token, Error<&[u8]>> {
    map(
        map_res(
            terminated(ident, preceded(multispace0, complete(tag("(")))),
            from_utf8,
        ),
        |s: &str| Token::Func(s.into(), None),
    ).parse(i)
}

fn lexpr(i: &[u8]) -> IResult<&[u8], Token, Error<&[u8]>> {
    delimited(
        multispace0,
        alt((number, func, tensor, var, negpos, lparen, not, bitnot, fact)),
        multispace0,
    ).parse(i)
}

fn after_rexpr(i: &[u8]) -> IResult<&[u8], Token, Error<&[u8]>> {
    delimited(
        multispace0,
        alt((binop, rparen, rbracket)),
        multispace0,
    ).parse(i)
}

fn after_rexpr_no_paren(i: &[u8]) -> IResult<&[u8], Token, Error<&[u8]>> {
    delimited(multispace0, binop, multispace0).parse(i)
}

fn after_rexpr_comma(i: &[u8]) -> IResult<&[u8], Token, Error<&[u8]>> {
    delimited(
        multispace0,
        alt((binop, rparen, rbracket, comma)),
        multispace0,
    ).parse(i)
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
                        if let Ok((rest2, _)) = delimited(multispace0, rbracket, multispace0).parse(rest) {
                            res.push(Token::Tensor(Some(0)));
                            s = rest2;
                            paren_stack.pop().expect("The paren_stack is empty!");
                            state = TokenizerState::AfterRExpr;
                            continue;
                        }
                    }
                    Token::Func(name, _) => {
                        paren_stack.push(ParenState::Func);
                        if let Ok((rest2, _)) = delimited(multispace0, rparen, multispace0).parse(rest) {
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
                return Err(ParseError::UnexpectedToken(1, s.len()));
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

#[cfg(test)]
mod tests {
    use nom::error;
    use nom::error::ErrorKind::{Alpha, Digit};
    use nom::Err::Error;
    use crate::ParseError;

    use super::*;

    #[test]
    fn test_binop() {
        assert_eq!(
            binop(b"+"),
            Ok((&b""[..], Token::Binary(Operation::Plus)))
        );
        assert_eq!(
            binop(b"-"),
            Ok((&b""[..], Token::Binary(Operation::Minus)))
        );
        assert_eq!(
            binop(b"*"),
            Ok((&b""[..], Token::Binary(Operation::Times)))
        );
        assert_eq!(
            binop(b"/"),
            Ok((&b""[..], Token::Binary(Operation::Div)))
        );
    }

    #[test]
    fn test_number() {
        assert_eq!(
            number(b"32143"),
            Ok((&b""[..], Token::Number(32143f64)))
        );
        assert_eq!(
            number(b"2."),
            Ok((&b""[..], Token::Number(2.0f64)))
        );
        assert_eq!(
            number(b"32143.25"),
            Ok((&b""[..], Token::Number(32143.25f64)))
        );
        assert_eq!(
            number(b"0.125e9"),
            Ok((&b""[..], Token::Number(0.125e9f64)))
        );
        assert_eq!(
            number(b"20.5E-3"),
            Ok((&b""[..], Token::Number(20.5E-3f64)))
        );
        assert_eq!(
            number(b"123423e+50"),
            Ok((&b""[..], Token::Number(123423e+50f64)))
        );

        assert_eq!(
            number(b""),
            Err(Error(error::Error {
                input: &b""[..],
                code: Digit
            }))
        );
        assert_eq!(
            number(b".2"),
            Err(Error(error::Error {
                input: &b".2"[..],
                code: Digit
            }))
        );
        assert_eq!(
            number(b"+"),
            Err(Error(error::Error {
                input: &b"+"[..],
                code: Digit
            }))
        );
        assert_eq!(
            number(b"e"),
            Err(Error(error::Error {
                input: &b"e"[..],
                code: Digit
            }))
        );
        assert_eq!(
            number(b"1E"),
            Err(Error(error::Error {
                input: &b""[..],
                code: Digit
            }))
        );
        assert_eq!(
            number(b"1e+"),
            Err(Error(error::Error {
                input: &b"+"[..],
                code: Digit
            }))
        );
    }

    #[test]
    fn test_var() {
        for &s in ["abc", "U0", "_034", "a_be45EA", "aAzZ_"].iter() {
            assert_eq!(
                var(s.as_bytes()),
                Ok((&b""[..], Token::Var(s.into())))
            );
        }
        for &s in ["\'a\'", "\"U0\"", "\"_034\"", "'*'", "\"+\""].iter() {
            assert_eq!(
                var(s.as_bytes()),
                Ok((&b""[..], tokenize(s).unwrap()[0].clone()))
            );
        }

        assert_eq!(
            var(b""),
            Err(Error(error::Error {
                input: &b""[..],
                code: Alpha
            }))
        );
        assert_eq!(
            var(b"0"),
            Err(Error(error::Error {
                input: &b"0"[..],
                code: Alpha
            }))
        );
    }

    #[test]
    fn test_func() {
        for &s in ["abc(", "u0(", "_034 (", "A_be45EA  ("].iter() {
            assert_eq!(
                func(s.as_bytes()),
                Ok((&b""[..], Token::Func(s[0..s.len() - 1].trim().into(), None)))
            );
        }

        assert_eq!(
            func(b""),
            Err(Error(error::Error {
                input: &b""[..],
                code: Alpha
            }))
        );
        assert_eq!(
            func(b"("),
            Err(Error(error::Error {
                input: &b"("[..],
                code: Alpha
            }))
        );
        assert_eq!(
            func(b"0("),
            Err(Error(error::Error {
                input: &b"0("[..],
                code: Alpha
            }))
        );
    }

    #[test]
    fn test_tokenize() {
        use super::Operation::*;
        use super::Token::*;

        assert_eq!(tokenize("a"), Ok(vec![Var("a".into())]));

        assert_eq!(
            tokenize("2 +(3--2) "),
            Ok(vec![
                Number(2f64),
                Binary(Plus),
                LParen,
                Number(3f64),
                Binary(Minus),
                Unary(Minus),
                Number(2f64),
                RParen,
            ])
        );

        assert_eq!(
            tokenize("-2^ ab0 *12 - C_0"),
            Ok(vec![
                Unary(Minus),
                Number(2f64),
                Binary(Pow),
                Var("ab0".into()),
                Binary(Times),
                Number(12f64),
                Binary(Minus),
                Var("C_0".into()),
            ])
        );

        assert_eq!(
            tokenize("-sin(pi * 3)^ cos(2) / Func2(x, f(y), z) * _buildIN(y)"),
            Ok(vec![
                Unary(Minus),
                Func("sin".into(), None),
                Var("pi".into()),
                Binary(Times),
                Number(3f64),
                RParen,
                Binary(Pow),
                Func("cos".into(), None),
                Number(2f64),
                RParen,
                Binary(Div),
                Func("Func2".into(), None),
                Var("x".into()),
                Comma,
                Func("f".into(), None),
                Var("y".into()),
                RParen,
                Comma,
                Var("z".into()),
                RParen,
                Binary(Times),
                Func("_buildIN".into(), None),
                Var("y".into()),
                RParen,
            ])
        );

        assert_eq!(
            tokenize("2 % 3"),
            Ok(vec![Number(2f64), Binary(Rem), Number(3f64)])
        );

        assert_eq!(
            tokenize("1 + !3 + 1"),
            Ok(vec![
                Number(1f64),
                Binary(Plus),
                Unary(Fact),
                Number(3f64),
                Binary(Plus),
                Number(1f64),
            ])
        );

        assert_eq!(tokenize("3!"), Err(ParseError::UnexpectedToken(1, 1)));
        assert_eq!(tokenize("()"), Err(ParseError::UnexpectedToken(1, 1)));
        assert_eq!(tokenize(""), Err(ParseError::MissingArgument));
        assert_eq!(tokenize("2)"), Err(ParseError::UnexpectedToken(1, 1)));
        assert_eq!(tokenize("2^"), Err(ParseError::MissingArgument));
        assert_eq!(tokenize("(((2)"), Err(ParseError::MissingRParen(2)));
        assert_eq!(tokenize("f(2,)"), Err(ParseError::UnexpectedToken(1, 1)));
        assert_eq!(tokenize("f(,2)"), Err(ParseError::UnexpectedToken(1, 3)));
    }

    #[test]
    fn test_func_with_no_para() {
        assert_eq!(
            tokenize("f()"),
            Ok(vec![Token::Func("f".to_string(), Some(0))])
        );
        assert_eq!(
            tokenize("f( )"),
            Ok(vec![Token::Func("f".to_string(), Some(0))])
        );
        assert!(tokenize("f(f2(1), f3())").is_ok());
        assert!(tokenize("f(f2(1), f3(), a)").is_ok());
        assert!(tokenize("f(a, b, f2(), f3(), c)").is_ok());
        assert!(tokenize("-sin(pi * 3)^ cos(2) / Func2(x, f(), z) * _buildIN()").is_ok());
    }

    #[test]
    fn test_show_latex() {
        //let test_token = tokenize("x1^2-10*x1+x2^2+8<=5*2").unwrap();
        //let test_token = tokenize("max((5*1)*x1+3*x2+2*x3+(10-3)*x4+4*x5)").unwrap();
        //let test_token = tokenize("1*3*x2+sin(8-2)*x3 - cos(pi)< 7").unwrap();
        //let test_token = tokenize("x1%5+3/3*x2+min(2,5)*x3*2e19 && 1").unwrap();
        //let test_token = tokenize("2!").unwrap();
        let test_token = tokenize("~x1").unwrap();
        println!("{:?}", test_token);
        for x in test_token {
            println!("{}", x);
        }
    }

    #[test]
    fn test_tensor() {
        assert_eq!(
            tokenize("[3]"),
            Ok(vec![Token::Tensor(None), Token::Number(3.), Token::RBracket])
        );
        assert!(tokenize("[[1,2],[3,4]]").is_ok());
    }
}
