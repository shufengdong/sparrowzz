use serde::{Deserialize, Serialize};

use eig_aoe::aoe::AoeModel;
use eig_domain::*;

// request and response topic
pub const TOPIC_REGISTER: &str = "register";
pub const TOPIC_WEB_REQUEST: &str = "request";
pub const TOPIC_WEB_RESPONSE: &str = "response";
// real time message topic
pub const REGISTER_REALTIME_MSG_TOPIC: &str = "registerRt";
pub const UPDATE_REALTIME_MSG_TOPIC: &str = "updateRt";

pub const TAG_GROUP_POINT: u8 = 1;

// 管理员为初始化用户，且不允许删除和修改ID，故为了方便判断，直接约定管理员ID为1
pub const USER_ADMIN_ID: u16 = 1;

/**
 * @apiDefine HisQuery
 * @apiQuery {String} [id] 测点id，多个id之间以,间隔
 * @apiQuery {u64} [start] 开始时间, 13位时间戳
 * @apiQuery {u64} [end] 结束时间, 13位时间戳，（start、end） 如果仅设置1个参数，则查询范围为start-当天结束 或 当天开始-end
 * @apiQuery {String} [date] 时间字符串，yyyy-MM-dd， （start、end）、date参数至少设定1个，如果同时设定，则以start、end为准。
 */
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct HisQuery {
    pub id: Option<String>,
    pub start: Option<u64>,
    pub end: Option<u64>,
    pub date: Option<String>,
    pub source: Option<u32>,
    pub last_only: Option<bool>,
    pub with_init: Option<bool>,
}

/**
 * @apiDefine HisSetPointQuery
 * @apiQuery {u64} [sender_id] sender_id
 * @apiQuery {u64} [point_id] 测点id
 * @apiQuery {u64} [start] 开始时间
 * @apiQuery {u64} [end] 结束时间
 * @apiQuery {String} [date] 时间字符串，yyyy-MM-dd
 */
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HisSetPointQuery {
    pub sender_id: Option<u64>,
    pub point_id: Option<u64>,
    pub start: Option<u64>,
    pub end: Option<u64>,
    pub date: Option<String>,
}

impl HisQuery {
    pub fn query_str(&self) -> String {
        let mut query_s = "?".to_string();
        if let Some(ids) = &self.id {
            query_s += &format!("id={}&", ids);
        }
        if let Some(start) = &self.start {
            query_s += &format!("start={}&", start);
        }
        if let Some(end) = &self.end {
            query_s += &format!("end={}&", end);
        }
        if let Some(date) = &self.date {
            query_s += &format!("date={}&", date);
        }
        if let Some(source) = &self.source {
            query_s += &format!("source={}&", source);
        }
        if let Some(last_only) = &self.last_only {
            query_s += &format!("last_only={}&", last_only);
        }
        if let Some(last_only) = &self.with_init {
            query_s += &format!("with_init={}", last_only);
        }
        if query_s.ends_with('&') {
            query_s[0..query_s.len() - 1].to_string()
        } else {
            query_s
        }
    }
}

impl HisSetPointQuery {
    pub fn query_str(&self) -> String {
        let mut query_s = "?".to_string();
        if let Some(point_id) = &self.point_id {
            query_s += &format!("point_id={}&", point_id);
        }
        if let Some(serder) = &self.sender_id {
            query_s += &format!("sender_id={}&", serder);
        }
        if let Some(start) = &self.start {
            query_s += &format!("start={}&", start);
        }
        if let Some(end) = &self.end {
            query_s += &format!("end={}&", end);
        }
        if let Some(date) = &self.date {
            query_s += &format!("date={}&", date);
        }
        if query_s.ends_with('&') {
            query_s[0..query_s.len() - 1].to_string()
        } else {
            query_s
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AoeQuery {
    pub id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PointsQuery {
    pub id: Option<String>,
    pub name: Option<String>,
}

// todo: api doc needed
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LogQuery {
    pub is_query_size: Option<bool>,
}

impl LogQuery {
    pub fn query_str(&self) -> String {
        let mut query_s = "?".to_string();
        if let Some(b) = &self.is_query_size {
            query_s += &format!("is_query_size={}", b);
            query_s
        } else {
            "".to_string()
        }
    }
}

/**
 * @api {EigRtRegister} /EigRtRegister EigRtRegister
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {String} listener_id 监听者，一般是自己的bee id, 界面使用时可以设为空，因为有socket session id作为唯一标志
 * @apiSuccess {String[]} lcc_id 对应的设备id，可以多个，注意plcc或mems的id都可以
 * @apiSuccess {u64[][]} measure 每个设备上的测点, 0 means all
 * @apiSuccess {bool[]} alarm 每个设备的告警
 * @apiSuccess {bool[]} aoe 每个设备的AOE
 * @apiSuccess {bool[]} command 每个设备的指令
 * @apiSuccess {bool[]} log 每个设备的log
 */
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EigRtRegister {
    /// 监听者，一般是自己的bee id, 界面使用时可以设为空，因为有socket session id作为唯一标志
    pub listener_id: String,
    // 对应的设备id，可以多个，注意plcc或mems的id都可以
    pub lcc_id: Vec<String>,
    /// 每个设备上的测点, vec![0] means all
    pub measure: Vec<Vec<(u32, Vec<u64>)>>,
    /// 每个设备的告警
    pub alarm: Vec<bool>,
    /// 每个设备的AOE
    pub aoe: Vec<bool>,
    /// 每个设备的指令
    pub command: Vec<bool>,
    /// 每个设备的log
    pub log: Vec<bool>,
}


/**
 * @api {UserPub} /UserPub UserPub
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u16} id 用户id
 * @apiSuccess {String} name 用户名
 * @apiSuccess {String} [desc] 描述
 * @apiSuccess {u16} user_group 所属用户组的id
 * @apiSuccess {u16[]} special_role 用户所属角色
 * @apiSuccess {String} [phone_number] 用户的手机号
 * @apiSuccess {String} [email] 用户的邮箱
 * @apiSuccess {u64} [expiration_time] 过期时间
 */
//用户 - 公开信息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserPub {
    //用户ID
    pub id: u16,
    //用户名称
    pub name: String,
    //描述
    pub desc: Option<String>,
    //所属用户组的id（用户与用户组关联表，一个用户只能属于一个用户组）
    pub user_group: u16,
    //特别分配的角色
    pub special_role: Vec<u16>,
    //用户的手机号
    pub phone_number: Option<String>,
    //用户的邮箱
    pub email: Option<String>,
    //过期时间
    pub expiration_time: Option<u64>,
}

/**
 * @api {User} /User User
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {UserPub} userpub 用户公开信息
 * @apiSuccess {u8[]} password 用户密码信息
 * @apiSuccess {u64} password_update_time 最近一次密码修改时间
 */
//用户 - 全部信息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    //可查询的用户公开信息
    pub pub_info: UserPub,
    //加密后的用户密码
    pub password: Vec<u8>,
    //最近一次密码修改时间
    pub password_update_time: u64,
}

/**
 * @api {UserGroup} /UserGroup UserGroup
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u16} id 用户组id
 * @apiSuccess {String} name 用户组名
 * @apiSuccess {u16[]} user_group2role 用户组关联的角色列表
 */
//用户组
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserGroup {
    //用户组ID
    pub id: u16,
    //用户组名称
    pub name: String,
    //用户组角色关联表，一个用户组可以拥有多个角色
    pub user_group2role: Vec<u16>,
}

/**
 * @api {Role} /Role Role
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u16} id 角色id
 * @apiSuccess {String} name 角色名
 * @apiSuccess {u16[]} role2authority 角色关联的权限列表
 * @apiSuccess {u16[]} role2menu 角色关联的菜单列表
 */
//角色
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Role {
    //角色ID
    pub id: u16,
    //角色名称
    pub name: String,
    //角色权限关联表，一个角色可以拥有多个权限
    pub role2authority: Vec<u16>,
    //角色菜单关联表，一个角色可以拥有多个菜单
    pub role2menu: Vec<u16>,
}

/**
 * @api {Authority} /Authority Authority
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u16} id 权限id
 * @apiSuccess {String} name 权限名
 * @apiSuccess {String} desc 权限描述
 * @apiSuccess {String} method 请求方法
 * @apiSuccess {String} url 权限可操作的url资源地址
 */
//权限
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Authority {
    //权限ID
    pub id: u16,
    //权限名称
    pub name: String,
    //描述
    pub desc: String,
    //请求方法
    pub method: String,
    //权限可操作的url资源地址
    pub url: String,
}

/**
 * @api {Menuitem} /Menuitem Menuitem
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u16} id 菜单id
 * @apiSuccess {String} name 菜单名
 * @apiSuccess {String} group 菜单分组
 * @apiSuccess {String} url 菜单对应的url地址
 */
//菜单
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Menuitem {
    //菜单ID
    pub id: u16,
    //名称
    pub name: String,
    //分组
    pub group: String,
    //菜单对应的url地址
    pub url: String,
}

/**
 * @api {告警通知形式} /AlarmNoticeSetting AlarmNoticeSetting
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {bool} popup_window 桌面弹窗
 * @apiSuccess {bool} sound_light 声光
 * @apiSuccess {bool} text_messages 短信
 */
// 告警通知形式
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AlarmNoticeSetting {
    //桌面弹窗
    pub popup_window: bool,
    //声光
    pub sound_light: bool,
    //短信
    pub text_messages: bool,
}

/**
 * @api {告警通知配置} /AlarmConfig AlarmConfig
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {AlarmNoticeSetting} emergency 紧急
 * @apiSuccess {AlarmNoticeSetting} important 严重
 * @apiSuccess {AlarmNoticeSetting} common 普通
 */
// 告警通知结构体
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AlarmConfig {
    pub emergency: AlarmNoticeSetting,
    pub important: AlarmNoticeSetting,
    pub common: AlarmNoticeSetting,
}

impl PointsQuery {
    pub fn query_str(&self) -> String {
        let mut query_s = "?".to_string();
        if let Some(ids) = &self.id {
            query_s += &format!("id={}&", ids);
        }
        if let Some(name) = &self.name {
            query_s += &format!("name={}&", name);
        }
        if query_s.ends_with('&') {
            query_s[0..query_s.len() - 1].to_string()
        } else {
            query_s
        }
    }
}

/**
 * @api {PointControl} /PointControl PointControl
 * @apiGroup A_Object
 * @apiSuccess {SetIntValue[]} discretes discretes
 * @apiSuccess {SetFloatValue[]} analogs analogs
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PointControl {
    pub discretes: Vec<SetIntValue>,
    pub analogs: Vec<SetFloatValue>,
}

/**
 * @api {AoeAction} /AoeAction AoeAction
 * @apiGroup A_Enum
 * @apiSuccess {Object} StartAoe 开始AOE，{"StartAoe": u64}
 * @apiSuccess {Object} StopAoe 停止AOE，{"StopAoe": u64}
 * @apiSuccess {Object} UpdateAoe 更新AOE，{"UpdateAoe": AoeModel}
 * @apiSuccess {Object} UpdateAoeCsv 更新AOE（字节数组），{"UpdateAoeCsv": u8[]}
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AoeAction {
    StartAoe(u64),
    StopAoe(u64),
    UpdateAoe(AoeModel),
    UpdateAoeCsv(Vec<u8>),
}

/**
 * @api {AoeControl} /AoeControl AoeControl
 * @apiGroup A_Object
 * @apiSuccess {AoeAction[]} AoeActions AOE指令列表
 */
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AoeControl {
    pub AoeActions: Vec<AoeAction>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum HisRequest {
    // 查询历史数据操作
    QueryMeasures(HisQuery),
    QuerySoes(HisQuery),
    QueryAoeResults(HisQuery),
    QueryAlarms(HisQuery),
    QueryUnconfirmedAlarms,
    QuerySetPointResults(HisSetPointQuery),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AuthRequest {
    CheckLogin(String, Vec<u8>),
    QueryUsers,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum CommonRequest {
    KvRequest(KvOperation),
    QueryIdsWithTag(u8, Vec<u16>),
    QueryTagDefs(u8),
    UpdateTags(u8, Vec<(String, Vec<u64>)>),
    DeleteTags(u8, Vec<(u16, u64)>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ControlRequest {
    Point(PointControl),
    Aoe(AoeControl),
    Reset,
    Recover,
    QuitForce,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum StatusRequest {
    RunningAoes,
    UnrunAoes,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EmsBroadcast {
    pub ip: String,
    pub rt_listen_port: u16,
    pub tcp_listen_port: u16,
}

/**
 * @api {枚举_键值对操作} /KvOperation KvOperation
 * @apiPrivate
 * @apiGroup A_Enum
 * @apiSuccess {Object} Query 查询，{"Query": u8[]}
 * @apiSuccess {Object} Update 更新，{"Update": tuple(u8[], u8[])}
 * @apiSuccess {Object} Delete 删除，{"Delete": u8[]}
 */
// 键值对操作
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum KvOperation {
    //查询
    Query(Vec<u8>),
    //增加
    Update(Vec<u8>, Vec<u8>),
    Update2(Vec<u8>, Vec<u8>, Vec<u8>),
    //删除
    Delete(Vec<u8>),
}

/**
 * @api {Lcc设备信息} /LccDevice LccDevice
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {String} lcc_id lcc_id
 * @apiSuccess {String} name lcc名称
 * @apiSuccess {String} desc lcc描述
 * @apiSuccess {String} ip lcc_ip
 * @apiSuccess {bool} is_ems 是否是ems
 */
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct LccDevice {
    pub lcc_id: String,
    pub name: String,
    pub desc: String,
    pub ip: String,
    pub is_ems: bool,
}


/**
 * @api {CommitNote} /CommitNote CommitNote
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u32} version 版本号
 * @apiSuccess {String} note 提交时的注释
 * @apiSuccess {String} tree_id 对应的tree_id
 */
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommitNote {
    // 版本号
    pub version: u32,
    // 提交时的注释
    pub note: String,
    // 对应的tree_id
    pub tree_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VersionData<T> {
    // 提交的信息
    pub note: CommitNote,
    // 对应的数据
    pub data: T,
}
