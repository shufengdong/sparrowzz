use serde::{Deserialize, Serialize};

/**
 * @api {计划对象} /DayPlan DayPlan
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} id 计划id
 * @apiSuccess {String} name 计划名称
 * @apiSuccess {String} [desc] 计划描述
 * @apiSuccess {tuple[]} plan 计划内容数组，tuple格式为(开始时间:u64, 结束时间:u64, 功率值:f64)
 */
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct DayPlan {
    pub id: u64,
    pub name: String,
    pub desc: String,
    pub plan: Vec<(u64, u64, f64)>,
}

/**
 * @api {计划树节点} /PlanTreeNode PlanTreeNode
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {String} path 路径
 * @apiSuccess {String} name 名称
 * @apiSuccess {String} [desc] 描述
 * @apiSuccess {u64} [ref_id] 计划ID，如果是普通节点，则为None
 */
/// 计划树节点
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct PlanTreeNode {
    pub path: String,
    pub name: String,
    pub desc: Option<String>,
    // 计划ID，如果是普通节点，则为None
    pub ref_id: Option<u64>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum ScriptTarget {
    Aoe,
    Dff,
}

/**
 * @api {MemsScript} /MemsScript MemsScript
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} id 脚本id
 * @apiSuccess {String} path 脚本路径
 * @apiSuccess {String} desc 脚本描述
 * @apiSuccess {bool} is_need_island 是否需要电气岛
 * @apiSuccess {u64[]} plans 计划列表
 * @apiSuccess {String} wasm_module_name wasm模块名称
 * @apiSuccess {u64} wasm_update_time wasm上传时间
 * @apiSuccess {bool} is_file_uploaded 文件是否已上传
 * @apiSuccess {bool} is_js 是否是javascript文件
 */
// 脚本
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MemsScript {
    pub id: u64,
    pub target : ScriptTarget,
    pub path: String,
    pub desc: String,
    // 生成aoe script
    pub wasm_module_name: String,
    pub wasm_update_time: u64,
    pub is_file_uploaded: bool,
    pub is_js: bool,
}

/**
 * @api {ScriptWasmFile} /ScriptWasmFile ScriptWasmFile
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} script_id 脚本id
 * @apiSuccess {String} module_name 模块名称
 * @apiSuccess {u8[]} wasm_file wasm文件
 * @apiSuccess {u8[]} js_file js文件
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScriptWasmFile {
    pub script_id: u64,
    pub module_name: String,
    pub wasm_file: Vec<u8>,
    pub js_file: Vec<u8>,
}

/**
 * @api {AoeMakeResult} /AoeMakeResult AoeMakeResult
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} script_id script_id
 * @apiSuccess {u64} make_time make_time
 * @apiSuccess {u64} aoe_model_id aoe_model_id
 * @apiSuccess {u32} island_version 电气岛版本号
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScriptResult {
    pub script_id: u64,
    pub make_time: u64,
    pub model_id: u64,
    pub target: ScriptTarget,
}