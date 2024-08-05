// flowing should as same as in sparrowzz
use std::collections::{HashMap, VecDeque};

use log::warn;

use eig_expr::Expr;
use eig_expr::Token::*;
use eig_expr::{Operation, Token};
use eig_expr::{factorial, ContextProvider};

pub trait N: ContextProvider + Sync {}

impl<T> N for T where T: ContextProvider + Sync {}

pub fn parse_linear_expr_str(
    expr_str: &str,
    x_name_pos: &HashMap<String, usize>,
) -> Option<Vec<(usize, Expr)>> {
    let r: Vec<&str> = expr_str.split('=').collect();
    if r.len() == 2 {
        let left = r[0].parse::<Expr>().ok()?;
        let right = r[1].parse::<Expr>().ok()?;
        let left = parse_linear_expr(left.rpn, x_name_pos)?;
        let mut right = parse_linear_expr(right.rpn, x_name_pos)?;
        if merge_expr_map(left, &mut right, Operation::Minus) {
            return create_linear_expr(right);
        }
    } else {
        let left = expr_str.parse::<Expr>().ok()?;
        let left = parse_linear_expr(left.rpn, x_name_pos)?;
        return create_linear_expr(left);
    }
    None
}

pub fn split_linear_expr(
    rpn: Vec<Token>,
    x_name_pos: &HashMap<String, usize>,
) -> Option<Vec<(usize, Expr)>> {
    let final_map = parse_linear_expr(rpn, x_name_pos)?;
    create_linear_expr(final_map)
}

pub fn create_linear_expr(map: HashMap<usize, VecDeque<Token>>) -> Option<Vec<(usize, Expr)>> {
    let mut result = Vec::with_capacity(map.len());
    // 收集结果
    for (key, value) in map {
        let expr = Expr::from_vec(Vec::from(value));
        result.push((key, expr));
    }
    // 按照expr0, expr1, expr2, ... , exprN 进行排序
    result.sort_by(|a, b| a.0.cmp(&b.0));
    Some(result)
}

/// 分解表达式,将 1+2*x1+(3*4)*x2这样的表达式分解为[1,2,12]
pub fn parse_linear_expr(
    rpn: Vec<Token>,
    x_name_pos: &HashMap<String, usize>,
) -> Option<HashMap<usize, VecDeque<Token>>> {
    // 每个map里面存储了expr0 + expr1 * x1 + expr2 * x2 + ... + exprn * xn的系数
    let mut stack: Vec<HashMap<usize, VecDeque<Token>>> = Vec::with_capacity(16);
    // check model
    for token in rpn {
        match token {
            Binary(op) => {
                if stack.len() < 2 {
                    return None;
                }
                let mut right = stack.pop().unwrap();
                let left = stack.pop().unwrap();
                // 合并两个map,结果存储在right中
                if !merge_expr_map(left, &mut right, op) {
                    return None;
                }
                stack.push(right);
            }
            Unary(op) => {
                if stack.is_empty() {
                    return None;
                }
                let mut x = stack.pop().unwrap();
                match op {
                    Operation::Plus => {} // 无须做任何改动
                    Operation::Minus => {
                        for expr in x.values_mut() {
                            if expr.len() == 1 {
                                // 如果是常数参数，直接计算
                                if let Number(f) = expr[0] {
                                    expr[0] = Number(-f);
                                    continue;
                                }
                            }
                            expr.push_back(Unary(Operation::Minus));
                        }
                    }
                    Operation::Not => {
                        // x 必须只含有exp0
                        if x.len() != 1 {
                            return None;
                        }
                        if let Some(expr0) = x.get_mut(&0) {
                            if expr0.len() == 1 {
                                // 如果是常数参数，直接计算
                                if let Number(f) = expr0[0] {
                                    expr0[0] = Number(if f > 0.0 { 0.0 } else { 1.0 });
                                    stack.push(x);
                                    continue;
                                }
                            }
                            expr0.push_back(Unary(Operation::Not));
                        } else {
                            return None;
                        }
                    }
                    Operation::BitNot => {
                        // x 必须只含有exp0
                        if x.len() != 1 {
                            return None;
                        }
                        if let Some(expr0) = x.get_mut(&0) {
                            if expr0.len() == 1 {
                                // 如果是常数参数，直接计算
                                if let Number(f) = expr0[0] {
                                    expr0[0] = Number(!(f as i64) as f64);
                                    stack.push(x);
                                    continue;
                                }
                            }
                            expr0.push_back(Unary(Operation::BitNot));
                        } else {
                            return None;
                        }
                    }
                    Operation::Fact => {
                        // x 必须只含有exp0
                        if x.len() != 1 {
                            return None;
                        }
                        if let Some(expr0) = x.get_mut(&0) {
                            if expr0.len() == 1 {
                                // 如果是常数参数，直接计算
                                if let Number(f) = expr0[0] {
                                    expr0[0] = Number(factorial(f).ok()?);
                                    stack.push(x);
                                    continue;
                                }
                                expr0.push_back(Unary(Operation::Fact));
                            }
                        } else {
                            return None;
                        }
                    }
                    _ => return None,
                };
                stack.push(x);
            }
            Func(_, Some(i)) => {
                if stack.len() < i {
                    return None;
                }
                let mut para = VecDeque::with_capacity(i + 1);
                let mut k = 0;
                for j in stack.len() - i..stack.len() {
                    // x不允许在函数里面，只能包含expr0
                    if stack[j].len() != 1 {
                        return None;
                    }
                    if let Some(mut expr0) = stack[j].remove(&0) {
                        if expr0.len() == 1 {
                            // 如果是常数参数
                            if let Number(_) = expr0[0] {
                                k += 1;
                            }
                        }
                        while !expr0.is_empty() {
                            para.push_back(expr0.pop_front().unwrap());
                        }
                    } else {
                        return None;
                    }
                }
                para.push_back(token.clone());
                // 如果可以直接计算
                let new_expr0: VecDeque<Token> = if k == i {
                    let rpn = Vec::from(para);
                    let result = Expr::from_vec(rpn).eval().ok()?;
                    [Number(result)].into()
                } else {
                    para
                };
                // 如果是常数参数，直接计算
                let nl = stack.len() - i;
                stack.truncate(nl);
                stack.push([(0, new_expr0)].into());
            }
            Number(_) => {
                // 存储expr0
                let mut c: HashMap<usize, VecDeque<Token>> = HashMap::new();
                c.insert(0, [token].into());
                stack.push(c);
            }
            Var(ref var) => {
                let mut c: HashMap<usize, VecDeque<Token>> = HashMap::new();
                // 获得xi的下标
                if let Some(i) = x_name_pos.get(var) {
                    c.insert(*i + 1, [Number(1.0)].into());
                } else {
                    c.insert(0, [token].into());
                }
                stack.push(c);
            }
            _ => return None,
        }
    }
    if stack.len() != 1 {
        return None;
    }
    Some(stack.pop().unwrap())
}

/// 分解表达式，根据最后一个操作符是函数或二目运算符分解成多个或2个表达式
pub fn get_expr_from_fun(rpn: Vec<Token>) -> Option<Vec<Expr>> {
    let mut stack = Vec::with_capacity(16);
    enum TokenGroup {
        T(Token),
        G(Vec<Token>),
    }
    let total_token_num = rpn.len();
    let mut index = 0;
    for token in rpn {
        index += 1;
        match &token {
            Var(_) => {
                stack.push(TokenGroup::T(token));
            }
            Number(_) => stack.push(TokenGroup::T(token)),
            Binary(_) => {
                // 已经是最后一个操作符
                if index == total_token_num {
                    return if stack.len() == 2 {
                        let mut result = Vec::with_capacity(stack.len());
                        for g in stack {
                            match g {
                                TokenGroup::T(t) => result.push(Expr::from_vec(vec![t])),
                                TokenGroup::G(v) => result.push(Expr::from_vec(v)),
                            }
                        }
                        Some(result)
                    } else {
                        warn!("!!Illegal expression");
                        None
                    };
                }
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();
                let mut tokens = Vec::new();
                match left {
                    TokenGroup::T(t) => tokens.push(t),
                    TokenGroup::G(v) => {
                        tokens.extend(v);
                    }
                }
                match right {
                    TokenGroup::T(t) => tokens.push(t),
                    TokenGroup::G(v) => {
                        tokens.extend(v);
                    }
                }
                tokens.push(token);
                stack.push(TokenGroup::G(tokens));
            }
            Unary(_) => {
                let x = stack.pop().unwrap();
                let mut tokens = Vec::new();
                match x {
                    TokenGroup::T(t) => tokens.push(t),
                    TokenGroup::G(v) => {
                        tokens.extend(v);
                    }
                }
                tokens.push(token);
                stack.push(TokenGroup::G(tokens));
            }
            Func(_, Some(i)) => {
                let len = stack.len();
                if len < *i {
                    warn!(
                        "!!eval: stack does not have enough arguments for function token {:?}",
                        token
                    );
                    return None;
                } else if len == *i && index == total_token_num {
                    // the last func
                    let mut result = Vec::with_capacity(len);
                    for g in stack {
                        match g {
                            TokenGroup::T(t) => result.push(Expr::from_vec(vec![t])),
                            TokenGroup::G(v) => result.push(Expr::from_vec(v)),
                        }
                    }
                    return Some(result);
                } else {
                    let mut tokens = Vec::with_capacity(*i);
                    for _ in (len - *i)..len {
                        let g = stack.pop().unwrap();
                        match g {
                            TokenGroup::T(t) => tokens.push(t),
                            TokenGroup::G(mut v) => loop {
                                let t = v.pop();
                                if t.is_none() {
                                    break;
                                }
                                tokens.push(t.unwrap());
                            },
                        }
                    }
                    tokens.reverse();
                    tokens.push(token);
                    stack.push(TokenGroup::G(tokens));
                }
            }
            _ => {
                warn!("!!Unrecognized token: {:?}", token);
                return None;
            }
        }
    }
    warn!("!!Illegal expression");
    None
}

/// 合并两个map,结果存储在right中
pub fn merge_expr_map(
    mut left: HashMap<usize, VecDeque<Token>>,
    right: &mut HashMap<usize, VecDeque<Token>>,
    op: Operation,
) -> bool {
    let is_left_const = left.len() == 1 && left.contains_key(&0);
    let is_right_const = right.len() == 1 && right.contains_key(&0);
    if is_left_const && is_right_const {
        let left_expr0 = left.remove(&0).unwrap();
        let right_expr0 = right.get_mut(&0).unwrap();
        return merge_two_linear_expr(left_expr0, right_expr0, op);
    } else if is_left_const && !is_right_const {
        if op != Operation::Times && op != Operation::Plus && op != Operation::Minus {
            return false;
        }
        if op == Operation::Times {
            let left_expr0 = left.remove(&0).unwrap();
            for right_expr in right.values_mut() {
                if !merge_two_linear_expr(left_expr0.clone(), right_expr, op) {
                    return false;
                }
            }
            return true;
        }
    } else if !is_left_const && is_right_const {
        let right_expr0 = right.remove(&0).unwrap();
        if op != Operation::Times
            && op != Operation::Div
            && op != Operation::Plus
            && op != Operation::Minus
        {
            return false;
        }
        if op == Operation::Times || op == Operation::Div {
            for (index, left_expr) in left {
                let mut new_expr = right_expr0.clone();
                if !merge_two_linear_expr(left_expr, &mut new_expr, op) {
                    return false;
                }
                right.insert(index, new_expr);
            }
            return true;
        }
        right.insert(0, right_expr0);
    } else if op != Operation::Plus && op != Operation::Minus {
        return false;
    }
    // // 合并两个map,结果存储在right中
    if op == Operation::Minus {
        for right_expr in right.values_mut() {
            let left_expr = [Number(-1.0)].into();
            if !merge_two_linear_expr(left_expr, right_expr, Operation::Times) {
                return false;
            }
        }
    }
    for (index, left_expr) in left {
        if let Some(right_expr) = right.get_mut(&index) {
            if !merge_two_linear_expr(left_expr, right_expr, Operation::Plus) {
                return false;
            }
        } else {
            right.insert(index, left_expr);
        }
    }
    true
}

/// 合并两个系数方程
pub fn merge_two_linear_expr(
    mut left: VecDeque<Token>,
    right: &mut VecDeque<Token>,
    op: Operation,
) -> bool {
    match op {
        Operation::Plus => {
            if let Some(Number(l)) = fetch_number_token(&left) {
                if let Some(Number(r)) = fetch_number_token(right) {
                    let t = Number(l + r);
                    right.clear();
                    right.push_front(t);
                    return true;
                }
            }
        }
        Operation::Minus => {
            if let Some(Number(l)) = fetch_number_token(&left) {
                if let Some(Number(r)) = fetch_number_token(right) {
                    let t = Number(l - r);
                    right.clear();
                    right.push_front(t);
                    return true;
                }
            }
        }
        Operation::Times => {
            if let Some(Number(l)) = fetch_number_token(&left) {
                if let Some(Number(r)) = fetch_number_token(right) {
                    let t = Number(l * r);
                    right.clear();
                    right.push_front(t);
                    return true;
                }
            }
        }
        Operation::Div => {
            if let Some(Number(l)) = fetch_number_token(&left) {
                if let Some(Number(r)) = fetch_number_token(right) {
                    let t = Number(l / r);
                    right.clear();
                    right.push_front(t);
                    return true;
                }
            }
        }
        Operation::Rem => {
            if let Some(Number(l)) = fetch_number_token(&left) {
                if let Some(Number(r)) = fetch_number_token(right) {
                    let t = Number(l % r);
                    right.clear();
                    right.push_front(t);
                    return true;
                }
            }
        }
        Operation::Pow => {
            if let Some(Number(l)) = fetch_number_token(&left) {
                if let Some(Number(r)) = fetch_number_token(right) {
                    let t = Number(l.powf(r));
                    right.clear();
                    right.push_front(t);
                    return true;
                }
            }
        }
        Operation::Equal => {
            if let Some(Number(l)) = fetch_number_token(&left) {
                if let Some(Number(r)) = fetch_number_token(right) {
                    let t = Number(if l == r { 1.0 } else { 0.0 });
                    right.clear();
                    right.push_front(t);
                    return true;
                }
            }
        }
        Operation::Unequal => {
            if let Some(Number(l)) = fetch_number_token(&left) {
                if let Some(Number(r)) = fetch_number_token(right) {
                    let t = Number(if l != r { 1.0 } else { 0.0 });
                    right.clear();
                    right.push_front(t);
                    return true;
                }
            }
        }
        Operation::LessThan => {
            if let Some(Number(l)) = fetch_number_token(&left) {
                if let Some(Number(r)) = fetch_number_token(right) {
                    let t = Number(if l < r { 1.0 } else { 0.0 });
                    right.clear();
                    right.push_front(t);
                    return true;
                }
            }
        }
        Operation::GreatThan => {
            if let Some(Number(l)) = fetch_number_token(&left) {
                if let Some(Number(r)) = fetch_number_token(right) {
                    let t = Number(if l > r { 1.0 } else { 0.0 });
                    right.clear();
                    right.push_front(t);
                    return true;
                }
            }
        }
        Operation::LtOrEqual => {
            if let Some(Number(l)) = fetch_number_token(&left) {
                if let Some(Number(r)) = fetch_number_token(right) {
                    let t = Number(if l <= r { 1.0 } else { 0.0 });
                    right.clear();
                    right.push_front(t);
                    return true;
                }
            }
        }
        Operation::GtOrEqual => {
            if let Some(Number(l)) = fetch_number_token(&left) {
                if let Some(Number(r)) = fetch_number_token(right) {
                    let t = Number(if l >= r { 1.0 } else { 0.0 });
                    right.clear();
                    right.push_front(t);
                    return true;
                }
            }
        }
        Operation::And => {
            if let Some(Number(l)) = fetch_number_token(&left) {
                if let Some(Number(r)) = fetch_number_token(right) {
                    let t = Number(if (l > 0.0) && (r > 0.0) { 1.0 } else { 0.0 });
                    right.clear();
                    right.push_front(t);
                    return true;
                }
            }
        }
        Operation::Or => {
            if let Some(Number(l)) = fetch_number_token(&left) {
                if let Some(Number(r)) = fetch_number_token(right) {
                    let t = Number(if (l > 0.0) || (r > 0.0) { 1.0 } else { 0.0 });
                    right.clear();
                    right.push_front(t);
                    return true;
                }
            }
        }
        Operation::BitAnd => {
            if let Some(Number(l)) = fetch_number_token(&left) {
                if let Some(Number(r)) = fetch_number_token(right) {
                    let t = Number((l as i64 & r as i64) as f64);
                    right.clear();
                    right.push_front(t);
                    return true;
                }
            }
        }
        Operation::BitOr => {
            if let Some(Number(l)) = fetch_number_token(&left) {
                if let Some(Number(r)) = fetch_number_token(right) {
                    let t = Number((l as i64 | r as i64) as f64);
                    right.clear();
                    right.push_front(t);
                    return true;
                }
            }
        }
        Operation::BitXor => {
            if let Some(Number(l)) = fetch_number_token(&left) {
                if let Some(Number(r)) = fetch_number_token(right) {
                    let t = Number((l as i64 ^ r as i64) as f64);
                    right.clear();
                    right.push_front(t);
                    return true;
                }
            }
        }
        Operation::BitShl => {
            if let Some(Number(l)) = fetch_number_token(&left) {
                if let Some(Number(r)) = fetch_number_token(right) {
                    let t = Number(((l as i64) << (r as i64)) as f64);
                    right.clear();
                    right.push_front(t);
                    return true;
                }
            }
        }
        Operation::BitShr => {
            if let Some(Number(l)) = fetch_number_token(&left) {
                if let Some(Number(r)) = fetch_number_token(right) {
                    let t = Number(((l as i64) >> (r as i64)) as f64);
                    right.clear();
                    right.push_front(t);
                    return true;
                }
            }
        }
        Operation::BitAt => {
            if let Some(Number(l)) = fetch_number_token(&left) {
                if let Some(Number(r)) = fetch_number_token(right) {
                    if !(1.0..=64.0).contains(&r) {
                        return false;
                    }
                    let f = if (l as i64) & 2_i64.pow(r as u32 - 1) != 0 {
                        1.0
                    } else {
                        0.0
                    };
                    let t = Number(f);
                    right.clear();
                    right.push_front(t);
                    return true;
                }
            }
        }
        _ => return false,
    }
    while !left.is_empty() {
        let t = left.pop_back().unwrap();
        right.push_front(t);
    }
    right.push_back(Binary(op));
    true
}

/// 从表达式中提取数字
fn fetch_number_token(expr: &VecDeque<Token>) -> Option<Token> {
    if expr.len() == 1 {
        // 如果是常数参数
        if let Number(_) = expr[0] {
            return Some(expr[0].clone());
        }
    }
    None
}

pub enum XError {
    Wrongx,
}

#[allow(clippy::type_complexity)]
pub fn get_x_info(
    x_expr_str: &str,
) -> Result<(Vec<String>, Vec<u8>, HashMap<String, usize>, Vec<(usize, Expr)>, Vec<(usize, Expr)>), XError> {
    // 首先确定变量名称和类型
    let x: Vec<&str> = x_expr_str.split(',').collect();
    let mut x_name = Vec::with_capacity(x.len());
    let mut binary_int_float: Vec<u8> = Vec::with_capacity(x.len());
    let mut x_name_pos = HashMap::with_capacity(x.len());
    let mut x_upper = Vec::new();
    let mut x_lower = Vec::new();
    for (pos, x_s) in x.into_iter().enumerate() {
        let name_type: Vec<&str> = x_s.split(':').collect();
        if name_type.len() != 2 {
            return Err(XError::Wrongx);
        }
        // 处理变量名称
        let name = name_type[0].trim().to_string();
        x_name_pos.insert(name.clone(), pos);
        x_name.push(name);
        // 处理变量的类型和上下限
        let x_settings = name_type[1].trim();
        if x_settings.starts_with('[') {
            if x_settings.len() < 3 {
                return Err(XError::Wrongx);
            }
            let tmp: Vec<&str> = x_settings[1..x_settings.len() - 1].split('/').collect();
            // 设置类型
            let var_type: u8 = tmp[0].parse().map_err(|_| XError::Wrongx)?;
            if !(1..=3).contains(&var_type) {
                // 1: binary, 2: integer 3: float
                return Err(XError::Wrongx);
            }
            binary_int_float.push(var_type);
            if (var_type == 2 || var_type == 3) && tmp.len() == 3 {
                if !tmp[1].trim().is_empty() {
                    if let Ok(l) = tmp[1].parse() {
                        x_lower.push((pos, l));
                    } else {
                        return Err(XError::Wrongx);
                    }
                }
                if !tmp[2].trim().is_empty() {
                    if let Ok(u) = tmp[2].parse() {
                        x_upper.push((pos, u));
                    } else {
                        return Err(XError::Wrongx);
                    }
                }
            }
        } else {
            // 如果只有类型，没有设置上下限，默认是>=0
            let var_type: u8 = x_settings.parse().map_err(|_| XError::Wrongx)?;
            if !(1..=3).contains(&var_type) {
                // 1: binary, 2: integer 3: float
                return Err(XError::Wrongx);
            }
            binary_int_float.push(var_type);
        }
    }
    Ok((x_name, binary_int_float, x_name_pos, x_upper, x_lower))
}

pub fn create_x_name_init_cx(x_define: &Vec<String>) -> Option<(Vec<String>, Vec<Expr>)> {
    let mut x_name = Vec::with_capacity(x_define.len());
    let mut x_init = Vec::with_capacity(x_define.len());
    for x_define_i in x_define {
        let name_and_init: Vec<&str> = x_define_i.split(':').collect();
        if name_and_init.len() == 1 {
            // 处理变量名称
            x_name.push(name_and_init[0].trim().to_string());
            // 默认的初值是0
            x_init.push(Expr::new());
        } else if name_and_init.len() == 2 {
            // 处理变量名称
            x_name.push(name_and_init[0].trim().to_string());
            // 处理初值
            if let Ok(init) = name_and_init[1].trim().parse() {
                x_init.push(init);
            } else {
                x_init.push(Expr::new());
            }
        } else {
            return None;
        }
    }
    Some((x_name, x_init))
}

pub fn create_x_name_init(x_define: &Vec<String>) -> Option<(Vec<String>, Vec<Expr>)> {
    let mut x_name = Vec::with_capacity(x_define.len());
    let mut x_init = Vec::with_capacity(x_define.len());
    for x_define_i in x_define {
        let name_and_init: Vec<&str> = x_define_i.split(':').collect();
        if name_and_init.len() == 1 {
            // 处理变量名称
            x_name.push(name_and_init[0].trim().to_string());
            x_init.push(Expr::new());
        } else if name_and_init.len() == 2 {
            // 处理变量名称
            x_name.push(name_and_init[0].trim().to_string());
            // 处理初值
            if let Ok(init) = name_and_init[1].trim().parse() {
                x_init.push(init);
            } else {
                x_init.push(Expr::new());
            }
        } else {
            return None;
        }
    }
    Some((x_name, x_init))
}

pub fn read_parameters_from_str(parameters_str: &[&str]) -> Result<HashMap<String, String>, usize> {
    let mut parameters = HashMap::new();
    for (i, parameter_str) in parameters_str.iter().enumerate() {
        if parameter_str.trim().is_empty() {
            continue;
        }
        let kvs: Vec<&str> = parameter_str.split(':').collect();
        if kvs.len() == 2 {
            parameters.insert(kvs[0].trim().to_string(), kvs[1].trim().to_string());
        } else {
            return Err(i + 1);
        }
    }
    Ok(parameters)
}
// above should as same as in sparrowzz