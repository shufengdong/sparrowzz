use std::collections::HashMap;
#[cfg(feature = "tools")]
use std::collections::HashSet;

#[cfg(feature = "tools")]
use eig_aoe::aoe::AoeModel;
#[cfg(feature = "tools")]
use eig_aoe::check_loop_in_computing_points;
use eig_domain::{get_csv_str, PbAlarmDefine, PbAlarmDefine_AlarmLevel, PbAlarmDefines};
#[cfg(feature = "tools")]
use eig_domain::{Measurement, MINIMUM_AOE_ID, MINIMUM_POINT_ID, Transport};
pub use model::*;


pub mod model;

pub const HEADER_TOKEN: &str = "access-token";

// ============= 对eig包中的对象进行apidoc注释-开始

/**
 * @api {枚举_文件操作类型} /PbFile_FileOperation PbFile_FileOperation
 * @apiPrivate
 * @apiGroup A_Enum
 * @apiSuccess {String} UPDATE 更新
 * @apiSuccess {String} DELETE 删除
 */

/**
 * @api {枚举_告警等级} /PbAlarmDefine_AlarmLevel PbAlarmDefine_AlarmLevel
 * @apiPrivate
 * @apiGroup A_Enum
 * @apiSuccess {String} Emergency 紧急
 * @apiSuccess {String} Important 严重
 * @apiSuccess {String} Common 普通
 */

/**
 * @api {枚举_告警类型} /PbEigAlarm_AlarmType PbEigAlarm_AlarmType
 * @apiPrivate
 * @apiGroup A_Enum
 * @apiSuccess {String} invalidPoints 无效测点
 * @apiSuccess {String} invalidTransport 无效通道
 * @apiSuccess {String} invalidAOE 无效AOE
 * @apiSuccess {String} alarmLevel1 告警等级1
 * @apiSuccess {String} alarmLevel2 告警等级2
 * @apiSuccess {String} badData 坏数据
 * @apiSuccess {String} userDefine 用户自定义
 */

/**
 * @api {枚举_告警状态} /PbEigAlarm_AlarmStatus PbEigAlarm_AlarmStatus
 * @apiPrivate
 * @apiGroup A_Enum
 * @apiSuccess {String} occur occur
 * @apiSuccess {String} disappear disappear
 */

/**
 * @api {枚举_设点状态} /PbSetPointResult_SetPointStatus PbSetPointResult_SetPointStatus
 * @apiPrivate
 * @apiGroup A_Enum
 * @apiSuccess {String} YkCreated YkCreated
 * @apiSuccess {String} YtCreated YtCreated
 * @apiSuccess {String} YkSuccess YkSuccess
 * @apiSuccess {String} YtSuccess YtSuccess
 * @apiSuccess {String} YkFailTimeout YkFailTimeout
 * @apiSuccess {String} YtFailTimeout YtFailTimeout
 * @apiSuccess {String} YkFailTooBusy YkFailTooBusy
 * @apiSuccess {String} YtFailTooBusy YtFailTooBusy
 * @apiSuccess {String} YkFailProtocol YkFailProtocol
 * @apiSuccess {String} YtFailProtocol YtFailProtocol
 */

/**
 * @api {枚举_事件结果} /PbEventResult_EventEvalResult PbEventResult_EventEvalResult
 * @apiPrivate
 * @apiGroup A_Enum
 * @apiSuccess {String} Happen 已发生
 * @apiSuccess {String} NotHappen 未发生
 * @apiSuccess {String} Canceled 取消
 * @apiSuccess {String} Error 错误
 */

/**
 * @api {枚举_动作结果} /PbActionResult_ActionExeResult PbActionResult_ActionExeResult
 * @apiPrivate
 * @apiGroup A_Enum
 * @apiSuccess {String} NotRun 未执行
 * @apiSuccess {String} Success 执行成功
 * @apiSuccess {String} Failed 执行失败
 */

/**
 * @api {枚举_HTTP方法} /PbRequest_RequestType PbRequest_RequestType
 * @apiPrivate
 * @apiGroup A_Enum
 * @apiSuccess {String} Get Get
 * @apiSuccess {String} Post Post
 * @apiSuccess {String} Put Put
 * @apiSuccess {String} Delete Delete
 * @apiSuccess {String} Test Test
 */

/**
 * @api {PbRequest} /PbRequest PbRequest
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} [id] 请求id
 * @apiSuccess {String} url 请求url
 * @apiSuccess {PbRequest_RequestType} [function] 请求方法
 * @apiSuccess {String} content 请求体,base64
 */

/**
 * @api {PbResponse} /PbResponse PbResponse
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} [request_id] 请求id
 * @apiSuccess {bool} [is_ok] 是否成功
 * @apiSuccess {String} content 返回内容
 */

/**
 * @api {PbFile} /PbFile PbFile
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {String} fileName 文件名
 * @apiSuccess {u8[]} fileContent 文件内容
 * @apiSuccess {PbFile_FileOperation} [op] 操作类型
 * @apiSuccess {bool} [is_zip] 是否是压缩文件
 */

/**
 * @api {PbPointValues} /PbPointValues PbPointValues
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {PbDiscreteValue[]} dValues dValues
 * @apiSuccess {PbAnalogValue[]} aValues aValues
 */

/**
 * @api {PbDiscreteValue} /PbDiscreteValue PbDiscreteValue
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} [pointId] 测点id
 * @apiSuccess {i64} [measValue] 量测值
 * @apiSuccess {u64} [timestamp] 时间戳
 * @apiSuccess {i64} [origValue] 原始值
 */

/**
 * @api {PbAnalogValue} /PbAnalogValue PbAnalogValue
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} [pointId] 测点id
 * @apiSuccess {f64} [measValue] 量测值
 * @apiSuccess {u64} [timestamp] 时间戳
 * @apiSuccess {f64} [origValue] 原始值
 */

/**
 * @api {PbEigPingRes} /PbEigPingRes PbEigPingRes
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {String} id id
 * @apiSuccess {String} name 名称
 * @apiSuccess {String} ip ip
 * @apiSuccess {String} desc 描述
 * @apiSuccess {bool} [is_ems] is_ems
 */

/**
 * @api {告警定义集} /PbAlarmDefines PbAlarmDefines
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {PbAlarmDefine[]} defines 告警定义列表
 */

/**
 * @api {告警定义} /PbAlarmDefine PbAlarmDefine
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u32} [id] 告警定义id
 * @apiSuccess {String} rule 告警规则
 * @apiSuccess {PbAlarmDefine_AlarmLevel} [level] 告警等级
 * @apiSuccess {String} name 名称
 * @apiSuccess {String} desc 描述
 * @apiSuccess {String} owner owner
 */

/**
 * @api {告警结果集} /PbEigAlarms PbEigAlarms
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {PbEigAlarm[]} alarms 告警列表
 */

/**
 * @api {告警结果} /PbEigAlarm PbEigAlarm
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} [timestamp] 时间戳
 * @apiSuccess {u64} [id] 告警id
 * @apiSuccess {PbEigAlarm_AlarmType} [alarm_type] 告警类型
 * @apiSuccess {PbEigAlarm_AlarmStatus} [status] 告警状态
 * @apiSuccess {u32} [define_id] 告警定义id
 * @apiSuccess {String} content 告警内容
 */

/**
 * @api {设点结果集} /PbSetPointResults PbSetPointResults
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {PbSetPointResult[]} results 设点结果列表
 */

/**
 * @api {设点结果} /PbSetPointResult PbSetPointResult
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} [sender_id] sender_id
 * @apiSuccess {u64} [point_id] 测点id
 * @apiSuccess {u64} [create_time] 创建时间
 * @apiSuccess {u64} [finish_time] 完成时间
 * @apiSuccess {u64} [command] command
 * @apiSuccess {PbSetPointResult_SetPointStatus} [status] 状态
 */

/**
 * @api {AOE执行结果集} /PbAoeResults PbAoeResults
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {PbAoeResult[]} results AOE执行结果列表
 */

/**
 * @api {AOE执行结果} /PbAoeResult PbAoeResult
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} [aoe_id] AOE_id
 * @apiSuccess {u64} [start_time] 开始时间
 * @apiSuccess {u64} [end_time] 结束时间
 * @apiSuccess {PbEventResult[]} event_results 事件结果列表
 * @apiSuccess {PbActionResult[]} action_results 动作结果列表
 */

/**
 * @api {事件结果} /PbEventResult PbEventResult
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} [id] id
 * @apiSuccess {u64} [start_time] 开始时间
 * @apiSuccess {u64} [end_time] 结束时间
 * @apiSuccess {PbEventResult_EventEvalResult} [final_result] 事件结果
 */

/**
 * @api {动作结果} /PbActionResult PbActionResult
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} [source_id] 源节点id
 * @apiSuccess {u64} [target_id] 目标节点id
 * @apiSuccess {u64} [start_time] 开始时间
 * @apiSuccess {u64} [end_time] 结束时间
 * @apiSuccess {PbActionResult_ActionExeResult} [final_result] 动作结果
 * @apiSuccess {u32} [fail_code] 失败code
 * @apiSuccess {u64[]} yk_points yk_points
 * @apiSuccess {i64[]} yk_values yk_values
 * @apiSuccess {u64[]} yt_points yt_points
 * @apiSuccess {f64[]} yt_values yt_values
 * @apiSuccess {String[]} variables variables
 * @apiSuccess {f64[]} var_values var_values
 */
// ============= 对eig包中的对象进行apidoc注释-结束

pub const DB_NAME: &str = "eig_model";
// 历史库维持在1G
pub const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
pub const DB_NAME_FORMAT: &str = "%Y/%m-%d";
pub const OPERATION_RECEIVE_BUFF_NUM: usize = 100;
// max items for one query, only for history data
pub const ONE_QUERY_LIMIT: usize = 10000;

pub fn export_alarm_def_csv2(defines: &PbAlarmDefines, text_map: &HashMap<String, String>) -> String {
    let vec: Vec<&PbAlarmDefine> = defines.defines.iter().collect();
    export_alarm_def_csv(vec.as_slice(), text_map)
}

pub fn export_alarm_def_csv(defines: &[&PbAlarmDefine], text_map: &HashMap<String, String>) -> String {
    // 表格抬头
    let mut result = format!(
        "{},{},{},{},{},{},{}\n",
        text_map.get("index").unwrap_or(&"Index".to_string()),
        text_map.get("alarm_define_id").unwrap_or(&"ID".to_string()),
        text_map.get("name").unwrap_or(&"Name".to_string()),
        text_map.get("rule").unwrap_or(&"Rule".to_string()),
        text_map.get("level").unwrap_or(&"Level".to_string()),
        text_map.get("desc").unwrap_or(&"Description".to_string()),
        text_map.get("owner").unwrap_or(&"Owner".to_string()),
    );
    //生成表格内容
    for i in 0..defines.len() {
        let d = &defines[i];
        let level = match d.level() {
            PbAlarmDefine_AlarmLevel::Emergency => "Emergency",
            PbAlarmDefine_AlarmLevel::Important => "Important",
            PbAlarmDefine_AlarmLevel::Common => "Common",
        };
        result += &format!(
            "{},{},{},\"{}\",{},{},{}\n",
            i + 1,
            d.id(),
            get_csv_str(d.name()),
            get_csv_str(d.rule()),
            level,
            get_csv_str(d.desc()),
            get_csv_str(d.owners())
        );
    }

    result
}

#[cfg(feature = "tools")]
fn check_points(points: &[Measurement]) -> bool {
    let mut map: HashMap<u64, Measurement> = HashMap::with_capacity(points.len());
    // 检查ID，若有重复则提示用户确认
    for p in points {
        if let std::collections::hash_map::Entry::Vacant(e) = map.entry(p.point_id) {
            e.insert(p.clone());
        } else {
            log::warn!("!!Check not pass: Same point id found: {}", p.point_id);
            return false;
        }
        if p.point_id < MINIMUM_POINT_ID {
            log::warn!("!!Check not pass: Point id is too small: {}", p.point_id);
            return false;
        }
    }

    // check alias name
    let mut alias_to_id: HashMap<String, u64> = HashMap::with_capacity(map.len());
    for m in map.values() {
        if !m.alias_id.is_empty() {
            let key = m.alias_id.clone();
            if let Some(item) = alias_to_id.get(&key) {
                log::warn!("!!Check not pass: Point {} and Point {} have same alias {}",
                    item, m.point_id, key);
                return false;
            }
            alias_to_id.insert(key, m.point_id);
        }
        // check lower limit and upper limit
        if m.lower_limit > m.upper_limit {
            log::warn!("!!Check not pass: The lower limit cannot exceed the upper limit");
            return false;
        }
        // check computing point's expression is not blank
        if m.is_computing_point && m.expression.is_empty() {
            log::warn!("!!Check not pass: Computing pint expression is null, id {}", m.point_id);
            return false;
        }
    }
    // check whether there is a loop in computing points
    if let Some(point_id) = check_loop_in_computing_points(&map, &alias_to_id) {
        log::warn!("!!There is loop in computing points, id {}", point_id);
        return false;
    }
    true
}

#[cfg(feature = "tools")]
fn check_transports(tps: &[Transport], all_points: &HashMap<u64, Measurement>,
                    all_serials: &Vec<String>) -> bool {
    use eig_domain::utils::check_transport;

    let mut occupied_paths = HashSet::with_capacity(tps.len());
    let mut remote_points = HashSet::with_capacity(tps.len());
    for tp in tps {
        if let Err(alarm) = check_transport(tp, all_points, &mut occupied_paths,
                                            &mut remote_points, all_serials) {
            log::warn!("!!Check not pass: {}", alarm.content());
            return false;
        }
    }
    true
}

#[cfg(feature = "tools")]
fn check_aoes(aoes: &[AoeModel], tp_ids: &HashSet<u64>) -> bool {
    use eig_aoe::aoe::model::check_trigger_type;

    let mut ids: HashSet<u64> = HashSet::with_capacity(aoes.len());
    for aoe in aoes {
        // check id
        if !ids.contains(&aoe.id) && !tp_ids.contains(&aoe.id) {
            ids.insert(aoe.id);
        } else {
            log::warn!("!!Check not pass: ID {} already exists", aoe.id);
            return false;
        }
        if aoe.id != 0 && aoe.id < MINIMUM_AOE_ID {
            log::warn!("!!Check not pass: AOE ID should be greater than {}", MINIMUM_AOE_ID);
            return false;
        }
        // check trigger type
        if !check_trigger_type(&aoe.trigger_type).is_empty() {
            return false;
        }
        // check for duplicate variable definitions
        for i in 0..aoe.variables.len() {
            let var_now = aoe.variables[i].0.clone();
            for j in (i + 1)..aoe.variables.len() {
                if var_now.eq(&aoe.variables[j].0) {
                    log::warn!("Check not pass: AOE variable {} has duplicate definition", var_now);
                    return false;
                }
            }
        }
    }
    true
}