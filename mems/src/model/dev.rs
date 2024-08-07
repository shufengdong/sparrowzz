use std::cmp::PartialOrd;
use std::collections::HashMap;
use std::fmt;

use std::hash::Hash;

use serde::{Deserialize, Serialize};

use eig_domain::prop::*;

/**
 * @api {枚举_电力设备类型} /PsRsrType PsRsrType
 * @apiPrivate
 * @apiGroup A_Enum
 * @apiSuccess {String} Switch Switch
 * @apiSuccess {String} Busbar Busbar
 * @apiSuccess {String} ACline ACline
 * @apiSuccess {String} Dcline Dcline
 * @apiSuccess {String} Winding Winding
 * @apiSuccess {String} SyncGenerator SyncGenerator
 * @apiSuccess {String} ESS ESS
 * @apiSuccess {String} PCS PCS
 * @apiSuccess {String} Transformer Transformer
 * @apiSuccess {String} Load Load
 * @apiSuccess {String} ShuntCompensator ShuntCompensator
 * @apiSuccess {String} SerialCompensator SerialCompensator
 * @apiSuccess {String} Feeder Feeder
 * @apiSuccess {String} Cable Cable
 * @apiSuccess {String} Regulator Regulator
 * @apiSuccess {String} Connector Connector
 * @apiSuccess {String} Company Company
 * @apiSuccess {String} SubIsland SubIsland
 * @apiSuccess {String} LoadArea LoadArea
 * @apiSuccess {String} Substation Substation
 * @apiSuccess {String} PowerPlant PowerPlant
 * @apiSuccess {String} VoltageLevel VoltageLevel
 * @apiSuccess {String} BaseVoltage BaseVoltage
 * @apiSuccess {String} UserDefine1 UserDefine1
 * @apiSuccess {String} UserDefine2 UserDefine2
 * @apiSuccess {String} UserDefine3 UserDefine3
 * @apiSuccess {String} UserDefine4 UserDefine4
 * @apiSuccess {String} UserDefine5 UserDefine5
 * @apiSuccess {String} UserDefine6 UserDefine6
 * @apiSuccess {String} UserDefine7 UserDefine7
 * @apiSuccess {String} UserDefine8 UserDefine8
 * @apiSuccess {String} UserDefine9 UserDefine9
 * @apiSuccess {String} UserDefine10 UserDefine10
 * @apiSuccess {String} Unknown Unknown
 */
/// 电力设备类型枚举
#[repr(u16)]
#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug, Copy, Hash)]
pub enum PsRsrType {
    // 输电网络
    Switch = 1,
    Busbar = 2,
    ACline = 3,
    DCline = 4,
    Winding = 5,
    SyncGenerator = 6,
    ESS = 7,
    PCS = 8,
    Transformer = 9,
    Load = 10,
    ShuntCompensator = 11,
    SerialCompensator = 12,
    // 配电网
    Feeder = 16,
    Cable,
    Regulator,
    Connector,
    // container
    Company = 10000,
    SubIsland,
    LoadArea,
    Substation,
    PowerPlant,
    VoltageLevel,
    BaseVoltage,

    // 自定义
    UserDefine1 = 30001,
    UserDefine2,
    UserDefine3,
    UserDefine4,
    UserDefine5,
    UserDefine6,
    UserDefine7,
    UserDefine8,
    UserDefine9,
    UserDefine10,
    Unknown = 65535,
}

impl From<&str> for PsRsrType {
    fn from(value: &str) -> Self {
        match value {
            "Switch" => PsRsrType::Switch,
            "Busbar" => PsRsrType::Busbar,
            "ACline" => PsRsrType::ACline,
            "DCline" => PsRsrType::DCline,
            "Winding" => PsRsrType::Winding,
            "SyncGenerator" => PsRsrType::SyncGenerator,
            "ESS" => PsRsrType::ESS,
            "PCS" => PsRsrType::PCS,
            "Transformer" => PsRsrType::Transformer,
            "Load" => PsRsrType::Load,
            "ShuntCompensator" => PsRsrType::ShuntCompensator,
            "SerialCompensator" => PsRsrType::SerialCompensator,
            "Feeder" => PsRsrType::Feeder,
            "Cable" => PsRsrType::Cable,
            "Regulator" => PsRsrType::Regulator,
            "Connector" => PsRsrType::Connector,
            "Company" => PsRsrType::Company,
            "SubIsland" => PsRsrType::SubIsland,
            "LoadArea" => PsRsrType::LoadArea,
            "Substation" => PsRsrType::Substation,
            "PowerPlant" => PsRsrType::PowerPlant,
            "VoltageLevel" => PsRsrType::VoltageLevel,
            "BaseVoltage" => PsRsrType::BaseVoltage,
            "UserDefine1" => PsRsrType::UserDefine1,
            "UserDefine2" => PsRsrType::UserDefine2,
            "UserDefine3" => PsRsrType::UserDefine3,
            "UserDefine4" => PsRsrType::UserDefine4,
            "UserDefine5" => PsRsrType::UserDefine5,
            "UserDefine6" => PsRsrType::UserDefine6,
            "UserDefine7" => PsRsrType::UserDefine7,
            "UserDefine8" => PsRsrType::UserDefine8,
            "UserDefine9" => PsRsrType::UserDefine9,
            "UserDefine10" => PsRsrType::UserDefine10,
            _ => PsRsrType::Unknown,
        }
    }
}

impl From<String> for PsRsrType {
    fn from(value: String) -> Self {
        PsRsrType::from(value.as_str())
    }
}

impl PsRsrType {
    /// 用于遍历所有设备类型列表
    pub const PS_DEV_TYPE: [PsRsrType; 34] = [
        PsRsrType::Switch,
        PsRsrType::Busbar,
        PsRsrType::ACline,
        PsRsrType::DCline,
        PsRsrType::Winding,
        PsRsrType::SyncGenerator,
        PsRsrType::ESS,
        PsRsrType::PCS,
        PsRsrType::Transformer,
        PsRsrType::Load,
        PsRsrType::ShuntCompensator,
        PsRsrType::SerialCompensator,
        PsRsrType::Feeder,
        PsRsrType::Cable,
        PsRsrType::Regulator,
        PsRsrType::Connector,
        PsRsrType::Company,
        PsRsrType::SubIsland,
        PsRsrType::LoadArea,
        PsRsrType::Substation,
        PsRsrType::PowerPlant,
        PsRsrType::VoltageLevel,
        PsRsrType::BaseVoltage,
        PsRsrType::UserDefine1,
        PsRsrType::UserDefine2,
        PsRsrType::UserDefine3,
        PsRsrType::UserDefine4,
        PsRsrType::UserDefine5,
        PsRsrType::UserDefine6,
        PsRsrType::UserDefine7,
        PsRsrType::UserDefine8,
        PsRsrType::UserDefine9,
        PsRsrType::UserDefine10,
        PsRsrType::Unknown,
    ];
}

/// 将枚举转换成字符串，调用to_string()方法
impl fmt::Display for PsRsrType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Copy, Clone, Hash)]
pub enum MeasPhase {
    Total,
    A,
    B,
    C,
    A0,
    B0,
    C0,
    CT,
    PT,
    Unknown,
}

impl From<&str> for MeasPhase {
    fn from(value: &str) -> Self {
        match value.to_uppercase().as_str() {
            "TOTAL" => MeasPhase::Total,
            "A" => MeasPhase::A,
            "B" => MeasPhase::B,
            "C" => MeasPhase::C,
            "A0" => MeasPhase::A0,
            "B0" => MeasPhase::B0,
            "C0" => MeasPhase::C0,
            "CT" => MeasPhase::CT,
            "PT" => MeasPhase::PT,
            _ => MeasPhase::Unknown,
        }
    }
}

impl From<String> for MeasPhase {
    fn from(value: String) -> Self {
        MeasPhase::from(value.as_str())
    }
}

impl fmt::Display for MeasPhase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/**
 * @api {属性定义} /PropDefine PropDefine
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} id 属性id
 * @apiSuccess {String} name 属性定义标识
 * @apiSuccess {String} desc 属性定义描述
 * @apiSuccess {PropType} data_type 属性类型
 * @apiSuccess {DataUnit} data_unit 属性单位
 */
/// 设备属性
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct PropDefine {
    /// 设备属性定义id
    pub id: u64,
    /// 属性定义标识
    pub name: String,
    /// 属性定义描述
    pub desc: String,
    /// 属性类型
    pub data_type: PropType,
    /// 属性单位
    pub data_unit: DataUnit,
}

/**
 * @api {属性分组定义} /PropGroupDefine PropGroupDefine
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {String} name 属性分组定义标识
 * @apiSuccess {String} desc 属性分组定义描述
 * @apiSuccess {u64[]} prop_defines 属性定义id列表
 */
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct PropGroupDefine {
    /// 属性定义标识
    pub name: String,
    /// 属性定义描述
    pub desc: String,
    /// 设备属性实际描述
    pub prop_defines: Vec<u64>,
}

/**
 * @api {设备属性分组} /RsrPropGroup RsrPropGroup
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} id 设备属性分组id
 * @apiSuccess {u64} rsr_id 资源id
 * @apiSuccess {String} name 分组名称，用于显示，以及匹配PropGroupDefine
 * @apiSuccess {u64[]} defines 设备属性定义列表
 * @apiSuccess {PropValue[]} props 设备属性值列表
 */
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct RsrPropGroup {
    pub id: u64,
    /// resource id
    pub rsr_id: u64,
    /// 分组名称，用于显示，以及匹配PropGroupDefine
    pub name: String,
    /// 设备属性定义列表
    pub defines: Vec<u64>,
    /// 设备属性实际描述
    pub props: Vec<PropValue>,
}

/**
 * @api {设备定义} /RsrDefine RsrDefine
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} id 定义id
 * @apiSuccess {PsRsrType} rsr_type 设备所属类型
 * @apiSuccess {String} name 设备类别名称
 * @apiSuccess {String} desc 设备定义的描述
 * @apiSuccess {u8} terminal_num 端口数量
 * @apiSuccess {PropGroupDefine[]} prop_groups 属性分组定义列表
 */
/// 设备定义
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct RsrDefine {
    /// 定义id
    pub id: u64,
    /// 设备所属类型
    pub rsr_type: PsRsrType,
    /// 设备类别名称
    pub name: String,
    /// 设备定义的描述
    pub desc: String,
    /// 端口数量
    pub terminal_num: u8,
    /// 设备属性
    pub prop_groups: Vec<PropGroupDefine>,
}

/**
 * @api {设备对象} /NetworkRsr NetworkRsr
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} id 设备id
 * @apiSuccess {u64} define_id 设备定义id
 * @apiSuccess {String} name 设备名称
 * @apiSuccess {String} desc 设备描述
 * @apiSuccess {u64} [container_id] 容器id
 * @apiSuccess {Terminal[]} terminals 端子列表
 * @apiSuccess {u64[]} prop_groups 设备属性分组，RsrPropGroup对象的id列表
 */
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct NetworkRsr {
    /// 设备id
    pub id: u64,
    /// 设备定义id
    pub define_id: u64,
    /// 设备名称
    pub name: String,
    /// 设备描述
    pub desc: String,
    // container id
    pub container_id: Option<u64>,
    /// 设备的端口
    pub terminals: Vec<Terminal>,
    /// 设备属性分组id列表
    pub prop_group_ids: Vec<u64>,
}

/**
 * @api {端口} /Terminal Terminal
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} device 设备id
 * @apiSuccess {u64} id 端口id
 */
//端口
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Terminal {
    pub device: u64,
    pub id: u64,
}

/**
 * @api {连接节点} /CN CN
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {Terminal[]} terminals 端子列表
 */
//Connective Node
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct CN {
    pub id: u64,
    pub terminals: Vec<u64>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Hash, Clone, Copy, PartialOrd)]
pub enum TNType {
    // 源节点
    Source,
    // 分布式电源
    DG,
    // 负荷节点
    Load,
    // 联络节点
    Link,
    None,
}

//Topology Node
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct TN {
    pub cns: Vec<CN>,
    pub tn_type: TNType,
    pub dev_ids: Vec<u64>,
}

impl TN {
    pub fn add_cn(&mut self, cn: CN) {
        self.cns.push(cn);
    }
    pub fn add_dev(&mut self, dev_id: u64) {
        self.dev_ids.push(dev_id);
    }
    // 合并两个TN
    pub fn merge(&mut self, other: TN) {
        self.cns.extend(other.cns);
        self.dev_ids.extend(other.dev_ids);
        if other.tn_type < self.tn_type {
            self.tn_type = other.tn_type;
        }
    }
    pub fn set_type(&mut self, tn_type: TNType) {
        if tn_type < self.tn_type {
            self.tn_type = tn_type;
        }
    }
}

/**
 * @api {测点定义} /MeasureDef MeasureDef
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} id 测点定义id
 * @apiSuccess {u64} point_id 测点id
 * @apiSuccess {u64} terminal_id 测点所属的端口的id
 * @apiSuccess {u64} dev_id 测点所属的设备的id
 */
// 测点定义
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MeasureDef {
    // 测点定义id
    pub id: u64,
    // 测点id
    pub point_id: u64,
    // 测点所属的端口的id
    pub terminal_id: u64,
    // 测点所属的设备的id
    pub dev_id: u64,
    // 测点phase
    pub phase: MeasPhase,
}

impl MeasureDef {
    pub fn get_csv_header(text_map: &HashMap<String, String>) -> String {
        format!(
            "{},{},{},{},{}",
            text_map.get("index").unwrap_or(&"Index".to_string()),
            text_map.get("id").unwrap_or(&"ID".to_string()),
            text_map.get("point_id").unwrap_or(&"Point ID".to_string()),
            text_map.get("dev_id").unwrap_or(&"Dev ID".to_string()),
            text_map.get("terminal_id").unwrap_or(&"Terminal ID".to_string()),
        )
    }

    pub fn to_csv_str(&self) -> String {
        format!("{},{},{},{}", self.id, self.point_id, self.dev_id, self.terminal_id)
    }
}

/**
 * @api {电气岛} /Island Island
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {Map} resources 资源，HashMap<id:u64, NetworkRsr>
 * @apiSuccess {Map} measures 测点，HashMap<id:u64, MeasureDef[]>
 * @apiSuccess {Map} prop_groups 属性分组，HashMap<id:u64, RsrPropGroup>
 * @apiSuccess {CN[]} cns 连接节点列表
 */
//电气岛，即集合
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Island {
    // key: dev_id
    pub resources: HashMap<u64, NetworkRsr>,
    // key: dev_id
    pub measures: HashMap<u64, Vec<MeasureDef>>,
    // key: prop_group_id
    pub prop_groups: HashMap<u64, RsrPropGroup>,
    pub cns: Vec<CN>,
}


impl NetworkRsr {
    // 获取设备类型
    pub fn get_rsr_type(&self, dev_defs: &HashMap<u64, RsrDefine>) -> PsRsrType {
        if let Some(def) = dev_defs.get(&self.define_id) {
            return def.rsr_type;
        } else {
            PsRsrType::Unknown
        }
    }

    // 获取具体设备的具体属性值
    pub fn get_prop_value(&self, prop_name: &str, prop_groups: &HashMap<u64, RsrPropGroup>,
                          prop_defs: &HashMap<u64, PropDefine>) -> PropValue {
        for prop_group_id in &self.prop_group_ids {
            if let Some(rpg) = prop_groups.get(prop_group_id) {
                for i in 0..rpg.defines.len() {
                    if let Some(prop_def) = prop_defs.get(&rpg.defines[i]) {
                        if prop_def.name == prop_name {
                            return rpg.props[i].clone();
                        }
                    }
                }
            }
        }
        PropValue::Unknown
    }

    pub fn get_prop_value2(&self, prop_name: &str, prop_groups: &HashMap<u64, RsrPropGroup>,
                           prop_defs: &HashMap<u64, &PropDefine>) -> PropValue {
        for prop_group_id in &self.prop_group_ids {
            if let Some(rpg) = prop_groups.get(prop_group_id) {
                for i in 0..rpg.defines.len() {
                    if let Some(prop_def) = prop_defs.get(&rpg.defines[i]) {
                        if prop_def.name == prop_name {
                            return rpg.props[i].clone();
                        }
                    }
                }
            }
        }
        PropValue::Unknown
    }
}