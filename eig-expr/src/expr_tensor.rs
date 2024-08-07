use std::f64::consts::PI;
use fnv::FnvHashMap;
use ndarray::{Array, Ix1, Ix2, IxDyn};
use num_complex::Complex64;

use crate::{CtxProvider, Expr, Operation, Token::*};
use crate::{ContextProvider, Error, factorial, FuncEvalError, MyCx, MyF};
use crate::expr::Context;
use crate::expr_complex::ContextCx;
use crate::tsfn_basic::*;


thread_local!(static DEFAULT_CONTEXT: Context<'static> = Context::new());
thread_local!(pub static DEFAULT_CONTEXT_TENSOR: ContextTensor<'static> = ContextTensor::new());

impl ContextProvider for CtxProvider {
    fn get_var(&self, name: &str) -> Option<f64> {
        self.var_values.get(name).cloned()
    }
    fn get_var_cx(&self, name: &str) -> Option<Complex64> {
        self.var_values_cx.get(name).cloned()
    }
    fn get_tensor(&self, name: &str) -> Option<Array<f64, IxDyn>> {
        self.var_values_tensor.get(name).cloned()
    }
    fn get_tensor_cx(&self, name: &str) -> Option<Array<Complex64, IxDyn>> {
        self.var_values_tensor_cx.get(name).cloned()
    }
    fn eval_func(&self, name: &str, args: &[f64]) -> Result<f64, FuncEvalError> {
        DEFAULT_CONTEXT_TENSOR.with(|ctx| ctx.eval_func(name, args))
    }
    fn eval_func_cx(&self, name: &str, args: &[Complex64]) -> Result<Complex64, FuncEvalError> {
        DEFAULT_CONTEXT_TENSOR.with(|ctx| ctx.eval_func_cx(name, args))
    }
    fn eval_func_tensor(&self, name: &str, args: &[MyF]) -> Result<MyF, FuncEvalError> {
        DEFAULT_CONTEXT_TENSOR.with(|ctx| ctx.eval_func_tensor(name, args))
    }
    fn eval_func_tensor_cx(&self, name: &str, args: &[MyCx]) -> Result<MyCx, FuncEvalError> {
        DEFAULT_CONTEXT_TENSOR.with(|ctx| ctx.eval_func_tensor_cx(name, args))
    }
}

impl Expr {
    pub fn eval_tensor(&self) -> Result<MyF, Error> {
        self.eval_tensor_with_ctx(ContextTensor::new())
    }

    pub fn eval_tensor_cx(&self) -> Result<MyCx, Error> {
        self.eval_tensor_with_ctx_cx(ContextTensor::new())
    }

    pub fn eval_tensor_with_ctx<C: ContextProvider>(&self, ctx: C) -> Result<MyF, Error> {
        let mut stack = Vec::with_capacity(16);
        if self.rpn.is_empty() {
            return Err(Error::EmptyExpression);
        }
        // 将后缀表达式转换成tensor
        for token in &self.rpn {
            match token {
                Var(n) => {
                    if let Some(f) = ctx.get_var(n) {
                        stack.push(MyF::F64(f));
                    } else if let Some(t) = ctx.get_tensor(n) {
                        stack.push(MyF::Tensor(t));
                    } else {
                        return Err(Error::UnknownVariable(n.clone()));
                    }
                }
                Number(f) => {
                    stack.push(MyF::F64(*f));
                }
                Tensor(size) => {
                    if size.is_none() {
                        return Err(Error::EvalError(format!(
                            "Tensor size is none: {:?}",
                            token
                        )));
                    }
                    let size = size.unwrap();
                    if stack.len() < size {
                        return Err(Error::EvalError(format!(
                            "eval: stack does not have enough arguments for function token {:?}",
                            token
                        )));
                    }
                    let mut floats = Vec::new();
                    let mut is_array = false;
                    let mut shape = match stack.last().unwrap() {
                        MyF::F64(_) => {
                            vec![]
                        }
                        MyF::Tensor(t) => {
                            is_array = true;
                            t.shape().to_vec()
                        }
                    };
                    for i in 0..size {
                        match &stack[stack.len() - size + i] {
                            MyF::F64(f) => {
                                if is_array {
                                    return Err(Error::EvalError(format!(
                                        "Not consistent type for tensor token : {:?}",
                                        token
                                    )));
                                }
                                floats.push(*f);
                            }
                            MyF::Tensor(t) => {
                                floats.extend(t.as_slice().unwrap());
                            }
                        }
                    }
                    let nl = stack.len() - size;
                    stack.truncate(nl);
                    shape.insert(0, size);
                    let array = Array::from_shape_vec(shape, floats).unwrap();
                    stack.push(MyF::Tensor(array.into_dyn()));
                }
                Binary(op) => {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    let r = match op {
                        Operation::Plus => match left {
                            MyF::F64(f1) => match right {
                                MyF::F64(f2) => MyF::F64(f1 + f2),
                                MyF::Tensor(t) => MyF::Tensor(f1 + t),
                            },
                            MyF::Tensor(t1) => match right {
                                MyF::F64(f) => MyF::Tensor(t1 + f),
                                MyF::Tensor(t2) => MyF::Tensor(t1 + t2),
                            },
                        },
                        Operation::Minus => match left {
                            MyF::F64(f1) => match right {
                                MyF::F64(f2) => MyF::F64(f1 - f2),
                                MyF::Tensor(t2) => MyF::Tensor(f1 - t2),
                            },
                            MyF::Tensor(t1) => match right {
                                MyF::F64(f2) => MyF::Tensor(t1 - f2),
                                MyF::Tensor(t2) => MyF::Tensor(t1 - t2),
                            },
                        },
                        Operation::Times => match left {
                            MyF::F64(f1) => match right {
                                MyF::F64(f2) => MyF::F64(f1 * f2),
                                MyF::Tensor(t) => MyF::Tensor(f1 * t),
                            },
                            MyF::Tensor(t1) => match right {
                                MyF::F64(f) => MyF::Tensor(t1 * f),
                                MyF::Tensor(t2) => match t1.shape().len() {
                                    1 => {
                                        let a = t1.into_dimensionality::<Ix1>().unwrap();
                                        if t2.shape().len() == 2 && a.shape()[0] == t2.shape()[0] {
                                            let b = t2.into_dimensionality::<Ix2>().unwrap();
                                            MyF::Tensor(a.dot(&b).into_dyn())
                                        } else {
                                            MyF::Tensor(a * t2)
                                        }
                                    }
                                    2 => {
                                        let a = t1.into_dimensionality::<Ix2>().unwrap();
                                        if t2.shape().len() == 1 && a.shape()[1] == t2.shape()[0] {
                                            let b = t2.into_dimensionality::<Ix1>().unwrap();
                                            MyF::Tensor(a.dot(&b).into_dyn())
                                        } else if t2.shape().len() == 2
                                            && a.shape()[1] == t2.shape()[0]
                                        {
                                            let b = t2.into_dimensionality::<Ix2>().unwrap();
                                            MyF::Tensor(a.dot(&b).into_dyn())
                                        } else {
                                            MyF::Tensor(a * t2)
                                        }
                                    }
                                    _ => MyF::Tensor(t1 * t2),
                                },
                            },
                        },
                        Operation::Div => match left {
                            MyF::F64(f1) => match right {
                                MyF::F64(f2) => MyF::F64(f1 / f2),
                                MyF::Tensor(t2) => MyF::Tensor(f1 / t2),
                            },
                            MyF::Tensor(t1) => match right {
                                MyF::F64(f2) => MyF::Tensor(t1 / f2),
                                MyF::Tensor(t2) => MyF::Tensor(t1 / t2),
                            },
                        },
                        Operation::Rem => match left {
                            MyF::F64(f1) => match right {
                                MyF::F64(f2) => MyF::F64(f1 % f2),
                                MyF::Tensor(t2) => MyF::Tensor(f1 % t2),
                            },
                            MyF::Tensor(t1) => match right {
                                MyF::F64(f2) => MyF::Tensor(t1 % f2),
                                MyF::Tensor(t2) => MyF::Tensor(t1 % t2),
                            },
                        },
                        Operation::Pow => match left {
                            MyF::F64(f1) => match right {
                                MyF::F64(f2) => MyF::F64(f1.powf(f2)),
                                _ => {
                                    return Err(Error::EvalError(format!(
                                        "Not equal type : {:?}",
                                        token
                                    )));
                                }
                            },
                            MyF::Tensor(t1) => {
                                match right {
                                    MyF::F64(f) => {
                                        let shape = t1.shape();
                                        if f == -1. && shape.len() == 2 && shape[0] == shape[1] {
                                            let t = ctx
                                                .matrix_inv(
                                                    &t1.into_dimensionality::<Ix2>().unwrap(),
                                                )
                                                .map_err(|e| {
                                                    Error::Function("pow".to_string(), e)
                                                })?;
                                            // let t = t1.into_dimensionality::<Ix2>().unwrap().inv()
                                            //     .map_err(|_| Error::Function("pow".to_string(), FuncEvalError::NumberArgs(0)))?;
                                            MyF::Tensor(t.into_dyn())
                                        } else {
                                            MyF::Tensor(t1.mapv(|a| a.powf(f)))
                                        }
                                    }
                                    _ => {
                                        return Err(Error::EvalError(format!(
                                            "Not equal type : {:?}",
                                            token
                                        )));
                                    }
                                }
                            }
                        },
                        Operation::LessThan => match left {
                            MyF::F64(f1) => match right {
                                MyF::F64(f2) => MyF::F64(if f1 < f2 { 1.0 } else { 0.0 }),
                                _ => {
                                    return Err(Error::EvalError(format!(
                                        "Not equal type : {:?}",
                                        token
                                    )));
                                }
                            },
                            _ => {
                                return Err(Error::EvalError(format!(
                                    "Not equal type : {:?}",
                                    token
                                )));
                            }
                        },
                        Operation::GreatThan => match left {
                            MyF::F64(f1) => match right {
                                MyF::F64(f2) => MyF::F64(if f1 > f2 { 1.0 } else { 0.0 }),
                                _ => {
                                    return Err(Error::EvalError(format!(
                                        "Not equal type : {:?}",
                                        token
                                    )));
                                }
                            },
                            _ => {
                                return Err(Error::EvalError(format!(
                                    "Not equal type : {:?}",
                                    token
                                )));
                            }
                        },
                        Operation::LtOrEqual => match left {
                            MyF::F64(f1) => match right {
                                MyF::F64(f2) => MyF::F64(if f1 <= f2 { 1.0 } else { 0.0 }),
                                _ => {
                                    return Err(Error::EvalError(format!(
                                        "Not equal type : {:?}",
                                        token
                                    )));
                                }
                            },
                            _ => {
                                return Err(Error::EvalError(format!(
                                    "Not equal type : {:?}",
                                    token
                                )));
                            }
                        },
                        Operation::GtOrEqual => match left {
                            MyF::F64(f1) => match right {
                                MyF::F64(f2) => MyF::F64(if f1 >= f2 { 1.0 } else { 0.0 }),
                                _ => {
                                    return Err(Error::EvalError(format!(
                                        "Not equal type : {:?}",
                                        token
                                    )));
                                }
                            },
                            _ => {
                                return Err(Error::EvalError(format!(
                                    "Not equal type : {:?}",
                                    token
                                )));
                            }
                        },
                        Operation::Equal => match left {
                            MyF::F64(f1) => match right {
                                MyF::F64(f2) => MyF::F64(if f1 == f2 { 1.0 } else { 0.0 }),
                                _ => {
                                    return Err(Error::EvalError(format!(
                                        "Not equal type : {:?}",
                                        token
                                    )));
                                }
                            },
                            _ => {
                                return Err(Error::EvalError(format!(
                                    "Not equal type : {:?}",
                                    token
                                )));
                            }
                        },
                        Operation::Unequal => match left {
                            MyF::F64(f1) => match right {
                                MyF::F64(f2) => MyF::F64(if f1 != f2 { 1.0 } else { 0.0 }),
                                _ => {
                                    return Err(Error::EvalError(format!(
                                        "Not equal type : {:?}",
                                        token
                                    )));
                                }
                            },
                            _ => {
                                return Err(Error::EvalError(format!(
                                    "Not equal type : {:?}",
                                    token
                                )));
                            }
                        },
                        Operation::And => match left {
                            MyF::F64(f1) => match right {
                                MyF::F64(f2) => {
                                    MyF::F64(if (f1 > 0.0) && (f2 > 0.0) { 1.0 } else { 0.0 })
                                }
                                _ => {
                                    return Err(Error::EvalError(format!(
                                        "Not equal type : {:?}",
                                        token
                                    )));
                                }
                            },
                            _ => {
                                return Err(Error::EvalError(format!(
                                    "Not equal type : {:?}",
                                    token
                                )));
                            }
                        },
                        Operation::Or => match left {
                            MyF::F64(f1) => match right {
                                MyF::F64(f2) => {
                                    MyF::F64(if (f1 > 0.0) || (f2 > 0.0) { 1.0 } else { 0.0 })
                                }
                                _ => {
                                    return Err(Error::EvalError(format!(
                                        "Not equal type : {:?}",
                                        token
                                    )));
                                }
                            },
                            _ => {
                                return Err(Error::EvalError(format!(
                                    "Not equal type : {:?}",
                                    token
                                )));
                            }
                        },
                        Operation::BitAnd => match left {
                            MyF::F64(f1) => match right {
                                MyF::F64(f2) => MyF::F64((f1 as i64 & f2 as i64) as f64),
                                _ => {
                                    return Err(Error::EvalError(format!(
                                        "Not equal type : {:?}",
                                        token
                                    )));
                                }
                            },
                            _ => {
                                return Err(Error::EvalError(format!(
                                    "Not equal type : {:?}",
                                    token
                                )));
                            }
                        },
                        Operation::BitOr => match left {
                            MyF::F64(f1) => match right {
                                MyF::F64(f2) => MyF::F64((f1 as i64 | f2 as i64) as f64),
                                _ => {
                                    return Err(Error::EvalError(format!(
                                        "Not equal type : {:?}",
                                        token
                                    )));
                                }
                            },
                            _ => {
                                return Err(Error::EvalError(format!(
                                    "Not equal type : {:?}",
                                    token
                                )));
                            }
                        },
                        Operation::BitXor => match left {
                            MyF::F64(f1) => match right {
                                MyF::F64(f2) => MyF::F64((f1 as i64 ^ f2 as i64) as f64),
                                _ => {
                                    return Err(Error::EvalError(format!(
                                        "Not equal type : {:?}",
                                        token
                                    )));
                                }
                            },
                            _ => {
                                return Err(Error::EvalError(format!(
                                    "Not equal type : {:?}",
                                    token
                                )));
                            }
                        },
                        Operation::BitShl => match left {
                            MyF::F64(f1) => match right {
                                MyF::F64(f2) => MyF::F64(((f1 as i64) << (f2 as i64)) as f64),
                                _ => {
                                    return Err(Error::EvalError(format!(
                                        "Not equal type : {:?}",
                                        token
                                    )));
                                }
                            },
                            _ => {
                                return Err(Error::EvalError(format!(
                                    "Not equal type : {:?}",
                                    token
                                )));
                            }
                        },
                        Operation::BitShr => match left {
                            MyF::F64(f1) => match right {
                                MyF::F64(f2) => MyF::F64(((f1 as i64) >> (f2 as i64)) as f64),
                                _ => {
                                    return Err(Error::EvalError(format!(
                                        "Not equal type : {:?}",
                                        token
                                    )));
                                }
                            },
                            _ => {
                                return Err(Error::EvalError(format!(
                                    "Not equal type : {:?}",
                                    token
                                )));
                            }
                        },
                        Operation::BitAt => match left {
                            MyF::F64(f1) => match right {
                                MyF::F64(f2) => {
                                    if f1 < 1. || f2 > 64. {
                                        return Err(Error::EvalError(format!(
                                            "Operation \"@\" ERROR:the {:?} bit doesn't exist.",
                                            right
                                        )));
                                    }
                                    MyF::F64(if (f1 as i64) & 2_i64.pow(f2 as u32 - 1) != 0 {
                                        1.0
                                    } else {
                                        0.0
                                    })
                                }
                                _ => {
                                    return Err(Error::EvalError(format!(
                                        "Not equal type : {:?}",
                                        token
                                    )));
                                }
                            },
                            _ => {
                                return Err(Error::EvalError(format!(
                                    "Not equal type : {:?}",
                                    token
                                )));
                            }
                        },
                        _ => {
                            return Err(Error::EvalError(format!("TypeUnsupported : {:?}", token)));
                        }
                    };
                    stack.push(r);
                }
                Unary(op) => {
                    let x = stack.pop().unwrap();
                    let r = match op {
                        Operation::Plus => x,
                        Operation::Minus => match x {
                            MyF::F64(f) => MyF::F64(-f),
                            MyF::Tensor(t) => MyF::Tensor(-t),
                        },
                        Operation::Not => match x {
                            MyF::F64(f) => MyF::F64(if f > 0. {1.0} else {0.}),
                            MyF::Tensor(t) => MyF::Tensor(t.mapv_into(|f| if f > 0. {1.0} else {0.} )),

                        }
                        Operation::BitNot => match x {
                            MyF::F64(f) => MyF::F64(!(f as i64) as f64),
                            MyF::Tensor(t) => MyF::Tensor(t.mapv_into(|f| !(f as i64) as f64)),
                        },
                        Operation::Fact => match x {
                            MyF::F64(f) => match factorial(f) {
                                Ok(res) => MyF::F64(res),
                                Err(e) => return Err(Error::EvalError(String::from(e))),
                            },
                            _ => {
                                return Err(Error::EvalError(format!(
                                    "Unimplemented unary operation: {:?}, {:?}",
                                    op, token
                                )));
                            }
                        },
                        _ => {
                            return Err(Error::EvalError(format!(
                                "Unimplemented unary operation: {:?}",
                                op
                            )));
                        }
                    };
                    stack.push(r);
                }
                Func(n, Some(i)) => {
                    if stack.len() < *i {
                        let msg = format!(
                            "stack does not have enough arguments for function token {:?}",
                            token
                        );
                        return Err(Error::EvalError(msg));
                    }
                    match ctx.eval_func_tensor(n, &stack[stack.len() - i..]) {
                        Ok(r) => {
                            let nl = stack.len() - i;
                            stack.truncate(nl);
                            stack.push(r);
                        }
                        Err(e) => return Err(Error::Function(n.to_owned(), e)),
                    }
                }
                Func(ref n, None) => match ctx.eval_func_tensor(n, &[]) {
                    Ok(r) => {
                        stack.push(r);
                    }
                    Err(e) => return Err(Error::Function(n.to_owned(), e)),
                },
                _ => {
                    return Err(Error::EvalError(format!("TypeUnsupported : {:?}", token)));
                }
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

    pub fn eval_tensor_with_ctx_cx<C: ContextProvider>(&self, ctx: C) -> Result<MyCx, Error> {
        let mut stack = Vec::with_capacity(16);
        if self.rpn.is_empty() {
            return Err(Error::EmptyExpression);
        }
        // 将后缀表达式转换成tensor
        for token in &self.rpn {
            match token {
                Var(n) => {
                    if let Some(f) = ctx.get_var(n) {
                        stack.push(MyCx::F64(Complex64::new(f, 0.)));
                    } else if let Some(c) = ctx.get_var_cx(n) {
                        stack.push(MyCx::F64(c));
                    } else if let Some(t) = ctx.get_tensor(n) {
                        let a = t.mapv(|f| Complex64::new(f, 0.));
                        stack.push(MyCx::Tensor(a));
                    } else if let Some(t) = ctx.get_tensor_cx(n) {
                        stack.push(MyCx::Tensor(t));
                    } else {
                        return Err(Error::UnknownVariable(n.clone()));
                    }
                }
                Number(f) => stack.push(MyCx::F64(Complex64::new(*f, 0.))),
                Tensor(size) => {
                    if size.is_none() {
                        return Err(Error::EvalError(format!(
                            "Tensor size is none: {:?}",
                            token
                        )));
                    }
                    let size = size.unwrap();
                    if stack.len() < size {
                        return Err(Error::EvalError(format!(
                            "eval: stack does not have enough arguments for function token {:?}",
                            token
                        )));
                    }
                    let mut floats = Vec::new();
                    let mut is_array = false;
                    let mut shape = match stack.last().unwrap() {
                        MyCx::F64(_) => {
                            vec![]
                        }
                        MyCx::Tensor(t) => {
                            is_array = true;
                            t.shape().to_vec()
                        }
                    };
                    for i in 0..size {
                        match &stack[stack.len() - size + i] {
                            MyCx::F64(f) => {
                                if is_array {
                                    return Err(Error::EvalError(format!(
                                        "Not consistent type for tensor token : {:?}",
                                        token
                                    )));
                                }
                                floats.push(*f);
                            }
                            MyCx::Tensor(t) => floats.extend(t.as_slice().unwrap()),
                        }
                    }
                    let nl = stack.len() - size;
                    stack.truncate(nl);
                    shape.insert(0, size);
                    let array = Array::from_shape_vec(shape, floats).unwrap();
                    stack.push(MyCx::Tensor(array.into_dyn()));
                }
                Binary(op) => {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    let r = match op {
                        Operation::Plus => match left {
                            MyCx::F64(f1) => match right {
                                MyCx::F64(f2) => MyCx::F64(f1 + f2),
                                MyCx::Tensor(t) => MyCx::Tensor(f1 + t),
                            },
                            MyCx::Tensor(t1) => match right {
                                MyCx::F64(f) => MyCx::Tensor(t1 + f),
                                MyCx::Tensor(t2) => MyCx::Tensor(t1 + t2),
                            },
                        },
                        Operation::Minus => match left {
                            MyCx::F64(f1) => match right {
                                MyCx::F64(f2) => MyCx::F64(f1 - f2),
                                MyCx::Tensor(t2) => MyCx::Tensor(f1 - t2),
                            },
                            MyCx::Tensor(t1) => match right {
                                MyCx::F64(f2) => MyCx::Tensor(t1 - f2),
                                MyCx::Tensor(t2) => MyCx::Tensor(t1 - t2),
                            },
                        },
                        Operation::Times => match left {
                            MyCx::F64(f1) => match right {
                                MyCx::F64(f2) => MyCx::F64(f1 * f2),
                                MyCx::Tensor(t) => MyCx::Tensor(f1 * t),
                            },
                            MyCx::Tensor(t1) => match right {
                                MyCx::F64(f) => MyCx::Tensor(t1 * f),
                                MyCx::Tensor(t2) => match t1.shape().len() {
                                    1 => {
                                        let a = t1.into_dimensionality::<Ix1>().unwrap();
                                        if t2.shape().len() == 2 && a.shape()[0] == t2.shape()[0] {
                                            let b = t2.into_dimensionality::<Ix2>().unwrap();
                                            MyCx::Tensor(a.dot(&b).into_dyn())
                                        } else {
                                            MyCx::Tensor(a * t2)
                                        }
                                    }
                                    2 => {
                                        let a = t1.into_dimensionality::<Ix2>().unwrap();
                                        if t2.shape().len() == 1 && a.shape()[1] == t2.shape()[0] {
                                            let b = t2.into_dimensionality::<Ix1>().unwrap();
                                            MyCx::Tensor(a.dot(&b).into_dyn())
                                        } else if t2.shape().len() == 2
                                            && a.shape()[1] == t2.shape()[0]
                                        {
                                            let b = t2.into_dimensionality::<Ix2>().unwrap();
                                            MyCx::Tensor(a.dot(&b).into_dyn())
                                        } else {
                                            MyCx::Tensor(a * t2)
                                        }
                                    }
                                    _ => MyCx::Tensor(t1 * t2),
                                },
                            },
                        },
                        Operation::Div => match left {
                            MyCx::F64(f1) => match right {
                                MyCx::F64(f2) => MyCx::F64(f1 / f2),
                                MyCx::Tensor(t2) => MyCx::Tensor(f1 / t2),
                            },
                            MyCx::Tensor(t1) => match right {
                                MyCx::F64(f2) => MyCx::Tensor(t1 / f2),
                                MyCx::Tensor(t2) => MyCx::Tensor(t1 / t2),
                            },
                        },
                        Operation::Rem => match left {
                            MyCx::F64(f1) => match right {
                                MyCx::F64(f2) => MyCx::F64(f1 % f2),
                                MyCx::Tensor(_) => {
                                    return Err(Error::EvalError(format!(
                                        "Not equal type : {:?}",
                                        token
                                    )));
                                }
                            },
                            MyCx::Tensor(t1) => match right {
                                MyCx::F64(f2) => MyCx::Tensor(t1 % f2),
                                MyCx::Tensor(t2) => MyCx::Tensor(t1 % t2),
                            },
                        },
                        Operation::Pow => match left {
                            MyCx::F64(f1) => match right {
                                MyCx::F64(f2) => MyCx::F64(f1.powf(f2.re)),
                                _ => {
                                    return Err(Error::EvalError(format!(
                                        "Not equal type : {:?}",
                                        token
                                    )));
                                }
                            },
                            MyCx::Tensor(t1) => {
                                match right {
                                    MyCx::F64(f) => {
                                        let shape = t1.shape();
                                        if f.re == -1. && shape.len() == 2 && shape[0] == shape[1] {
                                            let t = ctx
                                                .matrix_inv_cx(
                                                    &t1.into_dimensionality::<Ix2>().unwrap(),
                                                )
                                                .map_err(|e| {
                                                    Error::Function("pow".to_string(), e)
                                                })?;
                                            // let t = t1.into_dimensionality::<Ix2>().unwrap().inv()
                                            //     .map_err(|_| Error::Function("pow".to_string(), FuncEvalError::NumberArgs(0)))?;
                                            MyCx::Tensor(t.into_dyn())
                                        } else {
                                            MyCx::Tensor(t1.mapv(|a| a.powf(f.re)))
                                        }
                                    }
                                    _ => {
                                        return Err(Error::EvalError(format!(
                                            "Not equal type : {:?}",
                                            token
                                        )));
                                    }
                                }
                            }
                        },
                        Operation::LessThan => match left {
                            MyCx::F64(f1) => match right {
                                MyCx::F64(f2) => MyCx::F64(if f1.re < f2.re {
                                    Complex64::new(1., 0.)
                                } else {
                                    Complex64::new(0., 0.)
                                }),
                                _ => {
                                    return Err(Error::EvalError(format!(
                                        "Not equal type : {:?}",
                                        token
                                    )));
                                }
                            },
                            _ => {
                                return Err(Error::EvalError(format!(
                                    "Not equal type : {:?}",
                                    token
                                )));
                            }
                        },
                        Operation::GreatThan => match left {
                            MyCx::F64(f1) => match right {
                                MyCx::F64(f2) => MyCx::F64(if f1.re > f2.re {
                                    Complex64::new(1., 0.)
                                } else {
                                    Complex64::new(0., 0.)
                                }),
                                _ => {
                                    return Err(Error::EvalError(format!(
                                        "Not equal type : {:?}",
                                        token
                                    )));
                                }
                            },
                            _ => {
                                return Err(Error::EvalError(format!(
                                    "Not equal type : {:?}",
                                    token
                                )));
                            }
                        },
                        Operation::LtOrEqual => match left {
                            MyCx::F64(f1) => match right {
                                MyCx::F64(f2) => MyCx::F64(if f1.re <= f2.re {
                                    Complex64::new(1., 0.)
                                } else {
                                    Complex64::new(0., 0.)
                                }),
                                _ => {
                                    return Err(Error::EvalError(format!(
                                        "Not equal type : {:?}",
                                        token
                                    )));
                                }
                            },
                            _ => {
                                return Err(Error::EvalError(format!(
                                    "Not equal type : {:?}",
                                    token
                                )));
                            }
                        },
                        Operation::GtOrEqual => match left {
                            MyCx::F64(f1) => match right {
                                MyCx::F64(f2) => MyCx::F64(if f1.re >= f2.re {
                                    Complex64::new(1., 0.)
                                } else {
                                    Complex64::new(0., 0.)
                                }),
                                _ => {
                                    return Err(Error::EvalError(format!(
                                        "Not equal type : {:?}",
                                        token
                                    )));
                                }
                            },
                            _ => {
                                return Err(Error::EvalError(format!(
                                    "Not equal type : {:?}",
                                    token
                                )));
                            }
                        },
                        Operation::Equal => match left {
                            MyCx::F64(f1) => match right {
                                MyCx::F64(f2) => MyCx::F64(if f1.re == f2.re {
                                    Complex64::new(1., 0.)
                                } else {
                                    Complex64::new(0., 0.)
                                }),
                                _ => {
                                    return Err(Error::EvalError(format!(
                                        "Not equal type : {:?}",
                                        token
                                    )));
                                }
                            },
                            _ => {
                                return Err(Error::EvalError(format!(
                                    "Not equal type : {:?}",
                                    token
                                )));
                            }
                        },
                        Operation::Unequal => match left {
                            MyCx::F64(f1) => match right {
                                MyCx::F64(f2) => MyCx::F64(if f1.re != f2.re {
                                    Complex64::new(1., 0.)
                                } else {
                                    Complex64::new(0., 0.)
                                }),
                                _ => {
                                    return Err(Error::EvalError(format!(
                                        "Not equal type : {:?}",
                                        token
                                    )));
                                }
                            },
                            _ => {
                                return Err(Error::EvalError(format!(
                                    "Not equal type : {:?}",
                                    token
                                )));
                            }
                        },
                        Operation::And => match left {
                            MyCx::F64(f1) => match right {
                                MyCx::F64(f2) => MyCx::F64(if (f1.re > 0.0) && (f2.re > 0.0) {
                                    Complex64::new(1., 0.)
                                } else {
                                    Complex64::new(0., 0.)
                                }),
                                _ => {
                                    return Err(Error::EvalError(format!(
                                        "Not equal type : {:?}",
                                        token
                                    )));
                                }
                            },
                            _ => {
                                return Err(Error::EvalError(format!(
                                    "Not equal type : {:?}",
                                    token
                                )));
                            }
                        },
                        Operation::Or => match left {
                            MyCx::F64(f1) => match right {
                                MyCx::F64(f2) => MyCx::F64(if (f1.re > 0.0) || (f2.re > 0.0) {
                                    Complex64::new(1., 0.)
                                } else {
                                    Complex64::new(0., 0.)
                                }),
                                _ => {
                                    return Err(Error::EvalError(format!(
                                        "Not equal type : {:?}",
                                        token
                                    )));
                                }
                            },
                            _ => {
                                return Err(Error::EvalError(format!(
                                    "Not equal type : {:?}",
                                    token
                                )));
                            }
                        },
                        Operation::BitAnd => match left {
                            MyCx::F64(f1) => match right {
                                MyCx::F64(f2) => MyCx::F64(Complex64::new(
                                    (f1.re as i64 & f2.re as i64) as f64,
                                    0.,
                                )),
                                _ => {
                                    return Err(Error::EvalError(format!(
                                        "Not equal type : {:?}",
                                        token
                                    )));
                                }
                            },
                            _ => {
                                return Err(Error::EvalError(format!(
                                    "Not equal type : {:?}",
                                    token
                                )));
                            }
                        },
                        Operation::BitOr => match left {
                            MyCx::F64(f1) => match right {
                                MyCx::F64(f2) => MyCx::F64(Complex64::new(
                                    (f1.re as i64 | f2.re as i64) as f64,
                                    0.,
                                )),
                                _ => {
                                    return Err(Error::EvalError(format!(
                                        "Not equal type : {:?}",
                                        token
                                    )));
                                }
                            },
                            _ => {
                                return Err(Error::EvalError(format!(
                                    "Not equal type : {:?}",
                                    token
                                )));
                            }
                        },
                        Operation::BitXor => match left {
                            MyCx::F64(f1) => match right {
                                MyCx::F64(f2) => MyCx::F64(Complex64::new(
                                    (f1.re as i64 ^ f2.re as i64) as f64,
                                    0.,
                                )),
                                _ => {
                                    return Err(Error::EvalError(format!(
                                        "Not equal type : {:?}",
                                        token
                                    )));
                                }
                            },
                            _ => {
                                return Err(Error::EvalError(format!(
                                    "Not equal type : {:?}",
                                    token
                                )));
                            }
                        },
                        Operation::BitShl => match left {
                            MyCx::F64(f1) => match right {
                                MyCx::F64(f2) => MyCx::F64(Complex64::new(
                                    ((f1.re as i64) << (f2.re as i64)) as f64,
                                    0.,
                                )),
                                _ => {
                                    return Err(Error::EvalError(format!(
                                        "Not equal type : {:?}",
                                        token
                                    )));
                                }
                            },
                            _ => {
                                return Err(Error::EvalError(format!(
                                    "Not equal type : {:?}",
                                    token
                                )));
                            }
                        },
                        Operation::BitShr => match left {
                            MyCx::F64(f1) => match right {
                                MyCx::F64(f2) => MyCx::F64(Complex64::new(
                                    ((f1.re as i64) >> (f2.re as i64)) as f64,
                                    0.,
                                )),
                                _ => {
                                    return Err(Error::EvalError(format!(
                                        "Not equal type : {:?}",
                                        token
                                    )));
                                }
                            },
                            _ => {
                                return Err(Error::EvalError(format!(
                                    "Not equal type : {:?}",
                                    token
                                )));
                            }
                        },
                        Operation::BitAt => match left {
                            MyCx::F64(f1) => match right {
                                MyCx::F64(f2) => {
                                    if f1.re < 1. || f2.re > 64. {
                                        return Err(Error::EvalError(format!(
                                            "Operation \"@\" ERROR:the {:?} bit doesn't exist.",
                                            right
                                        )));
                                    }
                                    MyCx::F64(
                                        if (f1.re as i64) & 2_i64.pow(f2.re as u32 - 1) != 0 {
                                            Complex64::new(1., 0.)
                                        } else {
                                            Complex64::new(0., 0.)
                                        },
                                    )
                                }
                                _ => {
                                    return Err(Error::EvalError(format!(
                                        "Not equal type : {:?}",
                                        token
                                    )));
                                }
                            },
                            _ => {
                                return Err(Error::EvalError(format!(
                                    "Not equal type : {:?}",
                                    token
                                )));
                            }
                        },
                        _ => {
                            return Err(Error::EvalError(format!("TypeUnsupported : {:?}", token)));
                        }
                    };
                    stack.push(r);
                }
                Unary(op) => {
                    let x = stack.pop().unwrap();
                    let r = match op {
                        Operation::Plus => x,
                        Operation::Minus => match x {
                            MyCx::F64(f) => MyCx::F64(-f),
                            MyCx::Tensor(t) => MyCx::Tensor(-t),
                        },
                        Operation::Not => match x {
                            MyCx::F64(f) => MyCx::F64(Complex64::new(if f.re > 0. {1.0} else {0.}, 0.)),
                            _ => {
                                return Err(Error::EvalError(format!(
                                    "Unimplemented unary operation: {:?}, {:?}",
                                    op, token
                                )));
                            }
                        }
                        Operation::BitNot => match x {
                            MyCx::F64(f) => MyCx::F64(Complex64::new(!(f.re as i64) as f64, 0.)),
                            _ => {
                                return Err(Error::EvalError(format!(
                                    "Unimplemented unary operation: {:?}, {:?}",
                                    op, token
                                )));
                            }
                        },
                        Operation::Fact => match x {
                            MyCx::F64(f) => match factorial(f.re) {
                                Ok(res) => MyCx::F64(Complex64::new(res, 0.)),
                                Err(e) => return Err(Error::EvalError(String::from(e))),
                            },
                            _ => {
                                return Err(Error::EvalError(format!(
                                    "Unimplemented unary operation: {:?}, {:?}",
                                    op, token
                                )));
                            }
                        },
                        _ => {
                            return Err(Error::EvalError(format!(
                                "Unimplemented unary operation: {:?}",
                                op
                            )));
                        }
                    };
                    stack.push(r);
                }
                Func(n, Some(i)) => {
                    if stack.len() < *i {
                        let msg = format!(
                            "stack does not have enough arguments for function token {:?}",
                            token
                        );
                        return Err(Error::EvalError(msg));
                    }
                    match ctx.eval_func_tensor_cx(n, &stack[stack.len() - i..]) {
                        Ok(r) => {
                            let nl = stack.len() - i;
                            stack.truncate(nl);
                            stack.push(r);
                        }
                        Err(e) => return Err(Error::Function(n.to_owned(), e)),
                    }
                }
                Func(ref n, None) => match ctx.eval_func(n, &[]) {
                    Ok(r) => {
                        stack.push(MyCx::F64(Complex64::new(r, 0.)));
                    }
                    Err(e) => return Err(Error::Function(n.to_owned(), e)),
                },
                _ => {
                    return Err(Error::EvalError(format!("TypeUnsupported : {:?}", token)));
                }
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

#[derive(Clone)]
pub struct ContextTensor<'a> {
    tensors: FnvHashMap<String, Array<f64, IxDyn>>,
    tensors_cx: FnvHashMap<String, Array<Complex64, IxDyn>>,
    ctx: Context<'a>,
    ctx_cx: ContextCx<'a>,
}

impl<'a> ContextTensor<'a> {
    /// Creates a context with built-in constants and functions.
    pub fn new() -> ContextTensor<'a> {
        thread_local!(static DEFAULT_CONTEXT: ContextTensor<'static> = {
            ContextTensor::empty()
        });

        DEFAULT_CONTEXT.with(|ctx| ctx.clone())
    }

    /// Creates an empty contexts.
    pub fn empty() -> ContextTensor<'a> {
        ContextTensor {
            tensors: Default::default(),
            tensors_cx: Default::default(),
            ctx: Default::default(),
            ctx_cx: Default::default(),
        }
    }

    /// Adds a new variable/constant.
    pub fn var<S: Into<String>>(&mut self, var: S, value: f64) -> &mut Self {
        self.ctx.var(var.into(), value);
        self
    }

    pub fn var_cx<S: Into<String>>(&mut self, var: S, value: Complex64) -> &mut Self {
        self.ctx_cx.var_cx(var.into(), value);
        self
    }

    /// Adds a new variable/constant.
    pub fn tensor<S: Into<String>>(&mut self, var: S, value: Array<f64, IxDyn>) -> &mut Self {
        self.tensors.insert(var.into(), value);
        self
    }

    pub fn tensor_cx<S: Into<String>>(
        &mut self,
        var: S,
        value: Array<Complex64, IxDyn>,
    ) -> &mut Self {
        self.tensors_cx.insert(var.into(), value);
        self
    }

    pub fn clean(&mut self) {
        self.tensors.clear();
        self.tensors_cx.clear();
    }
 }

impl<'a> Default for ContextTensor<'a> {
    fn default() -> Self {
        ContextTensor::new()
    }
}

impl<'a> ContextProvider for ContextTensor<'a> {
    fn get_var(&self, name: &str) -> Option<f64> {
        self.ctx.get_var(name)
    }

    fn get_var_cx(&self, name: &str) -> Option<Complex64> {
        self.ctx_cx.get_var_cx(name)
    }

    fn get_tensor(&self, name: &str) -> Option<Array<f64, IxDyn>> {
        self.tensors.get(name).cloned()
    }

    fn get_tensor_cx(&self, name: &str) -> Option<Array<Complex64, IxDyn>> {
        self.tensors_cx.get(name).cloned()
    }

    fn eval_func_tensor(&self, name: &str, args: &[MyF]) -> Result<MyF, FuncEvalError> {
        let mut floats = Vec::with_capacity(args.len());
        for arg in args {
            match arg {
                MyF::F64(v) => floats.push(*v),
                MyF::Tensor(_) => break,
            }
        }
        if name.eq("eye") {
            return TsfnBasic::ts_eye(args)
        } else if name.eq("zeros") {
            return TsfnBasic::ts_zeros(args)
        } else if name.eq("ones") {
            return TsfnBasic::ts_ones(args)
        } else if name.eq("range") {
            return TsfnBasic::ts_range(args)
        }
        if floats.len() == args.len() {
            return Ok(MyF::F64(self.ctx.eval_func(name, &floats)?));
        }
        match name {
            "get" => TsfnBasic::ts_get(args),
            "slice" => TsfnBasic::ts_slice(args),
            "abs" => match &args[0] {
                MyF::F64(f) => Ok(MyF::F64(self.ctx.eval_func(name, &[*f])?)),
                MyF::Tensor(t) => Ok(MyF::Tensor(t.mapv(|a| a.abs()))),
            },
            "exp" => match &args[0] {
                MyF::F64(f) => Ok(MyF::F64(self.ctx.eval_func(name, &[*f])?)),
                MyF::Tensor(t) => Ok(MyF::Tensor(t.mapv(|a| a.exp()))),
            },
            "sin" => match &args[0] {
                MyF::F64(f) => Ok(MyF::F64(self.ctx.eval_func(name, &[*f])?)),
                MyF::Tensor(t) => Ok(MyF::Tensor(t.mapv(|a| a.sin()))),
            },
            "cos" => match &args[0] {
                MyF::F64(f) => Ok(MyF::F64(self.ctx.eval_func(name, &[*f])?)),
                MyF::Tensor(t) => Ok(MyF::Tensor(t.mapv(|a| a.cos()))),
            },
            "tan" => match &args[0] {
                MyF::F64(f) => Ok(MyF::F64(self.ctx.eval_func(name, &[*f])?)),
                MyF::Tensor(t) => Ok(MyF::Tensor(t.mapv(|a| a.tan()))),
            },
            "asin" => match &args[0] {
                MyF::F64(f) => Ok(MyF::F64(self.ctx.eval_func(name, &[*f])?)),
                MyF::Tensor(t) => Ok(MyF::Tensor(t.mapv(|a| a.asin()))),
            },
            "acos" => match &args[0] {
                MyF::F64(f) => Ok(MyF::F64(self.ctx.eval_func(name, &[*f])?)),
                MyF::Tensor(t) => Ok(MyF::Tensor(t.mapv(|a| a.acos()))),
            },
            "atan" => match &args[0] {
                MyF::F64(f) => Ok(MyF::F64(self.ctx.eval_func(name, &[*f])?)),
                MyF::Tensor(t) => Ok(MyF::Tensor(t.mapv(|a| a.atan()))),
            },
            "sinh" => match &args[0] {
                MyF::F64(f) => Ok(MyF::F64(self.ctx.eval_func(name, &[*f])?)),
                MyF::Tensor(t) => Ok(MyF::Tensor(t.mapv(|a| a.sinh()))),
            },
            "cosh" => match &args[0] {
                MyF::F64(f) => Ok(MyF::F64(self.ctx.eval_func(name, &[*f])?)),
                MyF::Tensor(t) => Ok(MyF::Tensor(t.mapv(|a| a.cosh()))),
            },
            "tanh" => match &args[0] {
                MyF::F64(f) => Ok(MyF::F64(self.ctx.eval_func(name, &[*f])?)),
                MyF::Tensor(t) => Ok(MyF::Tensor(t.mapv(|a| a.tanh()))),
            },
            "asinh" => match &args[0] {
                MyF::F64(f) => Ok(MyF::F64(self.ctx.eval_func(name, &[*f])?)),
                MyF::Tensor(t) => Ok(MyF::Tensor(t.mapv(|a| a.asinh()))),
            },
            "acosh" => match &args[0] {
                MyF::F64(f) => Ok(MyF::F64(self.ctx.eval_func(name, &[*f])?)),
                MyF::Tensor(t) => Ok(MyF::Tensor(t.mapv(|a| a.acosh()))),
            },
            "atanh" => match &args[0] {
                MyF::F64(f) => Ok(MyF::F64(self.ctx.eval_func(name, &[*f])?)),
                MyF::Tensor(t) => Ok(MyF::Tensor(t.mapv(|a| a.atanh()))),
            },
            "deg2rad" => match &args[0] {
                MyF::F64(f) => Ok(MyF::F64(f * PI / 180.)),
                MyF::Tensor(t) => Ok(MyF::Tensor(t * PI / 180.)),
            },
            "rad2deg" => match &args[0] {
                MyF::F64(f) => Ok(MyF::F64(f / PI * 180.)),
                MyF::Tensor(t) => Ok(MyF::Tensor(t / PI * 180.)),
            },
            "ln" => match &args[0] {
                MyF::F64(f) => Ok(MyF::F64(self.ctx.eval_func(name, &[*f])?)),
                MyF::Tensor(t) => Ok(MyF::Tensor(t.mapv(|a| a.ln()))),
            },
            "log10" => match &args[0] {
                MyF::F64(f) => Ok(MyF::F64(self.ctx.eval_func(name, &[*f])?)),
                MyF::Tensor(t) => Ok(MyF::Tensor(t.mapv(|a| a.log10()))),
            },
            "power" => TsfnBasic::ts_power(args),
            "sqrt" => match &args[0] {
                MyF::F64(f) => Ok(MyF::F64(self.ctx.eval_func(name, &[*f])?)),
                MyF::Tensor(t) => Ok(MyF::Tensor(t.mapv(|a| a.sqrt()))),
            },
            "floor" => match &args[0] {
                MyF::F64(f) => Ok(MyF::F64(self.ctx.eval_func(name, &[*f])?)),
                MyF::Tensor(t) => Ok(MyF::Tensor(t.mapv(|a| a.floor()))),
            },
            "ceil" => match &args[0] {
                MyF::F64(f) => Ok(MyF::F64(self.ctx.eval_func(name, &[*f])?)),
                MyF::Tensor(t) => Ok(MyF::Tensor(t.mapv(|a| a.ceil()))),
            },
            "round" => match &args[0] {
                MyF::F64(f) => Ok(MyF::F64(self.ctx.eval_func(name, &[*f])?)),
                MyF::Tensor(t) => Ok(MyF::Tensor(t.mapv(|a| a.round()))),
            },
            "signum" => match &args[0] {
                MyF::F64(f) => Ok(MyF::F64(self.ctx.eval_func(name, &[*f])?)),
                MyF::Tensor(t) => Ok(MyF::Tensor(t.mapv(|a| a.signum()))),
            },
            "sum_all" => match &args[0] {
                MyF::F64(f) => Ok(MyF::F64(*f)),
                MyF::Tensor(t) => Ok(MyF::F64(t.sum())),
            },
            "sum" => TsfnBasic::ts_sum(args),
            "transpose" => match &args[0] {
                MyF::F64(f) => Ok(MyF::F64(*f)),
                MyF::Tensor(t) => Ok(MyF::Tensor(t.clone().reversed_axes()))
            },
            "size" => TsfnBasic::ts_size(args),
            "sparse" => TsfnBasic::ts_sparse(args),
            "diag" => TsfnBasic::ts_diag(args),
            // "trace" => TsfnBasic::ts_trace(args),
            _ => Err(FuncEvalError::UnknownFunction),
        }
    }

    fn eval_func_tensor_cx(&self, name: &str, args: &[MyCx]) -> Result<MyCx, FuncEvalError> {
        let mut complex = Vec::with_capacity(args.len());
        for arg in args {
            match arg {
                MyCx::F64(v) => complex.push(*v),
                MyCx::Tensor(_) => break,
            }
        }
        if name.eq("eye") {
            return TsfnBasic::ts_eye_cx(args)
        } else if name.eq("zeros") {
            return TsfnBasic::ts_zeros_cx(args)
        } else if name.eq("ones") {
            return TsfnBasic::ts_ones_cx(args)
        }

        if complex.len() == args.len() {
            return Ok(MyCx::F64(self.ctx_cx.eval_func_cx(name, &complex)?));
        }
        match name {
            "get" => TsfnBasic::ts_get_cx(args),
            "slice" => TsfnBasic::ts_slice_cx(args),
            "abs" => match &args[0] {
                MyCx::F64(f) => Ok(MyCx::F64(self.ctx_cx.eval_func_cx(name, &[*f])?)),
                MyCx::Tensor(t) => Ok(MyCx::Tensor(
                    t.mapv(|a| self.ctx_cx.eval_func_cx(name, &[a]).unwrap()),
                )),
            },
            "exp" => match &args[0] {
                MyCx::F64(f) => Ok(MyCx::F64(self.ctx_cx.eval_func_cx(name, &[*f])?)),
                MyCx::Tensor(t) => Ok(MyCx::Tensor(t.mapv(|a| a.exp()))),
            },
            "sin" => match &args[0] {
                MyCx::F64(f) => Ok(MyCx::F64(self.ctx_cx.eval_func_cx(name, &[*f])?)),
                MyCx::Tensor(t) => Ok(MyCx::Tensor(t.mapv(|a| a.sin()))),
            },
            "cos" => match &args[0] {
                MyCx::F64(f) => Ok(MyCx::F64(self.ctx_cx.eval_func_cx(name, &[*f])?)),
                MyCx::Tensor(t) => Ok(MyCx::Tensor(t.mapv(|a| a.cos()))),
            },
            "tan" => match &args[0] {
                MyCx::F64(f) => Ok(MyCx::F64(self.ctx_cx.eval_func_cx(name, &[*f])?)),
                MyCx::Tensor(t) => Ok(MyCx::Tensor(t.mapv(|a| a.tan()))),
            },
            "asin" => match &args[0] {
                MyCx::F64(f) => Ok(MyCx::F64(self.ctx_cx.eval_func_cx(name, &[*f])?)),
                MyCx::Tensor(t) => Ok(MyCx::Tensor(t.mapv(|a| a.asin()))),
            },
            "acos" => match &args[0] {
                MyCx::F64(f) => Ok(MyCx::F64(self.ctx_cx.eval_func_cx(name, &[*f])?)),
                MyCx::Tensor(t) => Ok(MyCx::Tensor(t.mapv(|a| a.acos()))),
            },
            "atan" => match &args[0] {
                MyCx::F64(f) => Ok(MyCx::F64(self.ctx_cx.eval_func_cx(name, &[*f])?)),
                MyCx::Tensor(t) => Ok(MyCx::Tensor(t.mapv(|a| a.atan()))),
            },
            "sinh" => match &args[0] {
                MyCx::F64(f) => Ok(MyCx::F64(self.ctx_cx.eval_func_cx(name, &[*f])?)),
                MyCx::Tensor(t) => Ok(MyCx::Tensor(t.mapv(|a| a.sinh()))),
            },
            "cosh" => match &args[0] {
                MyCx::F64(f) => Ok(MyCx::F64(self.ctx_cx.eval_func_cx(name, &[*f])?)),
                MyCx::Tensor(t) => Ok(MyCx::Tensor(t.mapv(|a| a.cosh()))),
            },
            "tanh" => match &args[0] {
                MyCx::F64(f) => Ok(MyCx::F64(self.ctx_cx.eval_func_cx(name, &[*f])?)),
                MyCx::Tensor(t) => Ok(MyCx::Tensor(t.mapv(|a| a.tanh()))),
            },
            "asinh" => match &args[0] {
                MyCx::F64(f) => Ok(MyCx::F64(self.ctx_cx.eval_func_cx(name, &[*f])?)),
                MyCx::Tensor(t) => Ok(MyCx::Tensor(t.mapv(|a| a.asinh()))),
            },
            "acosh" => match &args[0] {
                MyCx::F64(f) => Ok(MyCx::F64(self.ctx_cx.eval_func_cx(name, &[*f])?)),
                MyCx::Tensor(t) => Ok(MyCx::Tensor(t.mapv(|a| a.acosh()))),
            },
            "atanh" => match &args[0] {
                MyCx::F64(f) => Ok(MyCx::F64(self.ctx_cx.eval_func_cx(name, &[*f])?)),
                MyCx::Tensor(t) => Ok(MyCx::Tensor(t.mapv(|a| a.atanh()))),
            },
            "deg2rad" => match &args[0] {
                MyCx::F64(f) => Ok(MyCx::F64(f * PI / 180.)),
                MyCx::Tensor(t) => Ok(MyCx::Tensor(t * PI / 180.)),
            },
            "rad2deg" => match &args[0] {
                MyCx::F64(f) => Ok(MyCx::F64(f / PI * 180.)),
                MyCx::Tensor(t) => Ok(MyCx::Tensor(t / PI * 180.)),
            },
            "ln" => match &args[0] {
                MyCx::F64(f) => Ok(MyCx::F64(self.ctx_cx.eval_func_cx(name, &[*f])?)),
                MyCx::Tensor(t) => Ok(MyCx::Tensor(t.mapv(|a| a.ln()))),
            },
            "log10" => match &args[0] {
                MyCx::F64(f) => Ok(MyCx::F64(self.ctx_cx.eval_func_cx(name, &[*f])?)),
                MyCx::Tensor(t) => Ok(MyCx::Tensor(t.mapv(|a| a.log10()))),
            },
            "power" => TsfnBasic::ts_power_cx(args),
            "sqrt" => match &args[0] {
                MyCx::F64(f) => Ok(MyCx::F64(self.ctx_cx.eval_func_cx(name, &[*f])?)),
                MyCx::Tensor(t) => Ok(MyCx::Tensor(t.mapv(|a| a.sqrt()))),
            },
            "floor" => match &args[0] {
                MyCx::F64(f) => Ok(MyCx::F64(self.ctx_cx.eval_func_cx(name, &[*f])?)),
                MyCx::Tensor(t) => Ok(MyCx::Tensor(
                    t.mapv(|a| self.ctx.eval_func_cx(name, &[a]).unwrap()),
                )),
            },
            "ceil" => match &args[0] {
                MyCx::F64(f) => Ok(MyCx::F64(self.ctx_cx.eval_func_cx(name, &[*f])?)),
                MyCx::Tensor(t) => Ok(MyCx::Tensor(
                    t.mapv(|a| self.ctx.eval_func_cx(name, &[a]).unwrap()),
                )),
            },
            "round" => match &args[0] {
                MyCx::F64(f) => Ok(MyCx::F64(self.ctx_cx.eval_func_cx(name, &[*f])?)),
                MyCx::Tensor(t) => Ok(MyCx::Tensor(
                    t.mapv(|a| self.ctx.eval_func_cx(name, &[a]).unwrap()),
                )),
            },
            "signum" => match &args[0] {
                MyCx::F64(f) => Ok(MyCx::F64(self.ctx_cx.eval_func_cx(name, &[*f])?)),
                MyCx::Tensor(t) => Ok(MyCx::Tensor(
                    t.mapv(|a| self.ctx.eval_func_cx(name, &[a]).unwrap()),
                )),
            },
            "sum_all" => match &args[0] {
                MyCx::F64(f) => Ok(MyCx::F64(*f)),
                MyCx::Tensor(t) => Ok(MyCx::F64(t.sum())),
            },
            "sum" => TsfnBasic::ts_sum_cx(args),
            "conj" => match &args[0] {
                MyCx::F64(f) => Ok(MyCx::F64(self.ctx_cx.eval_func_cx(name, &[*f])?)),
                MyCx::Tensor(t) => Ok(MyCx::Tensor(
                    t.mapv(|a| self.ctx_cx.eval_func_cx(name, &[a]).unwrap()),
                )),
            },
            "real" => match &args[0] {
                MyCx::F64(f) => Ok(MyCx::F64(self.ctx_cx.eval_func_cx(name, &[*f])?)),
                MyCx::Tensor(t) => Ok(MyCx::Tensor(
                    t.mapv(|a| self.ctx_cx.eval_func_cx(name, &[a]).unwrap()),
                )),
            },
            "imag" => match &args[0] {
                MyCx::F64(f) => Ok(MyCx::F64(self.ctx_cx.eval_func_cx(name, &[*f])?)),
                MyCx::Tensor(t) => Ok(MyCx::Tensor(
                    t.mapv(|a| self.ctx_cx.eval_func_cx(name, &[a]).unwrap()),
                )),
            },
            "angle" => match &args[0] {
                MyCx::F64(f) => Ok(MyCx::F64(self.ctx_cx.eval_func_cx("rad", &[*f])?)),
                MyCx::Tensor(t) => Ok(MyCx::Tensor(
                    t.mapv(|a| self.ctx_cx.eval_func_cx("rad", &[a]).unwrap()),
                )),
            },
            "transpose" => match &args[0] {
                MyCx::F64(f) => Ok(MyCx::F64(*f)),
                MyCx::Tensor(t) => Ok(MyCx::Tensor(t.clone().reversed_axes()))
            },
            "ctranspose" => match &args[0] {
                MyCx::F64(f) => Ok(MyCx::F64(self.ctx_cx.eval_func_cx("conj", &[*f])?)),
                MyCx::Tensor(t) => Ok(MyCx::Tensor(t.clone().reversed_axes()
                    .mapv(|a| self.ctx_cx.eval_func_cx("conj", &[a]).unwrap())))
            },
            "size" => TsfnBasic::ts_size_cx(args),
            // "eig" => TsfnBasic::ts_eig(args),
            // "diag" => TsfnBasic::ts_diag_cx(args),
            // "trace" => TsfnBasic::ts_trace_cx(args),
            _ => Err(FuncEvalError::UnknownFunction),
        }
    }
}