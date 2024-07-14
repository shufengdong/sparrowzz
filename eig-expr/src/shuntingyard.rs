//! Implementation of the shunting-yard algorithm for converting an infix expression to an
//! expression in reverse Polish notation (RPN).
//!
//! See the Wikipedia articles on the [shunting-yard algorithm][shunting] and on [reverse Polish
//! notation][RPN] for more details.
//!
//! [RPN]: https://en.wikipedia.org/wiki/Reverse_Polish_notation
//! [shunting]: https://en.wikipedia.org/wiki/Shunting-yard_algorithm

use crate::shuntingyard::Associativity::*;
use crate::{RPNError, Operation, Token};

#[derive(Debug, Clone, Copy)]
enum Associativity {
    Left,
    Right,
    NA,
}

/// Returns the operator precedence and associativity for a given token.
fn prec_assoc(token: &Token) -> (u32, Associativity) {
    use self::Associativity::*;
    use crate::Operation::*;
    use crate::Token::*;
    match *token {
        Binary(op) => match op {
            Or => (3, Left),
            And => (4, Left),
            BitOr => (5, Left),
            BitXor => (6, Left),
            BitAnd => (7, Left),
            Equal | Unequal => (8, Left),
            LessThan | GreatThan | LtOrEqual | GtOrEqual => (9, Left),
            BitShl | BitShr => (10, Left),
            Plus | Minus => (11, Left),
            Times | Div | Rem => (12, Left),
            BitAt => (13, Left),
            Pow => (14, Right),
            _ => unimplemented!(),
        },
        Unary(op) => match op {
            Plus | Minus | Not | BitNot => (13, NA),
            Fact => (15, NA),
            _ => unimplemented!(),
        },
        Var(_) | Number(_) | Func(..) | Tensor(_) | LParen | RParen | BigLParen | BigRParen
        | RBracket | Comma => (0, NA),
    }
}

/// Converts a tokenized infix expression to reverse Polish notation.
///
/// # Failure
///
/// Returns `Err` if the input expression is not well-formed.
pub fn to_rpn(input: &[Token]) -> Result<Vec<Token>, RPNError> {
    use crate::Token::*;

    let mut output = Vec::with_capacity(input.len());
    let mut stack = Vec::with_capacity(input.len());

    for (index, token) in input.iter().enumerate() {
        let token = token.clone();
        match token {
            Number(_) | Var(_) => output.push(token),
            Unary(_) => stack.push((index, token)),
            Binary(_) => {
                let pa1 = prec_assoc(&token);
                while !stack.is_empty() {
                    let pa2 = prec_assoc(&stack.last().unwrap().1);
                    match (pa1, pa2) {
                        ((i, Left), (j, _)) if i <= j => {
                            output.push(stack.pop().unwrap().1);
                        }
                        ((i, Right), (j, _)) if i < j => {
                            output.push(stack.pop().unwrap().1);
                        }
                        _ => {
                            break;
                        }
                    }
                }
                stack.push((index, token))
            }
            LParen => stack.push((index, token)),
            RParen => {
                let mut found = false;
                while let Some((_, t)) = stack.pop() {
                    match t {
                        LParen => {
                            found = true;
                            break;
                        }
                        Func(name, nargs) => {
                            found = true;
                            output.push(Func(name, Some(nargs.unwrap_or(0) + 1)));
                            break;
                        }
                        _ => output.push(t),
                    }
                }
                if !found {
                    return Err(RPNError::MismatchedRParen(index));
                }
            }
            RBracket => {
                let mut found = false;
                while let Some((_, t)) = stack.pop() {
                    match t {
                        Tensor(size) => {
                            found = true;
                            output.push(Tensor(Some(size.unwrap_or(0) + 1)));
                            break;
                        }
                        _ => output.push(t),
                    }
                }
                if !found {
                    return Err(RPNError::MismatchedRBracket(index));
                }
            }
            Comma => {
                let mut found = false;
                while let Some((i, t)) = stack.pop() {
                    match t {
                        LParen => {
                            return Err(RPNError::UnexpectedComma(index));
                        }
                        Func(name, nargs) => {
                            found = true;
                            stack.push((i, Func(name, Some(nargs.unwrap_or(0) + 1))));
                            break;
                        }
                        Tensor(size) => {
                            found = true;
                            stack.push((i, Tensor(Some(size.unwrap_or(0) + 1))));
                            break;
                        }
                        _ => output.push(t),
                    }
                }
                if !found {
                    return Err(RPNError::UnexpectedComma(index));
                }
            }
            Tensor(Some(0)) => output.push(token),
            Tensor(..) => stack.push((index, token)),
            Func(_, Some(0)) => output.push(token),
            Func(..) => stack.push((index, token)),
            _ => {}
        }
    }

    while let Some((index, token)) = stack.pop() {
        match token {
            Unary(_) | Binary(_) => output.push(token),
            Func(_, None) => output.push(token),
            Tensor(None) => output.push(token),
            LParen | Func(..) => return Err(RPNError::MismatchedLParen(index)),
            _ => panic!("Unexpected token on stack."),
        }
    }

    // verify rpn
    let mut n_operands = 0isize;
    for (index, token) in output.iter().enumerate() {
        match *token {
            Var(_) | Number(_) => n_operands += 1,
            Unary(_) => (),
            Binary(_) => n_operands -= 1,
            Func(_, None) => continue,
            Func(_, Some(n_args)) => n_operands -= n_args as isize - 1,
            Tensor(None) => continue,
            Tensor(Some(size)) => n_operands -= size as isize - 1,
            _ => panic!("Nothing else should be here"),
        }
        if n_operands <= 0 {
            return Err(RPNError::NotEnoughOperands(index));
        }
    }

    if n_operands > 1 {
        return Err(RPNError::TooManyOperands);
    }

    output.shrink_to_fit();
    Ok(output)
}

pub fn rpn_to_infix(input: &[Token]) -> Result<Vec<Token>, RPNError> {
    use self::Associativity::*;
    use crate::Operation::*;
    use crate::Token::*;

    if input.is_empty() {
        return Ok(vec![]);
    }
    let mut stack = Vec::with_capacity(input.len());

    for (index, token) in input.iter().enumerate() {
        let token = token.clone();
        match token {
            Number(_) | Var(_) => {
                let pa = prec_assoc(&token);
                stack.push((vec![token], pa));
            }
            Tensor(nargs) => {
                let nargs = nargs.unwrap_or(0);
                let pa = prec_assoc(&token);
                let mut infix = vec![RBracket];
                for i in 0..nargs {
                    if stack.is_empty() {
                        return Err(RPNError::NotEnoughOperands(index));
                    }
                    let mut argu = stack.pop().unwrap().0;
                    if i >= 1 {
                        argu.push(Comma);
                    }
                    argu.append(&mut infix);
                    infix = argu;
                }
                infix.insert(0, token);
                stack.push((infix, pa));
            }
            Unary(_) => {
                if stack.is_empty() {
                    return Err(RPNError::NotEnoughOperands(index));
                }
                let (i, assoc) = stack.last().unwrap().1;
                let mut infix1 = stack.pop().unwrap().0;
                let pa = prec_assoc(&token);
                match assoc {
                    NA => {
                        if i < pa.0 && i != 0 {
                            infix1.insert(0, LParen);
                            infix1.push(RParen);
                        }
                    }
                    _ => {
                        if i <= pa.0 && i != 0 {
                            infix1.insert(0, LParen);
                            infix1.push(RParen);
                        }
                    }
                }
                infix1.insert(0, token);
                stack.push((infix1, pa));
            }
            Binary(op) => {
                let pa = prec_assoc(&token);
                let prec = pa.0;
                if stack.is_empty() {
                    return Err(RPNError::NotEnoughOperands(index));
                }
                let (precr, assocr) = stack.last().unwrap().1; //右边
                let mut infixr = stack.pop().unwrap().0;
                if stack.is_empty() {
                    return Err(RPNError::NotEnoughOperands(index));
                }
                let (precl, _) = stack.last().unwrap().1; //左边
                let mut infixl = stack.pop().unwrap().0;
                // let mut laddparen = false;
                let mut raddparen = false;
                match op {
                    Plus | Times => {
                        // 对于+和*，同级运算符不需要加括号
                        if precl < prec && precl != 0 {
                            infixl.insert(0, LParen);
                            infixl.push(RParen);
                            // laddparen = true;
                        }
                        if precr < prec && precr != 0 {
                            infixr.insert(0, LParen);
                            infixr.push(RParen);
                            raddparen = true;
                        }
                    }
                    Pow => {
                        // 字符串的次幂用普通括号
                        if precl <= prec && precl != 0 {
                            infixl.insert(0, LParen);
                            infixl.push(RParen);
                            // laddparen = true;
                        }
                        if precr < prec && precr != 0 {
                            infixr.insert(0, LParen);
                            infixr.push(RParen);
                            raddparen = true;
                        }
                    }
                    _ => {
                        if precl < prec && precl != 0 {
                            infixl.insert(0, LParen);
                            infixl.push(RParen);
                        }
                        if precr <= prec && precr != 0 {
                            infixr.insert(0, LParen);
                            infixr.push(RParen);
                            raddparen = true;
                        }
                    }
                }

                // 左边的单目加括号
                // if !laddparen && matches!(assocl,NA) && precl == 13 {
                //     infixl.insert(0, LParen);
                //     infixl.push(RParen);
                // }

                // 右边的单目加括号，单目的优先级较高，但在数学式子中习惯加括号，如-a+-b习惯写作-a+(-b)
                if !raddparen && matches!(assocr, NA) && precr == 13 {
                    infixr.insert(0, LParen);
                    infixr.push(RParen);
                }

                infixl.push(token);
                infixl.append(&mut infixr);
                stack.push((infixl, pa));
            }
            Func(_, nargs) => {
                let nargs = nargs.unwrap_or(0);
                let pa = prec_assoc(&token);
                let mut infix = vec![RParen];
                for i in 0..nargs {
                    if stack.is_empty() {
                        return Err(RPNError::NotEnoughOperands(index));
                    }
                    let mut argu = stack.pop().unwrap().0;
                    if i >= 1 {
                        argu.push(Comma);
                    }
                    argu.append(&mut infix);
                    infix = argu;
                }
                infix.insert(0, token);
                stack.push((infix, pa));
            }
            _ => {}
        }
    }

    if stack.len() != 1 {
        return Err(RPNError::TooManyOperands);
    }
    let output = stack.pop().unwrap().0;
    Ok(output)
}

pub fn rpn_to_string(input: &[Token]) -> Result<String, RPNError> {
    let mut output = String::new();
    let infix = rpn_to_infix(input)?;
    for token in infix.iter() {
        match token {
            Token::Binary(op) => match op {
                Operation::Plus => output.push('+'),
                Operation::Minus => output.push('-'),
                Operation::Times => output.push('*'),
                Operation::Div => output.push('/'),
                Operation::Rem => output.push('%'),
                Operation::Pow => output.push('^'),
                Operation::Equal => output.push_str("=="),
                Operation::Unequal => output.push_str("!="),
                Operation::LessThan => output.push('<'),
                Operation::GreatThan => output.push('>'),
                Operation::LtOrEqual => output.push_str("<="),
                Operation::GtOrEqual => output.push_str(">="),
                Operation::And => output.push_str("&&"),
                Operation::Or => output.push_str("||"),
                Operation::BitAnd => output.push('&'),
                Operation::BitOr => output.push('|'),
                Operation::BitXor => output.push_str("^^"),
                Operation::BitShl => output.push_str("<<"),
                Operation::BitShr => output.push_str(">>"),
                Operation::BitAt => output.push('@'),
                _ => output.push_str("Unsupported"),
            },
            Token::Unary(op) => match op {
                Operation::Not => output.push_str("~~"),
                Operation::BitNot => output.push('~'),
                Operation::Fact => output.push('!'),
                Operation::Plus => output.push('+'),
                Operation::Minus => output.push('-'),
                _ => output.push_str("Unsupported"),
            },
            Token::LParen => output.push('('),
            Token::RParen => output.push(')'),
            Token::Comma => output.push(','),
            Token::Number(n) => output.push_str(&format!("{}", n)),
            Token::Var(v) => output.push_str(&v.to_string()),
            Token::Func(func, _) => output.push_str(&format!("{}(", func)),
            Token::Tensor(_) => output.push('['),
            Token::RBracket => output.push(']'),
            Token::BigLParen => output.push('{'),
            Token::BigRParen => output.push('}'),
        }
    }
    Ok(output)
}

pub fn rpn_to_infix_latex(input: &[Token]) -> Result<Vec<Token>, RPNError> {
    // 用于latex的中缀表达式，不同于rpn_to_infix，这里的pow使用{}包裹，除法均改为分式。
    use self::Associativity::*;
    use crate::Operation::*;
    use crate::Token::*;

    if input.is_empty() {
        return Ok(vec![]);
    }
    let mut stack = Vec::with_capacity(input.len());

    for (index, token) in input.iter().enumerate() {
        let token = token.clone();
        match token {
            Number(_) | Var(_) => {
                let pa = prec_assoc(&token);
                stack.push((vec![token], pa));
            }
            Tensor(nargs) => {
                let nargs = nargs.unwrap_or(0);
                let pa = prec_assoc(&token);
                let mut infix = vec![RBracket];
                for i in 0..nargs {
                    if stack.is_empty() {
                        return Err(RPNError::NotEnoughOperands(index));
                    }
                    let mut argu = stack.pop().unwrap().0;
                    if i >= 1 {
                        argu.push(Comma);
                    }
                    argu.append(&mut infix);
                    infix = argu;
                }
                infix.insert(0, token);
                stack.push((infix, pa));
            }
            Unary(op) => {
                if stack.is_empty() {
                    return Err(RPNError::NotEnoughOperands(index));
                }
                let (i, assoc) = stack.last().unwrap().1;
                let mut infix1 = stack.pop().unwrap().0;
                let pa = prec_assoc(&token);
                match assoc {
                    NA => {
                        if i < pa.0 && i != 0 {
                            infix1.insert(0, LParen);
                            infix1.push(RParen);
                        }
                    }
                    _ => {
                        if i <= pa.0 && i != 0 {
                            infix1.insert(0, LParen);
                            infix1.push(RParen);
                        }
                    }
                }
                match op {
                    Plus | Minus | Not | BitNot => infix1.insert(0, token),
                    Fact => infix1.push(token),
                    _ => unimplemented!(),
                }
                stack.push((infix1, pa));
            }
            Binary(op) => {
                let pa = prec_assoc(&token);
                let prec = pa.0;
                if stack.is_empty() {
                    return Err(RPNError::NotEnoughOperands(index));
                }
                let (precr, assocr) = stack.last().unwrap().1; //右边
                let mut infixr = stack.pop().unwrap().0;
                if stack.is_empty() {
                    return Err(RPNError::NotEnoughOperands(index));
                }
                let (precl, _) = stack.last().unwrap().1; //左边
                let mut infixl = stack.pop().unwrap().0;
                // let mut laddparen = false;
                let mut raddparen = false;
                match op {
                    Plus | Times => {
                        // 对于+和*，同级运算符不需要加括号
                        if precl < prec && precl != 0 {
                            infixl.insert(0, LParen);
                            infixl.push(RParen);
                            // laddparen = true;
                        }
                        if precr < prec && precr != 0 {
                            infixr.insert(0, LParen);
                            infixr.push(RParen);
                            raddparen = true;
                        }
                    }
                    Div => {
                        // \frac{l}{r} 除法不加括号，直接形成分式
                        infixl.insert(0, Binary(Div)); //类似函数的逻辑，先把\frac{放进去，再把分子分母放进去
                        infixl.push(BigRParen);
                        infixr.insert(0, BigLParen);
                        infixr.push(BigRParen);
                        infixl.append(&mut infixr);
                        stack.push((infixl, (prec, assocr)));
                        continue;
                    }
                    Pow => {
                        // latex的次幂用大括号
                        if precl <= prec && precl != 0 {
                            infixl.insert(0, LParen);
                            infixl.push(RParen);
                            // laddparen = true;
                        }
                        infixr.insert(0, BigLParen);
                        infixr.push(BigRParen);
                        raddparen = true;
                    }
                    _ => {
                        if precl < prec && precl != 0 {
                            infixl.insert(0, LParen);
                            infixl.push(RParen);
                        }
                        if precr <= prec && precr != 0 {
                            infixr.insert(0, LParen);
                            infixr.push(RParen);
                            raddparen = true;
                        }
                    }
                }

                // 左边的单目加括号
                // if !laddparen && matches!(assocl,NA) && precl == 13 {
                //     infixl.insert(0, LParen);
                //     infixl.push(RParen);
                // }

                // 右边的单目加括号，单目的优先级较高，但在数学式子中习惯加括号，如-a+-b习惯写作-a+(-b)
                if !raddparen && matches!(assocr, NA) && precr == 13 {
                    infixr.insert(0, LParen);
                    infixr.push(RParen);
                }

                infixl.push(token);
                infixl.append(&mut infixr);
                stack.push((infixl, pa));
            }
            Func(_, nargs) => {
                let nargs = nargs.unwrap_or(0);
                let pa = prec_assoc(&token);
                let mut infix = vec![RParen];
                for i in 0..nargs {
                    if stack.is_empty() {
                        return Err(RPNError::NotEnoughOperands(index));
                    }
                    let mut argu = stack.pop().unwrap().0;
                    if i >= 1 {
                        argu.push(Comma);
                    }
                    argu.append(&mut infix);
                    infix = argu;
                }
                infix.insert(0, token);
                stack.push((infix, pa));
            }
            _ => {}
        }
    }

    if stack.len() != 1 {
        return Err(RPNError::TooManyOperands);
    }
    let output = stack.pop().unwrap().0;
    Ok(output)
}

pub fn rpn_to_latex(input: &[Token]) -> Result<String, RPNError> {
    let mut output = String::new();
    let infix = rpn_to_infix_latex(input)?;
    for (_, token) in infix.iter().enumerate() {
        match token {
            Token::Binary(op) => {
                match op {
                    Operation::Plus => output.push('+'),
                    Operation::Minus => output.push('-'),
                    Operation::Times => output.push_str("\\times "),
                    Operation::Div => output.push_str("\\frac{"), // 除法
                    Operation::Rem => output.push_str("\\mid "),
                    Operation::Pow => output.push('^'),
                    Operation::Equal => output.push('='),
                    Operation::Unequal => output.push_str("\\neq "),
                    Operation::LessThan => output.push('<'),
                    Operation::GreatThan => output.push('>'),
                    Operation::LtOrEqual => output.push_str("\\le "),
                    Operation::GtOrEqual => output.push_str("\\ge "),
                    Operation::And => output.push_str("\\&\\&"),
                    Operation::Or => output.push_str("\\parallel "),
                    Operation::BitAnd => output.push_str("\\And "),
                    Operation::BitOr => output.push('|'),
                    Operation::BitXor => output.push_str("\\oplus "),
                    Operation::BitShl => output.push_str("<<"),
                    Operation::BitShr => output.push_str(">>"),
                    Operation::BitAt => output.push('@'),
                    _ => output.push_str("Unsupported"),
                }
            }
            Token::Unary(op) => match op {
                Operation::Not => output.push_str("~~"), //todo: here is a bug
                Operation::BitNot => output.push_str("\\sim "),
                Operation::Fact => output.push('!'),
                Operation::Plus => output.push('+'),
                Operation::Minus => output.push('-'),
                _ => output.push_str("Unsupported"),
            },
            Token::LParen => output.push('('),
            Token::RParen => output.push(')'),
            Token::BigLParen => output.push('{'),
            Token::BigRParen => output.push('}'),
            Token::Comma => output.push(','),
            Token::Number(n) => output.push_str(&format!("{}", n)),
            Token::Var(v) => output.push_str(&v.replace('_', "\\_")),
            Token::Func(func, _) => output.push_str(&format!("{}(", func)),
            Token::Tensor(_) => output.push('['),
            Token::RBracket => output.push(']'),
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use crate::Operation::*;
    use crate::Token::*;

    use super::*;

    #[test]
    fn test_to_rpn() {
        assert_eq!(to_rpn(&[Number(1.)]), Ok(vec![Number(1.)]));
        assert_eq!(
            to_rpn(&[Number(1.), Binary(Plus), Number(2.)]),
            Ok(vec![Number(1.), Number(2.), Binary(Plus)])
        );
        assert_eq!(
            to_rpn(&[Unary(Minus), Number(1.), Binary(Pow), Number(2.)]),
            Ok(vec![Number(1.), Number(2.), Binary(Pow), Unary(Minus)])
        );
        assert_eq!(
            to_rpn(&[Number(1.), Unary(Fact), Binary(Pow), Number(2.)]),
            Ok(vec![Number(1.), Unary(Fact), Number(2.), Binary(Pow)])
        );
        assert_eq!(
            to_rpn(&[
                Number(1.),
                Unary(Fact),
                Binary(Div),
                LParen,
                Number(2.),
                Binary(Plus),
                Number(3.),
                RParen,
                Unary(Fact)
            ]),
            Ok(vec![
                Number(1.),
                Unary(Fact),
                Number(2.),
                Number(3.),
                Binary(Plus),
                Unary(Fact),
                Binary(Div),
            ])
        );
        assert_eq!(
            to_rpn(&[
                Number(3.),
                Binary(Minus),
                Number(1.),
                Binary(Times),
                Number(2.)
            ]),
            Ok(vec![
                Number(3.),
                Number(1.),
                Number(2.),
                Binary(Times),
                Binary(Minus),
            ])
        );
        assert_eq!(
            to_rpn(&[
                LParen,
                Number(3.),
                Binary(Minus),
                Number(1.),
                RParen,
                Binary(Times),
                Number(2.)
            ]),
            Ok(vec![
                Number(3.),
                Number(1.),
                Binary(Minus),
                Number(2.),
                Binary(Times),
            ])
        );
        assert_eq!(
            to_rpn(&[
                Number(1.),
                Binary(Minus),
                Unary(Minus),
                Unary(Minus),
                Number(2.)
            ]),
            Ok(vec![
                Number(1.),
                Number(2.),
                Unary(Minus),
                Unary(Minus),
                Binary(Minus),
            ])
        );
        assert_eq!(
            to_rpn(&[Var("x".into()), Binary(Plus), Var("y".into())]),
            Ok(vec![Var("x".into()), Var("y".into()), Binary(Plus)])
        );

        assert_eq!(
            to_rpn(&[
                Func("max".into(), None),
                Func("sin".into(), None),
                Number(1f64),
                RParen,
                Comma,
                Func("cos".into(), None),
                Number(2f64),
                RParen,
                RParen
            ]),
            Ok(vec![
                Number(1f64),
                Func("sin".into(), Some(1)),
                Number(2f64),
                Func("cos".into(), Some(1)),
                Func("max".into(), Some(2)),
            ])
        );

        assert_eq!(to_rpn(&[Binary(Plus)]), Err(RPNError::NotEnoughOperands(0)));
        assert_eq!(
            to_rpn(&[Func("f".into(), None), Binary(Plus), RParen]),
            Err(RPNError::NotEnoughOperands(0))
        );
        assert_eq!(
            to_rpn(&[Var("x".into()), Number(1.)]),
            Err(RPNError::TooManyOperands)
        );
        assert_eq!(to_rpn(&[LParen]), Err(RPNError::MismatchedLParen(0)));
        assert_eq!(to_rpn(&[RParen]), Err(RPNError::MismatchedRParen(0)));
        // assert_eq!(
        //     to_rpn(&[Func("sin".into(), None)]),
        //     Err(RPNError::MismatchedLParen(0))
        // );
        assert_eq!(to_rpn(&[Comma]), Err(RPNError::UnexpectedComma(0)));
        // assert_eq!(
        //     to_rpn(&[Func("f".into(), None), Comma]),
        //     Err(RPNError::MismatchedLParen(0))
        // );
        assert_eq!(
            to_rpn(&[Func("f".into(), None), LParen, Comma, RParen]),
            Err(RPNError::UnexpectedComma(2))
        );

        assert_eq!(
            to_rpn(&[
                Number(4.),
                Binary(Minus),
                Number(3.),
                Binary(Minus),
                Number(1.),
                Binary(Times),
                Number(2.)
            ]),
            Ok(vec![
                Number(4.),
                Number(3.),
                Binary(Minus),
                Number(1.),
                Number(2.),
                Binary(Times),
                Binary(Minus),
            ])
        );

        assert_eq!(
            to_rpn(&[Tensor(None), Number(1.), Comma, Number(2.), RBracket]),
            Ok(vec![Number(1.), Number(2.), Tensor(Some(2))])
        );
    }

    #[test]
    fn test_to_infix() {
        assert_eq!(
            rpn_to_infix(&[
                Number(4.),
                Number(3.),
                Binary(Minus),
                Number(1.),
                Number(2.),
                Binary(Times),
                Binary(Minus),
            ]),
            Ok(vec![
                Number(4.),
                Binary(Minus),
                Number(3.),
                Binary(Minus),
                Number(1.),
                Binary(Times),
                Number(2.),
            ])
        );

        assert_eq!(rpn_to_infix(&[Number(1.)]), Ok(vec![Number(1.)]));
        assert_eq!(
            rpn_to_infix(&[Number(1.), Number(2.), Binary(Plus)]),
            Ok(vec![Number(1.), Binary(Plus), Number(2.)])
        );
        assert_eq!(
            rpn_to_infix(&[Number(1.), Number(2.), Binary(Pow), Unary(Minus)]),
            Ok(vec![Unary(Minus), Number(1.), Binary(Pow), Number(2.)])
        );
        assert_eq!(
            rpn_to_infix(&[Number(1.), Unary(Fact), Number(2.), Binary(Pow)]),
            Ok(vec![Number(1.), Unary(Fact), Binary(Pow), Number(2.)])
        );
        assert_eq!(
            rpn_to_infix(&[
                Number(1.),
                Unary(Fact),
                Number(2.),
                Number(3.),
                Binary(Plus),
                Unary(Fact),
                Binary(Div)
            ]),
            Ok(vec![
                Number(1.),
                Unary(Fact),
                Binary(Div),
                LParen,
                Number(2.),
                Binary(Plus),
                Number(3.),
                RParen,
                Unary(Fact),
            ])
        );
        assert_eq!(
            rpn_to_infix(&[
                Number(3.),
                Number(1.),
                Number(2.),
                Binary(Times),
                Binary(Minus)
            ]),
            Ok(vec![
                Number(3.),
                Binary(Minus),
                Number(1.),
                Binary(Times),
                Number(2.),
            ])
        );
        assert_eq!(
            rpn_to_infix(&[
                Number(3.),
                Number(1.),
                Binary(Minus),
                Number(2.),
                Binary(Times)
            ]),
            Ok(vec![
                LParen,
                Number(3.),
                Binary(Minus),
                Number(1.),
                RParen,
                Binary(Times),
                Number(2.),
            ])
        );
        assert_eq!(
            rpn_to_infix(&[
                Number(1.),
                Number(2.),
                Unary(Minus),
                Unary(Minus),
                Binary(Minus)
            ]),
            Ok(vec![
                Number(1.),
                Binary(Minus),
                LParen,
                Unary(Minus),
                Unary(Minus),
                Number(2.),
                RParen,
            ])
        );
        assert_eq!(
            rpn_to_infix(&[Var("x".into()), Var("y".into()), Binary(Plus)]),
            Ok(vec![Var("x".into()), Binary(Plus), Var("y".into())])
        );

        assert_eq!(
            rpn_to_infix(&[
                Number(1f64),
                Func("sin".into(), Some(1)),
                Number(2f64),
                Func("cos".into(), Some(1)),
                Func("max".into(), Some(2)),
            ]),
            Ok(vec![
                Func("max".into(), Some(2)),
                Func("sin".into(), Some(1)),
                Number(1f64),
                RParen,
                Comma,
                Func("cos".into(), Some(1)),
                Number(2f64),
                RParen,
                RParen,
            ])
        );
        assert_eq!(
            rpn_to_infix(&[Number(1.), Number(2.), Tensor(Some(2))]),
            Ok(vec![Tensor(Some(2)), Number(1.), Comma, Number(2.), RBracket])
        );
    }

    #[test]
    fn test_to_latex() {
        use crate::Expr;
        use crate::tokenizer::tokenize;
        use std::str::FromStr;
        // let expr = "max((5*1)*x1+3*x2+2*x3+(10-3)*x4+4*x5)";
        // let expr = "1*3*x2+sin(8-2)*x3 - cos(pi)< 7";
        // let expr = "x1%5+3/3*x2+min(2,5)*x3*2e19 && 1";
        // let expr = "(x1+3)^(-2+sin(4^9))^3!--10*x1+x2^(-2+-3)+8<=5*2";
        // let expr = "(a^b)!^c";
        // let expr = "[1,2]*[3,4]+[a,b]"; //向量
        // let expr = "c+-a*-b"; // 单目的括号，c+-a*(-b)，这里仍然有些别扭，暂时没有好的办法
        // let expr = "1+(2+3*(4*5))"; //乘法和加法多余的括号，可以去括号，但是后缀表达式的顺序会发生更改
        let expr = "a*(3*cos(x2)/-8+3)/(a+b)/(c+d)+e==2"; //除法变为分式
        let rpn = Expr::from_str(expr).unwrap().rpn;
        println!("{:?}", rpn);
        let string = rpn_to_string(&rpn).unwrap();
        println!("{}", string);
        let latex = rpn_to_latex(&rpn).unwrap();
        println!("{}", latex);
        let rpn_test = to_rpn(&tokenize(string).unwrap()).unwrap();
        assert_eq!(rpn, rpn_test);
    }
}
