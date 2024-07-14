use std::collections::HashMap;

use log::{trace, warn};
use serde::{Deserialize, Serialize};

use eig_expr::Expr;
use eig_expr::shuntingyard::{rpn_to_latex, rpn_to_string};
use eig_expr::Operation;
use eig_expr::Operation::*;
use eig_expr::Token::*;

use crate::find_points_in_expr;
use crate::solvers::utils::*;

/**
 * @api {由表达式组成的稀疏矩阵} /SparseMat SparseMat
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {usize} m m
 * @apiSuccess {usize} n n
 * @apiSuccess {tuple[]} u u，数组，tuple格式为(usize, usize, Expr)
 */
/// 由表达式组成的稀疏矩阵
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct SparseMat {
    pub m: usize,
    pub n: usize,
    pub v: Vec<(usize, usize, Expr)>,
}

/**
 * @api {由表达式组成的矩阵} /Mat Mat
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {usize} m m
 * @apiSuccess {usize} n n
 * @apiSuccess {Expr[]} v v
 */
/// 由表达式组成的矩阵
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Mat {
    pub(crate) m: usize,
    pub(crate) n: usize,
    pub(crate) v: Vec<Expr>,
}

/**
 * @api {混合整数线性规划求解器} /MILP MILP
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {String[]} x_name 变量名称
 * @apiSuccess {tuple[]} x_lower x_lower，数组，tuple格式为(usize, Expr)
 * @apiSuccess {tuple[]} x_upper x_upper，数组，tuple格式为(usize, Expr)
 * @apiSuccess {u8[]} binary_int_float 整数变量在x中的位置
 * @apiSuccess {Mat} a Ax >=/<= b
 * @apiSuccess {Expr[]} b b
 * @apiSuccess {Operation[]} constraint_type constraint_type
 * @apiSuccess {tuple[]} c min/max c^T*x，数组，tuple格式为(usize, Expr)
 * @apiSuccess {bool} min_or_max min: true, max: false
 * @apiSuccess {Map} parameters 参数Map，HashMap<String, String>
 */
/// 混合整数线性规划求解器
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct MILP {
    // 变量名称
    pub(crate) x_name: Vec<String>,
    pub x_lower: Vec<(usize, Expr)>,
    pub x_upper: Vec<(usize, Expr)>,
    // 整数变量在x中的位置
    pub binary_int_float: Vec<u8>,
    // Ax >=/<= b
    pub a: Mat,
    pub b: Vec<Expr>,
    pub constraint_type: Vec<Operation>,
    // min/max c^T*x
    pub c: Vec<Expr>,
    // min: true, max: false
    pub min_or_max: bool,
    pub parameters: HashMap<String, String>,
}

/**
 * @api {混合整数线性规划求解器，矩阵用稀疏矩阵} /SparseMILP SparseMILP
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {String[]} x_name 变量名称
 * @apiSuccess {tuple[]} x_lower x_lower，数组，tuple格式为(usize, Expr)
 * @apiSuccess {tuple[]} x_upper x_upper，数组，tuple格式为(usize, Expr)
 * @apiSuccess {u8[]} binary_int_float 整数变量在x中的位置
 * @apiSuccess {SparseMat} a Ax >=/<= b
 * @apiSuccess {Expr[]} b b
 * @apiSuccess {Operation[]} constraint_type constraint_type
 * @apiSuccess {tuple[]} c min/max c^T*x，数组，tuple格式为(usize, Expr)
 * @apiSuccess {bool} min_or_max min: true, max: false
 * @apiSuccess {Map} parameters 参数Map，HashMap<String, String>
 */
/// 混合整数线性规划求解器，矩阵用稀疏矩阵
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct SparseMILP {
    // 变量名称
    pub x_name: Vec<String>,
    pub x_lower: Vec<(usize, Expr)>,
    pub x_upper: Vec<(usize, Expr)>,
    // 整数变量在x中的位置
    pub binary_int_float: Vec<u8>,
    // Ax >=/<= b
    pub a: SparseMat,
    pub b: Vec<Expr>,
    pub constraint_type: Vec<Operation>,
    // min/max c^T*x
    pub c: Vec<(usize, Expr)>,
    // min: true, max: false
    pub min_or_max: bool,
    pub parameters: HashMap<String, String>,
}

/**
 * @api {稀疏线性方程组求解器} /SparseSolver SparseSolver
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {SparseMat} a a
 * @apiSuccess {Expr[]} b b
 * @apiSuccess {String[]} x_name x_name
 * @apiSuccess {Expr[]} x_init x_init
 * @apiSuccess {Map} parameters 参数Map，HashMap<String, String>
 */
/// 稀疏线性方程组Ax=b求解器
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct SparseSolver {
    pub a: SparseMat,
    pub b: Vec<Expr>,
    pub x_name: Vec<String>,
    pub x_init: Vec<Expr>,
    pub parameters: HashMap<String, String>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct SparseSolverCx {
    pub a: SparseMat,
    pub b: Vec<Expr>,
    pub x_name: Vec<String>,
    pub x_init: Vec<Expr>,
    pub parameters: HashMap<String, String>,
}

/// 线性方程组Ax=b求解器
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Solver {
    pub a: Mat,
    pub b: Vec<Expr>,
    pub x_name: Vec<String>,
    pub x_init: Vec<Expr>,
    pub parameters: HashMap<String, String>,
}

/**
 * @api {NLP} /NLP NLP
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {Expr} obj_expr min obj
 * @apiSuccess {String[]} x_name 变量名称
 * @apiSuccess {Expr[]} x_lower x_lower
 * @apiSuccess {Expr[]} x_upper x_upper
 * @apiSuccess {Expr[]} g g(x) >=/<=/== b
 * @apiSuccess {Expr[]} g_lower g_lower
 * @apiSuccess {Expr[]} g_upper g_upper
 * @apiSuccess {bool} min_or_max min: true, max: false
 * @apiSuccess {Map} parameters 参数Map，HashMap<String, String>
 */
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct NLP {
    // min obj
    pub obj_expr: Expr,
    // 变量名称
    pub x_name: Vec<String>,
    // 整数变量在x中的位置
    pub x_lower: Vec<Expr>,
    pub x_upper: Vec<Expr>,
    // g(x) >=/<=/== b
    pub g: Vec<Expr>,
    pub g_lower: Vec<Expr>,
    pub g_upper: Vec<Expr>,
    //x0
    pub x_init: Vec<Expr>,
    // min: true, max: false
    pub min_or_max: bool,
    pub parameters: HashMap<String, String>,
}

/**
 * @api {非线性方程求解器} /NewtonSolver NewtonSolver
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {Expr[]} f f
 * @apiSuccess {String[]} x_name x_name
 * @apiSuccess {Expr[]} x_init x_init
 * @apiSuccess {Expr[]} x_init_cx x_init_cx
 * @apiSuccess {Map} parameters 参数Map，HashMap<String, String>
 */
/// 非线性方程f(x)=b求解器
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct NewtonSolver {
    pub f: Vec<Expr>,
    pub x_name: Vec<String>,
    pub x_init: Vec<Expr>,
    pub x_init_cx: Vec<Expr>,
    pub parameters: HashMap<String, String>,
}

/// 加权最小二乘求解器
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct NewtonWls {
    pub f: Vec<Expr>,
    pub weight: Vec<Expr>,
    pub x_name: Vec<String>,
    pub x_init: Vec<Expr>,
    pub parameters: HashMap<String, String>,
}

impl Solver {
    /// 解析字符串表示的数学模型
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(expr_str: Vec<&str>) -> Result<Self, usize> {
        let mut expr_str = expr_str.clone();
        expr_str.retain(|expr| !expr.trim().is_empty());

        let s_n = expr_str.len();
        if s_n < 2 {
            warn!("!!Insufficient function num for solve Ax=b, content: {:?}",expr_str);
            return Err(s_n);
        }
        // 首先确定变量名称和类型
        let x_defines: Vec<String> = expr_str[s_n - 1]
            .split(',')
            .map(|s| s.to_string())
            .collect();
        if s_n != x_defines.len() + 1 {
            warn!("!!Insufficient function num for solve Ax=b, content: {:?}",expr_str);
            return Err(s_n);
        }
        let (x_name, x_init) = create_x_name_init(&x_defines).ok_or(s_n)?;
        let mut b = Vec::with_capacity(x_name.len());
        let mut a = Mat {
            m: x_name.len(),
            n: x_name.len(),
            v: Vec::new(),
        };
        for i in 0..x_name.len() {
            let s = expr_str[i];
            let constraint_expr = s.parse::<Expr>().map_err(|_| i + 1).map_err(|_| i + 1)?;
            let mut left_right = get_expr_from_fun(constraint_expr.rpn).ok_or(i + 1)?;
            let right = left_right.pop().ok_or(i + 1)?;
            b.push(right);
            let left_all = left_right.pop().ok_or(i + 1)?;
            let left = get_expr_from_fun(left_all.rpn).ok_or(i + 1)?;
            if left.len() != a.n {
                warn!("!!Insufficient expr num in A, content: {:?}", expr_str);
                return Err(i + 1);
            }
            for expr in left {
                a.v.push(expr);
            }
        }
        let parameters = HashMap::new();
        Ok(Solver { a, b, x_name, x_init, parameters })
    }

    pub fn from_str_with_parameters(
        expr_str: Vec<&str>,
        parameters_str: &[&str],
    ) -> Result<Solver, (usize, usize)> {
        match Solver::from_str(expr_str) {
            Ok(mut m) => match read_parameters_from_str(parameters_str) {
                Ok(parameters) => {
                    m.parameters = parameters;
                    Ok(m)
                }
                Err(n) => Err((1, n)),
            },

            Err(n) => Err((0, n)),
        }
    }

    // 分析相关的测点
    pub(crate) fn get_related_points(&self, alias: &HashMap<String, u64>) -> Vec<u64> {
        let mut result = Vec::new();
        for expr in &self.a.v {
            for (_, id, _) in find_points_in_expr(expr, alias) {
                if !result.contains(&id) {
                    result.push(id);
                }
            }
        }
        for expr in &self.b {
            for (_, id, _) in find_points_in_expr(expr, alias) {
                if !result.contains(&id) {
                    result.push(id);
                }
            }
        }
        result
    }
}

impl SparseSolver {
    /// 解析字符串表示的数学模型
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(expr_str: &[&str]) -> Result<Self, usize> {
        let mut expr_str = expr_str.to_vec();
        expr_str.retain(|expr| !expr.trim().is_empty());

        let s_n = expr_str.len();
        if s_n < 2 {
            warn!("!!Insufficient function num for solve Ax=b, content: {:?}",expr_str);
            return Err(s_n);
        }
        // 首先确定变量名称和类型
        let x_defines: Vec<String> = expr_str
            .pop()
            .unwrap()
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        if s_n != x_defines.len() + 1 {
            warn!("!!Insufficient function num for solve Ax=b, content: {:?}",expr_str);
            return Err(s_n);
        }
        let (x_name, x_init) = create_x_name_init(&x_defines).ok_or(s_n)?;
        let mut x_pos = HashMap::with_capacity(x_name.len());
        for (i, x_name_i) in x_name.iter().enumerate() {
            x_pos.insert(x_name_i.clone(), i);
        }
        let mut b = Vec::with_capacity(x_name.len());
        let mut a = SparseMat {
            m: x_name.len(),
            n: x_name.len(),
            v: Vec::new(),
        };
        for (i, expr_str_i) in expr_str.iter().enumerate() {
            let r = parse_linear_expr_str(expr_str_i, &x_pos).ok_or(i + 1)?;
            if r[0].0 != 0 {
                b.push(Expr::from_vec(vec![Number(0.0)]));
            }
            for (col, mut expr) in r {
                if col == 0 {
                    if expr.rpn.len() == 1 {
                        // 如果是常数参数，直接计算
                        if let Number(f) = expr.rpn[0] {
                            expr.rpn[0] = Number(-f);
                            b.push(expr);
                            continue;
                        }
                    }
                    expr.rpn.push(Unary(Minus));
                    b.push(expr);
                } else {
                    a.v.push((i, col - 1, expr));
                }
            }
        }
        let parameters = HashMap::new();
        Ok(SparseSolver { a, b, x_name, x_init, parameters })
    }

    pub fn from_str_with_parameters(
        expr_str: &[&str],
        parameters_str: &[&str],
    ) -> Result<SparseSolver, (usize, usize)> {
        match SparseSolver::from_str(expr_str) {
            Ok(mut m) => match read_parameters_from_str(parameters_str) {
                Ok(parameters) => {
                    m.parameters = parameters;
                    Ok(m)
                }
                Err(n) => Err((1, n)),
            },
            Err(n) => Err((0, n)),
        }
    }

    // 分析相关的测点
    pub(crate) fn get_related_points(&self, alias: &HashMap<String, u64>) -> Vec<u64> {
        let mut result = Vec::new();
        for (_, _, expr) in &self.a.v {
            for (_, id, _) in find_points_in_expr(expr, alias) {
                if !result.contains(&id) {
                    result.push(id);
                }
            }
        }
        for expr in &self.b {
            for (_, id, _) in find_points_in_expr(expr, alias) {
                if !result.contains(&id) {
                    result.push(id);
                }
            }
        }
        result
    }

    /// 将模型解析为字符串, (方程组, 变量定义, 求解参数)
    pub fn to_str(&self) -> (String, String, HashMap<String, String>) {
        // 方程组部分
        let mut result_fun = "".to_string();
        let mut v_index = 0;
        for i in 0..self.a.m {
            //m行
            let mut plus_flag = 0;
            for j in 0..self.a.n {
                //n列，系数矩阵
                if self.a.v[v_index].0 == i && self.a.v[v_index].1 == j {
                    //若此项存在
                    if let Ok(s) = rpn_to_string(&self.a.v[v_index].2.rpn) {
                        if plus_flag == 1 {
                            result_fun += "+"
                        }
                        if s != *"1" {
                            result_fun += &format!("{}*{}", s, self.x_name[j]);
                        } else {
                            result_fun += &self.x_name[j];
                        }
                        plus_flag = 1;
                    } else {
                        warn!("!!Failed to parse mat, row:{}, line:{}", i, j);
                    }
                    v_index += 1;
                    if v_index == self.a.v.len() {
                        break;
                    }
                }
            }
            if let Ok(s) = rpn_to_string(&self.b[i].rpn) {
                //常数项
                result_fun += &format!("={};", s);
            } else {
                warn!("!!Failed to parse b, row:{}", i);
            }
        }

        // 变量部分
        let mut result_var = "".to_string();
        for i in 0..self.x_name.len() {
            if i != 0 {
                result_var += ",";
            }
            result_var += &self.x_name[i];
            if let Ok(init) = rpn_to_string(&self.x_init[i].rpn) {
                if !init.is_empty() {
                    result_var += &format!(":{}", init);
                }
            }
        }

        // 求解参数部分
        let result_other = self.parameters.clone();
        // let mut result_other = "".to_string();
        // if !self.parameters.is_empty() {
        //     for (k, v) in &self.parameters {
        //         result_other += &format!("{}:{};\n", k, v);
        //     }
        // }

        (result_fun, result_var, result_other)
    }

    pub fn to_latex(&self) -> (String, String) {
        // 方程组部分
        let mut result_fun = "".to_string();
        let mut v_index = 0;
        for i in 0..self.a.m {
            //m行
            let mut plus_flag = 0;
            for j in 0..self.a.n {
                //n列，系数矩阵
                if self.a.v[v_index].0 == i && self.a.v[v_index].1 == j {
                    //若此项存在
                    if let Ok(s) = rpn_to_latex(&self.a.v[v_index].2.rpn) {
                        if plus_flag == 1 {
                            result_fun += "+"
                        }
                        if s != *"1" {
                            result_fun += &format!("{}\times {}", s, self.x_name[j]);
                        } else {
                            result_fun += &self.x_name[j];
                        }
                        plus_flag = 1;
                    } else {
                        warn!("!!Failed to parse mat, row:{}, line:{}", i, j);
                    }
                    v_index += 1;
                    if v_index == self.a.v.len() {
                        break;
                    }
                }
            }
            if let Ok(s) = rpn_to_latex(&self.b[i].rpn) {
                //常数项
                result_fun += &format!("={}\\\\\n", s);
            } else {
                warn!("!!Failed to parse b, row:{}", i);
            }
        }

        // 变量部分
        let mut result_var = "".to_string();
        for i in 0..self.x_name.len() {
            if i != 0 {
                result_var += ",";
            }
            result_var += &self.x_name[i];
            if let Ok(init) = rpn_to_latex(&self.x_init[i].rpn) {
                if !init.is_empty() {
                    result_var += &format!(":{}", init);
                }
            }
        }
        result_var += "\\\\\n";

        (result_fun, result_var)
    }
}

impl SparseSolverCx {
    /// 解析字符串表示的数学模型
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(expr_str: &[&str]) -> Result<Self, usize> {
        let mut expr_str = expr_str.to_vec();
        expr_str.retain(|expr| !expr.trim().is_empty());

        let s_n = expr_str.len();
        if s_n < 2 {
            warn!("!!Insufficient function num for solve Ax=b, content: {:?}",expr_str);
            return Err(s_n);
        }
        // 首先确定变量名称和类型
        let x_defines: Vec<String> = expr_str
            .pop()
            .unwrap()
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        if s_n != x_defines.len() + 1 {
            warn!("!!Insufficient function num for solve Ax=b, content: {:?}",expr_str);
            return Err(s_n);
        }
        let (x_name, x_init) = create_x_name_init_cx(&x_defines).ok_or(s_n)?;
        let mut x_pos = HashMap::with_capacity(x_name.len());
        for (i, x_name_i) in x_name.iter().enumerate() {
            x_pos.insert(x_name_i.clone(), i);
        }
        let mut b = Vec::with_capacity(x_name.len());
        let mut a = SparseMat {
            m: x_name.len(),
            n: x_name.len(),
            v: Vec::new(),
        };
        for (i, expr_str_i) in expr_str.iter().enumerate() {
            let r = parse_linear_expr_str(expr_str_i, &x_pos).ok_or(i + 1)?;
            if r[0].0 != 0 {
                b.push(Expr::from_vec(vec![Number(0.0)]));
            }
            for (col, mut expr) in r {
                if col == 0 {
                    if expr.rpn.len() == 1 {
                        // 如果是常数参数，直接计算
                        if let Number(f) = expr.rpn[0] {
                            expr.rpn[0] = Number(-f);
                            b.push(expr);
                            continue;
                        }
                    }
                    expr.rpn.push(Unary(Minus));
                    b.push(expr);
                } else {
                    a.v.push((i, col - 1, expr));
                }
            }
        }
        let parameters = HashMap::new();
        Ok(SparseSolverCx { a, b, x_name, x_init, parameters })
    }
}

impl MILP {
    /// 从字符串中获取模型
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(expr_str: &[&str]) -> Result<MILP, usize> {
        let mut expr_str = expr_str.to_vec();
        expr_str.retain(|expr| !expr.trim().is_empty());

        let s_n = expr_str.len();
        if s_n <= 2 {
            return Err(s_n);
        }
        // 首先确定变量名称和类型
        let (x_name, binary_int_float, _, x_upper, x_lower) =
            get_x_info(expr_str.pop().unwrap()).map_err(|_| s_n)?;
        // 处理目标函数
        let obj_exp: Expr = expr_str[0].parse().map_err(|_| 1usize)?;
        let n = obj_exp.len();
        let f = &obj_exp.rpn[n - 1];
        let min_or_max: bool;
        match f {
            Func(name, _) => {
                if name.to_uppercase() == "MIN" {
                    min_or_max = true;
                } else if name.to_uppercase() == "MAX" {
                    min_or_max = false;
                } else {
                    warn!("!!No min or max function found in obj expression: {:?}",obj_exp);
                    return Err(1);
                }
            }
            _ => {
                warn!("!!Not a function, obj expression: {:?}", obj_exp);
                return Err(1);
            }
        }
        let c: Vec<Expr> = get_expr_from_fun(obj_exp.rpn).ok_or(1usize)?;
        let m = expr_str.len() - 1;
        let n = x_name.len();
        let mut v = Vec::new();
        let mut b = Vec::with_capacity(m);
        let mut constraint_type = Vec::with_capacity(m);
        for (i, expr_str_i) in expr_str.iter().enumerate().skip(1) {
            let s = expr_str_i;
            let constraint_expr = s.parse::<Expr>().map_err(|_| i + 1)?;
            let token = &constraint_expr.rpn[constraint_expr.len() - 1];
            if let Binary(op) = token {
                if *op == GtOrEqual || *op == LtOrEqual || *op == Equal {
                    constraint_type.push(*op)
                } else {
                    warn!("!!The {}th constraint is wrong.", i);
                    return Err(i + 1);
                }
            } else {
                warn!("!!The {}th constraint is wrong.", i);
                return Err(i + 1);
            }
            let mut left_right = get_expr_from_fun(constraint_expr.rpn).ok_or(i + 1)?;
            b.push(left_right.pop().unwrap());
            let left = get_expr_from_fun(left_right.pop().unwrap().rpn).ok_or(i + 1)?;
            for expr in left {
                v.push(expr);
            }
        }
        let a = Mat { m, n, v };
        let parameters = HashMap::new();
        Ok(MILP {
            x_name,
            x_lower,
            x_upper,
            a,
            b,
            c,
            binary_int_float,
            constraint_type,
            min_or_max,
            parameters,
        })
    }

    pub fn from_str_with_parameters(
        expr_str: &[&str],
        parameters_str: &[&str],
    ) -> Result<MILP, (usize, usize)> {
        match MILP::from_str(expr_str) {
            Ok(mut m) => match read_parameters_from_str(parameters_str) {
                Ok(parameters) => {
                    m.parameters = parameters;
                    Ok(m)
                }
                Err(n) => Err((1, n)),
            },

            Err(n) => Err((0, n)),
        }
    }

    /// 分析相关的测点
    pub(crate) fn get_related_points(&self, alias: &HashMap<String, u64>) -> Vec<u64> {
        let mut result = Vec::new();
        for expr in &self.a.v {
            for (_, id, _) in find_points_in_expr(expr, alias) {
                if !result.contains(&id) {
                    result.push(id);
                }
            }
        }
        for expr in &self.b {
            for (_, id, _) in find_points_in_expr(expr, alias) {
                if !result.contains(&id) {
                    result.push(id);
                }
            }
        }
        for expr in &self.c {
            for (_, id, _) in find_points_in_expr(expr, alias) {
                if !result.contains(&id) {
                    result.push(id);
                }
            }
        }
        result
    }

    /// 将模型解析为字符串, (目标函数, 约束条件, 变量声明, 求解参数)
    pub fn to_str(&self) -> (String, String, String, HashMap<String, String>) {
        // 目标函数部分
        let mut result_fun = "".to_string();
        result_fun += if self.min_or_max { "min(" } else { "max(" };
        let mut plus_flag = 0;
        for i in 0..self.c.len() {
            if let Ok(s) = rpn_to_string(&self.c[i].rpn) {
                if plus_flag == 1 {
                    result_fun += ","
                }
                result_fun += &s.to_string();
                plus_flag = 1;
            } else {
                warn!("!!Failed to parse object function, index:{}", i);
            }
        }
        result_fun += ");";

        // 约束部分
        let mut result_cons = "".to_string();
        for i in 0..self.a.m {
            //m行
            let mut l = "".to_string();
            let mut plus_flag = 0;
            for j in i * self.a.n..(i + 1) * self.a.n {
                //n列，系数矩阵
                if plus_flag == 1 {
                    l += ",";
                }
                if let Ok(s) = rpn_to_string(&self.a.v[j].rpn) {
                    l += &s;
                    plus_flag = 1;
                } else {
                    warn!("!!Failed to parse mat");
                }
            }
            if let Ok(s) = rpn_to_string(&self.b[i].rpn) {
                //常数项
                let op = match self.constraint_type[i] {
                    Equal => "==",
                    Unequal => "!=",
                    LessThan => "<",
                    GreatThan => ">",
                    LtOrEqual => "<=",
                    GtOrEqual => ">=",
                    _ => "==",
                };
                result_cons += &format!("g({}){}{};", l, op, s);
            } else {
                warn!("!!Failed to parse constraint, row:{}", i);
            };
        }

        // 变量部分
        let mut result_var = "".to_string();
        for i in 0..self.x_name.len() {
            if i != 0 {
                result_var += ",";
            }
            result_var += &format!("{}:{}", self.x_name[i], self.binary_int_float[i]);
        }
        // 求解参数部分
        let result_other = self.parameters.clone();
        // let mut result_other = "".to_string();
        // if !self.parameters.is_empty() {
        //     for (k, v) in &self.parameters {
        //         result_other += &format!("{}:{};\n", k, v);
        //     }
        // }

        (result_fun, result_cons, result_var, result_other)
    }
}

impl SparseMILP {
    /// 从字符串中获取模型
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(expr_str: &[&str]) -> Result<SparseMILP, usize> {
        let mut expr_str = expr_str.to_vec();
        expr_str.retain(|expr| !expr.trim().is_empty());

        let s_n = expr_str.len();
        if s_n <= 2 {
            return Err(s_n);
        }
        // 首先确定变量名称和类型
        let (x_name, binary_int_float, x_pos, x_upper, x_lower) =
            get_x_info(expr_str.pop().unwrap()).map_err(|_| s_n)?;
        // 处理目标函数
        let mut obj_exp: Expr = expr_str[0].parse().map_err(|_| 1usize)?;
        // pop min or max func
        let f = obj_exp.rpn.pop().ok_or(1usize)?;
        let min_or_max: bool;
        match f {
            Func(name, _) => {
                trace!("obj func is {}.", name);
                if name.to_uppercase() == "MIN" {
                    min_or_max = true;
                } else if name.to_uppercase() == "MAX" {
                    min_or_max = false;
                } else {
                    warn!("!!No min or max function found in obj expression: {:?}",obj_exp);
                    return Err(1);
                }
            }
            _ => {
                warn!("!!Not a function, obj expression: {:?}", obj_exp);
                return Err(1);
            }
        }
        let mut c = split_linear_expr(obj_exp.rpn, &x_pos).ok_or(1usize)?;
        if c[0].0 == 0 {
            return Err(1);
        }
        // 下标从0开始，与MILP保持一致
        for c_i in c.iter_mut() {
            c_i.0 -= 1;
        }

        let m = expr_str.len() - 1;
        let n = x_name.len();
        let mut v = Vec::new();
        let mut b = Vec::with_capacity(m);
        let mut constraint_type = Vec::with_capacity(m);
        for (i, expr_str_i) in expr_str.iter().enumerate().skip(1) {
            let s = expr_str_i;
            let constraint_expr = s.parse::<Expr>().map_err(|_| i + 1)?;
            let token = &constraint_expr.rpn[constraint_expr.len() - 1];
            if let Binary(op) = token {
                if *op == GtOrEqual || *op == LtOrEqual || *op == Equal {
                    constraint_type.push(*op)
                } else {
                    warn!("!!The {}th constraint is wrong.", i);
                    return Err(i + 1);
                }
            } else {
                warn!("!!The {}th constraint is wrong.", i);
                return Err(i + 1);
            }
            let mut left_right = get_expr_from_fun(constraint_expr.rpn).ok_or(i + 1)?;
            if left_right.len() != 2 {
                return Err(i + 1);
            }
            let right = left_right.pop().unwrap();
            let left = left_right.pop().unwrap();
            let mut right = parse_linear_expr(right.rpn, &x_pos).ok_or(i + 1)?;
            let left = parse_linear_expr(left.rpn, &x_pos).ok_or(i + 1)?;
            if merge_expr_map(left, &mut right, Minus) {
                let r = create_linear_expr(right).ok_or(i + 1)?;
                if r[0].0 != 0 {
                    b.push(Expr::from_vec(vec![Number(0.0)]));
                }
                for (col, mut expr) in r {
                    if col == 0 {
                        if expr.rpn.len() == 1 {
                            // 如果是常数参数，直接计算
                            if let Number(f) = expr.rpn[0] {
                                expr.rpn[0] = Number(-f);
                                b.push(expr);
                                continue;
                            }
                        }
                        expr.rpn.push(Unary(Minus));
                        // check expression
                        if !expr.check_validity() {
                            return Err(i + 1);
                        }
                        b.push(expr);
                    } else {
                        // check expression
                        if !expr.check_validity() {
                            return Err(i + 1);
                        }
                        v.push((i - 1, col - 1, expr));
                    }
                }
            }
        }
        let a = SparseMat { m, n, v };
        let parameters = HashMap::new();
        Ok(SparseMILP {
            x_name,
            x_lower,
            x_upper,
            binary_int_float,
            a,
            b,
            constraint_type,
            c,
            min_or_max,
            parameters,
        })
    }

    pub fn from_str_with_parameters(
        expr_str: &[&str],
        parameters_str: &[&str],
    ) -> Result<SparseMILP, (usize, usize)> {
        match SparseMILP::from_str(expr_str) {
            Ok(mut m) => match read_parameters_from_str(parameters_str) {
                Ok(parameters) => {
                    m.parameters = parameters;
                    Ok(m)
                }
                Err(n) => Err((1, n)),
            },

            Err(n) => Err((0, n)),
        }
    }

    /// 分析相关的测点
    pub(crate) fn get_related_points(&self, alias: &HashMap<String, u64>) -> Vec<u64> {
        let mut result = Vec::new();
        for (_, _, expr) in &self.a.v {
            for (_, id, _) in find_points_in_expr(expr, alias) {
                if !result.contains(&id) {
                    result.push(id);
                }
            }
        }
        for expr in &self.b {
            for (_, id, _) in find_points_in_expr(expr, alias) {
                if !result.contains(&id) {
                    result.push(id);
                }
            }
        }
        for (_, expr) in &self.c {
            for (_, id, _) in find_points_in_expr(expr, alias) {
                if !result.contains(&id) {
                    result.push(id);
                }
            }
        }
        result
    }

    /// 将模型解析为字符串, (目标函数, 约束条件, 变量声明, 求解参数)
    pub fn to_str(&self) -> (String, String, String, HashMap<String, String>) {
        // 目标函数部分
        let mut result_fun = "".to_string();
        result_fun += if self.min_or_max { "min(" } else { "max(" };
        let mut plus_flag = 0;
        for i in 0..self.c.len() {
            if let Ok(s) = rpn_to_string(&self.c[i].1.rpn) {
                if plus_flag == 1 {
                    result_fun += "+";
                }
                if s != *"1" {
                    result_fun += &format!("{}*{}", s, &self.x_name[self.c[i].0]);
                } else {
                    result_fun += &self.x_name[self.c[i].0];
                }
                plus_flag = 1;
            } else {
                warn!("!!Failed to parse object function, index:{}", i);
            }
        }
        result_fun += ");";

        // 约束部分
        let mut result_cons = "".to_string();
        let mut v_index = 0;
        for i in 0..self.a.m {
            //m行
            let mut plus_flag = 0;
            for j in 0..self.a.n {
                //n列，系数矩阵
                if self.a.v[v_index].0 == i && self.a.v[v_index].1 == j {
                    //若此项存在
                    if let Ok(s) = rpn_to_string(&self.a.v[v_index].2.rpn) {
                        if plus_flag == 1 {
                            result_cons += "+";
                        }
                        if s != *"1" {
                            result_cons += &format!("{}*{}", s, &self.x_name[j]);
                        } else {
                            result_cons += &self.x_name[j];
                        }
                        plus_flag = 1;
                    } else {
                        warn!("!!Failed to parse mat, row:{}, line:{}", i, j);
                    }
                    v_index += 1;
                    if v_index == self.a.v.len() {
                        break;
                    }
                }
            }
            if let Ok(s) = rpn_to_string(&self.b[i].rpn) {
                //常数项
                let op = match self.constraint_type[i] {
                    Equal => "==",
                    Unequal => "!=",
                    LessThan => "<",
                    GreatThan => ">",
                    LtOrEqual => "<=",
                    GtOrEqual => ">=",
                    _ => "==",
                };
                result_cons += &format!("{}{};", op, s);
            } else {
                warn!("!!Failed to parse constraint, row:{}", i);
            }
            if v_index == self.a.v.len() {
                break;
            }
        }

        // 变量部分
        let mut result_var = "".to_string();
        for i in 0..self.x_name.len() {
            if i != 0 {
                result_var += ",";
            }
            result_var += &format!("{}:{}", self.x_name[i], self.binary_int_float[i]);
        }

        // 求解参数部分
        let result_other = self.parameters.clone();
        // let mut result_other = "".to_string();
        // if !self.parameters.is_empty() {
        //     for (k, v) in &self.parameters {
        //         result_other += &format!("{}:{};\n", k, v);
        //     }
        // }

        (result_fun, result_cons, result_var, result_other)
    }

    pub fn to_latex(&self) -> (String, String, String) {
        // 目标函数部分
        let mut result_fun = "".to_string();
        result_fun += if self.min_or_max { "\\min " } else { "\\max " };
        let mut x_index = 0;
        let mut plus_flag = 0;
        for i in 0..self.x_name.len() {
            if self.c[i].0 == x_index {
                //如果xi的系数存在
                if let Ok(s) = rpn_to_latex(&self.c[i].1.rpn) {
                    if plus_flag == 1 {
                        result_fun += "+";
                    }
                    if s != *"1" {
                        result_fun += &format!("{}\times {}", s, &self.x_name[i]);
                    } else {
                        result_fun += &self.x_name[i];
                    }
                    result_fun += &self.x_name[i];
                    plus_flag = 1;
                } else {
                    warn!("!!Failed to parse object function, index:{}", i);
                }
                x_index += 1;
            }
        }
        result_fun += ")\\\\\n";

        // 约束部分
        let mut result_cons = "".to_string();
        let mut v_index = 0;
        for i in 0..self.a.m {
            //m行
            let mut plus_flag = 0;
            for j in 0..self.a.n {
                //n列，系数矩阵
                if self.a.v[v_index].0 == i && self.a.v[v_index].1 == j {
                    //若此项存在
                    if let Ok(s) = rpn_to_latex(&self.a.v[v_index].2.rpn) {
                        if plus_flag == 1 {
                            result_cons += "+";
                        }
                        if s != *"1" {
                            result_cons += &format!("{}\times {}", s, &self.x_name[j]);
                        } else {
                            result_cons += &self.x_name[j];
                        }
                        plus_flag = 1;
                    } else {
                        warn!("!!Failed to parse mat, row:{}, line:{}", i, j);
                    }
                    v_index += 1;
                    if v_index == self.a.v.len() {
                        break;
                    }
                }
            }
            if let Ok(s) = rpn_to_latex(&self.b[i].rpn) {
                //常数项
                let op = match self.constraint_type[i] {
                    Equal => "=",
                    Unequal => "\\ne ",
                    LessThan => "<",
                    GreatThan => ">",
                    LtOrEqual => "\\le ",
                    GtOrEqual => "\\ge ",
                    _ => "=",
                };
                result_cons += &format!("{}{}\\\\\n", op, s);
            } else {
                warn!("!!Failed to parse constraint, row:{}", i);
            }
            if v_index == self.a.v.len() {
                break;
            }
        }

        // 变量部分
        let mut result_var = "".to_string();
        for i in 0..self.x_name.len() {
            let var_type = self.binary_int_float[i];
            if var_type == 1 {
                result_var += &format!("{} = 0 \\quad or \\quad 1", self.x_name[i]);
            } else if var_type == 2 {
                result_var += &format!("{} \\in \\mathbb{{z}}", self.x_name[i]);
            }
            result_var += "\\\\\n";
        }

        (result_fun, result_cons, result_var)
    }
}

impl NewtonSolver {
    // 解析字符串表示的数学模型
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(expr_str: &[&str]) -> Result<NewtonSolver, usize> {
        let mut expr_str = expr_str.to_vec();
        expr_str.retain(|expr| !expr.trim().is_empty());

        let s_n = expr_str.len();
        if s_n < 2 {
            warn!(
                "!!Insufficient function num for solve f(x)=0, content: {:?}",
                expr_str
            );
            return Err(s_n);
        }
        // 首先确定变量名称和类型
        let x_defines: Vec<String> = expr_str
            .pop()
            .unwrap()
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        let n = x_defines.len();
        if n != expr_str.len() {
            warn!("!!function num is not equal to x length, content: {:?}", expr_str);
            return Err(s_n);
        }
        // column of x
        let (x_name, x_init) = create_x_name_init(&x_defines).ok_or(s_n)?;
        let mut f = Vec::with_capacity(n);
        for (i, expr_str_i) in expr_str.iter().enumerate() {
            let expr: Expr = expr_str_i.parse().map_err(|_| i + 1)?;
            // 对方程进行校验
            if !expr.check_validity() {
                return Err(i + 1);
            }
            f.push(expr);
        }
        let parameters = HashMap::new();
        Ok(NewtonSolver {
            f,
            x_name,
            x_init,
            x_init_cx: vec![],
            parameters,
        })
    }

    pub fn from_str_with_parameters(
        expr_str: &[&str],
        parameters_str: &[&str],
    ) -> Result<NewtonSolver, (usize, usize)> {
        match NewtonSolver::from_str(expr_str) {
            Ok(mut m) => match read_parameters_from_str(parameters_str) {
                Ok(parameters) => {
                    m.parameters = parameters;
                    Ok(m)
                }
                Err(n) => Err((1, n)),
            },

            Err(n) => Err((0, n)),
        }
    }

    // 分析相关的测点
    pub(crate) fn get_related_points(&self, alias: &HashMap<String, u64>) -> Vec<u64> {
        let mut result = Vec::new();
        for nl_expr in &self.f {
            for (_, id, _) in find_points_in_expr(nl_expr, alias) {
                if !result.contains(&id) {
                    result.push(id);
                }
            }
        }
        result
    }

    /// 将模型解析为字符串, (方程组，变量声明，求解参数)
    pub fn to_str(&self) -> (String, String, HashMap<String, String>) {
        // 方程组部分
        let mut result_fun = "".to_string();
        for i in 0..self.f.len() {
            if let Ok(s) = rpn_to_string(&self.f[i].rpn) {
                result_fun += &format!("{};", s);
            } else {
                warn!("!!Failed to parse mat, row:{}", i);
            }
        }

        // 变量部分
        let mut result_var = "".to_string();
        for i in 0..self.x_name.len() {
            if i != 0 {
                result_var += ",";
            }
            result_var += &self.x_name[i];
            if let Ok(init) = rpn_to_string(&self.x_init[i].rpn) {
                if !init.is_empty() {
                    result_var += &format!(":{}", init);
                }
            }
        }

        // 求解参数部分
        let result_other = self.parameters.clone();
        // let mut result_other = "".to_string();
        // if !self.parameters.is_empty() {
        //     for (k, v) in &self.parameters {
        //         result_other += &format!("{}:{};\n", k, v);
        //     }
        // }

        (result_fun, result_var, result_other)
    }

    pub fn to_latex(&self) -> (String, String) {
        // 方程组部分
        let mut result_fun = "".to_string();
        for i in 0..self.f.len() {
            if let Ok(s) = rpn_to_latex(&self.f[i].rpn) {
                result_fun += &format!("{} = 0\\\\\n", s);
            } else {
                warn!("!!Failed to parse mat, row:{}", i);
            }
        }

        // 变量部分
        let mut result_var = "".to_string();
        for i in 0..self.x_name.len() {
            if i != 0 {
                result_var += ",";
            }
            result_var += &self.x_name[i];
            if let Ok(init) = rpn_to_latex(&self.x_init[i].rpn) {
                if !init.is_empty() {
                    result_var += &format!(":{}", init);
                }
            }
        }

        (result_fun, result_var)
    }
}

impl NewtonWls {
    /// 从字符串中获取模型
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(expr_str: Vec<&str>) -> Result<NewtonWls, usize> {
        let mut expr_str = expr_str.clone();
        expr_str.retain(|expr| !expr.trim().is_empty());
        let sn = expr_str.len();
        if sn < 3 {
            warn!("!!Insufficient function num for solve wls, content: {:?}", expr_str);
            return Err(sn);
        }
        // 首先确定变量名称和类型
        let x_defines: Vec<String> = expr_str
            .pop()
            .unwrap()
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        // column of x
        let (x_name, x_init) = create_x_name_init(&x_defines).ok_or(sn)?;
        let mut f = Vec::with_capacity(sn - 1);
        let mut weight = Vec::with_capacity(sn - 1);
        for (i, expr_str_i) in expr_str.iter().enumerate() {
            let f_s = expr_str_i;
            let f_and_weight: Vec<&str> = f_s.split(':').collect();
            if f_and_weight.len() == 1 {
                weight.push(Expr::from_vec(vec![Number(1.0)]));
            } else if f_and_weight.len() == 2 {
                let expr: Expr = f_and_weight[1].parse().map_err(|_| i + 1)?;
                // 校验
                if !expr.check_validity() {
                    return Err(i + 1);
                }
                weight.push(expr);
            } else if f_and_weight.len() > 2 {
                return Err(i + 1);
            };
            let expr: Expr = f_and_weight[0].parse().map_err(|_| i + 1)?;
            // 对方程进行校验
            if !expr.check_validity() {
                return Err(i + 1);
            }
            f.push(expr);
        }
        let parameters = HashMap::new();
        Ok(NewtonWls { f, weight, x_name, x_init, parameters })
    }

    pub fn from_str_with_parameters(
        expr_str: Vec<&str>,
        parameters_str: &[&str],
    ) -> Result<NewtonWls, (usize, usize)> {
        match NewtonWls::from_str(expr_str) {
            Ok(mut m) => match read_parameters_from_str(parameters_str) {
                Ok(parameters) => {
                    m.parameters = parameters;
                    Ok(m)
                }
                Err(n) => Err((1, n)),
            },

            Err(n) => Err((0, n)),
        }
    }
}

impl NLP {
    /// 从字符串中获取模型
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(expr_str: &[&str]) -> Result<NLP, usize> {
        let mut expr_str = expr_str.to_vec();
        expr_str.retain(|expr| !expr.trim().is_empty());
        let n = expr_str.len();
        if n < 2 {
            return Err(n);
        }
        // 首先确定变量名称和类型
        let x: Vec<&str> = expr_str.pop().unwrap().split(',').collect();
        let mut x_name = Vec::with_capacity(x.len());
        let mut x_init: Vec<Expr> = Vec::with_capacity(x.len());
        let mut x_lower: Vec<Expr> = Vec::with_capacity(x.len());
        let mut x_upper: Vec<Expr> = Vec::with_capacity(x.len());
        for x_s in x {
            let name_and_limit_and_init: Vec<&str> = x_s.split(':').collect();
            if name_and_limit_and_init.len() != 2 {
                return Err(n);
            }
            x_name.push(name_and_limit_and_init[0].trim().to_string());
            let limits_and_init = name_and_limit_and_init[1].trim();
            if limits_and_init.len() < 3 {
                return Err(n);
            }
            let tmp: Vec<&str> = limits_and_init[1..limits_and_init.len() - 1]
                .split('/')
                .collect();
            if (tmp.len() != 2) && (tmp.len() != 3) {
                return Err(n);
            }
            if tmp[0].trim().is_empty() {
                x_lower.push(Expr::from_vec(vec![Number(f64::MIN)]));
            } else if let Ok(l) = tmp[0].parse() {
                x_lower.push(l);
            }
            if tmp[1].trim().is_empty() {
                x_upper.push(Expr::from_vec(vec![Number(f64::MAX)]));
            } else if let Ok(l) = tmp[1].parse() {
                x_upper.push(l);
            }
            if tmp.len() == 3 && !tmp[2].trim().is_empty() {
                if let Ok(l) = tmp[2].parse() {
                    x_init.push(l);
                }
            } else {
                x_init.push(Expr::new());
            }
        }
        // 处理目标函数
        let mut obj_expr: Expr = expr_str[0].parse().map_err(|_| n)?;
        if !obj_expr.check_validity() {
            return Err(n);
        }
        // pop min or max func
        let f = obj_expr.rpn.pop().ok_or(1usize)?;
        let mut min_or_max = true;
        match &f {
            Func(name, _) => {
                if name.to_uppercase() == "MIN" {
                    min_or_max = true;
                } else if name.to_uppercase() == "MAX" {
                    min_or_max = false;
                } else {
                    obj_expr.rpn.push(f);
                }
            }
            _ => {
                obj_expr.rpn.push(f);
            }
        }
        let m = expr_str.len() - 1;
        let mut g = Vec::with_capacity(m);
        let mut g_lower = Vec::with_capacity(m);
        let mut g_upper = Vec::with_capacity(m);
        for (i, expr_str_i) in expr_str.iter().enumerate().skip(1) {
            let g_s = expr_str_i;
            let g_and_limit: Vec<&str> = g_s.split(':').collect();
            if g_and_limit.len() != 2 {
                return Err(i + 1);
            }
            let constraint_expr = g_and_limit[0].trim().parse::<Expr>().map_err(|_| i + 1)?;
            if !constraint_expr.check_validity() {
                return Err(i + 1);
            }
            g.push(constraint_expr);
            // find limit of constraints
            let limits = g_and_limit[1].trim();
            let tmp: Vec<&str> = limits[1..limits.len() - 1].split('/').collect();
            if tmp.len() != 2 {
                return Err(i + 1);
            }
            if tmp[0].trim().is_empty() {
                g_lower.push(Expr::from_vec(vec![Number(f64::MIN)]));
            } else if let Ok(l) = tmp[0].parse() {
                g_lower.push(l);
            }
            if tmp[1].trim().is_empty() {
                g_upper.push(Expr::from_vec(vec![Number(f64::MAX)]));
            } else if let Ok(l) = tmp[1].parse() {
                g_upper.push(l);
            }
        }
        let parameters = HashMap::new();
        Ok(NLP {
            x_name,
            x_lower,
            x_upper,
            g,
            g_upper,
            g_lower,
            obj_expr,
            x_init,
            min_or_max,
            parameters,
        })
    }

    /// 从字符串中获取模型及求解参数
    pub fn from_str_with_parameters(
        expr_str: &[&str],
        parameters_str: &[&str],
    ) -> Result<NLP, (usize, usize)> {
        match NLP::from_str(expr_str) {
            Ok(mut m) => match read_parameters_from_str(parameters_str) {
                Ok(parameters) => {
                    m.parameters = parameters;
                    Ok(m)
                }
                Err(n) => Err((1, n)),
            },

            Err(n) => Err((0, n)),
        }
    }

    pub(crate) fn get_related_points(&self, alias: &HashMap<String, u64>) -> Vec<u64> {
        let mut result = Vec::new();
        for (_, id, _) in find_points_in_expr(&self.obj_expr, alias) {
            if !result.contains(&id) {
                result.push(id);
            }
        }
        for expr in &self.x_lower {
            for (_, id, _) in find_points_in_expr(expr, alias) {
                if !result.contains(&id) {
                    result.push(id);
                }
            }
        }
        for expr in &self.x_upper {
            for (_, id, _) in find_points_in_expr(expr, alias) {
                if !result.contains(&id) {
                    result.push(id);
                }
            }
        }
        for expr in &self.g {
            for (_, id, _) in find_points_in_expr(expr, alias) {
                if !result.contains(&id) {
                    result.push(id);
                }
            }
        }
        for expr in &self.g_lower {
            for (_, id, _) in find_points_in_expr(expr, alias) {
                if !result.contains(&id) {
                    result.push(id);
                }
            }
        }
        for expr in &self.g_upper {
            for (_, id, _) in find_points_in_expr(expr, alias) {
                if !result.contains(&id) {
                    result.push(id);
                }
            }
        }
        result
    }

    /// 将模型解析为字符串, (目标函数, 约束条件, 变量定义, 求解参数)
    pub fn to_str(&self) -> (String, String, String, HashMap<String, String>) {
        // 目标函数部分
        let mut result_fun = "".to_string();
        if let Ok(s) = rpn_to_string(&self.obj_expr) {
            if self.min_or_max {
                result_fun = format!("{};", s);
            } else {
                result_fun = format!("max({});", s);
            }
        } else {
            warn!("!!Failed to parse object function");
        }

        // 约束部分
        let mut result_cons = "".to_string();
        for i in 0..self.g.len() {
            if let Ok(s) = rpn_to_string(&self.g[i].rpn) {
                if let Ok(mut l) = rpn_to_string(&self.g_lower[i].rpn) {
                    if let Ok(mut u) = rpn_to_string(&self.g_upper[i].rpn) {
                        if self.g_lower[i].rpn[0] == Number(f64::MIN) {
                            l = "".to_string();
                        }
                        if self.g_upper[i].rpn[0] == Number(f64::MAX) {
                            u = "".to_string();
                        }
                        result_cons += &format!("{}:[{}/{}];", s, l, u);
                    }
                }
            }
        }

        // 变量部分
        let mut result_var = "".to_string();
        for i in 0..self.x_name.len() {
            if let Ok(mut l) = rpn_to_string(&self.x_lower[i].rpn) {
                //下限
                if let Ok(mut u) = rpn_to_string(&self.x_upper[i].rpn) {
                    //上限
                    if self.x_lower[i].rpn[0] == Number(f64::MIN) {
                        l = "".to_string();
                    }
                    if self.x_upper[i].rpn[0] == Number(f64::MAX) {
                        u = "".to_string();
                    }
                    let mut init = "".to_string();
                    if !self.x_init[i].rpn.is_empty() {
                        if let Ok(v) = rpn_to_string(&self.x_init[i].rpn) {
                            init = "/".to_string() + &v;
                        }
                    }
                    result_var += &format!("{}:[{}/{}{}]", self.x_name[i], l, u, init);
                }
            }
            if i != self.x_name.len() - 1 {
                result_var += ",";
            }
        }

        // 求解参数部分
        let result_other = self.parameters.clone();
        // let mut result_other = "".to_string();
        // if !self.parameters.is_empty() {
        //     for (k, v) in &self.parameters {
        //         result_other += &format!("{}:{};\n", k, v);
        //     }
        // }

        (result_fun, result_cons, result_var, result_other)
    }

    pub fn to_latex(&self) -> (String, String, String) {
        // 目标函数部分
        let mut result_fun = "".to_string();
        result_fun += if self.min_or_max { "\\min " } else { "\\max " };
        if let Ok(s) = rpn_to_latex(&self.obj_expr) {
            result_fun += &format!("{}\\\\\n", s);
        } else {
            warn!("!!Failed to parse object function");
        }

        // 约束部分
        let mut result_cons = "".to_string();
        for i in 0..self.g.len() {
            if let Ok(s) = rpn_to_latex(&self.g[i].rpn) {
                if let Ok(mut l) = rpn_to_latex(&self.g_lower[i].rpn) {
                    l += "\\le ";
                    if let Ok(mut u) = rpn_to_latex(&self.g_upper[i].rpn) {
                        if self.g_lower[i].rpn[0] == self.g_upper[i].rpn[0] {
                            l = "".to_string();
                            u = "=".to_string() + &u;
                        } else {
                            u = "\\le ".to_string() + &u;
                            if self.g_lower[i].rpn[0] == Number(f64::MIN) {
                                l = "".to_string();
                            }
                            if self.g_upper[i].rpn[0] == Number(f64::MAX) {
                                u = "".to_string();
                            }
                        }
                        result_cons += &format!("{}{}{}\\\\\n", l, s, u);
                    }
                }
            }
        }

        // 变量部分
        let mut result_var = "".to_string();
        for i in 0..self.x_name.len() {
            if let Ok(mut l) = rpn_to_latex(&self.x_lower[i].rpn) {
                //下限
                l += "\\le ";
                if let Ok(mut u) = rpn_to_latex(&self.x_upper[i].rpn) {
                    //上限
                    if self.x_lower[i].rpn[0] == self.x_upper[i].rpn[0] {
                        l = "".to_string();
                        u = "=".to_string() + &u;
                    } else {
                        u = "\\le ".to_string() + &u;
                        if self.x_lower[i].rpn[0] == Number(f64::MIN) {
                            l = "".to_string();
                        }
                        if self.x_upper[i].rpn[0] == Number(f64::MAX) {
                            u = "".to_string();
                        }
                    }
                    let mut init = "".to_string();
                    if !self.x_init[i].rpn.is_empty() {
                        if let Ok(v) = rpn_to_latex(&self.x_init[i].rpn) {
                            init = "(".to_string() + &v + ")";
                        }
                    }

                    if !l.is_empty() || !u.is_empty() {
                        result_var += &format!("{}{}{}{}\\\\\n", l, self.x_name[i], init, u);
                    }
                }
            }
        }

        (result_fun, result_cons, result_var)
    }
}

impl NewtonSolver {
    pub fn to_nlp(&self) -> NLP{
        NLP {
            x_name: self.x_name.clone(),
            x_init: self.x_init.clone(),
            x_lower: vec![Expr::from_vec(vec![Number(f64::MIN)]); self.f.len()],
            x_upper: vec![Expr::from_vec(vec![Number(f64::MAX)]); self.f.len()],
            g: self.f.clone(),
            g_lower: vec![Expr::from_vec(vec![Number(0.)]); self.f.len()],
            g_upper: vec![Expr::from_vec(vec![Number(0.)]); self.f.len()],
            obj_expr: Expr::from_vec(vec![Number(0.)]),
            min_or_max: true,
            parameters: HashMap::new(),
        }
    }
}

impl SparseSolver {
    pub fn to_sparsemilp(&self) -> SparseMILP {
        let mut x_lower = Vec::with_capacity(self.x_name.len());
        let mut x_upper = Vec::with_capacity(self.x_name.len());
        for i in 0..self.x_name.len() {
            x_lower.push((i, Expr::from_vec(vec![Number(f64::MIN)])));
            x_upper.push((i, Expr::from_vec(vec![Number(f64::MAX)])));
        }

        SparseMILP {
            x_name: self.x_name.clone(),
            x_lower,
            x_upper,
            binary_int_float: vec![3_u8; self.x_name.len()],
            a: self.a.clone(),
            b: self.b.clone(),
            constraint_type: vec![Operation::Equal; self.b.len()],
            c: vec![(0, Expr::from_vec(vec![Number(0.)]))],
            min_or_max: true,
            parameters: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::solvers::{MILP, NewtonSolver, NLP, SparseMILP, SparseSolver};

    #[test]
    fn test_parse() {
        let x = "1";
        let y: Vec<&str> = x.split('/').collect();
        assert_eq!(1, y.len());
        let x = "1//";
        let y: Vec<&str> = x.split('/').collect();
        assert_eq!(3, y.len());
        let x = "3/-4/";
        let y: Vec<&str> = x.split('/').collect();
        assert_eq!(3, y.len());
        let x = "1/";
        let y: Vec<&str> = x.split('/').collect();
        assert_eq!(2, y.len());
    }

    #[test]
    fn test_solve_to_str() {
        let s = "+224.7319*x1-35.5872*x2-32.8947*x4-156.25*x5=2.1;\n
                        -35.5872*x1+128.1798*x2-92.5926*x3=-3;\n
                        -92.5926*x2+126.2626*x3-33.67*x4=0.2349;\n
                        x4=0;\n
                        -156.25*x1-33.67*x4+3*x4-2*(x1+x2)=4.6651;\n
                        x1,x2,x3,x4,x5";
        let expr_str: Vec<&str> = s.split(';').collect();
        let model = SparseSolver::from_str(&expr_str).unwrap();
        let to_str_temp = model.to_str();
        let to_str = format!("{}\n{}", to_str_temp.0, to_str_temp.1);
        // println!("{}",to_str);
        let expr_str2: Vec<&str> = to_str.split(';').collect();
        let str_to_model = SparseSolver::from_str(&expr_str2).unwrap();
        assert_eq!(model, str_to_model);
        let to_latex_temp = model.to_latex();
        let to_latex = format!("{}{}", to_latex_temp.0, to_latex_temp.1);
        println!("{}", to_latex)
    }

    #[test]
    fn test_nlsolve_to_str() {
        let s = "2.10000000-0.00000000-V1*(V1*(22.25068569)+V2*(-3.52348402*cos(THETA1-THETA2)+35.23484021*sin(THETA1-THETA2))+V4*(-3.25690464*cos(THETA1-THETA4)+32.56904638*sin(THETA1-THETA4))+V5*(-15.47029703*cos(THETA1-THETA5)+154.70297030*sin(THETA1-THETA5)));
                        0.00000000-3.00000000-V2*(V1*(-3.52348402*cos(THETA2-THETA1)+35.23484021*sin(THETA2-THETA1))+V2*(12.69106745)+V3*(-9.16758343*cos(THETA2-THETA3)+91.67583425*sin(THETA2-THETA3)));
                        3.23490000-3.00000000-V3*(V2*(-9.16758343*cos(THETA3-THETA2)+91.67583425*sin(THETA3-THETA2))+V3*(12.50125013)+V4*(-3.33366670*cos(THETA3-THETA4)+33.33666700*sin(THETA3-THETA4)));
                        PG_balancenode-4.00000000-V4*(V1*(-3.25690464*cos(THETA4-THETA1)+32.56904638*sin(THETA4-THETA1))+V3*(-3.33366670*cos(THETA4-THETA3)+33.33666700*sin(THETA4-THETA3))+V4*(9.92423804)+V5*(-3.33366670*cos(THETA4-THETA5)+33.33666700*sin(THETA4-THETA5)));
                        4.66510000-0.00000000-V5*(V1*(-15.47029703*cos(THETA5-THETA1)+154.70297030*sin(THETA5-THETA1))+V4*(-3.33366670*cos(THETA5-THETA4)+33.33666700*sin(THETA5-THETA4))+V5*(18.80396373));
                        QG1+QG2-0.00000000-V1*(V1*(--222.48437689)+V2*(-3.52348402*sin(THETA1-THETA2)-35.23484021*cos(THETA1-THETA2))+V4*(-3.25690464*sin(THETA1-THETA4)-32.56904638*cos(THETA1-THETA4))+V5*(-15.47029703*sin(THETA1-THETA5)-154.70297030*cos(THETA1-THETA5)));
                        -0.98610000-V2*(V1*(-3.52348402*sin(THETA2-THETA1)-35.23484021*cos(THETA2-THETA1))+V2*(--126.89785446)+V3*(-9.16758343*sin(THETA2-THETA3)-91.67583425*cos(THETA2-THETA3)));
                        QG3-0.98610000-V3*(V2*(-9.16758343*sin(THETA3-THETA2)-91.67583425*cos(THETA3-THETA2))+V3*(--124.99987125)+V4*(-3.33366670*sin(THETA3-THETA4)-33.33666700*cos(THETA3-THETA4)));
                        QG4-1.31470000-V4*(V1*(-3.25690464*sin(THETA4-THETA1)-32.56904638*cos(THETA4-THETA1))+V3*(-3.33366670*sin(THETA4-THETA3)-33.33666700*cos(THETA4-THETA3))+V4*(--99.23235038)+V5*(-3.33366670*sin(THETA4-THETA5)-33.33666700*cos(THETA4-THETA5)));
                        QG5-0.00000000-V5*(V1*(-15.47029703*sin(THETA5-THETA1)-154.70297030*cos(THETA5-THETA1))+V4*(-3.33366670*sin(THETA5-THETA4)-33.33666700*cos(THETA5-THETA4))+V5*(--188.02063730));
                        V1-1.00000000;
                        V3-1.00000000;
                        V4-1.00000000;
                        V5-1.00000000;
                        THETA4;
                        QG1/0.40000000-QG2/1.70000000;
                        V1:1,V2:1,V3:1,V4:1,V5:1,THETA1,THETA2,THETA3,THETA4,THETA5,QG1,QG2,QG3,QG4,QG5,PG_balancenode";
        let expr_str: Vec<&str> = s.split(';').collect();
        let model = NewtonSolver::from_str(&expr_str).unwrap();
        let to_str_temp = model.to_str();
        let to_str = format!("{}\n{}", to_str_temp.0, to_str_temp.1);
        println!("{}", to_str);
        let expr_str2: Vec<&str> = to_str.split(';').collect();
        let str_to_model = NewtonSolver::from_str(&expr_str2).unwrap();
        assert_eq!(model, str_to_model);
        let to_latex_temp = model.to_latex();
        let to_latex = format!("{}\n{}", to_latex_temp.0, to_latex_temp.1);
        println!("{}", to_latex);
    }

    #[test]
    fn test_simple_milp_to_str() {
        let s = "max( 3*x2+5*x1+2*x3+(10-3)*x4+4*x5);
                        2*x1+(2*4)*x2+4*x3+2*x4+max(1,5)*x5 <= 5*2;
                        2 + x1 >= 3;
                        x1:1,x2:1,x3:1,x4:1,x5:1";
        let expr_str: Vec<&str> = s.split(';').collect();
        let model = SparseMILP::from_str(&expr_str).unwrap();

        let to_str_temp = model.to_str();
        let to_str = format!("{}\n{}{}", to_str_temp.0, to_str_temp.1, to_str_temp.2);
        // println!("{}",to_str);
        let expr_str2: Vec<&str> = to_str.split(';').collect();
        let str_to_model = SparseMILP::from_str(&expr_str2).unwrap();
        assert_eq!(model, str_to_model);

        let to_latex_temp = model.to_latex();
        let to_latex = format!("{}{}{}", to_latex_temp.0, to_latex_temp.1, to_latex_temp.2);
        println!("{}", to_latex);
    }

    #[test]
    fn test_milp_to_str() {
        let expr_str = [
            "max(4+1,3,0,10-3,4)",              // 目标函数
            "g(2, 2*4, 4, 2, max(1,5)) <= 5*2", // 约束
            "g(2, 2*4, 0, 0, 3) <= 5*2",        // 约束
            "t(1,2,3,4,5) <= 8",
            "x1:1,x2:1,x3:1,x4:1,x5:1", // 变量的名称、顺序及其类型
        ]
        .to_vec();
        let model = MILP::from_str(&expr_str).unwrap();
        let to_str_temp = model.to_str();
        let to_str = format!("{}\n{}{}", to_str_temp.0, to_str_temp.1, to_str_temp.2);
        let expr_str2: Vec<&str> = to_str.split(';').collect();
        // println!("{}",to_str);
        let str_to_model = MILP::from_str(&expr_str2).unwrap();
        assert_eq!(model, str_to_model);
    }

    #[test]
    fn test_nlp_to_str() {
        let s = "x1*x4*(x1+x2+x3)+x3;
                        x1*x2*x3*x4:[25/2e19];
                        x1^2+x2^2+x3^2+x4^2:[40/40];
                        x1:[/],x2:[1/1/3],x3:[/5],x4:[1/]";
        let expr_str: Vec<&str> = s.split(';').collect();
        let model = NLP::from_str(&expr_str).unwrap();
        let to_str_temp = model.to_str();
        let to_str = format!("{}\n{}{}", to_str_temp.0, to_str_temp.1, to_str_temp.2);
        let expr_str2: Vec<&str> = to_str.split(';').collect();
        let str_to_model = NLP::from_str(&expr_str2).unwrap();
        assert_eq!(model, str_to_model);

        let to_latex_temp = model.to_latex();
        let to_latex = format!("{}{}{}", to_latex_temp.0, to_latex_temp.1, to_latex_temp.2);
        println!("{}", to_latex);
    }
}
