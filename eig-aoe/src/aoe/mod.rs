// flowing should as same as in sparrowzz
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
// above should as same as in sparrowzz