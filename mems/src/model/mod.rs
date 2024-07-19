use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use eig_aoe::aoe::AoeModel;
use eig_domain::{DataUnit, Measurement, MeasureValue};

use crate::model::dev::{Island, PropDefine, RsrDefine};

pub mod dev;
pub mod plan;
pub mod web;

pub const SCRIPT_FILE_DIR: &str = "scripts";
pub const WEB_PLUGIN_FILE_DIR: &str = "plugins";
// 实时消息websocket地址
pub const URL_RT_MESSAGE: &str = "/ws/v1/rtmsg";
// ============= PLCC特有的URL，用于PLCC监控界面
// models
// controls
pub const LCC_QUIT_FORCE_URL: &str = "/api/v1/controls/quit_force";
pub const LCC_RESET_URL: &str = "/api/v1/controls/reset";
pub const LCC_RECOVER_URL: &str = "/api/v1/controls/recover";

// MEMS URLs
// ======================= common
// 树
pub const URL_FILE_TREE: &str = "/api/v1/file_tree_cbor";
// north
pub const URL_RESTART_NORTH: &str = "/api/v1/north/restart";

// ======================= PLCC管理和查询接口
// 查询所有在线的设备列表
pub const URL_LCC_LIST: &str = "/api/v1/lcc_list_cbor";
pub const URL_EMS_LIST: &str = "/api/v1/ems_list_cbor";
pub const URL_EMS_REQUEST: &str = "/api/v1/ems/request_bytes/";
// 历史数据查询
pub const URL_LCC_MEASURES: &str = "/api/v1/lcc/measures_bytes/";
pub const URL_LCC_AOE_RESULTS: &str = "/api/v1/lcc/aoe_results_bytes/";
pub const URL_LCC_ALARM_RESULTS: &str = "/api/v1/lcc/alarms_bytes/";
pub const URL_LCC_SOE_RESULTS: &str = "/api/v1/lcc/soes_bytes/";
pub const URL_LCC_COMMANDS: &str = "/api/v1/lcc/commands_bytes/";
pub const URL_LCC_LOGS: &str = "/api/v1/lcc/logs_bytes/";
pub const URL_LCC_ALL_MODEL: &str = "/api/v1/lcc/allmodels_bytes/";
// alarm define
pub const URL_LCC_ALARM_COUNT: &str = "/api/v1/lcc/alarm/count_cbor/";
pub const URL_LCC_ALARM_DEFINES: &str = "/api/v1/lcc/alarm/defines_bytes/";
pub const URL_LCC_ALARM_DEFINE: &str = "/api/v1/lcc/alarm/define_bytes/";
pub const URL_LCC_ALARM_CONFIG: &str = "/api/v1/lcc/alarm/config_cbor/";
pub const URL_LCC_ALARM_STATUS: &str = "/api/v1/lcc/alarm/confirm_status_cbor/";
pub const URL_LCC_ALARM_UNCONFIRMED_NUM: &str = "/api/v1/lcc/alarm/unconfirmed_number_cbor/";
pub const URL_LCC_ALARM_UNCONFIRMED: &str = "/api/v1/lcc/alarms/unconfirmed_bytes/";
pub const URL_LCC_ALARM_CONFIRM: &str = "/api/v1/lcc/alarm/confirm_cbor/";
// 工具
pub const URL_LCC_COMMON_MAP: &str = "/api/v1/lcc/common_map_cbor/";
pub const URL_LCC_TAG_DEFINES: &str = "/api/v1/lcc/tag_defines_cbor/";
pub const URL_LCC_TAGS: &str = "/api/v1/lcc/tags_cbor/";

// 控制接口，目前数据格式还未统一
pub const URL_LCC_CONFIG: &str = "/api/v1/lcc/config_cbor/";

// 模型配置接口
pub const URL_LCC_TRANSPORTS: &str = "/api/v1/lcc/transports/models_cbor/";
pub const URL_LCC_POINTS: &str = "/api/v1/lcc/points/models_cbor/";
pub const URL_LCC_AOES: &str = "/api/v1/lcc/aoes/models_cbor/";

// AOE相关接口
pub const URL_LCC_RUNNING_AOE: &str = "/api/v1/lcc/running_aoes_cbor/";
pub const URL_LCC_UNRUN_AOE: &str = "/api/v1/lcc/unrun_aoes_cbor/";
pub const URL_LCC_CONTROL: &str = "/api/v1/lcc/controls_cbor/";

pub const URL_LCC_USERS: &str = "/api/v1/lcc/auth/users_cbor/";

// ======================== 测点
// plcc and mems are same
pub const URL_POINTS: &str = "/api/v1/points/models_cbor";
// 以下都是MEMS特有的URL
pub const URL_IMPORT_POINTS: &str = "/api/v1/lcc/points/models_from";
pub const URL_POINTS_VERSION: &str = "/api/v1/points/version_cbor";
pub const URL_SET_POINT: &str = "/api/v1/controls_cbor/points";
pub const URL_SET_POINT2: &str = "/api/v1/controls_cbor/points_by_alias";
pub const URL_SET_POINT3: &str = "/api/v1/controls_cbor/points_by_expr";
// followings are mems only
pub const URL_APPLY_POINTS: &str = "/api/v1/pscpu/points_cbor";
pub const URL_APPLY_POINTS_MODELS: &str = "/api/v1/pscpu/points/models_cbor";
pub const URL_APPLY_POINTS_VERSION: &str = "/api/v1/pscpu/points/version_cbor";
// 获取可编辑的测点集合
pub const URL_GET_POINTS: &str = "/api/v1/mems/points_cbor?version=0";
// LCC设备ID与其测点号的对应关系
pub const URL_LCC_TO_POINT_IDS: &str = "/api/v1/points/remote_cbor";

// ========================= AOE
// plcc and mems are same
// config and ping
pub const URL_CONFIG: &str = "/api/v1/config_cbor";
pub const URL_PING: &str = "/api/v1/ping_bytes";
pub const URL_AOES: &str = "/api/v1/aoes/models_cbor";
pub const URL_CONTROL_AOE: &str = "/api/v1/controls_cbor/aoes";
pub const URL_RUNNING_AOES: &str = "/api/v1/running_aoes_cbor";
pub const URL_UNRUN_AOES: &str = "/api/v1/unrun_aoes_cbor";
// mems only
pub const URL_AOES_APPLY: &str = "/api/v1/aoes/models_cbor/for_apply";
pub const URL_QUERY_AOES_BY_VERSION: &str = "/api/v1/aoes/models_cbor/by_version/";
pub const URL_AOES_VERSION: &str = "/api/v1/aoes/version_cbor";
pub const URL_APPLY_AOES: &str = "/api/v1/pscpu/aoes_cbor";
pub const URL_APPLY_AOES_MODELS: &str = "/api/v1/pscpu/aoes/models_cbor";
pub const URL_APPLY_AOES_VERSION: &str = "/api/v1/pscpu/aoes/version_cbor";
pub const URL_UPDATE_HIS_DB_INITS: &str = "/api/v1/measureinits/";
// ============================= transport
// plcc only
pub const URL_TRANSPORT: &str = "/api/v1/transports/models_cbor";

// ======================== 设备相关
// 设备定义
pub const URL_MEASURE_TYPES: &str = "/api/v1/devices/measure_types_cbor";
pub const URL_DEV_TYPES: &str = "/api/v1/devices/dev_types_cbor";
pub const URL_DEV_PROPERTY_TYPE: &str = "/api/v1/devices/prop_types_cbor";
pub const URL_DEV_DATA_UNIT: &str = "/api/v1/devices/data_units_cbor";
pub const URL_DEV_NODE_TYPES: &str = "/api/v1/devices/dev_node_types_cbor";
pub const URL_DEV_DEFINES: &str = "/api/v1/devices/defines_cbor";
pub const URL_DEV_PROPERTY: &str = "/api/v1/devices/prop_defines_cbor";
pub const URL_DEV: &str = "/api/v1/devices/devs_cbor";
pub const URL_DEV_PATHS: &str = "/api/v1/devices/paths_cbor";
pub const URL_DEV_PROP_GROUP: &str = "/api/v1/devices/prop_groups_cbor";
pub const URL_DEV_STATIONS: &str = "/api/v1/devices/stations_cbor";
pub const URL_DEV_VOLTAGE_LEVELS: &str = "/api/v1/devices/voltage_levels_cbor";
pub const URL_DEV_MEASURE_DEFS: &str = "/api/v1/devices/measure_defs_cbor";
pub const URL_CNS: &str = "/api/v1/devices/cns_cbor";
pub const URL_DEV_VERSION: &str = "/api/v1/devices/version_cbor";
pub const URL_DEV_MULTI_IMPORT: &str = "/api/v1/multi_import_bytes";

pub const URL_APPLY_ISLAND_MODELS: &str = "/api/v1/pscpu/island/models_cbor";
pub const URL_APPLY_ISLAND_PATHS: &str = "/api/v1/pscpu/island/paths_cbor";
pub const URL_APPLY_ISLAND_VERSION: &str = "/api/v1/pscpu/island/version_cbor";
pub const URL_APPLY_POINT_TREE: &str = "/api/v1/pscpu/island/point_tree_cbor";
pub const URL_POINT_TREE: &str = "/api/v1/devices/point_tree_cbor";
pub const URL_ISLAND: &str = "/api/v1/devices/islands_cbor";


// pscpu
pub const URL_PSCPU_PROFILE: &str = "/api/v1/pscpu/info_cbor";
pub const URL_PSCPU_START: &str = "/api/v1/pscpu/start";
pub const URL_PSCPU_STOP: &str = "/api/v1/pscpu/stop";
pub const URL_PSCPU_RESET: &str = "/api/v1/pscpu/reset";

// ====================== 用户权限
pub const URL_USERS: &str = "/api/v1/auth/users_cbor";
pub const URL_ROLES: &str = "/api/v1/auth/roles_cbor";
pub const URL_USER_GROUPS: &str = "/api/v1/auth/user_groups_cbor";
pub const URL_LOGIN: &str = "/api/v1/auth/login_cbor";
pub const URL_REGISTER: &str = "/api/v1/auth/register_cbor";
pub const URL_AUTHS: &str = "/api/v1/auth/auths_cbor";
pub const URL_MENUS: &str = "/api/v1/auth/menus_cbor";

// ====================== 查询项
// 历史数据查询，plcc和mems是一样的
pub const URL_MEASURES: &str = "/api/v1/measures_bytes";
pub const URL_AOE_RESULTS: &str = "/api/v1/aoe_results_bytes";
pub const URL_ALARM_RESULTS: &str = "/api/v1/alarms_bytes";
pub const URL_SOE_RESULTS: &str = "/api/v1/soes_bytes";
pub const URL_COMMANDS: &str = "/api/v1/commands_bytes";
pub const URL_LOGS: &str = "/api/v1/logs_bytes";
pub const URL_ALL_MODEL: &str = "/api/v1/allmodels_bytes";


// ==================== alarm defines
pub const URL_ALARM_COUNT: &str = "/api/v1/alarm/count_cbor";
pub const URL_ALARM_DEFINES: &str = "/api/v1/alarm/defines_bytes";
pub const URL_ALARM_DEFINE: &str = "/api/v1/alarm/define_bytes";
pub const URL_ALARM_CONFIG: &str = "/api/v1/alarm/config_cbor";
pub const URL_ALARM_STATUS: &str = "/api/v1/alarm/confirm_status_cbor";
pub const URL_ALARM_UNCONFIRMED_NUM: &str = "/api/v1/alarm/unconfirmed_number_cbor";
pub const URL_ALARM_UNCONFIRMED: &str = "/api/v1/alarms/unconfirmed_bytes";
pub const URL_ALARM_CONFIRM: &str = "/api/v1/alarm/confirm_cbor";

// ====================== 报表
pub const URL_DFF_MODELS: &str = "/api/v1/flows/models_cbor";
pub const URL_SIMPLE_DFF_MODELS: &str = "/api/v1/flows/simple_models_cbor";
pub const URL_DFF_MODELS_FILE: &str = "/api/v1/flows/models_file";
pub const URL_DFF_MODELS_URL: &str = "/api/v1/flows/models_str";
pub const URL_DFF_RESULT_KEYS: &str = "/api/v1/flows/result_keys_cbor";
pub const URL_DFF_RESULTS: &str = "/api/v1/flows/results_cbor";
pub const URL_DFF_BRIEF_RESULTS: &str = "/api/v1/flows/brief_results_cbor";
pub const URL_RUNNING_FLOWS: &str = "/api/v1/flows/running_cbor";
pub const URL_UNRUN_FLOWS: &str = "/api/v1/flows/unrun_cbor";
pub const URL_CONTROL_FLOW: &str = "/api/v1/flows/controls_cbor";
pub const URL_DFF_VIEW_MODELS: &str = "/api/v1/flows/view_cbor";
pub const URL_DFF_DEBUG: &str = "/api/v1/flows/debug_cbor";

// ====================== 脚本
pub const URL_SCRIPTS: &str = "/api/v1/scripts_cbor";
pub const URL_SCRIPT_WASM_FILE: &str = "/api/v1/script_wasm_cbor";
pub const URL_SCRIPT_RESULTS: &str = "/api/v1/script_results_cbor";
pub const URL_SCRIPT_7Z_FILE: &str = "/api/v1/script_file_cbor";
pub const URL_SCRIPT_MD5: &str = "/api/v1/script_md5_cbor";

// ====================== 计划
pub const URL_PLAN_MODELS: &str = "/api/v1/plans/models_cbor";
pub const URL_PLAN_BY_IDS: &str = "/api/v1/plans/models_cbor/by_ids/";
pub const URL_PLAN_PATHS: &str = "/api/v1/plans/paths_cbor";

// ====================== SVG
pub const URL_GRAPH_MODELS: &str = "/api/v1/graphs/models_cbor";
pub const URL_GRAPH_PATHS: &str = "/api/v1/graphs/paths_cbor";
pub const URL_GRAPH_VERSION: &str = "/api/v1/graphs/version_cbor";
pub const URL_APPLY_GRAPH_MODELS: &str = "/api/v1/graphs/apply/models_cbor";
pub const URL_APPLY_GRAPH_PATHS: &str = "/api/v1/graphs/apply/paths_cbor";
pub const URL_APPLY_GRAPH_VERSION: &str = "/api/v1/graphs/apply/version_cbor";
pub const URL_APPLY_GRAPH_ADDITIONAL: &str = "/api/v1/graphs/apply/additional_cbor";

// ====================== 工具
// 存储key和value的api
pub const URL_COMMON_MAP: &str = "/api/v1/common_map_cbor";
pub const URL_TAG_DEFINES: &str = "/api/v1/tag_defines_cbor/";
pub const URL_TAGS: &str = "/api/v1/tags_cbor/";
pub const URL_WEB_PLUGINS: &str = "/api/v1/webplugins_cbor";
pub const URL_WEB_PLUGIN_7Z_FILE: &str = "/api/v1/webplugin_file_cbor";
pub const URL_WEB_PLUGIN_MD5: &str = "/api/v1/webplugin_md5_cbor";
/**
 * @api {SysPoints} /SysPoints SysPoints
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u32} version 版本号
 * @apiSuccess {String} commit_msg 版本描述
 * @apiSuccess {Measurement[]} points 测点列表
 * @apiSuccess {Map} paths 路径Map，HashMap<路径名:String, 测点id:u64>
 * @apiSuccess {tuple[]} beeid_to_points beeId和测点列表对应的数组，tuple格式为(beeId:String, 测点列表:u64[])
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SysPoints {
    pub version: u32,
    pub commit_msg: String,
    pub points: Vec<Measurement>,
    pub beeid_to_points: Vec<(String, Vec<u64>)>,
}

/**
 * @api {SysAoes} /SysAoes SysAoes
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u32} version 版本号
 * @apiSuccess {String} commit_msg 版本描述
 * @apiSuccess {AoeModel[]} aoes AOE列表
 * @apiSuccess {Map} paths 路径Map，HashMap<路径名:String, AOE_id:u64>
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SysAoes {
    pub version: u32,
    pub commit_msg: String,
    pub aoes: Vec<AoeModel>,
}

/**
 * @api {SysIsland} /SysIsland SysIsland
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u32} version 版本号
 * @apiSuccess {String} commit_msg 版本描述
 * @apiSuccess {Island} island 电气岛
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SysIsland {
    pub version: u32,
    pub commit_msg: String,
    pub island: Island,
    pub rsr_defs: Vec<RsrDefine>,
    pub prop_defs: Vec<PropDefine>,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum ModelType {
    Island,
    Meas,
    File(Vec<String>),
    Outgoing(Vec<String>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PluginInput {
    pub model: Vec<ModelType>,
    pub model_len: Vec<u32>,
    pub dfs: Vec<String>,
    pub dfs_len: Vec<u32>,
    pub bytes: Vec<u8>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PluginOutput {
    pub error_msg: Option<String>,
    pub schema: Option<Vec<arrow_schema::Schema>>,
    pub csv_bytes: Vec<(String, Vec<u8>)>,
}

pub fn get_island_from_plugin_input(input: &PluginInput) -> Result<(Island, Vec<PropDefine>, HashMap<u64, RsrDefine>), String> {
    let mut from = 0;
    let mut index = 0;
    for i in 0..input.model.len() {
        match input.model[i] {
            ModelType::Island => {
                if input.model_len.len() <= index {
                    return Err("model_len length error".to_string());
                }
                let size = input.model_len[index] as usize;
                let end = from + size;
                let r = ciborium::from_reader(&input.bytes[from..end]);
                if r.is_err() {
                    return Err(format!("{:?}", r));
                }
                from += size;
                let island = r.unwrap();
                index += 1;
                if input.model_len.len() <= index {
                    return Err("model_len length error".to_string());
                }
                let size = input.model_len[index] as usize;
                let end = from + size;
                let r = ciborium::from_reader(&input.bytes[from..end]);
                if r.is_err() {
                    return Err(format!("{:?}", r));
                }
                from += size;
                let defines = r.unwrap();
                index += 1;
                if input.model_len.len() <= index {
                    return Err("model_len length error".to_string());
                }
                let size = input.model_len[index] as usize;
                let end = from + size;
                let r = ciborium::from_reader(&input.bytes[from..end]);
                if r.is_err() {
                    return Err(format!("{:?}", r));
                }
                let prop_defs = r.unwrap();
                return Ok((island, prop_defs, defines));
            }
            ModelType::Meas => {
                if input.model_len.len() <= index + 2 {
                    return Err("model_len length error".to_string());
                }
                let size1 = input.model_len[index] as usize;
                let size2 = input.model_len[index + 1] as usize;
                from += size1;
                from += size2;
                index += 2;
            }
            _ => {}
        }
    }
    Err("Island not found in plugin input".to_string())
}

pub fn get_meas_from_plugin_input(input: &PluginInput) -> Result<(Vec<MeasureValue>, HashMap<u64, DataUnit>), String> {
    let mut from = 0;
    let mut index = 0;
    for i in 0..input.model.len() {
        match input.model[i] {
            ModelType::Meas => {
                if input.model_len.len() < index {
                    return Err("model_len length error".to_string());
                }
                let size = input.model_len[index] as usize;
                let end = from + size;
                let r = ciborium::from_reader(&input.bytes[from..end]);
                if r.is_err() {
                    return Err(format!("{:?}", r));
                }
                from += size;
                let meas = r.unwrap();
                index += 1;
                if input.model_len.len() <= index {
                    return Err("model_len length error".to_string());
                }
                let size = input.model_len[index] as usize;
                let end = from + size;
                let r = ciborium::from_reader(&input.bytes[from..end]);
                if r.is_err() {
                    return Err(format!("{:?}", r));
                }
                let units = r.unwrap();
                return Ok((meas, units));
            }
            ModelType::Island => {
                if input.model_len.len() < index + 3 {
                    return Err("model_len length error".to_string());
                }
                let size1 = input.model_len[index] as usize;
                let size2 = input.model_len[index + 1] as usize;
                let size3 = input.model_len[index + 2] as usize;
                from += size1;
                from += size2;
                from += size3;
                index += 3;
            }
            _ => {}
        }
    }
    Err("Measure not found in plugin input".to_string())
}

pub fn get_df_from_in_plugin(input: &PluginInput) -> Result<usize, String> {
    let mut from = 0;
    let mut index = 0;
    for i in 0..input.model.len() {
        match input.model[i] {
            ModelType::Meas => {
                if input.model_len.len() < index + 2 {
                    return Err(format!("model_len length error, expect more then {}, actual {}",
                                       index + 2, input.model_len.len()));
                }
                let size1 = input.model_len[index] as usize;
                let size2 = input.model_len[index + 1] as usize;
                from += size1;
                from += size2;
                index += 2;
            }
            ModelType::Island => {
                if input.model_len.len() < index + 3 {
                    return Err(format!("model_len length error, expect more then {}, actual {}",
                                       index + 3, input.model_len.len()));
                }
                let size1 = input.model_len[index] as usize;
                let size2 = input.model_len[index + 1] as usize;
                let size3 = input.model_len[index + 2] as usize;
                from += size1;
                from += size2;
                from += size3;
                index += 3;
            }
            _ => {}
        }
    }
    Ok(from)
}

// #[inline]
// pub fn get_wasm_result(output: PluginOutput) -> u64 {
//     // 下面的unwrap是必要的，否则输出的字节无法解析
//     let mut v = Vec::new();
//     ciborium::into_writer(&output, &mut v).unwrap();
//     v.shrink_to_fit();
//     let offset = v.as_ptr() as i32;
//     let len = v.len() as u32;
//     let mut bytes = BytesMut::with_capacity(8);
//     bytes.put_i32(offset);
//     bytes.put_u32(len);
//     return bytes.get_u64();
// }

#[inline]
pub fn get_csv_str(s: &str) -> String {
    if s.contains(',') || s.contains('\n') || s.contains('"')
        || s.starts_with(' ') || s.ends_with(' ') {
        format!("\"{}\"", s.replace('\"', "\"\""))
    } else {
        s.to_string()
    }
}



