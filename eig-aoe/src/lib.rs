use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Iter;
use log::error;

use petgraph::algo;
use petgraph::graphmap::DiGraphMap;

use eig_domain::{Measurement, MeasureValue};
use eig_expr::{Expr, Token};

pub mod aoe;
pub mod solvers;

pub const AOE_RESULT_BUF: usize = 100;

const ERR_SUFFIX: &str = "_err";
const DT_SUFFIX: &str = "_dt";
const DDT_SUFFIX: &str = "_ddt";
const T_SUFFIX: &str = "_t";
const PUB_T_SUFFIX: &str = "_pub_t";
const PUB_V_SUFFIX: &str = "_pub_v";

/// 测点变量取值，可以取值、取偏差、取时间
/// 偏差指的是最新量测和上一次发送到网络量测之间的偏差
#[derive(Debug, Clone)]
pub enum PointVarType {
    /// 取测点值
    Value,
    /// 取测点值偏差(当前采样的值与最近一次发布值之间的偏差）
    Error,
    /// 取测点导数
    Gradient,
    /// 取测点采集时间
    Time,
    /// 取测点时间偏差（当前采集时间和最近一次发布时间之差）
    TimeErr,
    /// 取最近一次发布时间
    PubTime,
    /// 去最近一次发布的测点值
    PubValue,
}

#[derive(Debug, Clone, Default)]
pub struct MeasureBuf {
    /// 别名对应的point id
    pub alias_to_id: HashMap<String, u64>,
    /// 最新的量测值
    pub current_mvs: HashMap<u64, MeasureValue>,
    /// last time updated measurements
    last_mvs: HashMap<u64, MeasureValue>,
    /// 目前的告警状态
    current_alarm: HashMap<u64, u8>,
    /// 上次handle的测点
    last_handled: HashMap<u64, MeasureValue>,
}

#[derive(Debug, Clone)]
pub struct ExprGraph {
    /// 存储测点号
    pub graph: DiGraphMap<u64, u8>,
    /// 存储表达式
    pub exprs: HashMap<u64, Expr>,
    /// 记录每个计算点的层数，同一层的测点互不影响
    pub layer: HashMap<u64, u32>,
    /// 记录每个计算节点所对应的变量名
    pub var_names: HashMap<u64, Vec<String>>,
}


/// 检查计算点中可能存在的环路问题
pub fn check_loop_in_computing_points(
    map: &HashMap<u64, Measurement>,
    alias: &HashMap<String, u64>,
) -> Option<u64> {
    let mut exprs = HashMap::new();
    let mut in_degree: HashMap<u64, u8> = HashMap::with_capacity(map.len());
    let graph = form_graph(map, alias, &mut exprs, &mut in_degree);
    // 拓扑排序
    if let Err(e) = algo::toposort(&graph, None) {
        error!("!!!There is loop in computing points");
        let node_id = e.node_id();
        // 给出错误的计算点
        Some(node_id)
    } else {
        None
    }
}

/// 从变量字符串中获得测点变量的类型和测点号，如果没有找到测点，则测点号为0
fn find_points_in_var(
    mut var_str: String,
    all_alias: &HashMap<String, u64>,
) -> (PointVarType, u64) {
    // 首先判断是要获取测点值or测点偏差（和上次发布相比）or测点时间
    let var_type = if var_str.ends_with(PUB_T_SUFFIX) {
        let len = var_str.len() - PUB_T_SUFFIX.len();
        var_str.truncate(len);
        PointVarType::PubTime
    } else if var_str.ends_with(PUB_V_SUFFIX) {
        let len = var_str.len() - PUB_V_SUFFIX.len();
        var_str.truncate(len);
        PointVarType::PubValue
    } else if var_str.ends_with(DDT_SUFFIX) {
        let len = var_str.len() - DDT_SUFFIX.len();
        var_str.truncate(len);
        PointVarType::Gradient
    } else if var_str.ends_with(ERR_SUFFIX) {
        let len = var_str.len() - ERR_SUFFIX.len();
        var_str.truncate(len);
        PointVarType::Error
    } else if var_str.ends_with(DT_SUFFIX) {
        let len = var_str.len() - DT_SUFFIX.len();
        var_str.truncate(len);
        PointVarType::TimeErr
    } else if var_str.ends_with(T_SUFFIX) {
        let len = var_str.len() - T_SUFFIX.len();
        var_str.truncate(len);
        PointVarType::Time
    } else {
        PointVarType::Value
    };

    // 获取测点号
    let point_id = if let Some(id) = all_alias.get(&var_str) {
        *id
    } else if var_str.starts_with('_') || var_str.starts_with('$') {
        if let Ok(point_id) = var_str.as_str()[1..].parse::<u64>() {
            point_id
        } else {
            0
        }
    } else {
        0
    };
    (var_type, point_id)
}

/// 注意这个方法可能会返回重复的点号
pub fn find_points_in_expr(
    expr: &Expr,
    all_alias: &HashMap<String, u64>,
) -> Vec<(PointVarType, u64, String)> {
    let mut r = Vec::new();
    for token in expr.iter() {
        if let Token::Var(s) = token {
            let (pv_type, point_id) = find_points_in_var(s.clone(), all_alias);
            // 如果找到了测点
            if point_id > 0 {
                r.push((pv_type, point_id, s.clone()));
            }
        }
    }
    r
}

/// 建立测点之间的有向图
pub fn form_graph(
    map: &HashMap<u64, Measurement>,
    alias: &HashMap<String, u64>,
    exprs: &mut HashMap<u64, Expr>,
    in_degree: &mut HashMap<u64, u8>,
) -> DiGraphMap<u64, u8> {
    let mut graph = DiGraphMap::<u64, u8>::with_capacity(map.len(), map.len());
    // 首先构建一个包含全部计算点及其相关测点的图
    for (id, m) in map {
        if m.is_computing_point {
            let expr: Expr = m.expression.parse().unwrap();
            if !graph.contains_node(*id) {
                graph.add_node(*id);
            }
            for (_, point_id, _) in find_points_in_expr(&expr, alias) {
                // 添加节点和边
                if !graph.contains_node(point_id) {
                    graph.add_node(point_id);
                }
                if graph.add_edge(point_id, *id, 1).is_none() {
                    if let Some(degree) = in_degree.get_mut(id) {
                        *degree += 1;
                    } else {
                        in_degree.insert(*id, 1);
                    }
                }
            }
            exprs.insert(*id, expr);
        }
    }
    graph
}

impl MeasureBuf {
    pub fn new(
        current_mvs: HashMap<u64, MeasureValue>,
        alias_to_id: HashMap<String, u64>,
    ) -> MeasureBuf {
        let last_handled = HashMap::with_capacity(current_mvs.len());
        let last_mvs = current_mvs.clone();
        MeasureBuf {
            current_mvs,
            last_mvs,
            current_alarm: Default::default(),
            alias_to_id,
            last_handled,
        }
    }

    pub fn initial_point(&mut self, mvs: HashMap<u64, MeasureValue>, alias: HashMap<String, u64>) {
        self.current_mvs = mvs;
        self.alias_to_id = alias;
        self.last_mvs.clear();
        self.last_handled.clear();
        self.current_mvs.shrink_to_fit();
        self.alias_to_id.shrink_to_fit();
    }

    pub fn copy_sub(&self, ids: &HashSet<u64>, is_copy_alias: bool) -> MeasureBuf {
        let v: Vec<(Option<MeasureValue>, Option<MeasureValue>, Option<MeasureValue>)> = ids.iter().map(|id| {
            let mv = self.current_mvs.get(id).cloned();
            let last_mv = self.last_mvs.get(id).cloned();
            let last_handled = self.last_handled.get(id).cloned();
            (mv, last_mv, last_handled)
        }).collect::<_>();
        let alias_to_id = if is_copy_alias {
            let mut alias_to_id = HashMap::with_capacity(ids.len());
            for (alias, id) in &self.alias_to_id {
                if ids.contains(id) {
                    alias_to_id.insert(alias.clone(), *id);
                }
            }
            alias_to_id.shrink_to_fit();
            alias_to_id
        } else {
            HashMap::with_capacity(0)
        };
        let mut current_mvs = HashMap::with_capacity(ids.len());
        let mut last_mvs = HashMap::with_capacity(ids.len());
        let mut last_handled = HashMap::with_capacity(ids.len());
        for (current_mv, last_mv, last_handle) in v {
            if current_mv.is_some() {
                let mv = current_mv.unwrap();
                current_mvs.insert(mv.point_id, mv);
            }
            if last_mv.is_some() {
                let mv = last_mv.unwrap();
                last_mvs.insert(mv.point_id, mv);
            }
            if last_handle.is_some() {
                let mv = last_handle.unwrap();
                last_handled.insert(mv.point_id, mv);
            }
        }
        MeasureBuf {
            alias_to_id,
            current_mvs,
            last_mvs,
            last_handled,
            current_alarm: Default::default(),
        }
    }

    pub fn contains_point(&self, point_id: &u64) -> bool {
        self.current_mvs.contains_key(point_id)
    }

    // pub fn update_mv(&mut self, new_m: &MeasureValue) {
    //     let point_id = new_m.point_id;
    //     if self.contains_point(&point_id) {
    //         let cloned_last = self.get_mut(&point_id).clone();
    //         self.update_last_handled(cloned_last.clone());
    //         self.update_last_mv(cloned_last);
    //         self.get_mut(&point_id).update(new_m);
    //     }
    // }

    // pub fn update_mvs(&mut self, v: &[MeasureValue]) {
    //     for m in v {
    //         // 在update_mv方法中已经判断了是否存在该测点，因此这里不需要再判断一次
    //         self.update_mv(m);
    //     }
    // }

    pub fn update_buf(&mut self, buf: &MeasureBuf) {
        for (id, mv) in &buf.current_mvs {
            if let Some(m) = self.current_mvs.get_mut(id) {
                m.update(mv);
            }
        }
        for (id, mv) in &buf.last_mvs {
            if !self.contains_point(id) {
                continue
            }
            if let Some(m) = self.last_mvs.get_mut(id) {
                m.update(mv);
            } else {
                self.last_mvs.insert(*id, mv.clone());
            }
        }
        for (id, mv) in &buf.last_handled {
            if !self.contains_point(id) {
                continue
            }
            if let Some(m) = self.last_handled.get_mut(id) {
                m.update(mv);
            } else {
                self.last_handled.insert(*id, mv.clone());
            }
        }
    }

    pub fn get_mut(&mut self, point_id: &u64) -> &mut MeasureValue {
        self.current_mvs.get_mut(point_id).unwrap()
    }

    pub fn get_mv(&self, point_id: &u64) -> Option<&MeasureValue> {
        self.current_mvs.get(point_id)
    }

    pub fn get_mv_count(&self) -> usize {
        self.current_mvs.len()
    }

    pub fn get_mvs(&self) -> Iter<'_, u64, MeasureValue> {
        self.current_mvs.iter()
    }

    pub fn get_alarm_status(&self, point_id: &u64) -> u8 {
        if let Some(status) = self.current_alarm.get(point_id) {
            *status
        } else {
            0
        }
    }

    pub fn get_last_handled(&self, point_id: &u64) -> Option<&MeasureValue> {
        self.last_handled.get(point_id)
    }

    pub fn get_last_updated(&self, point_id: &u64) -> Option<&MeasureValue> {
        self.last_mvs.get(point_id)
    }

    pub fn update_last_handled(&mut self, m: MeasureValue) {
        self.last_handled.insert(m.point_id, m);
    }

    pub fn update_last_mv(&mut self, m: MeasureValue) {
        self.last_mvs.insert(m.point_id, m);
    }

    pub fn update_alarm_status(&mut self, point_id: u64, status: u8) {
        self.current_alarm.insert(point_id, status);
    }

    pub fn get_bool_measure(&self, point_id: &u64) -> bool {
        let status = self.get_mv(point_id);
        status.is_some() && status.unwrap().discrete_value > 0
    }
}