use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use std::str::FromStr;
use std::time::Duration;

use async_channel::Receiver;
use async_channel::RecvError;
use async_channel::Sender;
use async_trait::async_trait;
use ndarray::{Array, IxDyn};
use petgraph::prelude::*;
use serde::{Deserialize, Serialize};

use eig_domain::{SetFloatValue, SetIntValue};
use eig_expr::{Expr, MyF};

use crate::{ExprGraph, MeasureBuf};

const AOE_MEAS_BUF_NUM: usize = 100;
#[async_trait]
pub trait Action: Sync {
    /// 执行
    async fn do_action(&self, aoe: &Aoe) -> ActionResult;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ActionExeResult {
    NotRun,
    Success,
    Failed(u32),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EventEvalResult {
    Happen,
    NotHappen,
    Canceled,
    Error,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActionResult {
    pub start_time: u64,
    pub end_time: u64,
    pub yk_ids: Option<Vec<(u64, i64)>>,
    pub yt_ids: Option<Vec<(u64, f64)>>,
    pub num_result: HashMap<String, f64>,
    #[serde(skip)]
    pub tensor_result: HashMap<String, Array<f64, IxDyn>>,
    pub final_result: ActionExeResult,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventResult {
    pub start_time: u64,
    pub end_time: u64,
    pub final_result: EventEvalResult,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AoeResult {
    pub aoe_id: u64,
    pub start_time: u64,
    pub end_time: u64,
    pub action_result: Vec<(u64, u64, ActionResult)>,
    pub event_result: Vec<(u64, EventResult)>,
}

pub enum OuterMsg {
    Cancel,
    VarOrMeasure(VarOrMeasures),
    QueueError(RecvError),
}

/**
 * @api {时间对象} /Duration Duration
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} secs 秒
 * @apiSuccess {u32} nanos 纳秒
 */
/**
 * @api {枚举_启动方式} /TriggerType TriggerType
 * @apiPrivate
 * @apiGroup A_Enum
 * @apiSuccess {Object} SimpleRepeat 简单固定周期触发，{"SimpleRepeat": Duration}
 * @apiSuccess {Object} TimeDrive cron expression，{"TimeDrive": String}
 * @apiSuccess {String} EventDrive 事件驱动，AOE开始节点条件满足即触发
 * @apiSuccess {Object} EventRepeatMix 事件驱动 && Simple drive，{"EventRepeatMix": Duration}
 * @apiSuccess {Object} EventTimeMix 事件驱动 && Time drive，{"EventTimeMix": String}
 */
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum TriggerType {
    // 简单固定周期触发
    SimpleRepeat(Duration),
    // cron expression
    TimeDrive(String),
    // 事件驱动，AOE开始节点条件满足即触发
    EventDrive,
    // 事件驱动 && Simple drive
    EventRepeatMix(Duration),
    // 事件驱动 && Time drive
    EventTimeMix(String),
}

impl Display for TriggerType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/**
 * @api {枚举_失败模式} /FailureMode FailureMode
 * @apiPrivate
 * @apiGroup A_Enum
 * @apiSuccess {String} Default 如果存在指向该节点的动作运行成功(可以理解为有路径到达该事件),则后续动作继续进行
 * @apiSuccess {String} Ignore 忽略，不影响其他action
 * @apiSuccess {String} StopAll 停止整个aoe
 * @apiSuccess {String} StopFailed 只停止受影响的节点
 */
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum FailureMode {
    // 如果存在指向该节点的动作运行成功(可以理解为有路径到达该事件),则后续动作继续进行
    Default,
    // 忽略，不影响其他action
    Ignore,
    // 停止整个aoe
    StopAll,
    // 只停止受影响的节点
    StopFailed,
}

impl Display for FailureMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/**
 * @api {枚举_节点类型} /NodeType NodeType
 * @apiPrivate
 * @apiGroup A_Enum
 * @apiSuccess {String} ConditionNode 带表达式的节点，表达式结果>0说明事件发生，进入后续事件
 * @apiSuccess {String} SwitchNode 带表达式的节点，表达式结果>0进入第一条支路，否则进入第二条支路
 * @apiSuccess {String} SwitchOfActionResult 不带表达式的节点，前序Action运行成功进入第一条支路，否则进入第二条支路
 */
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum NodeType {
    // 带表达式的节点，表达式结果>0说明事件发生，进入后续事件
    ConditionNode,
    // 带表达式的节点，表达式结果>0进入第一条支路，否则进入第二条支路
    SwitchNode,
    // 不带表达式的节点，前序Action运行成功进入第一条支路，否则进入第二条支路
    SwitchOfActionResult,
}

impl Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/**
 * @api {EventNode} /EventNode EventNode
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} id 节点id
 * @apiSuccess {u64} aoe_id AOE_id
 * @apiSuccess {String} name 节点名
 * @apiSuccess {NodeType} node_type 节点类型
 * @apiSuccess {Expr} expr 表达式
 * @apiSuccess {u64} timeout 事件还未发生的等待超时时间
 */
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct EventNode {
    pub id: u64,
    pub aoe_id: u64,
    pub name: String,
    pub node_type: NodeType,
    pub expr: Expr,
    /// 事件还未发生的等待超时时间
    pub timeout: u64,
}

/**
 * @api {ActionEdge} /ActionEdge ActionEdge
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} aoe_id AOE_id
 * @apiSuccess {String} name 节点名
 * @apiSuccess {u64} source_node 源节点
 * @apiSuccess {u64} target_node 目标节点
 * @apiSuccess {FailureMode} failure_mode action失败时的处理方式
 * @apiSuccess {EigAction} action 动作定义
 */
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ActionEdge {
    pub aoe_id: u64,
    pub name: String,
    pub source_node: u64,
    pub target_node: u64,
    /// action失败时的处理方式
    pub failure_mode: FailureMode,
    pub action: EigAction,
}

/**
 * @api {枚举_动作} /EigAction EigAction
 * @apiPrivate
 * @apiGroup A_Enum
 * @apiSuccess {String} None 无动作
 * @apiSuccess {Object} SetPoints 设点动作，{"SetPoints": SetPoints}
 * @apiSuccess {Object} SetPointsWithCheck 设点动作，{"SetPointsWithCheck": SetPoints}
 * @apiSuccess {Object} SetPoints2 设点动作，{"SetPoints2": SetPoints2}
 * @apiSuccess {Object} SetPointsWithCheck2 设点动作，{"SetPointsWithCheck2": SetPoints2}
 * @apiSuccess {Object} Solve 求方程，{"Solve": SparseSolver}
 * @apiSuccess {Object} Nlsolve Nlsolve，{"Nlsolve": NewtonSolver}
 * @apiSuccess {Object} Milp 混合整数线性规划稀疏表示，{"Milp": SparseMILP}
 * @apiSuccess {Object} SimpleMilp 混合整数线性规划稠密表示，{"SimpleMilp": MILP}
 * @apiSuccess {Object} Nlp 非整数线性规划，{"Nlp": NLP}
 * @apiSuccess {Object} Url 调用webservice获取EigAction并执行，{"Url": String}
 */
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum EigAction {
    /// 无动作
    None,
    /// 设点动作
    SetPoints(SetPoints),
    /// 设点动作
    SetPointsWithCheck(SetPoints),
    /// 设点动作
    SetPoints2(SetPoints2),
    /// 设点动作
    SetPointsWithCheck2(SetPoints2),
    /// 求方程
    Solve(crate::solvers::SparseSolver),
    /// Nlsolve
    Nlsolve(crate::solvers::NewtonSolver),
    /// 混合整数线性规划稀疏表示
    Milp(crate::solvers::SparseMILP),
    /// 混合整数线性规划稠密表示
    SimpleMilp(crate::solvers::MILP),
    /// 非整数线性规划
    Nlp(crate::solvers::NLP),
    /// 调用webservice获取EigAction并执行
    Url(String),
}

/**
 * @api {SetPoints} /SetPoints SetPoints
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {String[]} discrete_id discrete_id
 * @apiSuccess {Expr[]} discrete_v discrete_v
 * @apiSuccess {String[]} analog_id analog_id
 * @apiSuccess {Expr[]} analog_v analog_v
 */
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct SetPoints {
    pub discrete_id: Vec<String>,
    pub discrete_v: Vec<Expr>,
    pub analog_id: Vec<String>,
    pub analog_v: Vec<Expr>,
}

/**
 * @api {PointsToExp} /PointsToExp PointsToExp
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {String[]} ids id列表
 * @apiSuccess {Expr} expr 表达式
 */
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct PointsToExp {
    pub ids: Vec<String>,
    pub expr: Expr,
}

/**
 * @api {SetPoints2} /SetPoints2 SetPoints2
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {PointsToExp[]} discretes discretes
 * @apiSuccess {PointsToExp[]} analogs analogs
 */
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct SetPoints2 {
    pub discretes: Vec<PointsToExp>,
    pub analogs: Vec<PointsToExp>,
}

#[derive(Debug, Clone)]
pub enum VarOrMeasures {
    Vars(Vec<(String, MyF)>),
    Measures(MeasureBuf),
}

/**
 * @api {AoeModel} /AoeModel AoeModel
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} id AOE_id
 * @apiSuccess {String} name AOE名
 * @apiSuccess {EventNode[]} events 节点
 * @apiSuccess {ActionEdge[]} actions 边
 * @apiSuccess {TriggerType} trigger_type 启动的方式
 * @apiSuccess {tuple[]} variables 用户自定义的变量，这些变量不在计算点的范围，tuple格式为(变量名:String, 变量表达式:Expr)
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AoeModel {
    /// aoe id
    pub id: u64,
    /// aoe name
    pub name: String,
    /// 节点
    pub events: Vec<EventNode>,
    /// 边
    pub actions: Vec<ActionEdge>,
    /// aoe启动的方式
    pub trigger_type: TriggerType,
    /// 用户自定义的变量，这些变量不在计算点的范围
    pub variables: Vec<(String, Expr)>,
}

impl Default for AoeModel {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::default(),
            events: vec![],
            actions: vec![],
            trigger_type: TriggerType::EventDrive,
            variables: vec![],
        }
    }
}

impl PartialEq for AoeModel {
    fn eq(&self, other: &Self) -> bool {
        let b = self.id.eq(&other.id)
            && self.name.eq(&other.name)
            && self.trigger_type.eq(&other.trigger_type)
            && self.variables.eq(&other.variables)
            && self.events.len() == other.events.len()
            && self.actions.len() == other.actions.len();
        if b {
            for i in 0..self.events.len() {
                if self.events[i] != other.events[i] {
                    return false;
                }
            }
            for i in 0..self.actions.len() {
                if self.actions[i] != other.actions[i] {
                    return false;
                }
            }
        }
        b
    }
}

#[derive(Debug, Clone)]
pub struct Aoe {
    /// 接收更新的测点值
    pub(crate) measure_rx: Receiver<VarOrMeasures>,
    /// 提供给外部的发送端
    measure_tx: Sender<VarOrMeasures>,
    /// 接收取消指令
    pub(crate) cancel_rx: Receiver<()>,
    /// 提供给外部的控制端
    cancel_tx: Sender<()>,
    /// 发送控点指令的队列
    control_rx: Receiver<(Vec<SetIntValue>, Vec<SetFloatValue>)>,
    pub control_tx: Sender<(Vec<SetIntValue>, Vec<SetFloatValue>)>,
    pub model: AoeModel,
    /// 存储Aoe网络, 节点表示事件，边表示action实体的id
    pub(crate) graph: DiGraph<usize, (usize, usize)>,
    /// 拓扑排序的结果
    pub(crate) toposort_nodes: Vec<u64>,
    /// 存储node的map
    pub(crate) nodes: HashMap<u64, NodeIndex>,
    /// 存储aoe对应的测点的值
    pub measure_buf: MeasureBuf,
    /// 上下文，用于存储变量计算结果
    pub context: HashMap<String, MyF>,
    // 用户自定义变量的编号，其中包括上面对应公式的，也包括计算产生的
    // 对应公式的index从1开始，方便找到它在variables中的位置
    // 要求用户0-1000不能用于测点号，否则会有问题
    pub(crate) var_index: HashMap<String, u64>,
    // 上面定义变量相互之间的关系
    pub(crate) expr_graph: Option<ExprGraph>,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use async_channel::{bounded, Receiver, Sender};
    use log::debug;

    use eig_domain::*;
    use eig_domain::utils::get_timestamp;
    use crate::aoe::*;

    use crate::aoe::dispatcher::Dispatcher;
    use crate::aoe::parse::{from_file2, initial_aoe_points};

    #[tokio::test]
    async fn test_fire_alarm_aoe() {
        // let log_env = env_logger::Env::default().default_filter_or("trace");
        // env_logger::Builder::from_env(log_env).init();

        let name = "fire alarm".to_string();
        let mut aoe = Aoe::new(1, name, TriggerType::EventDrive);
        let start = EventNode {
            id: 1,
            aoe_id: 1,
            name: "火警信号".to_string(),
            node_type: NodeType::ConditionNode,
            expr: "FIRE_ALARM > 0".parse().unwrap(),
            timeout: 0,
        };
        let end = EventNode {
            id: 2,
            aoe_id: 1,
            name: "结束".to_string(),
            node_type: NodeType::ConditionNode,
            expr: "PCS_STOP == 1 && PCS_P_1 < 1e-4 && PCS_P_2 < 1e-4 && PCS_P_3 < 1e-4 "
                .parse()
                .unwrap(),
            timeout: 5000,
        };
        aoe.model.events.push(start);
        aoe.model.events.push(end);
        let set_points = SetPoints {
            discrete_id: vec!["2".to_string()],
            discrete_v: vec!["1".parse().unwrap()],
            analog_id: vec!["3".to_string(), "4".to_string(), "5".to_string()],
            analog_v: vec![
                "0.0".parse().unwrap(),
                "0.0".parse().unwrap(),
                "0.0".parse().unwrap(),
            ],
        };
        aoe.model.actions.push(ActionEdge {
            aoe_id: 1,
            name: "shut down system".to_string(),
            source_node: 1,
            target_node: 2,
            failure_mode: FailureMode::Default,
            action: EigAction::SetPoints(set_points),
        });
        let mut current_mvs: HashMap<u64, Measurement> = HashMap::new();
        let mut alias_to_id: HashMap<String, u64> = HashMap::new();
        current_mvs.insert(1, init_discrete_point(1, 1));
        current_mvs.insert(2, init_discrete_point(2, 0));
        current_mvs.insert(3, init_analog_point(3, 10.0));
        current_mvs.insert(4, init_analog_point(4, 10.0));
        current_mvs.insert(5, init_analog_point(5, 10.0));
        current_mvs.get_mut(&1).unwrap().alias_id = "FIRE_ALARM".to_string();
        current_mvs.get_mut(&2).unwrap().alias_id = "PCS_STOP".to_string();
        current_mvs.get_mut(&3).unwrap().alias_id = "PCS_P_1".to_string();
        current_mvs.get_mut(&4).unwrap().alias_id = "PCS_P_2".to_string();
        current_mvs.get_mut(&5).unwrap().alias_id = "PCS_P_3".to_string();
        for (id, m) in &current_mvs {
            alias_to_id.insert(m.alias_id.clone(), *id);
        }
        // 结束构建AOE
        let check_result = aoe.finish_and_check();
        // 初始化测量点
        aoe.initial_points(&current_mvs, &alias_to_id);
        assert_eq!(check_result, None);
        // 监听AOE发出来的命令
        start_measure_loop(aoe.control_receiver(), aoe.measure_sender());
        // 启动AOE
        assert!(aoe.should_start());
        aoe.start().await;
        let m = aoe.measure_buf.get_mv(&2);
        assert!(m.is_some());
        assert_eq!(1, m.unwrap().discrete_value);
        let m = aoe.measure_buf.get_mv(&3);
        assert!(m.is_some());
        assert_eq!(0.0, m.unwrap().analog_value);
        let m = aoe.measure_buf.get_mv(&4);
        assert!(m.is_some());
        assert_eq!(0.0, m.unwrap().analog_value);
        let m = aoe.measure_buf.get_mv(&5);
        assert!(m.is_some());
        assert_eq!(0.0, m.unwrap().analog_value);
    }

    #[tokio::test]
    async fn test_aoe_file1() {
        // let log_env = env_logger::Env::default().default_filter_or("trace");
        // env_logger::Builder::from_env(log_env).init();

        let r = from_file2("tests/test_zq/aoe-test1.csv", false);
        assert!(r.is_ok());
        let (points, _) = from_csv2("tests/test_zq/points-zq.csv", false).unwrap();
        let mut aoes = r.unwrap();
        for aoe in &mut aoes {
            let r = initial_aoe_points(aoe, &points);
            assert_eq!(None, r);
            // 监听AOE发出来的命令
            start_measure_loop(aoe.control_receiver(), aoe.measure_sender());
        }
        let mut dispatcher = Dispatcher::new();
        let rx = dispatcher.result_receiver();
        let job = tokio::spawn(async move {
            let mut count = 0;
            loop {
                match rx.recv().await {
                    Ok(r) => {
                        println!("aoe finish: {}", r.aoe_id);
                        if r.aoe_id == 1 || r.aoe_id == 2 {
                            assert_eq!(2, r.event_result.len());
                            assert_eq!(1, r.action_result.len());
                        }
                        if r.aoe_id == 3 {
                            assert_eq!(3, r.event_result.len());
                        }
                        count += 1;
                    }
                    Err(_) => {
                        break;
                    }
                }
                if count == 4 {
                    break;
                }
            }
        });
        dispatcher.schedule(aoes);
        job.await.unwrap();
        dispatcher.shutdown().await;
        // tokio::time::sleep(Duration::from_secs(5)).await;
    }

    #[tokio::test]
    async fn test_aoe_file_heater() {
        // let log_env = env_logger::Env::default().default_filter_or("trace");
        // env_logger::Builder::from_env(log_env).init();

        let r = from_file2("tests/test_househeat/aoe-househeat_test.csv", false);
        println!("Aoe parse result : {:?}", r);
        assert!(r.is_ok());
        let (points, _) = from_csv2("tests/test_househeat/points-aoe-househeat.csv", false).unwrap();
        let mut aoes = r.unwrap();
        for aoe in &mut aoes {
            let r = initial_aoe_points(aoe, &points);
            assert_eq!(None, r);
            // 监听AOE发出来的命令
            start_measure_loop(aoe.control_receiver(), aoe.measure_sender());
        }
        let mut dispatcher = Dispatcher::new();
        let rx = dispatcher.result_receiver();
        let job = tokio::spawn(async move {
            let mut count = 0;
            loop {
                match rx.recv().await {
                    Ok(r) => {
                        println!("aoe finish: {}", r.aoe_id);
                        assert_eq!(4, r.event_result.len());
                        assert_eq!(3, r.action_result.len());
                        count += 1;
                    }
                    Err(_) => {
                        break;
                    }
                }
                if count == 4 {
                    break;
                }
            }
        });
        dispatcher.schedule(aoes);
        job.await.unwrap();
        dispatcher.shutdown().await;
    }

    #[tokio::test]
    async fn test_aoe_get_time() {
        // let log_env = env_logger::Env::default().default_filter_or("trace");
        // env_logger::Builder::from_env(log_env).init();
        do_aoe_get_time("tests/test_time/aoe-time-test.csv").await;
    }

    #[tokio::test]
    async fn test_aoe_get_time2() {
        do_aoe_get_time("tests/test_time/aoe-time-test2.csv").await;
    }

    async fn do_aoe_get_time(aoe_file: &str) {
        let r = from_file2(aoe_file, false);
        println!("Aoe parse result : {:?}", r);
        assert!(r.is_ok());
        let (points, _) = from_csv2("tests/test_time/points-timetest.csv", false).unwrap();
        // 设为离散点
        assert!(!points.get(&400133025).unwrap().is_discrete);
        assert!(!points.get(&400133026).unwrap().is_discrete);

        let mut alias: HashMap<String, u64> = HashMap::with_capacity(points.len());
        // 先形成别名的map
        for m in points.values() {
            if !m.alias_id.is_empty() {
                let key = m.alias_id.clone();
                alias.insert(key, m.point_id);
            }
        }

        let mut aoes = r.unwrap();
        let set_measure_tx = aoes[0].measure_sender();
        for aoe in &mut aoes {
            let r = initial_aoe_points(aoe, &points);
            assert_eq!(None, r);
            // 监听AOE发出来的命令
            start_measure_loop(aoe.control_receiver(), aoe.measure_sender());
        }

        let mut dispatcher = Dispatcher::new();
        let rx = dispatcher.result_receiver();
        let job = tokio::spawn(async move {
            let mut count = 0;
            loop {
                match rx.recv().await {
                    Ok(r) => {
                        println!("aoe finish: {}", r.aoe_id);
                        assert_eq!(2, r.event_result.len());
                        assert_eq!(1, r.action_result.len());
                        count += 1;
                    }
                    Err(_) => {
                        break;
                    }
                }
                if count == 1 {
                    break;
                }
            }
        });
        dispatcher.schedule(aoes);

        // 设置初始值
        set_points("", "pcs_304:18;pcs_305:47", &alias, &set_measure_tx).await;

        job.await.unwrap();
        dispatcher.shutdown().await;
    }

    #[cfg(feature = "milp")]
    #[tokio::test]
    async fn test_aoe_file2() {
        use log::debug;

        let log_env = env_logger::Env::default().default_filter_or("trace");
        env_logger::Builder::from_env(log_env).init();

        // 8 PCS start
        let agc_mode1 = "EMS_MODE_AGC_POINT:1";
        let bms_status_str1 = "BAMS1_WORK_STATUS_POINT:1;BAMS2_WORK_STATUS_POINT:1;BAMS3_WORK_STATUS_POINT:1;BAMS4_WORK_STATUS_POINT:1;BAMS5_WORK_STATUS_POINT:1;BAMS6_WORK_STATUS_POINT:1;BAMS7_WORK_STATUS_POINT:1;BAMS8_WORK_STATUS_POINT:1";

        // Cross-sectional data set 1 (RESULT: Discharge strategy 1)
        let mileage_str1 = "BAMS1_MILEAGE_TOTAL_POINT:0;BAMS2_MILEAGE_TOTAL_POINT:0;BAMS3_MILEAGE_TOTAL_POINT:0;BAMS4_MILEAGE_TOTAL_POINT:0;BAMS5_MILEAGE_TOTAL_POINT:0;BAMS6_MILEAGE_TOTAL_POINT:0;BAMS7_MILEAGE_TOTAL_POINT:0;BAMS8_MILEAGE_TOTAL_POINT:0";
        let rated_mileage_str1 = "BAMS1_RATEDMILEAGE:1000;BAMS2_RATEDMILEAGE:1000;BAMS3_RATEDMILEAGE:1000;BAMS4_RATEDMILEAGE:1000;BAMS5_RATEDMILEAGE:1000;BAMS6_RATEDMILEAGE:1000;BAMS7_RATEDMILEAGE:1000;BAMS8_RATEDMILEAGE:1000";
        let bms_capacity_str1 = "BAMS1_CAPACITY:1000;BAMS2_CAPACITY:1000;BAMS3_CAPACITY:1000;BAMS4_CAPACITY:1000;BAMS5_CAPACITY:1000;BAMS6_CAPACITY:1000;BAMS7_CAPACITY:1000;BAMS8_CAPACITY:1000";
        let max_charging_power_str1 = "BAMS1_CMAX_POINT:1500;BAMS2_CMAX_POINT:1500;BAMS3_CMAX_POINT:1500;BAMS4_CMAX_POINT:1500;BAMS5_CMAX_POINT:1500;BAMS6_CMAX_POINT:1500;BAMS7_CMAX_POINT:1500;BAMS8_CMAX_POINT:1500";
        let max_discharging_power_str1 = "BAMS1_DMAX_POINT:1500;BAMS2_DMAX_POINT:1500;BAMS3_DMAX_POINT:1500;BAMS4_DMAX_POINT:1500;BAMS5_DMAX_POINT:1500;BAMS6_DMAX_POINT:1500;BAMS7_DMAX_POINT:1500;BAMS8_DMAX_POINT:1500";
        let soc_str1 = "BAMS1_SOC_POINT:70;BAMS2_SOC_POINT:70;BAMS3_SOC_POINT:60;BAMS4_SOC_POINT:70;BAMS5_SOC_POINT:70;BAMS6_SOC_POINT:20;BAMS7_SOC_POINT:70;BAMS8_SOC_POINT:70";
        let bms_p1 = "BAMS1_P_POINT:0;BAMS2_P_POINT:0;BAMS3_P_POINT:0;BAMS4_P_POINT:0;BAMS5_P_POINT:0;BAMS6_P_POINT:0;BAMS7_P_POINT:0;BAMS8_P_POINT:0";
        let total_power_str1 = "GEN_TOTAL_P_POINT:2400";
        let dstr1 = bms_status_str1.to_string() + ";" + agc_mode1;
        let astr1 = soc_str1.to_string()
            + ";"
            + mileage_str1
            + ";"
            + rated_mileage_str1
            + ";"
            + bms_capacity_str1
            + ";"
            + max_charging_power_str1
            + ";"
            + max_discharging_power_str1
            + ";"
            + total_power_str1
            + ";"
            + bms_p1;

        // Cross-sectional data set 2 (RESULT: Charge strategy 1)
        let soc_str2 = "BAMS1_SOC_POINT:30;BAMS2_SOC_POINT:30;BAMS3_SOC_POINT:30;BAMS4_SOC_POINT:20;BAMS5_SOC_POINT:30;BAMS6_SOC_POINT:30;BAMS7_SOC_POINT:30;BAMS8_SOC_POINT:70";
        let total_power_str2 = "GEN_TOTAL_P_POINT:-3000";
        let dstr2 = bms_status_str1.to_string() + ";" + agc_mode1;
        let astr2 = soc_str2.to_string()
            + ";"
            + mileage_str1
            + ";"
            + rated_mileage_str1
            + ";"
            + bms_capacity_str1
            + ";"
            + max_charging_power_str1
            + ";"
            + max_discharging_power_str1
            + ";"
            + total_power_str2
            + ";"
            + bms_p1;

        // Cross-sectional data set 3 (RESULT: Discharge strategy 2)
        let soc_str3 = "BAMS1_SOC_POINT:50;BAMS2_SOC_POINT:70;BAMS3_SOC_POINT:60;BAMS4_SOC_POINT:50;BAMS5_SOC_POINT:50;BAMS6_SOC_POINT:50;BAMS7_SOC_POINT:50;BAMS8_SOC_POINT:50";
        let total_power_str3 = "GEN_TOTAL_P_POINT:4000";
        let dstr3 = bms_status_str1.to_string() + ";" + agc_mode1;
        let astr3 = soc_str3.to_string()
            + ";"
            + mileage_str1
            + ";"
            + rated_mileage_str1
            + ";"
            + bms_capacity_str1
            + ";"
            + max_charging_power_str1
            + ";"
            + max_discharging_power_str1
            + ";"
            + total_power_str3
            + ";"
            + bms_p1;

        // Cross-sectional data set 4 (RESULT: Charge strategy 2)
        let soc_str4 = "BAMS1_SOC_POINT:30;BAMS2_SOC_POINT:40;BAMS3_SOC_POINT:60;BAMS4_SOC_POINT:70;BAMS5_SOC_POINT:80;BAMS6_SOC_POINT:70;BAMS7_SOC_POINT:70;BAMS8_SOC_POINT:70";
        let total_power_str4 = "GEN_TOTAL_P_POINT:-4000";
        let dstr4 = bms_status_str1.to_string() + ";" + agc_mode1;
        let astr4 = soc_str4.to_string()
            + ";"
            + mileage_str1
            + ";"
            + rated_mileage_str1
            + ";"
            + bms_capacity_str1
            + ";"
            + max_charging_power_str1
            + ";"
            + max_discharging_power_str1
            + ";"
            + total_power_str4
            + ";"
            + bms_p1;

        // Cross-sectional data set 5 (RESULT: Discharge optimal strategy)
        let soc_str5 = "BAMS1_SOC_POINT:60;BAMS2_SOC_POINT:70;BAMS3_SOC_POINT:40;BAMS4_SOC_POINT:40;BAMS5_SOC_POINT:30;BAMS6_SOC_POINT:30;BAMS7_SOC_POINT:30;BAMS8_SOC_POINT:30";
        let total_power_str5 = "GEN_TOTAL_P_POINT:3500";
        let dstr5 = bms_status_str1.to_string() + ";" + agc_mode1;
        let astr5 = soc_str5.to_string()
            + ";"
            + mileage_str1
            + ";"
            + rated_mileage_str1
            + ";"
            + bms_capacity_str1
            + ";"
            + max_charging_power_str1
            + ";"
            + max_discharging_power_str1
            + ";"
            + total_power_str5
            + ";"
            + bms_p1;

        // Cross-sectional data set 6 (RESULT: Charge optimal strategy)
        let soc_str6 = "BAMS1_SOC_POINT:30;BAMS2_SOC_POINT:40;BAMS3_SOC_POINT:60;BAMS4_SOC_POINT:60;BAMS5_SOC_POINT:70;BAMS6_SOC_POINT:60;BAMS7_SOC_POINT:60;BAMS8_SOC_POINT:60";
        let total_power_str6 = "GEN_TOTAL_P_POINT:-3500";
        let dstr6 = bms_status_str1.to_string() + ";" + agc_mode1;
        let astr6 = soc_str6.to_string()
            + ";"
            + mileage_str1
            + ";"
            + rated_mileage_str1
            + ";"
            + bms_capacity_str1
            + ";"
            + max_charging_power_str1
            + ";"
            + max_discharging_power_str1
            + ";"
            + total_power_str6
            + ";"
            + bms_p1;

        // Cross-sectional data set 7 (RESULT: Discharge strategy 2)
        let soc_str7 = "BAMS1_SOC_POINT:80;BAMS2_SOC_POINT:70;BAMS3_SOC_POINT:50;BAMS4_SOC_POINT:20;BAMS5_SOC_POINT:20;BAMS6_SOC_POINT:20;BAMS7_SOC_POINT:50;BAMS8_SOC_POINT:20";
        let total_power_str7 = "GEN_TOTAL_P_POINT:8500";
        let dstr7 = bms_status_str1.to_string() + ";" + agc_mode1;
        let astr7 = soc_str7.to_string()
            + ";"
            + mileage_str1
            + ";"
            + rated_mileage_str1
            + ";"
            + bms_capacity_str1
            + ";"
            + max_charging_power_str1
            + ";"
            + max_discharging_power_str1
            + ";"
            + total_power_str7
            + ";"
            + bms_p1;

        let r = from_file2("tests/test_zq_dispatch/aoe-zq-power-dispatch.csv", false);
        if let Err(rc) = r {
            println!("parse aoe failed: {:?}", rc);
        }
        assert!(r.is_ok());
        let (points, _) = from_csv2("tests/test_zq_dispatch/points-zq.csv", false).unwrap();
        let mut alias: HashMap<String, u64> = HashMap::with_capacity(points.len());
        // 先形成别名的map
        for m in points.values() {
            if !m.alias_id.is_empty() {
                let key = m.alias_id.clone();
                alias.insert(key, m.point_id);
            }
        }
        let mut aoes = r.unwrap();
        let set_measure_tx = aoes[0].measure_sender();
        for aoe in &mut aoes {
            let r = initial_aoe_points(aoe, &points);
            assert_eq!(None, r);
            // 监听AOE发出来的命令
            start_measure_loop(aoe.control_receiver(), aoe.measure_sender());
        }
        let mut dispatcher = Dispatcher::new();
        let rx = dispatcher.result_receiver();
        let cloned_set_measure_tx = set_measure_tx.clone();
        let cloned_alias = alias.clone();
        let job = tokio::spawn(async move {
            let mut count = 0;
            loop {
                match rx.recv().await {
                    Ok(r) => {
                        debug!("aoe finish: {}", r.aoe_id);
                        if r.aoe_id == 30 {
                            for (from, to, r) in &r.action_result {
                                debug!("{}->{}", from, to);
                                debug!("{:?}", r.num_result);
                            }
                        }
                        count += 1;
                        match count {
                            // Case1 RESULT: Discharge strategy 1
                            1 => {
                                debug!("###### CASE2:Charge strategy 1 ######"); // RESULT: Charge strategy 1
                                set_points(
                                    dstr2.as_str(),
                                    astr2.as_str(),
                                    &cloned_alias,
                                    &cloned_set_measure_tx,
                                )
                                .await;
                            }
                            2 => {
                                debug!("###### CASE3:Discharge strategy 2 ######"); // RESULT: Discharge strategy 2
                                set_points(
                                    dstr3.as_str(),
                                    astr3.as_str(),
                                    &cloned_alias,
                                    &cloned_set_measure_tx,
                                )
                                .await;
                            }
                            3 => {
                                debug!("###### CASE4:Charge strategy 2 ######"); // RESULT: Charge strategy 2
                                set_points(
                                    dstr4.as_str(),
                                    astr4.as_str(),
                                    &cloned_alias,
                                    &cloned_set_measure_tx,
                                )
                                .await;
                            }
                            4 => {
                                debug!("###### CASE5:Discharge optimal strategy ######"); // RESULT: Discharge optimal strategy
                                set_points(
                                    dstr5.as_str(),
                                    astr5.as_str(),
                                    &cloned_alias,
                                    &cloned_set_measure_tx,
                                )
                                .await;
                            }
                            5 => {
                                debug!("###### CASE6:Charge optimal strategy ######"); // RESULT: Discharge optimal strategy
                                set_points(
                                    dstr6.as_str(),
                                    astr6.as_str(),
                                    &cloned_alias,
                                    &cloned_set_measure_tx,
                                )
                                .await;
                            }
                            6 => {
                                debug!("###### CASE7:Discharge strategy 2 ######"); // RESULT: Discharge optimal strategy
                                set_points(
                                    dstr7.as_str(),
                                    astr7.as_str(),
                                    &cloned_alias,
                                    &cloned_set_measure_tx,
                                )
                                .await;
                            }
                            _ => {
                                break;
                            }
                        }
                    }
                    Err(_) => {
                        break;
                    }
                }
                if count == 7 {
                    break;
                }
            }
        });
        dispatcher.schedule(aoes);

        // 设置初始值
        debug!("###### CASE1:Discharge strategy 1 ######"); // RESULT: Discharge strategy 1
        set_points(dstr1.as_str(), astr1.as_str(), &alias, &set_measure_tx).await;

        job.await.unwrap();
        dispatcher.shutdown().await;
    }

    #[tokio::test]
    async fn test_aoe_file3() {
        use log::debug;

        // let log_env = env_logger::Env::default().default_filter_or("trace");
        // env_logger::Builder::from_env(log_env).init();

        // Cross-sectional data set 1 (RESULT: Discharge strategy 1)
        let black_start_str1 = "START_TEST3:1";
        let dstr1 = black_start_str1.to_string();
        let astr1 = "".to_string();

        let black_start_str2 = "START_TEST3:0;VT_PCS1_START_COMMAND:1;VT_PCS2_START_COMMAND:1";
        let dstr2 = black_start_str2.to_string();
        let astr2 = "".to_string();

        let black_start_str3 =
            "START_TEST3:1;XA_PCS1_ED00:2;VT_PCS1_ERROR:0;XA_PCS2_ED00:2;VT_PCS2_ERROR:0";
        let dstr3 = black_start_str3.to_string();
        let astr3 = "".to_string();

        // let black_start_str32 = "XA_PCS1_ED00:2;VT_PCS1_ERROR:0;XA_PCS2_ED00:0;VT_PCS2_ERROR:1";
        // let dstr32 =  black_start_str32.to_string();
        // let astr32 = "".to_string();

        let black_start_str4 = "VT_RJ1_START_COMMAND:1";
        let dstr4 = black_start_str4.to_string();
        // let astr4 = "".to_string();
        //
        // let black_start_str5 = "VT_RJ_LACK:2;XA_RJ1toEMS_bIfOnline:0;XA_RJ8toEMS_bPermissiRun:0;XA_RJ1toEMS_bPermissiRun:1;XA_RJ5toEMS_bPermissiRun:1;XA_INTERFACE_RJ1Select:1;XA_INTERFACE_RJ5Select:1";
        // let dstr5 =  black_start_str5.to_string();
        // let astr5 = "".to_string();

        // let r = from_file2("tests/aoe-zq-power-dispatch.csv");
        // assert!(r.is_ok());
        // let points = from_csv2("tests/points-zq.csv").unwrap();
        let r = from_file2("tests/test_xa/aoe-xa-EMS20200711.csv", false);
        assert!(r.is_ok());
        let r2 = from_csv2("tests/test_xa/points-xa20210723.csv", false);
        if let Err(rc) = r2 {
            println!("Error: {:?}", rc);
        }
        assert!(r2.is_ok());
        let (points, _) = r2.unwrap();
        let mut alias: HashMap<String, u64> = HashMap::with_capacity(points.len());
        // 先形成别名的map
        for m in points.values() {
            if !m.alias_id.is_empty() {
                let key = m.alias_id.clone();
                alias.insert(key, m.point_id);
            }
        }
        let mut aoes = r.unwrap();
        let set_measure_tx0 = aoes[0].measure_sender();
        let set_measure_tx1 = aoes[1].measure_sender();
        let set_measure_tx2 = aoes[2].measure_sender();
        let set_measure_tx3 = aoes[3].measure_sender();
        // let set_measure_tx4 = aoes[4].measure_sender();
        for aoe in &mut aoes {
            let r = initial_aoe_points(aoe, &points);
            assert_eq!(None, r);
            // 监听AOE发出来的命令
            start_measure_loop(aoe.control_receiver(), aoe.measure_sender());
        }
        let mut dispatcher = Dispatcher::new();
        let rx = dispatcher.result_receiver();
        let cloned_set_measure_tx0 = set_measure_tx0.clone();
        let cloned_set_measure_tx1 = set_measure_tx1.clone();
        let cloned_set_measure_tx2 = set_measure_tx2.clone();
        let cloned_set_measure_tx3 = set_measure_tx3.clone();
        // let cloned_set_measure_tx4 = set_measure_tx4.clone();
        let cloned_alias = alias.clone();
        let job = tokio::spawn(async move {
            let mut count = 0;
            loop {
                match rx.recv().await {
                    Ok(r) => {
                        debug!("aoe finish: {}", r.aoe_id);
                        // if r.aoe_id == 1 {
                        //     for (from, to, r) in &r.action_result {
                        //         debug!("{}->{}", from, to);
                        //         debug!("{:?}", r.num_result);
                        //     }
                        // }
                        count += 1;
                        match count {
                            // Case1 RESULT: Discharge strategy 1
                            1 => {
                                debug!("###### CASE2:BLACKSTART strategy 2 ######"); // RESULT: BLACKSTART strategy 2
                                set_points(
                                    dstr2.as_str(),
                                    astr2.as_str(),
                                    &cloned_alias,
                                    &cloned_set_measure_tx0,
                                )
                                .await;
                                set_points(
                                    dstr2.as_str(),
                                    astr2.as_str(),
                                    &cloned_alias,
                                    &cloned_set_measure_tx1,
                                )
                                .await;
                                // set_points(dstr2.as_str(), astr2.as_str(), &cloned_alias, &cloned_set_measure_tx2).await;
                                // set_points(dstr2.as_str(), astr2.as_str(), &cloned_alias, &cloned_set_measure_tx3).await;
                                // set_points(dstr2.as_str(), astr2.as_str(), &cloned_alias, &cloned_set_measure_tx4).await;
                            }
                            2 => {
                                debug!("###### CASE3:BLACKSTART strategy 31 ######"); // RESULT: BLACKSTART strategy 3
                                                                                      // set_points(dstr3.as_str(), astr3.as_str(), &cloned_alias, &cloned_set_measure_tx0).await;
                                                                                      // set_points(dstr3.as_str(), astr3.as_str(), &cloned_alias, &cloned_set_measure_tx1).await;
                                                                                      // set_points(dstr3.as_str(), astr3.as_str(), &cloned_alias, &cloned_set_measure_tx2).await;
                                set_points(
                                    dstr3.as_str(),
                                    astr3.as_str(),
                                    &cloned_alias,
                                    &cloned_set_measure_tx3,
                                )
                                .await;
                                // set_points(dstr3.as_str(), astr3.as_str(), &cloned_alias, &cloned_set_measure_tx4).await;
                            }
                            3 => {
                                debug!("###### CASE4:BLACKSTART strategy 4 ######"); // RESULT: BLACKSTART strategy 4
                                                                                     // set_points(dstr4.as_str(), astr3.as_str(), &cloned_alias, &cloned_set_measure_tx0).await;
                                                                                     // set_points(dstr4.as_str(), astr3.as_str(), &cloned_alias, &cloned_set_measure_tx1).await;
                                set_points(
                                    dstr4.as_str(),
                                    astr3.as_str(),
                                    &cloned_alias,
                                    &cloned_set_measure_tx2,
                                )
                                .await;
                                // set_points(dstr4.as_str(), astr3.as_str(), &cloned_alias, &cloned_set_measure_tx3).await;
                                // set_points(dstr4.as_str(), astr3.as_str(), &cloned_alias, &cloned_set_measure_tx4).await;
                            }
                            // 4 => {
                            //     debug!("###### CASE5:BLACKSTART strategy 5 ######"); // RESULT: Charge strategy 5
                            //     // set_points(dstr5.as_str(), astr3.as_str(), &cloned_alias, &cloned_set_measure_tx0).await;
                            //     // set_points(dstr5.as_str(), astr3.as_str(), &cloned_alias, &cloned_set_measure_tx1).await;
                            //     // set_points(dstr5.as_str(), astr3.as_str(), &cloned_alias, &cloned_set_measure_tx2).await;
                            //     // set_points(dstr5.as_str(), astr3.as_str(), &cloned_alias, &cloned_set_measure_tx3).await;
                            //     set_points(dstr5.as_str(), astr3.as_str(), &cloned_alias, &cloned_set_measure_tx4).await;
                            // }
                            _ => {
                                break;
                            }
                        }
                    }
                    Err(_) => {
                        break;
                    }
                }
                if count == 4 {
                    break;
                }
            }
        });
        dispatcher.schedule(aoes);
        // 设置初始值
        debug!("###### CASE1:BLACKSTART strategy 1 ######"); // RESULT: Discharge strategy 1
        set_points(dstr1.as_str(), astr1.as_str(), &alias, &set_measure_tx3).await;
        // debug!("###### CASE5:BLACKSTART strategy 1 ######"); // RESULT: Discharge strategy 1
        // set_points(dstr5.as_str(), astr1.as_str(), &alias, &set_measure_tx4).await;
        // 该策略不会被触发，因为计算点变量未被传送至AOE
        job.await.unwrap();
        dispatcher.shutdown().await;
    }

    #[tokio::test]
    async fn test_aoe_file4() {
        // let log_env = env_logger::Env::default().default_filter_or("trace");
        // env_logger::Builder::from_env(log_env).init();

        let xm1 = "P_xiafa:70;P_zhiling:70;SOC:50;";
        let dstr1 = xm1.to_string();
        let astr1 = "".to_string();

        let xm2 = "ifxiafa:1";
        let dstr2 = xm2.to_string();
        let astr2 = "".to_string();

        let xm3 = "P_xiafa:50;P_zhiling:70;SOC:50;";
        let dstr3 = xm3.to_string();
        let astr3 = "".to_string();

        let r = from_file2("tests/test_xm/aoe-xm-EMS20220215.csv", false);
        // let r = from_file2("tests/test_xm2/aoe-xm-EMS20220225.csv");
        println!("Aoe parse result : {:?}", r);
        assert!(r.is_ok());
        let (points, _) = from_csv2("tests/test_xm/points-20220214.csv", false).unwrap();
        // let points = from_csv2("tests/test_xm2/points-20220225.csv").unwrap();
        let mut alias: HashMap<String, u64> = HashMap::with_capacity(points.len());
        // 先形成别名的map
        for m in points.values() {
            if !m.alias_id.is_empty() {
                let key = m.alias_id.clone();
                alias.insert(key, m.point_id);
            }
        }
        let cloned_alias = alias.clone();
        println!("{:?}", cloned_alias);
        let mut aoes = r.unwrap();
        for aoe in &mut aoes {
            let r = initial_aoe_points(aoe, &points);
            assert_eq!(None, r);
            // 监听AOE发出来的命令
            start_measure_loop(aoe.control_receiver(), aoe.measure_sender());
        }
        let mut dispatcher = Dispatcher::new();
        let rx = dispatcher.result_receiver();
        let set_measure_tx0 = aoes[0].measure_sender();
        let cloned_set_measure_tx0 = set_measure_tx0.clone();
        set_points(
            dstr1.as_str(),
            astr1.as_str(),
            &cloned_alias,
            &cloned_set_measure_tx0,
        )
        .await;
        set_points(
            dstr2.as_str(),
            astr2.as_str(),
            &cloned_alias,
            &cloned_set_measure_tx0,
        )
        .await;
        let job = tokio::spawn(async move {
            let mut count = 0;
            loop {
                match rx.recv().await {
                    Ok(_) => {
                        count += 1;
                        match count {
                            // Case1 RESULT: Discharge strategy 1
                            1 => {
                                set_points(
                                    dstr3.as_str(),
                                    astr3.as_str(),
                                    &cloned_alias,
                                    &cloned_set_measure_tx0,
                                )
                                .await;
                                set_points(
                                    dstr2.as_str(),
                                    astr2.as_str(),
                                    &cloned_alias,
                                    &cloned_set_measure_tx0,
                                )
                                .await;
                            }
                            2 => {
                                set_points(
                                    dstr3.as_str(),
                                    astr3.as_str(),
                                    &cloned_alias,
                                    &cloned_set_measure_tx0,
                                )
                                .await;
                                set_points(
                                    dstr2.as_str(),
                                    astr2.as_str(),
                                    &cloned_alias,
                                    &cloned_set_measure_tx0,
                                )
                                .await;
                            }
                            3 => {
                                set_points(
                                    dstr3.as_str(),
                                    astr3.as_str(),
                                    &cloned_alias,
                                    &cloned_set_measure_tx0,
                                )
                                .await;
                                set_points(
                                    dstr2.as_str(),
                                    astr2.as_str(),
                                    &cloned_alias,
                                    &cloned_set_measure_tx0,
                                )
                                .await;
                            }
                            4 => {
                                set_points(
                                    dstr3.as_str(),
                                    astr3.as_str(),
                                    &cloned_alias,
                                    &cloned_set_measure_tx0,
                                )
                                .await;
                                set_points(
                                    dstr2.as_str(),
                                    astr2.as_str(),
                                    &cloned_alias,
                                    &cloned_set_measure_tx0,
                                )
                                .await;
                            }
                            5 => {
                                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                                set_points(
                                    dstr3.as_str(),
                                    astr3.as_str(),
                                    &cloned_alias,
                                    &cloned_set_measure_tx0,
                                )
                                .await;
                                set_points(
                                    dstr2.as_str(),
                                    astr2.as_str(),
                                    &cloned_alias,
                                    &cloned_set_measure_tx0,
                                )
                                .await;
                            }

                            _ => {
                                break;
                            }
                        }
                    }
                    Err(_) => {
                        break;
                    }
                }
                if count == 5 {
                    break;
                }
            }
        });
        dispatcher.schedule(aoes);
        job.await.unwrap();
        dispatcher.shutdown().await;
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }

    #[tokio::test]
    async fn test_aoe_file5() {
        // let log_env = env_logger::Env::default().default_filter_or("trace");
        // env_logger::Builder::from_env(log_env).init();

        let xm1 = "SOC:100;";
        let xm11 = "P_xiafa:-150;";
        let dstr1 = xm1.to_string();
        let astr1 = xm11.to_string();

        let xm2 = "ifxiafa:1";
        let dstr2 = xm2.to_string();
        let astr2 = "".to_string();

        let xm3 = "SOC:100;";
        let xm33 = "P_xiafa:-150;";
        let dstr3 = xm3.to_string();
        let astr3 = xm33.to_string();

        // let r = from_file2("tests/test_xm/aoe-xm-EMS20220215.csv");
        let r = from_file2("tests/test_xm2/aoe-xm-EMS20220225.csv", false);
        println!("Aoe parse result : {:?}", r);
        assert!(r.is_ok());
        // let points = from_csv2("tests/test_xm/points-20220214.csv").unwrap();
        let (points, _) = from_csv2("tests/test_xm2/points-20220225.csv", false).unwrap();
        let mut alias: HashMap<String, u64> = HashMap::with_capacity(points.len());
        // 先形成别名的map
        for m in points.values() {
            if !m.alias_id.is_empty() {
                let key = m.alias_id.clone();
                alias.insert(key, m.point_id);
            }
        }
        let cloned_alias = alias.clone();
        println!("{:?}", cloned_alias);
        let mut aoes = r.unwrap();
        for aoe in &mut aoes {
            let r = initial_aoe_points(aoe, &points);
            assert_eq!(None, r);
            // 监听AOE发出来的命令
            start_measure_loop(aoe.control_receiver(), aoe.measure_sender());
        }
        let mut dispatcher = Dispatcher::new();
        let rx = dispatcher.result_receiver();
        let set_measure_tx0 = aoes[0].measure_sender();
        let cloned_set_measure_tx0 = set_measure_tx0.clone();
        set_points(
            dstr1.as_str(),
            astr1.as_str(),
            &cloned_alias,
            &cloned_set_measure_tx0,
        )
        .await;
        set_points(
            dstr2.as_str(),
            astr2.as_str(),
            &cloned_alias,
            &cloned_set_measure_tx0,
        )
        .await;
        let job = tokio::spawn(async move {
            let mut count = 0;
            loop {
                match rx.recv().await {
                    Ok(_) => {
                        count += 1;
                        match count {
                            // Case1 RESULT: Discharge strategy 1
                            1 => {
                                set_points(
                                    dstr3.as_str(),
                                    astr3.as_str(),
                                    &cloned_alias,
                                    &cloned_set_measure_tx0,
                                )
                                .await;
                                set_points(
                                    dstr2.as_str(),
                                    astr2.as_str(),
                                    &cloned_alias,
                                    &cloned_set_measure_tx0,
                                )
                                .await;
                            }
                            2 => {
                                set_points(
                                    dstr3.as_str(),
                                    astr3.as_str(),
                                    &cloned_alias,
                                    &cloned_set_measure_tx0,
                                )
                                .await;
                                set_points(
                                    dstr2.as_str(),
                                    astr2.as_str(),
                                    &cloned_alias,
                                    &cloned_set_measure_tx0,
                                )
                                .await;
                            }
                            3 => {
                                set_points(
                                    dstr3.as_str(),
                                    astr3.as_str(),
                                    &cloned_alias,
                                    &cloned_set_measure_tx0,
                                )
                                .await;
                                set_points(
                                    dstr2.as_str(),
                                    astr2.as_str(),
                                    &cloned_alias,
                                    &cloned_set_measure_tx0,
                                )
                                .await;
                            }
                            4 => {
                                set_points(
                                    dstr3.as_str(),
                                    astr3.as_str(),
                                    &cloned_alias,
                                    &cloned_set_measure_tx0,
                                )
                                .await;
                                set_points(
                                    dstr2.as_str(),
                                    astr2.as_str(),
                                    &cloned_alias,
                                    &cloned_set_measure_tx0,
                                )
                                .await;
                            }
                            5 => {
                                set_points(
                                    dstr2.as_str(),
                                    astr2.as_str(),
                                    &cloned_alias,
                                    &cloned_set_measure_tx0,
                                )
                                .await;
                            }
                            6 => {
                                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                                let discretes = dstr3.as_str();
                                let analogs = astr3.as_str();
                                set_points(discretes, analogs, &cloned_alias, &cloned_set_measure_tx0, ).await;
                                set_points(dstr2.as_str(), astr2.as_str(), &cloned_alias, &cloned_set_measure_tx0).await;
                            }
                            _ => {
                                break;
                            }
                        }
                    }
                    Err(_) => {
                        break;
                    }
                }
                if count == 6 {
                    break;
                }
            }
        });
        dispatcher.schedule(aoes);
        job.await.unwrap();
        dispatcher.shutdown().await;
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }

    async fn set_points(
        discretes: &str,
        analogs: &str,
        alias: &HashMap<String, u64>,
        tx: &Sender<VarOrMeasures>,
    ) {
        let v: Vec<&str> = discretes.split(';').collect();
        let mut result = HashMap::new();
        for s in v {
            if s.is_empty() {
                continue;
            }
            let bias_to_v: Vec<&str> = s.split(':').collect();
            let point_id = alias.get(bias_to_v[0]).unwrap();
            result.insert(*point_id, MeasureValue {
                point_id: *point_id,
                is_discrete: true,
                timestamp: get_timestamp(),
                analog_value: 0.0,
                discrete_value: bias_to_v[1].parse().unwrap(),
                is_transformed: false,
                transformed_analog: 0.0,
                transformed_discrete: 0,
            });
        }
        let v: Vec<&str> = analogs.split(';').collect();
        for s in v {
            if s.is_empty() {
                continue;
            }
            let bias_to_v: Vec<&str> = s.split(':').collect();
            let point_id = alias.get(bias_to_v[0]).unwrap();
            result.insert(*point_id, MeasureValue {
                point_id: *point_id,
                is_discrete: false,
                timestamp: get_timestamp(),
                analog_value: bias_to_v[1].parse().unwrap(),
                discrete_value: 0,
                is_transformed: false,
                transformed_analog: 0.0,
                transformed_discrete: 0,
            });
        }
        let buf = MeasureBuf::new(result, HashMap::with_capacity(0));
        tx.send(VarOrMeasures::Measures(buf)).await.unwrap();
    }

    fn start_measure_loop(
        rx: Receiver<(Vec<SetIntValue>, Vec<SetFloatValue>)>,
        tx: Sender<VarOrMeasures>,
    ) {
        let (meas_sender, meas_receiver) = bounded(100);
        tokio::spawn(async move {
            loop {
                match meas_receiver.recv().await {
                    Ok(measures) => {
                        let msg = VarOrMeasures::Measures(measures);
                        if let Err(e) = tx.send(msg).await {
                            log::warn!("!!Failed to send measures to aoe, {:?}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        log::warn!("!!Failed to receive measures, {:?}", e);
                        break;
                    }
                }
            }
        });
        tokio::spawn(async move {
            while let Ok((d, a)) = rx.recv().await {
                let mut result = HashMap::new();
                for v in d {
                    result.insert(v.point_id, MeasureValue {
                        point_id: v.point_id,
                        is_discrete: true,
                        timestamp: get_timestamp(),
                        analog_value: 0.0,
                        discrete_value: v.yk_command,
                        is_transformed: false,
                        transformed_analog: 0.0,
                        transformed_discrete: 0,
                    });
                }
                for v in a {
                    result.insert(v.point_id, MeasureValue {
                        point_id: v.point_id,
                        is_discrete: false,
                        timestamp: get_timestamp(),
                        analog_value: v.yt_command,
                        discrete_value: 0,
                        is_transformed: false,
                        transformed_analog: 0.0,
                        transformed_discrete: 0,
                    });
                }
                let buf =MeasureBuf::new(result, HashMap::new());
                meas_sender.send(buf).await.unwrap();
            }
        });
    }

    #[tokio::test]
    async fn test_set_points_check() {
        // let log_env = env_logger::Env::default().default_filter_or("trace");
        // env_logger::Builder::from_env(log_env).init();

        let r = from_file2("tests/other/aoe-set-points-with-check.csv", false);
        assert!(r.is_ok());
        let (points, _) = from_csv2("tests/other/points-set-points-with-check.csv", false).unwrap();
        let mut aoes = r.unwrap();
        for aoe in &mut aoes {
            let r = initial_aoe_points(aoe, &points);
            assert_eq!(None, r);
            assert_eq!(aoe.context.len(), 3);
            // 监听AOE发出来的命令
            start_measure_loop(aoe.control_receiver(), aoe.measure_sender());
        }
        let mut dispatcher = Dispatcher::new();
        let rx = dispatcher.result_receiver();
        let job = tokio::spawn(async move {
            if let Ok(r) = rx.recv().await {
                println!("aoe finish: {:?}", r);
                assert_eq!(3, r.event_result.len());
                assert_eq!(2, r.action_result.len());
            }
        });
        dispatcher.schedule(aoes);
        job.await.unwrap();
        dispatcher.shutdown().await;
    }
}
