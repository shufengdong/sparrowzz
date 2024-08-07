// flowing should as same as in sparrowzz
use ndarray::{arr1, Array1, Array2, Axis, IxDyn, SliceInfo, SliceInfoElem};
use num_traits::ToPrimitive;
use crate::{FuncEvalError, MyCx, MyF};
use num_complex::Complex64;

pub trait TsLinalgFn {
    fn ts_eig(_: &[MyCx]) -> Result<MyCx, FuncEvalError> {
        Err(FuncEvalError::UnknownFunction)
    }

    fn ts_trace(_: &[MyF]) -> Result<MyF, FuncEvalError> {
        Err(FuncEvalError::UnknownFunction)
    }

    fn ts_trace_cx(_: &[MyCx]) -> Result<MyCx, FuncEvalError> {
        Err(FuncEvalError::UnknownFunction)
    }
}

pub struct TsfnBasic {
}

impl TsfnBasic {
    pub fn ts_get(args: &[MyF]) -> Result<MyF, FuncEvalError> {
        match &args[0] {
            MyF::F64(_) => Err(FuncEvalError::UnknownFunction),
            MyF::Tensor(t) => {
                if args.len() == 1 {
                    Ok(MyF::Tensor(t.clone()))
                } else {
                    let mut index = Vec::with_capacity(args.len() - 1);
                    for (i, arg) in args.iter().enumerate().skip(1) {
                        match arg {
                            MyF::F64(f) => index.push(*f as usize),
                            MyF::Tensor(_) => return Err(FuncEvalError::NumberArgs(i)),
                        }
                    }
                    match t.get(&*index) {
                        None => Err(FuncEvalError::NumberArgs(0)),
                        Some(v) => Ok(MyF::F64(*v)),
                    }
                }
            }
        }
    }

    pub fn ts_get_cx(args: &[MyCx]) -> Result<MyCx, FuncEvalError> {
        match &args[0] {
            MyCx::F64(_) => Err(FuncEvalError::UnknownFunction),
            MyCx::Tensor(t) => {
                if args.len() == 1 {
                    Ok(MyCx::Tensor(t.clone()))
                } else {
                    let mut index = Vec::with_capacity(args.len() - 1);
                    for (i, arg) in args.iter().enumerate().skip(1) {
                        match arg {
                            MyCx::F64(f) => index.push(f.re as usize),
                            MyCx::Tensor(_) => return Err(FuncEvalError::NumberArgs(i)),
                        }
                    }
                    match t.get(&*index) {
                        None => Err(FuncEvalError::NumberArgs(0)),
                        Some(v) => Ok(MyCx::F64(*v)),
                    }
                }
            }
        }
    }

    pub(crate) fn ts_slice(args: &[MyF]) -> Result<MyF, FuncEvalError> {
        match &args[0] {
            MyF::F64(_) => Err(FuncEvalError::UnknownFunction),
            MyF::Tensor(t) => {
                if args.len() == 1 {
                    Ok(MyF::Tensor(t.clone()))
                } else {
                    let mut indices = Vec::with_capacity(args.len() - 1);
                    for (i, arg) in args.iter().enumerate().skip(1) {
                        match arg {
                            MyF::F64(f) => {
                                let s = SliceInfoElem::Index(*f as isize);
                                indices.push(s);
                            }
                            MyF::Tensor(t) => {
                                let v = t.as_slice().ok_or(FuncEvalError::NumberArgs(i))?;
                                let s = match v.len() {
                                    0 => {
                                        SliceInfoElem::NewAxis
                                    }
                                    1 => {
                                        SliceInfoElem::Slice {
                                            start: v[0] as isize,
                                            end: None,
                                            step: 1,
                                        }
                                    }
                                    2 => {
                                        SliceInfoElem::Slice {
                                            start: v[0] as isize,
                                            end: Some(v[1] as isize),
                                            step: 1,
                                        }
                                    }
                                    3 => {
                                        SliceInfoElem::Slice {
                                            start: v[0] as isize,
                                            end: Some(v[1] as isize),
                                            step: v[2] as isize,
                                        }
                                    }
                                    _ => {
                                        return Err(FuncEvalError::NumberArgs(i));
                                    }
                                };
                                indices.push(s);
                            }
                        }
                    }
                    let iter: SliceInfo<Vec<SliceInfoElem>, IxDyn, IxDyn> = SliceInfo::try_from(indices).map_err(
                        |_| FuncEvalError::NumberArgs(0),
                    )?;
                    Ok(MyF::Tensor(t.slice(iter).into_dyn().to_owned()))
                }
            }
        }
    }
    pub(crate) fn ts_slice_cx(args: &[MyCx]) -> Result<MyCx, FuncEvalError> {
        match &args[0] {
            MyCx::F64(_) => Err(FuncEvalError::UnknownFunction),
            MyCx::Tensor(t) => {
                if args.len() == 1 {
                    Ok(MyCx::Tensor(t.clone()))
                } else {
                    let mut indices = Vec::with_capacity(args.len() - 1);
                    for (i, arg) in args.iter().enumerate().skip(1) {
                        match arg {
                            MyCx::F64(f) => {
                                let s = SliceInfoElem::Index(f.re as isize);
                                indices.push(s);
                            }
                            MyCx::Tensor(t) => {
                                let v = t.as_slice().ok_or(FuncEvalError::NumberArgs(i))?;
                                let s = match v.len() {
                                    0 => {
                                        SliceInfoElem::NewAxis
                                    }
                                    1 => {
                                        SliceInfoElem::Slice {
                                            start: v[0].re as isize,
                                            end: None,
                                            step: 1,
                                        }
                                    }
                                    2 => {
                                        SliceInfoElem::Slice {
                                            start: v[0].re as isize,
                                            end: Some(v[1].re as isize),
                                            step: 1,
                                        }
                                    }
                                    3 => {
                                        SliceInfoElem::Slice {
                                            start: v[0].re as isize,
                                            end: Some(v[1].re as isize),
                                            step: v[2].re as isize,
                                        }
                                    }
                                    _ => {
                                        return Err(FuncEvalError::NumberArgs(i));
                                    }
                                };
                                indices.push(s);
                            }
                        }
                    }
                    let iter: SliceInfo<Vec<SliceInfoElem>, IxDyn, IxDyn> = SliceInfo::try_from(indices).map_err(
                        |_| FuncEvalError::NumberArgs(0),
                    )?;
                    Ok(MyCx::Tensor(t.slice(iter).into_dyn().to_owned()))
                }
            }
        }
    }

    pub fn ts_sum(args: &[MyF]) -> Result<MyF, FuncEvalError> {
        match &args[0] {
            MyF::F64(f) => Ok(MyF::F64(*f)),
            MyF::Tensor(t) => {
                match args.len() {
                    1 => {
                        if t.ndim() > 1 {
                            if t.shape().len() == 2 && t.shape()[1] == 1 {
                                Ok(MyF::F64(t.sum()))
                            } else {
                                Ok(MyF::Tensor(t.sum_axis(Axis(0))))
                            }
                        } else {
                            Ok(MyF::F64(t.sum()))
                        }
                    },
                    _ => {
                        match &args[1] {
                            MyF::F64(dim_f) => {
                                match dim_f.to_usize() {
                                    Some(dim) => {
                                        if dim < t.ndim() {
                                            Ok(MyF::Tensor(t.sum_axis(Axis(dim))))
                                        } else {
                                            Err(FuncEvalError::NumberArgs(1))
                                        }
                                    },
                                    None => Err(FuncEvalError::NumberArgs(1)),
                                }
                            },
                            MyF::Tensor(vexdim_f) => {
                                match vexdim_f.shape().len() {
                                    0 => {
                                        if t.ndim() > 1 {
                                            if t.shape().len() == 2 && t.shape()[1] == 1 {
                                                Ok(MyF::F64(t.sum()))
                                            } else {
                                                Ok(MyF::Tensor(t.sum_axis(Axis(0))))
                                            }
                                        } else {
                                            Ok(MyF::F64(t.sum()))
                                        }
                                    },
                                    1 => {
                                        let mut sum_t = t.clone();
                                        let mut count = 0;
                                        for dim_f in vexdim_f {
                                            match dim_f.to_usize() {
                                                Some(dim) =>
                                                    if dim < t.ndim() {
                                                        sum_t = sum_t.sum_axis(Axis(dim - count))
                                                    } else {
                                                        return Err(FuncEvalError::NumberArgs(1))
                                                    },
                                                None => return Err(FuncEvalError::NumberArgs(1)),
                                            };
                                            count += 1;
                                        }
                                        Ok(MyF::Tensor(sum_t))
                                    },
                                    _ => Err(FuncEvalError::NumberArgs(1)),
                                }
                            },
                        }
                    },
                }
            }
        }
    }

    pub fn ts_sum_cx(args: &[MyCx]) -> Result<MyCx, FuncEvalError> {
        match &args[0] {
            MyCx::F64(f) => Ok(MyCx::F64(*f)),
            MyCx::Tensor(t) => {
                match args.len() {
                    1 => {
                        if t.ndim() > 1 {
                            if t.shape().len() == 2 && t.shape()[1] == 1 {
                                Ok(MyCx::F64(t.sum()))
                            } else {
                                Ok(MyCx::Tensor(t.sum_axis(Axis(0))))
                            }
                        } else {
                            Ok(MyCx::F64(t.sum()))
                        }
                    },
                    _ => {
                        match &args[1] {
                            MyCx::F64(dim_f) => {
                                match dim_f.re.to_usize() {
                                    Some(dim) => {
                                        if dim < t.ndim() {
                                            Ok(MyCx::Tensor(t.sum_axis(Axis(dim))))
                                        } else {
                                            Err(FuncEvalError::NumberArgs(1))
                                        }
                                    },
                                    None => Err(FuncEvalError::NumberArgs(1)),
                                }
                            },
                            MyCx::Tensor(vexdim_f) => {
                                match vexdim_f.shape().len() {
                                    0 => {
                                        if t.ndim() > 1 {
                                            if t.shape().len() == 2 && t.shape()[1] == 1 {
                                                Ok(MyCx::F64(t.sum()))
                                            } else {
                                                Ok(MyCx::Tensor(t.sum_axis(Axis(0))))
                                            }
                                        } else {
                                            Ok(MyCx::F64(t.sum()))
                                        }
                                    },
                                    1 => {
                                        let mut sum_t = t.clone();
                                        for (count, dim_f) in vexdim_f.iter().enumerate() {
                                            match dim_f.re.to_usize() {
                                                Some(dim) =>
                                                    if dim < t.ndim() {
                                                        sum_t = sum_t.sum_axis(Axis(dim - count))
                                                    } else {
                                                        return Err(FuncEvalError::NumberArgs(1))
                                                    },
                                                None => return Err(FuncEvalError::NumberArgs(1)),
                                            };
                                        }
                                        Ok(MyCx::Tensor(sum_t))
                                    },
                                    _ => Err(FuncEvalError::NumberArgs(1)),
                                }
                            },
                        }
                    },
                }
            }
        }
    }

    pub fn ts_power(args: &[MyF]) -> Result<MyF, FuncEvalError> {
        match args.len() {
            0 => Err(FuncEvalError::TooFewArguments),
            1 => Err(FuncEvalError::TooFewArguments),
            2 => {
                match &args[1] {
                    MyF::F64(b) => {
                        match &args[0] {
                            MyF::F64(f) => Ok(MyF::F64(f.powf(*b))),
                            MyF::Tensor(t) => Ok(MyF::Tensor(t.mapv(|a| a.powf(*b)))),
                        }
                    },
                    MyF::Tensor(_) => Err(FuncEvalError::NumberArgs(2)),
                }
            },
            _ => Err(FuncEvalError::TooManyArguments),
        }
    }

    pub fn ts_power_cx(args: &[MyCx]) -> Result<MyCx, FuncEvalError> {
        match args.len() {
            0 => Err(FuncEvalError::TooFewArguments),
            1 => Err(FuncEvalError::TooFewArguments),
            2 => {
                match &args[1] {
                    MyCx::F64(b) => {
                        match &args[0] {
                            MyCx::F64(f) => Ok(MyCx::F64(f.powc(*b))),
                            MyCx::Tensor(t) => Ok(MyCx::Tensor(t.mapv(|a| a.powc(*b)))),
                        }
                    },
                    MyCx::Tensor(_) => Err(FuncEvalError::NumberArgs(2)),
                }
            },
            _ => Err(FuncEvalError::TooManyArguments),
        }
    }

    pub fn ts_diag(args: &[MyF]) -> Result<MyF, FuncEvalError> {
        match &args[0] {
            MyF::F64(f) => Ok(MyF::F64(*f)),
            MyF::Tensor(t) => {
                if t.ndim() > 1 {
                    if t.shape().len() == 2 && (t.shape()[0] == 1 || t.shape()[1] == 1) {
                        Ok(MyF::Tensor(Array2::from_diag(&arr1(t.clone().into_raw_vec_and_offset().0.as_slice())).into_dyn()))
                    } else {
                        Ok(MyF::Tensor(t.diag().into_dyn().to_owned()))
                    }
                } else {
                    Ok(MyF::Tensor(Array2::from_diag(&arr1(t.clone().into_raw_vec_and_offset().0.as_slice())).into_dyn()))
                }
            }
        }
    }

    pub(crate) fn ts_diag_cx(args: &[MyCx]) -> Result<MyCx, FuncEvalError> {
        match &args[0] {
            MyCx::F64(f) => Ok(MyCx::F64(*f)),
            MyCx::Tensor(t) => {
                if t.ndim() > 1 {
                    if t.shape().len() == 2 && t.shape()[1] == 1 {
                        Ok(MyCx::Tensor(Array2::from_diag(&arr1(t.clone().into_raw_vec_and_offset().0.as_slice())).into_dyn()))
                    } else {
                        Ok(MyCx::Tensor(t.diag().into_dyn().to_owned()))
                    }
                } else {
                    Ok(MyCx::Tensor(Array2::from_diag(&arr1(t.clone().into_raw_vec_and_offset().0.as_slice())).into_dyn()))
                }
            }
        }
    }

    pub fn ts_eye(args: &[MyF]) -> Result<MyF, FuncEvalError> {
        match &args[0] {
            MyF::F64(f) => {
                if *f < 1. {
                    Err(FuncEvalError::NumberArgs(0))
                } else {
                    Ok(MyF::Tensor(Array2::eye(f.to_usize().unwrap()).into_dyn()))
                }
            },
            MyF::Tensor(_) => Err(FuncEvalError::NumberArgs(0)),
        }
    }

    pub fn ts_eye_cx(args: &[MyCx]) -> Result<MyCx, FuncEvalError> {
        match &args[0] {
            MyCx::F64(f) => {
                if f.re < 1. {
                    Err(FuncEvalError::NumberArgs(0))
                } else {
                    Ok(MyCx::Tensor(Array2::eye(f.re.to_usize().unwrap()).into_dyn()))
                }
            },
            MyCx::Tensor(_) => Err(FuncEvalError::NumberArgs(0)),
        }
    }

    pub fn ts_zeros(args: &[MyF]) -> Result<MyF, FuncEvalError> {
        match &args[0] {
            MyF::F64(f1) => {
                match &args[1] {
                    MyF::F64(f2) => {
                        if *f1 < 1. {
                            Err(FuncEvalError::NumberArgs(0))
                        } else if *f2 < 1. {
                            Err(FuncEvalError::NumberArgs(1))
                        } else {
                            Ok(MyF::Tensor(Array2::zeros([f1.to_usize().unwrap(), f2.to_usize().unwrap()]).into_dyn()))
                        }
                    }
                    MyF::Tensor(_) => Err(FuncEvalError::NumberArgs(1)),
                }
            },
            MyF::Tensor(_) => Err(FuncEvalError::NumberArgs(0)),
        }
    }

    pub fn ts_zeros_cx(args: &[MyCx]) -> Result<MyCx, FuncEvalError> {
        match &args[0] {
            MyCx::F64(f1) => {
                match &args[1] {
                    MyCx::F64(f2) => {
                        if f1.re < 1. {
                            Err(FuncEvalError::NumberArgs(0))
                        } else if f2.re < 1. {
                            Err(FuncEvalError::NumberArgs(1))
                        } else {
                            Ok(MyCx::Tensor(Array2::zeros([f1.re.to_usize().unwrap(), f2.re.to_usize().unwrap()]).into_dyn()))
                        }
                    }
                    MyCx::Tensor(_) => Err(FuncEvalError::NumberArgs(1)),
                }
            },
            MyCx::Tensor(_) => Err(FuncEvalError::NumberArgs(0)),
        }
    }

    pub fn ts_ones(args: &[MyF]) -> Result<MyF, FuncEvalError> {
        match &args[0] {
            MyF::F64(f1) => {
                match &args[1] {
                    MyF::F64(f2) => {
                        if *f1 < 1. {
                            Err(FuncEvalError::NumberArgs(0))
                        } else if *f2 < 1. {
                            Err(FuncEvalError::NumberArgs(1))
                        } else {
                            Ok(MyF::Tensor(Array2::ones([f1.to_usize().unwrap(), f2.to_usize().unwrap()]).into_dyn()))
                        }
                    }
                    MyF::Tensor(_) => Err(FuncEvalError::NumberArgs(1)),
                }
            },
            MyF::Tensor(_) => Err(FuncEvalError::NumberArgs(0)),
        }
    }

    pub fn ts_ones_cx(args: &[MyCx]) -> Result<MyCx, FuncEvalError> {
        match &args[0] {
            MyCx::F64(f1) => {
                match &args[1] {
                    MyCx::F64(f2) => {
                        if f1.re < 1. {
                            Err(FuncEvalError::NumberArgs(0))
                        } else if f2.re < 1. {
                            Err(FuncEvalError::NumberArgs(1))
                        } else {
                            Ok(MyCx::Tensor(Array2::ones([f1.re.to_usize().unwrap(), f2.re.to_usize().unwrap()]).into_dyn()))
                        }
                    }
                    MyCx::Tensor(_) => Err(FuncEvalError::NumberArgs(1)),
                }
            },
            MyCx::Tensor(_) => Err(FuncEvalError::NumberArgs(0)),
        }
    }

    pub fn ts_range(args: &[MyF]) -> Result<MyF, FuncEvalError> {
        match args.len() {
            0 => Err(FuncEvalError::TooFewArguments),
            1 => Err(FuncEvalError::TooFewArguments),
            2 => Err(FuncEvalError::TooFewArguments),
            3 => {
                match &args[0] {
                    MyF::F64(start) => {
                        match &args[1] {
                            MyF::F64(end) => {
                                match &args[2] {
                                    MyF::F64(step) => {
                                        Ok(MyF::Tensor(Array1::range(*start, *end, *step).into_dyn()))
                                    }
                                    MyF::Tensor(_) => Err(FuncEvalError::NumberArgs(2)),
                                }
                            },
                            MyF::Tensor(_) => Err(FuncEvalError::NumberArgs(1)),
                        }
                    },
                    MyF::Tensor(_) => Err(FuncEvalError::NumberArgs(0)),
                }
            },
            _ => Err(FuncEvalError::TooManyArguments),
        }
    }

    pub fn ts_sparse(args: &[MyF]) -> Result<MyF, FuncEvalError> {
        if args.len() < 5 {
            return Err(FuncEvalError::TooFewArguments)
        } else if args.len() > 5 {
            return Err(FuncEvalError::TooManyArguments)
        }
        match &args[0] {
            MyF::F64(_) => Err(FuncEvalError::NumberArgs(0)),
            MyF::Tensor(i) => {
                match &args[1] {
                    MyF::F64(_) => Err(FuncEvalError::NumberArgs(1)),
                    MyF::Tensor(j) => {
                        match &args[2] {
                            MyF::F64(_) => Err(FuncEvalError::NumberArgs(2)),
                            MyF::Tensor(v) => {
                                match &args[3] {
                                    MyF::F64(m) => {
                                        match &args[4] {
                                            MyF::F64(n) => {
                                                if *m < 1. {
                                                    Err(FuncEvalError::NumberArgs(3))
                                                } else if *n < 1. {
                                                    Err(FuncEvalError::NumberArgs(4))
                                                } else {
                                                    let mut eq_size = i.len() == j.len();
                                                    if j.len() != v.len() {
                                                        eq_size = false;
                                                    }
                                                    if eq_size {
                                                        let mut matrix = Array2::zeros([m.to_usize().unwrap(), n.to_usize().unwrap()]);
                                                        let i_vec = i.to_owned().into_raw_vec_and_offset().0;
                                                        let j_vec = j.to_owned().into_raw_vec_and_offset().0;
                                                        let v_vec = v.to_owned().into_raw_vec_and_offset().0;
                                                        for k in 0..i.len() {
                                                            if i_vec[k] >= *m {
                                                                return Err(FuncEvalError::NumberArgs(0))
                                                            }
                                                            if j_vec[k] >= *n {
                                                                return Err(FuncEvalError::NumberArgs(1))
                                                            }
                                                            *matrix.get_mut([i_vec[k].to_usize().unwrap(), j_vec[k].to_usize().unwrap()]).unwrap() = v_vec[k];
                                                        }
                                                        Ok(MyF::Tensor(matrix.into_dyn()))
                                                    } else {
                                                        Err(FuncEvalError::NumberArgs(0))
                                                    }
                                                }
                                            }
                                            MyF::Tensor(_) => Err(FuncEvalError::NumberArgs(4)),
                                        }
                                    },
                                    MyF::Tensor(_) => Err(FuncEvalError::NumberArgs(3)),
                                }
                            }
                        }
                    },
                }
            },
        }
    }

    pub fn ts_size(args: &[MyF]) -> Result<MyF, FuncEvalError> {
        match &args[0] {
            MyF::Tensor(t) => {
                Ok(MyF::Tensor(Array1::from_vec(t.shape().iter().map(|e| e.to_f64().unwrap()).collect::<Vec<f64>>()).into_dyn()))
            },
            MyF::F64(_) => Err(FuncEvalError::NumberArgs(0)),
        }
    }

    pub fn ts_size_cx(args: &[MyCx]) -> Result<MyCx, FuncEvalError> {
        match &args[0] {
            MyCx::Tensor(t) => {
                Ok(MyCx::Tensor(Array1::from_vec(t.shape().iter().map(|e| Complex64::new(e.to_f64().unwrap(), 0.)).collect::<Vec<Complex64>>()).into_dyn()))
            },
            MyCx::F64(_) => Err(FuncEvalError::NumberArgs(0)),
        }
    }
}
// above should as same as in sparrowzz