use std::collections::HashMap;

use ndarray::{Array, Array2, Ix2};
use num_complex::{Complex64, ComplexFloat};

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

fn get_pq_of_acline(r_x: Array<Complex64, Ix2>, tn1: u64, tn2: u64) -> Option<(Vec<String>, u32)> {
    let mut mode:u32 = 0; //判断相位的模式
    if r_x[[0, 0]] != Complex64::new(0.0, 0.0) {
        mode += 1;
    }
    if r_x[[1, 1]] != Complex64::new(0.0, 0.0) {
        mode += 2;
    }
    if r_x[[2, 2]] != Complex64::new(0.0, 0.0) {
        mode += 4;
    }
    let mut result = Vec::new();
    // 计算导纳阵
    match mode {
        // A 或 B 或 C   r_x[[2,2]].inv().unwrap()
        1 => {
            let gb = r_x[[0, 0]].inv();
            let g = gb.re();
            let b = gb.im();
            // P: V1a*(V1a*g-V2a*(g*cos(t1a-t2a)+b*sin(t1a-t2a)))
            // Q: V1a*(-V1a*b-V2a*(g*sin(t1a-t2a)-b*cos(t1a-t2a)))
            // = -V1a*(V1a*b+V2a*(g*sin(t1a-t2a)-b*cos(t1a-t2a)))
            //P_A
            result.push(
                format!("V_{tn1}_A*(V_{tn1}_A*{g:.4}-V_{tn2}_A*({g:.4}*cos(D_{tn1}_A-D_{tn2}_A)+{b:.4}*sin(D_{tn1}_A-D_{tn2}_A)))",)
            );
            //Q_A
            result.push(
                format!("-V_{tn1}_A*(V_{tn1}_A*{b:.4}+V_{tn2}_A*({g:.4}*sin(D_{tn1}_A-D_{tn2}_A)-{b:.4}*cos(D_{tn1}_A-D_{tn2}_A)))",),
            )
        }
        2 => {
            let gb = r_x[[1, 1]].inv();
            let g = gb.re();
            let b = gb.im();
            //P_B
            result.push(
                format!("V_{tn1}_B*(V_{tn1}_B*{g:.4}-V_{tn2}_B*({g:.4}*cos(D_{tn1}_B-D_{tn2}_B)+{b:.4}*sin(D_{tn1}_B-D_{tn2}_B)))",),
            );
            //Q_B
            result.push(
                format!("-V_{tn1}_B*(V_{tn1}_B*{b:.4}+V_{tn2}_B*({g:.4}*sin(D_{tn1}_B-D_{tn2}_B)-{b:.4}*cos(D_{tn1}_B-D_{tn2}_B)))",),
            )
        }
        4 => {
            let gb = r_x[[2, 2]].inv();
            let g = gb.re();
            let b = gb.im();
            //P_C
            result.push(
                format!("V_{tn1}_C*(V_{tn1}_C*{g:.4}-V_{tn2}_C*({g:.4}*cos(D_{tn1}_C-D_{tn2}_C)+{b:.4}*sin(D_{tn1}_C-D_{tn2}_C)))",),
            );
            //Q_C
            result.push(
                format!("-V_{tn1}_C*(V_{tn1}_C*{b:.4}+V_{tn2}_C*({g:.4}*sin(D_{tn1}_C-D_{tn2}_C)-{b:.4}*cos(D_{tn1}_C-D_{tn2}_C)))",),
            )
        }
        // AB
        3 => {
            let rx = nalgebra::Matrix2::new(
                r_x[[0, 0]], r_x[[0, 1]],
                r_x[[1, 0]], r_x[[1, 1]]);
            let gb = rx.try_inverse().unwrap();
            let (g_aa, b_aa) = (gb.m11.re, gb.m11.im);
            let (g_ab, b_ab) = (gb.m12.re, gb.m12.im);
            let (g_ba, b_ba) = (gb.m21.re, gb.m21.im);
            let (g_bb, b_bb) = (gb.m22.re, gb.m22.im);
            //P_A
            result.push(
              format!(
                  "V_{tn1}_A*V_{tn1}_A*{g_aa:.4}\
                  -V_{tn1}_A*V_{tn2}_A*({g_aa:.4}*cos(D_{tn1}_A-D_{tn2}_A)+{b_aa:.4}*sin(D_{tn1}_A-D_{tn2}_A))\
                  +V_{tn1}_A*(V_{tn1}_B*({g_ab:.4}*cos(D_{tn1}_A-D_{tn1}_B)+{b_ab:.4}*sin(D_{tn1}_A-D_{tn1}_B)))\
                  -V_{tn1}_A*(V_{tn2}_B*({g_ab:.4}*cos(D_{tn1}_A-D_{tn2}_B)+{b_ab:.4}*sin(D_{tn1}_A-D_{tn2}_B))",),
            );
            //Q_A
            result.push(
                format!(
                    "-V_{tn1}_A*V_{tn1}_A*{b_aa:.4}\
                  +V_{tn1}_A*V_{tn2}_A*({b_aa:.4}*cos(D_{tn1}_A-D_{tn2}_A)-{g_aa:.4}*sin(D_{tn1}_A-D_{tn2}_A))\
                  +V_{tn1}_A*(V_{tn1}_B*({g_ab:.4}*sin(D_{tn1}_A-D_{tn1}_B)-{b_ab:.4}*cos(D_{tn1}_A-D_{tn1}_B)))\
                  +V_{tn1}_A*(V_{tn2}_B*({b_ab:.4}*cos(D_{tn1}_A-D_{tn2}_B)-{g_ab:.4}*sin(D_{tn1}_A-D_{tn2}_B))",),
            );
            //P_B
            result.push(
                format!(
                    "V_{tn1}_B*V_{tn1}_B*{g_bb:.4}\
                  -V_{tn1}_B*V_{tn2}_B*({g_bb:.4}*cos(D_{tn1}_B-D_{tn2}_B)+{b_bb:.4}*sin(D_{tn1}_B-D_{tn2}_B))\
                  +V_{tn1}_B*(V_{tn1}_A*({g_ba:.4}*cos(D_{tn1}_B-D_{tn1}_A)+{b_ba:.4}*sin(D_{tn1}_B-D_{tn1}_A)))\
                  -V_{tn1}_B*(V_{tn2}_A*({g_ba:.4}*cos(D_{tn1}_B-D_{tn2}_A)+{b_ba:.4}*sin(D_{tn1}_B-D_{tn2}_A))",),
            );
            //Q_B
            result.push(
                format!(
                    "-V_{tn1}_B*V_{tn1}_B*{b_bb:.4}\
                  +V_{tn1}_B*V_{tn2}_B*({b_bb:.4}*cos(D_{tn1}_B-D_{tn2}_B)-{g_bb:.4}*sin(D_{tn1}_B-D_{tn2}_B))\
                  +V_{tn1}_B*(V_{tn1}_A*({g_ba:.4}*sin(D_{tn1}_B-D_{tn1}_A)-{b_ba:.4}*cos(D_{tn1}_B-D_{tn1}_A)))\
                  +V_{tn1}_B*(V_{tn2}_A*({b_ba:.4}*cos(D_{tn1}_B-D_{tn2}_A)-{g_ba:.4}*sin(D_{tn1}_B-D_{tn2}_A))",),
            );
        }
        // AC
        5 => {
            let rx = nalgebra::Matrix2::new(
                r_x[[0, 0]], r_x[[0, 2]],
                r_x[[2, 0]], r_x[[2, 2]]);
            let gb = rx.try_inverse().unwrap();
            let (g_aa, b_aa) = (gb.m11.re, gb.m11.im);
            let (g_ac, b_ac) = (gb.m12.re, gb.m12.im);
            let (g_ca, b_ca) = (gb.m21.re, gb.m21.im);
            let (g_cc, b_cc) = (gb.m22.re, gb.m22.im);
            //P_A
            result.push(
                format!(
                    "V_{tn1}_A*V_{tn1}_A*{g_aa:.4}\
                  -V_{tn1}_A*V_{tn2}_A*({g_aa:.4}*cos(D_{tn1}_A-D_{tn2}_A)+{b_aa:.4}*sin(D_{tn1}_A-D_{tn2}_A))\
                  +V_{tn1}_A*(V_{tn1}_C*({g_ac:.4}*cos(D_{tn1}_A-D_{tn1}_C)+{b_ac:.4}*sin(D_{tn1}_A-D_{tn1}_C)))\
                  -V_{tn1}_A*(V_{tn2}_C*({g_ac:.4}*cos(D_{tn1}_A-D_{tn2}_C)+{b_ac:.4}*sin(D_{tn1}_A-D_{tn2}_C))",),
            );
            //Q_A
            result.push(
                format!(
                    "-V_{tn1}_A*V_{tn1}_A*{b_aa:.4}\
                  +V_{tn1}_A*V_{tn2}_A*({b_aa:.4}*cos(D_{tn1}_A-D_{tn2}_A)-{g_aa:.4}*sin(D_{tn1}_A-D_{tn2}_A))\
                  +V_{tn1}_A*(V_{tn1}_C*({g_ac:.4}*sin(D_{tn1}_A-D_{tn1}_C)-{b_ac:.4}*cos(D_{tn1}_A-D_{tn1}_C)))\
                  +V_{tn1}_A*(V_{tn2}_C*({b_ac:.4}*cos(D_{tn1}_A-D_{tn2}_C)-{g_ac:.4}*sin(D_{tn1}_A-D_{tn2}_C))",),
            );
            //P_C
            result.push(
                format!(
                    "V_{tn1}_C*V_{tn1}_C*{g_cc:.4}\
                  -V_{tn1}_C*V_{tn2}_C*({g_cc:.4}*cos(D_{tn1}_C-D_{tn2}_C)+{b_cc:.4}*sin(D_{tn1}_C-D_{tn2}_C))\
                  +V_{tn1}_C*(V_{tn1}_A*({g_ca:.4}*cos(D_{tn1}_C-D_{tn1}_A)+{b_ca:.4}*sin(D_{tn1}_C-D_{tn1}_A)))\
                  -V_{tn1}_C*(V_{tn2}_A*({g_ca:.4}*cos(D_{tn1}_C-D_{tn2}_A)+{b_ca:.4}*sin(D_{tn1}_C-D_{tn2}_A))",),
            );
            //Q_C
            result.push(
                format!(
                    "-V_{tn1}_C*V_{tn1}_C*{b_cc:.4}\
                  +V_{tn1}_C*V_{tn2}_C*({b_cc:.4}*cos(D_{tn1}_C-D_{tn2}_C)-{g_cc:.4}*sin(D_{tn1}_C-D_{tn2}_C))\
                  +V_{tn1}_C*(V_{tn1}_A*({g_ca:.4}*sin(D_{tn1}_C-D_{tn1}_A)-{b_ca:.4}*cos(D_{tn1}_C-D_{tn1}_A)))\
                  +V_{tn1}_C*(V_{tn2}_A*({b_ca:.4}*cos(D_{tn1}_C-D_{tn2}_A)-{g_ca:.4}*sin(D_{tn1}_C-D_{tn2}_A))",),
            );
        }
        // BC
        6 => {
            let rx = nalgebra::Matrix2::new(
                r_x[[1, 1]], r_x[[1, 2]],
                r_x[[2, 1]], r_x[[2, 2]]);
            let gb = rx.try_inverse().unwrap();
            let (g_bb, b_bb) = (gb.m11.re, gb.m11.im);
            let (g_bc, b_bc) = (gb.m12.re, gb.m12.im);
            let (g_cb, b_cb) = (gb.m21.re, gb.m21.im);
            let (g_cc, b_cc) = (gb.m22.re, gb.m22.im);
            //P_B
            result.push(
                format!(
                    "V_{tn1}_B*V_{tn1}_B*{g_bb:.4}\
                  -V_{tn1}_B*V_{tn2}_B*({g_bb:.4}*cos(D_{tn1}_B-D_{tn2}_B)+{b_bb:.4}*sin(D_{tn1}_B-D_{tn2}_B))\
                  +V_{tn1}_B*(V_{tn1}_C*({g_bc:.4}*cos(D_{tn1}_B-D_{tn1}_C)+{b_bc:.4}*sin(D_{tn1}_B-D_{tn1}_C)))\
                  -V_{tn1}_B*(V_{tn2}_C*({g_bc:.4}*cos(D_{tn1}_B-D_{tn2}_C)+{b_bc:.4}*sin(D_{tn1}_B-D_{tn2}_C))",),
            );
            //Q_B
            result.push(
                format!(
                    "-V_{tn1}_B*V_{tn1}_B*{b_bb:.4}\
                  +V_{tn1}_B*V_{tn2}_B*({b_bb:.4}*cos(D_{tn1}_B-D_{tn2}_B)-{g_bb:.4}*sin(D_{tn1}_B-D_{tn2}_B))\
                  +V_{tn1}_B*(V_{tn1}_C*({g_bc:.4}*sin(D_{tn1}_B-D_{tn1}_C)-{b_bc:.4}*cos(D_{tn1}_B-D_{tn1}_C)))\
                  +V_{tn1}_B*(V_{tn2}_C*({b_bc:.4}*cos(D_{tn1}_B-D_{tn2}_C)-{g_bc:.4}*sin(D_{tn1}_B-D_{tn2}_C))",),
            );
            //P_C
            result.push(
                format!(
                    "V_{tn1}_C*V_{tn1}_C*{g_cc:.4}\
                  -V_{tn1}_C*V_{tn2}_C*({g_cc:.4}*cos(D_{tn1}_C-D_{tn2}_C)+{b_cc:.4}*sin(D_{tn1}_C-D_{tn2}_C))\
                  +V_{tn1}_C*(V_{tn1}_B*({g_cb:.4}*cos(D_{tn1}_C-D_{tn1}_B)+{b_cb:.4}*sin(D_{tn1}_C-D_{tn1}_B)))\
                  -V_{tn1}_C*(V_{tn2}_B*({g_cb:.4}*cos(D_{tn1}_C-D_{tn2}_B)+{b_cb:.4}*sin(D_{tn1}_C-D_{tn2}_B))",),
            );
            //Q_C
            result.push(
                format!(
                    "-V_{tn1}_C*V_{tn1}_C*{b_cc:.4}\
                  +V_{tn1}_C*V_{tn2}_C*({b_cc:.4}*cos(D_{tn1}_C-D_{tn2}_C)-{g_cc:.4}*sin(D_{tn1}_C-D_{tn2}_C))\
                  +V_{tn1}_C*(V_{tn1}_B*({g_cb:.4}*sin(D_{tn1}_C-D_{tn1}_B)-{b_cb:.4}*cos(D_{tn1}_C-D_{tn1}_B)))\
                  +V_{tn1}_C*(V_{tn2}_B*({b_cb:.4}*cos(D_{tn1}_C-D_{tn2}_B)-{g_cb:.4}*sin(D_{tn1}_C-D_{tn2}_B))",),
            );
        }
        // ABC
        7 => {
            let rx = nalgebra::Matrix3::new(
                r_x[[0, 0]], r_x[[0, 1]], r_x[[0, 2]],
                r_x[[1, 0]], r_x[[1, 1]], r_x[[1, 2]],
                r_x[[2, 0]], r_x[[2, 1]], r_x[[2, 2]]);
            let gb = rx.try_inverse().unwrap();
            let (g_aa, b_aa) = (gb.m11.re, gb.m11.im);
            let (g_ab, b_ab) = (gb.m12.re, gb.m12.im);
            let (g_ac, b_ac) = (gb.m13.re, gb.m13.im);
            let (g_ba, b_ba) = (gb.m21.re, gb.m21.im);
            let (g_bb, b_bb) = (gb.m22.re, gb.m22.im);
            let (g_bc, b_bc) = (gb.m23.re, gb.m23.im);
            let (g_ca, b_ca) = (gb.m31.re, gb.m31.im);
            let (g_cb, b_cb) = (gb.m32.re, gb.m32.im);
            let (g_cc, b_cc) = (gb.m33.re, gb.m33.im);
            //P_A
            result.push(
                format!(
                    "V_{tn1}_A*V_{tn1}_A*{g_aa:.4}\
                  -V_{tn1}_A*V_{tn2}_A*({g_aa:.4}*cos(D_{tn1}_A-D_{tn2}_A)+{b_aa:.4}*sin(D_{tn1}_A-D_{tn2}_A))\
                  +V_{tn1}_A*(V_{tn1}_B*({g_ab:.4}*cos(D_{tn1}_A-D_{tn1}_B)+{b_ab:.4}*sin(D_{tn1}_A-D_{tn1}_B))\
                  -V_{tn1}_A*(V_{tn2}_B*({g_ab:.4}*cos(D_{tn1}_A-D_{tn2}_B)+{b_ab:.4}*sin(D_{tn1}_A-D_{tn2}_B))\
                  +V_{tn1}_A*(V_{tn1}_C*({g_ac:.4}*cos(D_{tn1}_A-D_{tn1}_C)+{b_ac:.4}*sin(D_{tn1}_A-D_{tn1}_C)))\
                  -V_{tn1}_A*(V_{tn2}_C*({g_ac:.4}*cos(D_{tn1}_A-D_{tn2}_C)+{b_ac:.4}*sin(D_{tn1}_A-D_{tn2}_C))",),
            );
            //Q_A
            result.push(
                format!(
                    "-V_{tn1}_A*V_{tn1}_A*{b_aa:.4}\
                  +V_{tn1}_A*V_{tn2}_A*({b_aa:.4}*cos(D_{tn1}_A-D_{tn2}_A)-{g_aa:.4}*sin(D_{tn1}_A-D_{tn2}_A))\
                  +V_{tn1}_A*(V_{tn1}_B*({g_ab:.4}*sin(D_{tn1}_A-D_{tn1}_B)-{b_ab:.4}*cos(D_{tn1}_A-D_{tn1}_B))\
                  +V_{tn1}_A*(V_{tn2}_B*({b_ab:.4}*cos(D_{tn1}_A-D_{tn2}_B)-{g_ab:.4}*sin(D_{tn1}_A-D_{tn2}_B))\
                  +V_{tn1}_A*(V_{tn1}_C*({g_ac:.4}*sin(D_{tn1}_A-D_{tn1}_C)-{b_ac:.4}*cos(D_{tn1}_A-D_{tn1}_C))\
                  +V_{tn1}_A*(V_{tn2}_C*({b_ac:.4}*cos(D_{tn1}_A-D_{tn2}_C)-{g_ac:.4}*sin(D_{tn1}_A-D_{tn2}_C))",),
            );
            //P_B
            result.push(
                format!(
                    "V_{tn1}_B*V_{tn1}_B*{g_bb:.4}\
                  -V_{tn1}_B*V_{tn2}_B*({g_bb:.4}*cos(D_{tn1}_B-D_{tn2}_B)+{b_bb:.4}*sin(D_{tn1}_B-D_{tn2}_B))\
                  +V_{tn1}_B*(V_{tn1}_A*({g_ba:.4}*cos(D_{tn1}_B-D_{tn1}_A)+{b_ba:.4}*sin(D_{tn1}_B-D_{tn1}_A))\
                  -V_{tn1}_B*(V_{tn2}_A*({g_ba:.4}*cos(D_{tn1}_B-D_{tn2}_A)+{b_ba:.4}*sin(D_{tn1}_B-D_{tn2}_A))\
                  +V_{tn1}_B*(V_{tn1}_C*({g_bc:.4}*cos(D_{tn1}_B-D_{tn1}_C)+{b_bc:.4}*sin(D_{tn1}_B-D_{tn1}_C)))\
                  -V_{tn1}_B*(V_{tn2}_C*({g_bc:.4}*cos(D_{tn1}_B-D_{tn2}_C)+{b_bc:.4}*sin(D_{tn1}_B-D_{tn2}_C))",),
            );
            //Q_B
            result.push(
                format!(
                    "-V_{tn1}_B*V_{tn1}_B*{b_bb:.4}\
                  +V_{tn1}_B*V_{tn2}_B*({b_bb:.4}*cos(D_{tn1}_B-D_{tn2}_B)-{g_bb:.4}*sin(D_{tn1}_B-D_{tn2}_B))\
                  +V_{tn1}_B*(V_{tn1}_A*({g_ba:.4}*sin(D_{tn1}_B-D_{tn1}_A)-{b_ba:.4}*cos(D_{tn1}_B-D_{tn1}_A))\
                  +V_{tn1}_B*(V_{tn2}_A*({b_ba:.4}*cos(D_{tn1}_B-D_{tn2}_A)-{g_ba:.4}*sin(D_{tn1}_B-D_{tn2}_A))\
                  +V_{tn1}_B*(V_{tn1}_C*({g_bc:.4}*sin(D_{tn1}_B-D_{tn1}_C)-{b_bc:.4}*cos(D_{tn1}_B-D_{tn1}_C))\
                  +V_{tn1}_B*(V_{tn2}_C*({b_bc:.4}*cos(D_{tn1}_B-D_{tn2}_C)-{g_bc:.4}*sin(D_{tn1}_B-D_{tn2}_C))",),
            );
            //P_C
            result.push(
                format!(
                    "V_{tn1}_C*V_{tn1}_C*{g_cc:.4}\
                  -V_{tn1}_C*V_{tn2}_C*({g_cc:.4}*cos(D_{tn1}_C-D_{tn2}_C)+{b_cc:.4}*sin(D_{tn1}_C-D_{tn2}_C))\
                  +V_{tn1}_C*(V_{tn1}_A*({g_ca:.4}*cos(D_{tn1}_C-D_{tn1}_A)+{b_ca:.4}*sin(D_{tn1}_C-D_{tn1}_A))\
                  -V_{tn1}_C*(V_{tn2}_A*({g_ca:.4}*cos(D_{tn1}_C-D_{tn2}_A)+{b_ca:.4}*sin(D_{tn1}_C-D_{tn2}_A))\
                  +V_{tn1}_C*(V_{tn1}_B*({g_cb:.4}*cos(D_{tn1}_C-D_{tn1}_B)+{b_cb:.4}*sin(D_{tn1}_C-D_{tn1}_B)))\
                  -V_{tn1}_C*(V_{tn2}_B*({g_cb:.4}*cos(D_{tn1}_C-D_{tn2}_B)+{b_cb:.4}*sin(D_{tn1}_C-D_{tn2}_B))",),
            );
            //Q_C
            result.push(
                format!(
                    "-V_{tn1}_C*V_{tn1}_C*{b_cc:.4}\
                  +V_{tn1}_C*V_{tn2}_C*({b_cc:.4}*cos(D_{tn1}_C-D_{tn2}_C)-{g_cc:.4}*sin(D_{tn1}_C-D_{tn2}_C))\
                  +V_{tn1}_C*(V_{tn1}_A*({g_ca:.4}*sin(D_{tn1}_C-D_{tn1}_A)-{b_ca:.4}*cos(D_{tn1}_C-D_{tn1}_A))\
                  +V_{tn1}_C*(V_{tn2}_A*({b_ca:.4}*cos(D_{tn1}_C-D_{tn2}_A)-{g_ca:.4}*sin(D_{tn1}_C-D_{tn2}_A))\
                  +V_{tn1}_C*(V_{tn1}_B*({g_cb:.4}*sin(D_{tn1}_C-D_{tn1}_B)-{b_cb:.4}*cos(D_{tn1}_C-D_{tn1}_B))\
                  +V_{tn1}_C*(V_{tn2}_B*({b_cb:.4}*cos(D_{tn1}_C-D_{tn2}_B)-{g_cb:.4}*sin(D_{tn1}_C-D_{tn2}_B))",),
            );
        }
        _ => { return None; }
    };
    Some((result, mode))
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
        let (p, q) = get_pq_of_acline(arr,1,2).unwrap();
        assert_eq!(p[0],
                   "V_1_A*V_1_A*0.4338\
                   -V_1_A*V_2_A*(0.4338*cos(D_1_A-D_2_A)+-1.2502*sin(D_1_A-D_2_A))\
                   +V_1_A*(V_1_B*(-0.1840*cos(D_1_A-D_1_B)+0.4622*sin(D_1_A-D_1_B))\
                   -V_1_A*(V_2_B*(-0.1840*cos(D_1_A-D_2_B)+0.4622*sin(D_1_A-D_2_B))\
                   +V_1_A*(V_1_C*(-0.1008*cos(D_1_A-D_1_C)+0.3455*sin(D_1_A-D_1_C)))\
                   -V_1_A*(V_2_C*(-0.1008*cos(D_1_A-D_2_C)+0.3455*sin(D_1_A-D_2_C))"
        );
        let arr = array![  [Complex64::new(0.0,0.0), Complex64::new(0.0,0.0), Complex64::new(0.0,0.0)],
                                            [Complex64::new(0.0,0.0), Complex64::new(0.0,0.0), Complex64::new(0.0,0.0)],
                                            [Complex64::new(0.0,0.0), Complex64::new(0.0,0.0), Complex64::new(0.3414,1.0348)]];
        // 0.2875 - 0.8715i
        let (p, q) = get_pq_of_acline(arr,1,2).unwrap();
        assert_eq!(p[0], "V_1_C*(V_1_C*0.2875-V_2_C*(0.2875*cos(D_1_C-D_2_C)+-0.8715*sin(D_1_C-D_2_C)))");
    }
}