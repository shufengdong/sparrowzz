use std::collections::HashMap;

use ndarray::{Array, Array2, Ix2};
use num_complex::Complex64;

use eig_domain::DataUnit;
use mems::model::dev::MeasPhase;

pub fn get_pf_nlp_constraints(
    // cn, tn
    dyn_topo: Vec<Vec<u64>>,
    // terminal, cn, tn, dev
    dev_topo: Vec<Vec<u64>>,
    dev_matrix: HashMap<u64, Vec<Array2<f64>>>,
    input_tns: Vec<u64>,
    input_phases: Vec<MeasPhase>,
    input_types: Vec<DataUnit>,
    input_values: Vec<f64>
) -> Option<Vec<String>> {
    let mut constraint = Vec::with_capacity(dyn_topo.len());
    for i in 0..input_tns.len() {
        match input_types[i] {
            DataUnit::kW => {
                let mut active_exp = String::new();
                constraint.push(active_exp);
                match input_phases[i] {
                    MeasPhase::Total => {}
                    MeasPhase::A => {}
                    MeasPhase::B => {}
                    MeasPhase::C => {}
                    MeasPhase::A0 => {}
                    MeasPhase::B0 => {}
                    MeasPhase::C0 => {}
                    MeasPhase::CT => {}
                    MeasPhase::PT => {}
                    MeasPhase::Unknown => return None,
                }
            }
            DataUnit::kVar => {
                let mut reactive_exp = String::new();
                constraint.push(reactive_exp);
            }
            DataUnit::kV => {}
            _ => {}
        }
    }
    Some(constraint)
}

pub fn get_pf_nlp_variables(tns: &[u64]) -> String {
    let mut variable = String::new();
    // 生成变量名
    for tn in tns {
        variable.push_str(&format!("V_{tn}_A:[0/2],D_{tn}_A:[-3.2/3.2],V_{tn}_B:[0/2],D_{tn}_B:[-3.2/3.2],V_{tn}_C:[0/2],D_{tn}_C:[-3.2/3.2]"));
    }
    variable
}

fn get_pq_of_acline(r_x: Array<Complex64, Ix2>) -> Option<(String, String)> {
    let mut mode = 0; //判断相位的模式
    if r_x[[0, 0]] != Complex64::new(0.0, 0.0) {
        mode += 1;
    }
    if r_x[[1, 1]] != Complex64::new(0.0, 0.0) {
        mode += 2;
    }
    if r_x[[2, 2]] != Complex64::new(0.0, 0.0) {
        mode += 4;
    }
    // 计算导纳阵
    let result = match mode {
        // A 或 B 或 C   r_x[[2,2]].inv().unwrap()
        1 => {
            let gb = r_x[[0, 0]].inv();
            format!("{}*x1", gb.re)
        }
        2 => {
            let gb = r_x[[1, 1]].inv();
            format!("{}*x1", gb.re)
        }
        4 => {
            let gb = r_x[[2, 2]].inv();
            format!("{}*x1", gb.re)
        }
        // AB
        3 => {
            let rx = nalgebra::Matrix2::new(
                r_x[[0, 0]], r_x[[0, 1]],
                r_x[[1, 0]], r_x[[1, 1]]);
            let gb = rx.try_inverse().unwrap();
            format!("{}*x1-{}*x2", gb.m11, gb.m12)
        }
        // AC
        5 => {
            let rx = nalgebra::Matrix2::new(
                r_x[[0, 0]], r_x[[0, 2]],
                r_x[[2, 0]], r_x[[2, 2]]);
            let gb = rx.try_inverse().unwrap();
            format!("{}*x1", gb.m11)
        }
        // BC
        6 => {
            let rx = nalgebra::Matrix2::new(
                r_x[[1, 1]], r_x[[1, 2]],
                r_x[[2, 1]], r_x[[2, 2]]);
            let gb = rx.try_inverse().unwrap();
            format!("{}*x1", gb.m11)
        }
        // ABC
        7 => {
            let rx = nalgebra::Matrix3::new(
                r_x[[0, 0]], r_x[[0, 1]], r_x[[0, 2]],
                r_x[[1, 0]], r_x[[1, 1]], r_x[[1, 2]],
                r_x[[2, 0]], r_x[[2, 1]], r_x[[2, 2]]);
            let gb = rx.try_inverse().unwrap();
            format!("{:.4}*x1-{:.4}*x2-{:.4}*x3", gb.m11.re, gb.m12.re, gb.m13.re)
        }
        _ => { return None; }
    };
    Some((result.clone(), result.clone()))
}

// test
#[cfg(test)]
mod test {
    use ndarray::array;
    use super::*;

    #[test]
    fn test_get_pq_of_acline() {
        // 原矩阵：
        // 0.3465+1.0179j  0.1560+0.5017j  0.1580+0.4236j
        // 0.1560+0.5017j  0.3375+1.0478j  0.1535+0.3849j
        // 0.1580+0.4236j  0.1535+0.3849j  0.3414+1.0348j
        // 求逆的结果：
        //   0.4338 - 1.2502i  -0.1840 + 0.4622i  -0.1008 + 0.3455i
        //   -0.1840 + 0.4622i   0.3798 - 1.1847i  -0.0478 + 0.2639i
        //   -0.1008 + 0.3455i  -0.0478 + 0.2639i   0.3359 - 1.1176i
        let arr = array![  [Complex64::new(0.3465,1.0179), Complex64::new(0.1560,0.5017), Complex64::new(0.1580,0.4236)],
                                            [Complex64::new(0.1560,0.5017), Complex64::new(0.3375,1.0478), Complex64::new(0.1535,0.3849)],
                                            [Complex64::new(0.1580,0.4236), Complex64::new(0.1535,0.3849), Complex64::new(0.3414,1.0348)]];
        let (p, q) = get_pq_of_acline(arr).unwrap();
        assert_eq!(p, "0.4338*x1--0.1840*x2--0.1008*x3");
        let arr = array![  [Complex64::new(0.0,0.0), Complex64::new(0.0,0.0), Complex64::new(0.0,0.0)],
                                            [Complex64::new(0.0,0.0), Complex64::new(0.0,0.0), Complex64::new(0.0,0.0)],
                                            [Complex64::new(0.0,0.0), Complex64::new(0.0,0.0), Complex64::new(0.3414,1.0348)]];
        let (p, q) = get_pq_of_acline(arr).unwrap();
    }
}