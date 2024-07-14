use std::f64::consts::PI;
use std::rc::Rc;

use fnv::FnvHashMap;
use num_complex::Complex64;
use num_traits::identities::One;
use num_traits::Zero;

use crate::{Expr, FuncEvalError};
use crate::{ContextProvider, Error, factorial};
use crate::Operation;
use crate::Token::*;

impl Expr {
    pub fn eval_complex(&self) -> Result<Complex64, Error> {
        self.eval_complex_with_ctx(ContextCx::new())
    }

    pub fn eval_complex_with_ctx<C: ContextProvider>(&self, ctx: C) -> Result<Complex64, Error> {
        let mut stack = Vec::with_capacity(16);
        if self.rpn.is_empty() {
            return Err(Error::EmptyExpression);
        }

        for token in &self.rpn {
            match *token {
                Var(ref n) => {
                    if let Some(v) = ctx.get_var(n) {
                        stack.push(Complex64::new(v, 0.));
                    } else if let Some(v) = ctx.get_var_cx(n) {
                        stack.push(v);
                    } else {
                        return Err(Error::UnknownVariable(n.clone()));
                    }
                }
                Number(f) => stack.push(Complex64::new(f, 0.)),
                Binary(op) => {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    let r = match op {
                        Operation::Plus => left + right,
                        Operation::Minus => left - right,
                        Operation::Times => left * right,
                        Operation::Div => left / right,
                        Operation::Rem => left % right,
                        Operation::Pow => left.powf(right.re),
                        // added by dsf, 2021.3
                        Operation::LessThan => {
                            if !left.im.is_zero() || !right.im.is_zero() {
                                return Err(Error::EvalError(format!(
                                    "Wrong input of for op : {:?}",
                                    op
                                )));
                            }
                            if left.re < right.re {
                                Complex64::one()
                            } else {
                                Complex64::zero()
                            }
                        }
                        Operation::GreatThan => {
                            if !left.im.is_zero() || !right.im.is_zero() {
                                return Err(Error::EvalError(format!(
                                    "Wrong input of for op : {:?}",
                                    op
                                )));
                            }
                            if left.re > right.re {
                                Complex64::one()
                            } else {
                                Complex64::zero()
                            }
                        }
                        Operation::LtOrEqual => {
                            if !left.im.is_zero() || !right.im.is_zero() {
                                return Err(Error::EvalError(format!(
                                    "Wrong input of for op : {:?}",
                                    op
                                )));
                            }
                            if left.re <= right.re {
                                Complex64::one()
                            } else {
                                Complex64::zero()
                            }
                        }
                        Operation::GtOrEqual => {
                            if !left.im.is_zero() || !right.im.is_zero() {
                                return Err(Error::EvalError(format!(
                                    "Wrong input of for op : {:?}",
                                    op
                                )));
                            }
                            if left.re >= right.re {
                                Complex64::one()
                            } else {
                                Complex64::zero()
                            }
                        }
                        Operation::Equal => {
                            if !left.im.is_zero() || !right.im.is_zero() {
                                return Err(Error::EvalError(format!(
                                    "Wrong input of for op : {:?}",
                                    op
                                )));
                            }
                            if left.re == right.re {
                                Complex64::one()
                            } else {
                                Complex64::zero()
                            }
                        }
                        Operation::Unequal => {
                            if left != right {
                                Complex64::one()
                            } else {
                                Complex64::zero()
                            }
                        }
                        Operation::And => {
                            if !left.im.is_zero() || !right.im.is_zero() {
                                return Err(Error::EvalError(format!(
                                    "Wrong input of for op : {:?}",
                                    op
                                )));
                            }
                            if (left.re > 0.0) && (right.re > 0.0) {
                                Complex64::one()
                            } else {
                                Complex64::zero()
                            }
                        }
                        Operation::Or => {
                            if !left.im.is_zero() || !right.im.is_zero() {
                                return Err(Error::EvalError(format!(
                                    "Wrong input of for op : {:?}",
                                    op
                                )));
                            }
                            if (left.re > 0.0) || (right.re > 0.0) {
                                Complex64::one()
                            } else {
                                Complex64::zero()
                            }
                        }
                        Operation::BitAnd => {
                            if !left.im.is_zero() || !right.im.is_zero() {
                                return Err(Error::EvalError(format!(
                                    "Wrong input of for op : {:?}",
                                    op
                                )));
                            }
                            Complex64::new((left.re as i64 & right.re as i64) as f64, 0.)
                        }
                        Operation::BitOr => {
                            if !left.im.is_zero() || !right.im.is_zero() {
                                return Err(Error::EvalError(format!(
                                    "Wrong input of for op : {:?}",
                                    op
                                )));
                            }
                            Complex64::new((left.re as i64 | right.re as i64) as f64, 0.)
                        }
                        Operation::BitXor => {
                            if left.im.is_zero() || right.im.is_zero() {
                                return Err(Error::EvalError(format!(
                                    "Wrong input of for op : {:?}",
                                    op
                                )));
                            }
                            Complex64::new((left.re as i64 ^ right.re as i64) as f64, 0.)
                        }
                        Operation::BitShl => {
                            if !left.im.is_zero() || !right.im.is_zero() {
                                return Err(Error::EvalError(format!(
                                    "Wrong input of for op : {:?}",
                                    op
                                )));
                            }
                            Complex64::new(((left.re as i64) << (right.re as i64)) as f64, 0.)
                        }
                        Operation::BitShr => {
                            if !left.im.is_zero() || !right.im.is_zero() {
                                return Err(Error::EvalError(format!(
                                    "Wrong input of for op : {:?}",
                                    op
                                )));
                            }
                            Complex64::new(((left.re as i64) >> (right.re as i64)) as f64, 0.)
                        }
                        Operation::BitAt => {
                            if !left.im.is_zero() || !right.im.is_zero() {
                                return Err(Error::EvalError(format!(
                                    "Wrong input of for op : {:?}",
                                    op
                                )));
                            }
                            if right.re < 1. || right.re > 64. {
                                return Err(Error::EvalError(format!(
                                    "Operation \"@\" ERROR:the {:?} bit doesn't exist.",
                                    right
                                )));
                            }
                            if (left.re as i64) & 2_i64.pow(right.re as u32 - 1) != 0 {
                                Complex64::one()
                            } else {
                                Complex64::zero()
                            }
                        }
                        _ => {
                            return Err(Error::EvalError(format!(
                                "Unimplemented binary operation: {:?}",
                                op
                            )));
                        }
                    };
                    stack.push(r);
                }
                Unary(op) => {
                    let x = stack.pop().unwrap();
                    let r = match op {
                        Operation::Plus => x,
                        Operation::Minus => -x,
                        Operation::Not => {
                            if x.re > 0. {
                                Complex64::one()
                            } else {
                                Complex64::zero()
                            }
                        }
                        Operation::BitNot => {
                            Complex64::new(!(x.re as i64) as f64, 0.)
                        }
                        Operation::Fact => {
                            // Check to make sure x has no fractional component (can be converted to int without loss)
                            match factorial(x.re) {
                                Ok(res) => Complex64::new(res, 0.),
                                Err(e) => return Err(Error::EvalError(String::from(e))),
                            }
                        }
                        _ => {
                            return Err(Error::EvalError(format!(
                                "Unimplemented unary operation: {:?}",
                                op
                            )));
                        }
                    };
                    stack.push(r);
                }
                Func(ref n, Some(i)) => {
                    if stack.len() < i {
                        return Err(Error::EvalError(format!(
                            "eval: stack does not have enough arguments for function token \
                             {:?}",
                            token
                        )));
                    }
                    match ctx.eval_func_cx(n, &stack[stack.len() - i..]) {
                        Ok(r) => {
                            let nl = stack.len() - i;
                            stack.truncate(nl);
                            stack.push(r);
                        }
                        Err(e) => return Err(Error::Function(n.to_owned(), e)),
                    }
                }
                Func(ref n, None) => match ctx.eval_func_cx(n, &[]) {
                    Ok(r) => {
                        stack.push(r);
                    }
                    Err(e) => return Err(Error::Function(n.to_owned(), e)),
                },
                _ => return Err(Error::EvalError(format!("Unrecognized token: {:?}", token))),
            }
        }

        let r = stack.pop().expect("Stack is empty, this is impossible.");
        if !stack.is_empty() {
            return Err(Error::EvalError(format!(
                "There are still {} items on the stack.",
                stack.len()
            )));
        }
        Ok(r)
    }
}

#[doc(hidden)]
pub fn new_cx(r: Complex64, i: Complex64) -> Complex64 {
    Complex64::new(r.re, i.re)
}

#[doc(hidden)]
pub fn new_cx_rad(r: Complex64, i: Complex64) -> Complex64 {
    Complex64::new(r.re * i.re.cos(), r.re * i.re.sin())
}

#[doc(hidden)]
pub fn new_cx_angle(r: Complex64, i: Complex64) -> Complex64 {
    let rad = PI * i.re / 180.;
    Complex64::new(r.re * rad.cos(), r.re * rad.sin())
}

#[doc(hidden)]
#[cfg(feature = "with_rand")]
pub fn random() -> Complex64 {
    use rand::Rng;
    Complex64::new(rand::thread_rng().gen::<f64>(), 0.)
}
#[doc(hidden)]
#[cfg(feature = "with_rand")]
pub fn random2(lower: Complex64, upper: Complex64) -> Complex64 {
    use rand::Rng;
    Complex64::new(rand::thread_rng().gen_range(lower.re..upper.re), 0.)
}

#[doc(hidden)]
pub fn abs(v: Complex64) -> Complex64 {
    Complex64::new(v.norm(), 0.)
}

#[doc(hidden)]
pub fn floor(v: Complex64) -> Complex64 {
    Complex64::new(v.re.floor(), 0.)
}

#[doc(hidden)]
pub fn ceil(v: Complex64) -> Complex64 {
    Complex64::new(v.re.ceil(), 0.)
}

#[doc(hidden)]
pub fn round(v: Complex64) -> Complex64 {
    Complex64::new(v.re.round(), 0.)
}

#[doc(hidden)]
pub fn signum(v: Complex64) -> Complex64 {
    Complex64::new(v.re.signum(), 0.)
}

#[doc(hidden)]
pub fn conjugate(v: Complex64) -> Complex64 {
    Complex64::new(v.re, -v.im)
}

#[doc(hidden)]
pub fn real(v: Complex64) -> Complex64 {
    Complex64::new(v.re, 0.)
}

#[doc(hidden)]
pub fn imag(v: Complex64) -> Complex64 {
    Complex64::new(0., v.im)
}

#[doc(hidden)]
pub fn radian(v: Complex64) -> Complex64 {
    Complex64::new(v.im.atan2(v.re), 0.)
}

#[doc(hidden)]
pub fn atan2(v1: Complex64, v2: Complex64) -> Complex64 {
    Complex64::new(v1.re.atan2(v2.re), 0.)
}

#[doc(hidden)]
pub fn max_array(xs: &[Complex64]) -> Complex64 {
    xs.iter()
        .fold(Complex64::new(f64::NEG_INFINITY, 0.), |m, &x| {
            Complex64::new(m.re.max(x.re), 0.)
        })
}

#[doc(hidden)]
pub fn min_array(xs: &[Complex64]) -> Complex64 {
    xs.iter().fold(Complex64::new(f64::INFINITY, 0.), |m, &x| {
        Complex64::new(m.re.min(x.re), 0.)
    })
}

#[derive(Clone)]
pub struct ContextCx<'a> {
    vars: FnvHashMap<String, f64>,
    vars_cx: FnvHashMap<String, Complex64>,
    funcs: FnvHashMap<String, GuardedFuncCx<'a>>,
    // tensors: ContextHashMap<String, Tensor<'a, f32>>,
}

impl<'a> ContextCx<'a> {
    /// Creates a context with built-in constants and functions.
    pub fn new() -> ContextCx<'a> {
        thread_local!(static DEFAULT_CONTEXT: ContextCx<'static> = {
            let mut ctx = ContextCx::empty();
            ctx.var("pi", PI);
            ctx.var("PI", PI);
            ctx.var("e", std::f64::consts::E);
            #[cfg(feature = "with_rand")]
            ctx.func0("rand", random);
            ctx.func1("abs", abs);
            ctx.func1("sqrt", Complex64::sqrt);
            ctx.func1("exp", Complex64::exp);
            ctx.func1("ln", Complex64::ln);
            ctx.func1("log10", Complex64::log10);
            ctx.func1("sin", Complex64::sin);
            ctx.func1("cos", Complex64::cos);
            ctx.func1("tan", Complex64::tan);
            ctx.func1("asin", Complex64::asin);
            ctx.func1("acos", Complex64::acos);
            ctx.func1("atan", Complex64::atan);
            ctx.func1("sinh", Complex64::sinh);
            ctx.func1("cosh", Complex64::cosh);
            ctx.func1("tanh", Complex64::tanh);
            ctx.func1("asinh", Complex64::asinh);
            ctx.func1("acosh", Complex64::acosh);
            ctx.func1("atanh", Complex64::atanh);
            ctx.func1("floor", floor);
            ctx.func1("ceil", ceil);
            ctx.func1("round", round);
            ctx.func1("signum", signum);
            ctx.func1("conj", conjugate);
            ctx.func1("real", real);
            ctx.func1("imag", imag);
            ctx.func1("rad", radian);
            ctx.func2("atan2", atan2);
            // 建立复数的函数
            ctx.func2("c", new_cx);
            // 用弧度建立复数
            ctx.func2("c1", new_cx_rad);
            // 用角度建立复数
            ctx.func2("c2", new_cx_angle);
            #[cfg(feature = "with_rand")]
            ctx.func2("rand2", random2);
            ctx.funcn("max", max_array, 1..);
            ctx.funcn("min", min_array, 1..);
            ctx
        });

        DEFAULT_CONTEXT.with(|ctx| ctx.clone())
    }

    /// Creates an empty contexts.
    pub fn empty() -> ContextCx<'a> {
        ContextCx {
            vars: FnvHashMap::default(),
            vars_cx: Default::default(),
            funcs: FnvHashMap::default(),
        }
    }

    /// Adds a new variable/constant.
    pub fn var<S: Into<String>>(&mut self, var: S, value: f64) -> &mut Self {
        self.vars.insert(var.into(), value);
        self
    }

    pub fn var_cx<S: Into<String>>(&mut self, var: S, value: Complex64) -> &mut Self {
        self.vars_cx.insert(var.into(), value);
        self
    }

    /// Adds a new function of one argument.
    pub fn func0<S, F>(&mut self, name: S, func: F) -> &mut Self
    where
        S: Into<String>,
        F: Fn() -> Complex64 + 'a,
    {
        self.funcs.insert(name.into(), Rc::new(move |_| Ok(func())));
        self
    }

    /// Adds a new function of one argument.
    pub fn func1<S, F>(&mut self, name: S, func: F) -> &mut Self
    where
        S: Into<String>,
        F: Fn(Complex64) -> Complex64 + 'a,
    {
        self.funcs.insert(
            name.into(),
            Rc::new(move |args: &[Complex64]| {
                if args.len() == 1 {
                    Ok(func(args[0]))
                } else {
                    Err(FuncEvalError::NumberArgs(1))
                }
            }),
        );
        self
    }

    pub fn func2<S, F>(&mut self, name: S, func: F) -> &mut Self
    where
        S: Into<String>,
        F: Fn(Complex64, Complex64) -> Complex64 + 'a,
    {
        self.funcs.insert(
            name.into(),
            Rc::new(move |args: &[Complex64]| {
                if args.len() == 2 {
                    Ok(func(args[0], args[1]))
                } else {
                    Err(FuncEvalError::NumberArgs(2))
                }
            }),
        );
        self
    }

    /// Adds a new function of three arguments.
    pub fn func3<S, F>(&mut self, name: S, func: F) -> &mut Self
    where
        S: Into<String>,
        F: Fn(Complex64, Complex64, Complex64) -> Complex64 + 'a,
    {
        self.funcs.insert(
            name.into(),
            Rc::new(move |args: &[Complex64]| {
                if args.len() == 3 {
                    Ok(func(args[0], args[1], args[2]))
                } else {
                    Err(FuncEvalError::NumberArgs(3))
                }
            }),
        );
        self
    }

    pub fn funcn<S, F, N>(&mut self, name: S, func: F, n_args: N) -> &mut Self
    where
        S: Into<String>,
        F: Fn(&[Complex64]) -> Complex64 + 'a,
        N: ArgGuardCx,
    {
        self.funcs.insert(name.into(), n_args.to_arg_guard(func));
        self
    }
}

impl<'a> Default for ContextCx<'a> {
    fn default() -> Self {
        ContextCx::new()
    }
}

type GuardedFuncCx<'a> = Rc<dyn Fn(&[Complex64]) -> Result<Complex64, FuncEvalError> + 'a>;

pub trait ArgGuardCx {
    fn to_arg_guard<'a, F: Fn(&[Complex64]) -> Complex64 + 'a>(self, func: F) -> GuardedFuncCx<'a>;
}

impl ArgGuardCx for usize {
    fn to_arg_guard<'a, F: Fn(&[Complex64]) -> Complex64 + 'a>(self, func: F) -> GuardedFuncCx<'a> {
        Rc::new(move |args: &[Complex64]| {
            if args.len() == self {
                Ok(func(args))
            } else {
                Err(FuncEvalError::NumberArgs(1))
            }
        })
    }
}

impl ArgGuardCx for std::ops::RangeFrom<usize> {
    fn to_arg_guard<'a, F: Fn(&[Complex64]) -> Complex64 + 'a>(self, func: F) -> GuardedFuncCx<'a> {
        Rc::new(move |args: &[Complex64]| {
            if args.len() >= self.start {
                Ok(func(args))
            } else {
                Err(FuncEvalError::TooFewArguments)
            }
        })
    }
}

impl ArgGuardCx for std::ops::RangeTo<usize> {
    fn to_arg_guard<'a, F: Fn(&[Complex64]) -> Complex64 + 'a>(self, func: F) -> GuardedFuncCx<'a> {
        Rc::new(move |args: &[Complex64]| {
            if args.len() < self.end {
                Ok(func(args))
            } else {
                Err(FuncEvalError::TooManyArguments)
            }
        })
    }
}

impl ArgGuardCx for std::ops::Range<usize> {
    fn to_arg_guard<'a, F: Fn(&[Complex64]) -> Complex64 + 'a>(self, func: F) -> GuardedFuncCx<'a> {
        Rc::new(move |args: &[Complex64]| {
            if args.len() >= self.start && args.len() < self.end {
                Ok(func(args))
            } else if args.len() < self.start {
                Err(FuncEvalError::TooFewArguments)
            } else {
                Err(FuncEvalError::TooManyArguments)
            }
        })
    }
}

impl ArgGuardCx for std::ops::RangeFull {
    fn to_arg_guard<'a, F: Fn(&[Complex64]) -> Complex64 + 'a>(self, func: F) -> GuardedFuncCx<'a> {
        Rc::new(move |args: &[Complex64]| Ok(func(args)))
    }
}

impl<'a> ContextProvider for ContextCx<'a> {
    fn get_var(&self, name: &str) -> Option<f64> {
        self.vars.get(name).cloned()
    }
    fn get_var_cx(&self, name: &str) -> Option<Complex64> {
        self.vars_cx.get(name).cloned()
    }
    fn eval_func_cx(&self, name: &str, args: &[Complex64]) -> Result<Complex64, FuncEvalError> {
        self.funcs
            .get(name)
            .map_or(Err(FuncEvalError::UnknownFunction), |f| f(args))
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;
    use std::ops::Mul;
    use std::str::FromStr;

    use approx::assert_relative_eq;
    use ndarray::array;
    use num_complex::{Complex, Complex64};

    use crate::Expr;
    use crate::expr_complex::ContextCx;

    #[test]
    fn it_works() {
        let expr = Expr::from_str("1+2").unwrap();
        let r = expr.eval_complex();
        assert_eq!(r, Ok(Complex64::new(3., 0.)));

        let expr = Expr::from_str("c(0,1)+c(2,0.)").unwrap();
        let r = expr.eval_complex();
        assert_eq!(r, Ok(Complex64::new(2., 1.)));

        let expr = Expr::from_str("abs(c(0,1))+c(2,0.)").unwrap();
        let r = expr.eval_complex();
        assert_eq!(r, Ok(Complex64::new(3., 0.)));

        let mut cc = ContextCx::new();
        cc.var_cx("a", Complex::new(1., 1.));
        let expr = Expr::from_str("a+c(2,0.)").unwrap();
        let r = expr.eval_complex_with_ctx(cc.clone());
        assert_eq!(r, Ok(Complex64::new(3., 1.)));
    }

    #[test]
    fn test_2_1() {
        let mut cc = ContextCx::new();
        cc.var("GMRabc", 0.00744);
        cc.var("GMRn", 0.00248);
        cc.var("rabc", 0.190);
        cc.var("rn", 0.368);
        cc.var("Dab", 0.7622);
        cc.var("Dbc", 1.3720);
        cc.var("Dca", 2.1342);
        cc.var("Dan", 1.7247);
        cc.var("Dbn", 1.3025);
        cc.var("Dcn", 1.5244);
        // let zaa = Complex64::new(rabc + 0.0493, 0.0628 * (((1.0 / GMRabc) as f64).ln() + 8.02517));
        let expr =
            Expr::from_str("c(rabc + 0.0493, 0.0628 * (ln(1.0 / GMRabc) + 8.02517))").unwrap();
        let zaa = expr.eval_complex_with_ctx(cc.clone()).unwrap();
        assert_relative_eq!(zaa.re, 0.2393, max_relative = 1e-4);
        assert_relative_eq!(zaa.im, 0.8118, max_relative = 1e-4);
        let expr = Expr::from_str("c(0.0493, 0.0628 * (ln(1.0 / Dab) + 8.02517))").unwrap();
        let zab = expr.eval_complex_with_ctx(cc.clone()).unwrap();
        assert_relative_eq!(zab.im, 0.5210, max_relative = 1e-4);
        let expr = Expr::from_str("c(0.0493, 0.0628 * (ln(1.0 / Dca) + 8.02517))").unwrap();
        let zac = expr.eval_complex_with_ctx(cc.clone()).unwrap();
        assert_relative_eq!(zac.im, 0.4564, max_relative = 1e-4);
        let expr = Expr::from_str("c(0.0493, 0.0628 * (ln(1.0 / Dbc) + 8.02517))").unwrap();
        let zbc = expr.eval_complex_with_ctx(cc.clone()).unwrap();
        assert_relative_eq!(zbc.im, 0.4841, max_relative = 1e-4);
        let expr = Expr::from_str("c(0.0493, 0.0628 * (ln(1.0 / Dan) + 8.02517))").unwrap();
        let zan = expr.eval_complex_with_ctx(cc.clone()).unwrap();
        assert_relative_eq!(zan.im, 0.4698, max_relative = 1e-3);
        let expr = Expr::from_str("c(0.0493, 0.0628 * (ln(1.0 / Dbn) + 8.02517))").unwrap();
        let zbn = expr.eval_complex_with_ctx(cc.clone()).unwrap();
        assert_relative_eq!(zbn.im, 0.4874, max_relative = 1e-4);
        let expr = Expr::from_str("c(0.0493, 0.0628 * (ln(1.0 / Dcn) + 8.02517))").unwrap();
        let zcn = expr.eval_complex_with_ctx(cc.clone()).unwrap();
        assert_relative_eq!(zcn.im, 0.4775, max_relative = 1e-4);
        let expr = Expr::from_str("c(rn + 0.0493, 0.0628 * (ln(1.0 / GMRn) + 8.02517))").unwrap();
        let znn = expr.eval_complex_with_ctx(cc.clone()).unwrap();
        assert_relative_eq!(znn.re, 0.4173, max_relative = 1e-4);
        assert_relative_eq!(znn.im, 0.8807, max_relative = 1e-4);
        let zij = array![[zaa, zab, zac], [zab, zaa, zbc], [zac, zbc, zaa]];
        let zin = array![[zan], [zbn], [zcn]];
        let znj = array![zan, zbn, zcn];
        let zabc = zij - zin.mul(array![Complex64::new(1.0, 0.0)] / znn).mul(znj);
        println!("{:?}", zabc);
        let a = Complex64::new(f64::cos(2.0 * PI / 3.0), f64::sin(2.0 * PI / 3.0));
        let matrixes = array![
            [
                Complex64::new(1.0, 0.0),
                Complex64::new(1.0, 0.0),
                Complex64::new(1.0, 0.0)
            ],
            [Complex64::new(1.0, 0.0), a * a, a],
            [Complex64::new(1.0, 0.0), a, a * a]
        ];
        let matrixes_inv = array![
            [
                Complex64::new(1.0 / 3.0, 0.0),
                Complex64::new(1.0 / 3.0, 0.0),
                Complex64::new(1.0 / 3.0, 0.0)
            ],
            [
                Complex64::new(1.0 / 3.0, 0.0),
                a * Complex64::new(1.0 / 3.0, 0.0),
                a * a * Complex64::new(1.0 / 3.0, 0.0)
            ],
            [
                Complex64::new(1.0 / 3.0, 0.0),
                a * a * Complex64::new(1.0 / 3.0, 0.0),
                a * Complex64::new(1.0 / 3.0, 0.0)
            ]
        ];
        let temp = matrixes_inv.dot(&zabc);
        let z012 = temp.dot(&matrixes);
        assert_relative_eq!(z012.get([0, 0]).unwrap().re, 0.5050, max_relative = 1e-3);
    }
}
