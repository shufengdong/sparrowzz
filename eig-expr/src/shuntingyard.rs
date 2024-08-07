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