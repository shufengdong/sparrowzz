use std::cmp::PartialOrd;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::hash::Hash;
use std::io::Write;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use eig_domain::{csv_f64, csv_str, csv_string, csv_u64, csv_u8, DataUnit, Measurement, MeasureValue, prop::*};
use eig_domain::excel::transfer_to_utf8;

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

pub fn write_to_file(path: &str, content: &[u8]) {
    let mut file = File::create(path).unwrap();
    file.write_all(content).unwrap();
    file.flush().unwrap();
}

pub fn prop_def_from_csv(path: &str) -> Result<Vec<PropDefine>, (usize, usize)> {
    let content = std::fs::read(path).map_err(|_| (0, 0))?;
    // if env::IS_ENCRYPT {
    //     let plain_t = decrypt_vec(content.as_slice());
    //     prop_def_from_csv_bytes(plain_t.as_slice(), true)
    // } else {
    //     prop_def_from_csv_bytes(content.as_slice(), true)
    // }
    prop_def_from_csv_bytes(content.as_slice(), true)
}

pub fn prop_def_from_csv_bytes(
    content: &[u8],
    has_headers: bool,
) -> Result<Vec<PropDefine>, (usize, usize)> {
    let content_new = transfer_to_utf8(content.to_vec()).map_err(|_| (0, 0))?;
    let content = content_new.as_slice();
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(has_headers)
        .from_reader(content);
    let start_row = if has_headers { 1 } else { 0 };
    let mut records = rdr.records();
    let offset: usize = 1;
    let mut row: usize = start_row;
    let mut props = Vec::new();
    while let Some(Ok(record)) = records.next() {
        let rc = (row, offset);
        let id = csv_u64(&record, rc.1).ok_or(rc)?;
        let rc = (row, offset + 1);
        let name = csv_string(&record, rc.1).ok_or(rc)?;
        let rc = (row, offset + 2);
        let desc = csv_string(&record, rc.1).ok_or(rc)?;
        let rc = (row, offset + 3);
        let data_type_str = csv_str(&record, rc.1).ok_or(rc)?;
        let data_type = PropType::from_str(data_type_str).unwrap_or_default();
        let rc = (row, offset + 4);
        let data_unit_str = csv_str(&record, rc.1).ok_or(rc)?;
        let data_unit = DataUnit::from_str(data_unit_str).map_err(|_| rc)?;
        props.push(PropDefine { id, name, desc, data_type, data_unit });
        row += 1;
    }
    props.shrink_to_fit();
    Ok(props)
}

pub fn dev_def_from_csv(path: &str) -> Result<Vec<RsrDefine>, (usize, usize)> {
    let content = std::fs::read(path).map_err(|_| (0, 0))?;
    // if env::IS_ENCRYPT {
    //     let plain_t = decrypt_vec(content.as_slice());
    //     dev_def_from_csv_bytes(plain_t.as_slice(), true)
    // } else {
    //     dev_def_from_csv_bytes(content.as_slice(), true)
    // }
    dev_def_from_csv_bytes(content.as_slice(), true)
}

pub fn dev_def_from_csv_bytes(
    content: &[u8],
    has_headers: bool,
) -> Result<Vec<RsrDefine>, (usize, usize)> {
    let content_new = transfer_to_utf8(content.to_vec()).map_err(|_| (0, 0))?;
    let content = content_new.as_slice();
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(has_headers)
        .from_reader(content);
    let start_row = if has_headers { 1 } else { 0 };
    let mut records = rdr.records();
    let offset: usize = 1;
    let mut row: usize = start_row;
    let mut defines = vec![];
    while let Some(Ok(record)) = records.next() {
        let rc = (row, offset);
        let id = csv_u64(&record, rc.1).ok_or(rc)?;
        let rc = (row, offset + 1);
        let dev_type_str = csv_string(&record, rc.1).ok_or(rc)?;
        let rsr_type = PsRsrType::from(dev_type_str);
        let rc = (row, offset + 2);
        let name = csv_string(&record, rc.1).ok_or(rc)?;
        let rc = (row, offset + 3);
        let desc = csv_string(&record, rc.1).ok_or(rc)?;
        let rc = (row, offset + 4);
        let terminal_num = csv_u8(&record, rc.1).ok_or(rc)?;
        let mut col = 5;
        let mut prop_groups = Vec::new();
        loop {
            // 如果当前列已经超出记录的总长度了，则退出循环
            if record.len() <= offset + col {
                break;
            }
            let rc = (row, offset + col);
            let name = csv_string(&record, rc.1).ok_or(rc)?;
            if name.is_empty() {
                break;
            }
            let rc = (row, offset + col + 1);
            let desc = csv_string(&record, rc.1).ok_or(rc)?;
            let rc = (row, offset + col + 2);
            let prop_def_ids = csv_str(&record, rc.1).ok_or(rc)?;
            let r: Vec<Result<u64, (usize, usize)>> = if !prop_def_ids.is_empty() {
                prop_def_ids.split(';')
                    .map(|s| s.parse::<u64>().map_err(|_| rc))
                    .collect()
            } else {
                vec![]
            };
            let mut prop_defines = Vec::with_capacity(r.len());
            for id in r {
                prop_defines.push(id?);
            }
            prop_groups.push(PropGroupDefine { name, desc, prop_defines });
            col += 3;
        }
        prop_groups.shrink_to_fit();
        defines.push(RsrDefine { id, rsr_type, name, desc, terminal_num, prop_groups });
        row += 1;
    }
    defines.shrink_to_fit();
    Ok(defines)
}

pub fn dev_from_csv(
    path: &str,
    defines: &HashMap<u64, RsrDefine>,
    prop_defines: &HashMap<u64, PropDefine>,
) -> Result<(HashMap<u64, NetworkRsr>, HashMap<u64, RsrPropGroup>), (usize, usize)> {
    let content = std::fs::read(path).map_err(|_| (0, 0))?;
    // if env::IS_ENCRYPT {
    //     let plain_t = decrypt_vec(content.as_slice());
    //     dev_from_csv_bytes(plain_t.as_slice(), true, defines, prop_defines)
    // } else {
    //     dev_from_csv_bytes(content.as_slice(), true, defines, prop_defines)
    // }
    dev_from_csv_bytes(content.as_slice(), true, defines, prop_defines)
}

pub fn dev_from_csv_bytes(
    content: &[u8],
    has_headers: bool,
    defines: &HashMap<u64, RsrDefine>,
    prop_defines: &HashMap<u64, PropDefine>,
) -> Result<(HashMap<u64, NetworkRsr>, HashMap<u64, RsrPropGroup>), (usize, usize)> {
    let content_new = transfer_to_utf8(content.to_vec()).map_err(|_| (0, 0))?;
    let content = content_new.as_slice();
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(has_headers)
        .from_reader(content);
    let start_row = if has_headers { 1 } else { 0 };
    let mut records = rdr.records();
    let offset: usize = 1;
    let mut row: usize = start_row;
    let mut devs: HashMap<u64, NetworkRsr> = HashMap::new();
    let mut prop_groups: HashMap<u64, RsrPropGroup> = HashMap::new();
    while let Some(Ok(record)) = records.next() {
        let rc = (row, offset);
        let rsr_id = csv_u64(&record, rc.1).ok_or(rc)?;
        let rc = (row, offset + 1);
        let dev_define_id = csv_u64(&record, rc.1).ok_or(rc)?;
        let define = defines.get(&dev_define_id).ok_or(rc)?;
        let rc = (row, offset + 6);
        let prop_group_name = csv_str(&record, rc.1).ok_or(rc)?;
        let mut prop_group_id = None;
        if !prop_group_name.is_empty() {
            for prop_def in define.prop_groups.iter() {
                if prop_group_name == prop_def.name {
                    let rc = (row, offset + 7);
                    let group_id = csv_u64(&record, rc.1).ok_or(rc)?;
                    let mut col = offset + 8;
                    let mut props = Vec::with_capacity(prop_def.prop_defines.len());
                    let mut defines = Vec::with_capacity(prop_def.prop_defines.len());
                    for _ in 0..prop_def.prop_defines.len() {
                        let rc = (row, col);
                        if let Some(prop_def_id) = csv_u64(&record, rc.1) {
                            let rc = (row, col + 1);
                            if let Some(prop_def) = prop_defines.get(&prop_def_id) {
                                if let Some(s) = csv_str(&record, rc.1) {
                                    let t = prop_def.data_type;
                                    let prop = PropValue::from_str(t, s).ok_or(rc)?;
                                    props.push(prop);
                                    defines.push(prop_def_id);
                                }
                            }
                        }
                        col += 2;
                    }
                    let group = RsrPropGroup {
                        id: group_id,
                        name: prop_group_name.to_string(),
                        rsr_id,
                        defines,
                        props,
                    };
                    prop_groups.insert(group_id, group);
                    prop_group_id = Some(group_id);
                    break;
                }
            }
        }
        if let Some(dev) = devs.get_mut(&rsr_id) {
            if let Some(group_id) = &prop_group_id {
                dev.prop_group_ids.push(*group_id);
            }
        } else {
            let rc = (row, offset + 2);
            let name = csv_string(&record, rc.1).ok_or(rc)?;
            let rc = (row, offset + 3);
            let desc = csv_string(&record, rc.1).ok_or(rc)?;
            let rc = (row, offset + 4);
            let terminal_id_str = csv_str(&record, rc.1).ok_or(rc)?;
            let r: Vec<Result<u64, (usize, usize)>> = if terminal_id_str.is_empty() {
                vec![]
            } else {
                terminal_id_str.split(';')
                    .map(|s| s.parse::<u64>().map_err(|_| rc))
                    .collect()
            };
            let mut dev_terminals = Vec::with_capacity(r.len());
            for tid in r {
                let terminal_id = tid?;
                dev_terminals.push(Terminal {
                    device: rsr_id,
                    id: terminal_id,
                });
            }
            let rc = (row, offset + 5);
            let container_id = csv_string(&record, rc.1).ok_or(rc)?;
            let container_id = if container_id.is_empty() {
                None
            } else {
                Some(container_id.parse::<u64>().map_err(|_| rc)?)
            };
            let prop_groups = if let Some(group_id) = &prop_group_id {
                vec![*group_id]
            } else {
                vec![]
            };
            let rsr = NetworkRsr {
                id: rsr_id,
                define_id: dev_define_id,
                name,
                desc,
                container_id,
                terminals: dev_terminals,
                prop_group_ids: prop_groups,
            };
            devs.insert(rsr_id, rsr);
        }
        row += 1;
    }
    Ok((devs, prop_groups))
}

pub fn meas_def_from_csv(
    path: &str,
) -> Result<Vec<MeasureDef>, (usize, usize)> {
    let content = std::fs::read(path).map_err(|_| (0, 0))?;
    // if env::IS_ENCRYPT {
    //     let plain_t = decrypt_vec(content.as_slice());
    //     meas_def_from_csv_bytes(plain_t.as_slice(), true)
    // } else {
    //     meas_def_from_csv_bytes(content.as_slice(), true)
    // }
    meas_def_from_csv_bytes(content.as_slice(), true)
}

pub fn meas_def_from_csv_bytes(
    content: &[u8],
    has_headers: bool,
) -> Result<Vec<MeasureDef>, (usize, usize)> {
    let content_new = transfer_to_utf8(content.to_vec()).map_err(|_| (0, 0))?;
    let content = content_new.as_slice();
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(has_headers)
        .from_reader(content);
    let start_row = if has_headers { 1 } else { 0 };
    let mut records = rdr.records();
    let offset: usize = 1;
    let mut row: usize = start_row;
    let mut defs: Vec<MeasureDef> = Vec::new();
    while let Some(Ok(record)) = records.next() {
        let rc = (row, offset);
        let id = csv_u64(&record, rc.1).ok_or(rc)?;
        let rc = (row, offset + 1);
        let point_id = csv_u64(&record, rc.1).ok_or(rc)?;
        let rc = (row, offset + 2);
        let dev_id = csv_u64(&record, rc.1).ok_or(rc)?;
        let rc = (row, offset + 3);
        let terminal_id = csv_u64(&record, rc.1).ok_or(rc)?;
        let rc = (row, offset + 4);
        let phase = if let Some(s) = csv_str(&record, rc.1) {
            MeasPhase::from(s)
        } else {
            MeasPhase::Unknown
        };
        defs.push(MeasureDef { id, point_id, terminal_id, dev_id, phase });
        row += 1;
    }
    Ok(defs)
}


pub fn measures_from_csv(
    path: &str,
) -> Result<Vec<MeasureValue>, (usize, usize)> {
    let content = std::fs::read(path).map_err(|_| (0, 0))?;
    // if env::IS_ENCRYPT {
    //     let plain_t = decrypt_vec(content.as_slice());
    //     measures_from_csv_bytes(plain_t.as_slice(), true)
    // } else {
    //     measures_from_csv_bytes(content.as_slice(), true)
    // }
    measures_from_csv_bytes(content.as_slice(), true)
}

pub fn measures_from_csv_bytes(
    content: &[u8],
    has_headers: bool,
) -> Result<Vec<MeasureValue>, (usize, usize)> {
    let content_new = transfer_to_utf8(content.to_vec()).map_err(|_| (0, 0))?;
    let content = content_new.as_slice();
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(has_headers)
        .from_reader(content);
    let start_row = if has_headers { 1 } else { 0 };
    let mut records = rdr.records();
    let offset: usize = 1;
    let mut row: usize = start_row;
    let mut defs: Vec<MeasureValue> = Vec::new();
    while let Some(Ok(record)) = records.next() {
        let rc = (row, offset);
        let point_id = csv_u64(&record, rc.1).ok_or(rc)?;
        let rc = (row, offset + 1);
        let value = csv_f64(&record, rc.1).ok_or(rc)?;
        defs.push(MeasureValue {
            point_id,
            is_discrete: false,
            timestamp: 0,
            analog_value: value,
            discrete_value: value as i64,
            is_transformed: false,
            transformed_analog: 0.0,
            transformed_discrete: 0,
        });
        row += 1;
    }
    Ok(defs)
}

pub fn cns_from_csv(
    path: &str,
) -> Result<Vec<CN>, (usize, usize)> {
    let content = std::fs::read(path).map_err(|_| (0, 0))?;
    // if env::IS_ENCRYPT {
    //     let plain_t = decrypt_vec(content.as_slice());
    //     cns_from_csv_bytes(plain_t.as_slice(), true)
    // } else {
    //     cns_from_csv_bytes(content.as_slice(), true)
    // }
    cns_from_csv_bytes(content.as_slice(), true)
}

pub fn cns_from_csv_bytes(
    content: &[u8],
    has_headers: bool,
) -> Result<Vec<CN>, (usize, usize)> {
    let content_new = transfer_to_utf8(content.to_vec()).map_err(|_| (0, 0))?;
    let content = content_new.as_slice();
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(has_headers)
        .from_reader(content);
    let start_row = if has_headers { 1 } else { 0 };
    let mut records = rdr.records();
    let offset: usize = 0;
    let mut row: usize = start_row;
    let mut cns: HashMap<u64, CN> = HashMap::new();
    while let Some(Ok(record)) = records.next() {
        let rc = (row, offset);
        let index = csv_u64(&record, rc.1).ok_or(rc)?;
        let rc = (row, offset + 1);
        let terminal = csv_u64(&record, rc.1).ok_or(rc)?;
        cns.entry(index).or_insert(CN { id: index, terminals: vec![] }).terminals.push(terminal);
        row += 1;
    }
    Ok(cns.into_values().collect())
}
impl CN {
    pub fn get_csv_header(text_map: &HashMap<String, String>) -> String {
        format!(
            "{},{}",
            text_map.get("CN").unwrap_or(&"CN".to_string()),
            text_map.get("terminal").unwrap_or(&"Terminal".to_string()),
        )
    }

    pub fn to_csv_str(&self) -> String {
        let mut result = String::new();
        for t in &self.terminals {
            result.push_str(&format!("\n{},{}", self.id, t));
        }
        result
    }
}

impl RsrDefine {
    pub fn create_rsr(&self) -> NetworkRsr {
        NetworkRsr {
            id: 0,
            define_id: self.id,
            name: "".to_string(),
            desc: "".to_string(),
            container_id: None,
            terminals: Vec::with_capacity(self.terminal_num as usize),
            prop_group_ids: Vec::with_capacity(self.prop_groups.len()),
        }
    }

    pub fn to_csv_str(&self, max_prop_group_len: usize) -> String {
        let mut prop_group_result = "".to_string();
        for group_index in 0..max_prop_group_len {
            if self.prop_groups.len() > group_index {
                let prop_group = &self.prop_groups[group_index];
                let prop_defines_result = &prop_group.prop_defines
                    .iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<_>>()
                    .join(";");
                prop_group_result += &format!(",{},{},{}", prop_group.name.clone(), prop_group.desc.clone(), prop_defines_result);
            } else {
                prop_group_result += ",,,";
            }
        }
        format!("{},{},{},{},{}{}", self.id, self.rsr_type, self.name,
                self.desc, self.terminal_num, prop_group_result)
    }

    pub fn get_csv_header(text_map: &HashMap<String, String>, max_prop_group_len: usize) -> String {
        // 组建表头
        let mut group_header = "".to_string();
        for i in 0..max_prop_group_len {
            let group_header_name = &format!("{}{}_{}",
                                             text_map.get("my_devpropgroupdefine")
                                                 .unwrap_or(&"Property Group".to_string()),
                                             i + 1,
                                             text_map.get("name").unwrap_or(&"Name".to_string()),
            );
            let group_header_desc = &format!("{}{}_{}",
                                             text_map.get("my_devpropgroupdefine")
                                                 .unwrap_or(&"Property Group".to_string()),
                                             i + 1,
                                             text_map.get("desc").unwrap_or(&"Description".to_string()),
            );
            let group_header_prop = &format!("{}{}_{}",
                                             text_map.get("my_devpropgroupdefine")
                                                 .unwrap_or(&"Property Group".to_string()),
                                             i + 1,
                                             text_map.get("dev_property")
                                                 .unwrap_or(&"Property".to_string()),
            );
            group_header += &format!(",{},{},{}",
                                     group_header_name,
                                     group_header_desc,
                                     group_header_prop,
            );
        }
        format!(
            "{},{},{},{},{},{}{}",
            text_map.get("index").unwrap_or(&"Index".to_string()),
            text_map.get("devdefine_id").unwrap_or(&"ID".to_string()),
            text_map.get("rsr_type").unwrap_or(&"Resource Type".to_string()),
            text_map.get("devdefine_name").unwrap_or(&"Name".to_string()),
            text_map.get("devdefine_desc").unwrap_or(&"Description".to_string()),
            text_map.get("devdefine_terminal_num").unwrap_or(&"Terminal Number".to_string()),
            group_header
        )
    }
}

impl PropDefine {
    pub fn get_csv_header(text_map: &HashMap<String, String>) -> String {
        format!(
            "{},{},{},{},{}, {}",
            text_map.get("index").unwrap_or(&"Index".to_string()),
            text_map.get("devpropdefine_id").unwrap_or(&"ID".to_string()),
            text_map.get("devpropdefine_name").unwrap_or(&"Name".to_string()),
            text_map.get("devpropdefine_desc").unwrap_or(&"Description".to_string()),
            text_map.get("devpropdefine_datatype").unwrap_or(&"Data Type".to_string()),
            text_map.get("devpropdefine_dataunit").unwrap_or(&"Data Unit".to_string()),
        )
    }

    pub fn to_csv_str(&self) -> String {
        format!("{},{},{},{},{}", self.id, self.name, self.desc, self.data_type, self.data_unit)
    }
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

    pub fn get_prop_value_by_desc(&self, prop_desc: &str, prop_groups: &HashMap<u64, RsrPropGroup>,
                                  prop_defs: &HashMap<u64, PropDefine>) -> PropValue {
        for prop_group_id in &self.prop_group_ids {
            if let Some(rpg) = prop_groups.get(prop_group_id) {
                for i in 0..rpg.defines.len() {
                    if let Some(prop_def) = prop_defs.get(&rpg.defines[i]) {
                        if prop_def.desc == prop_desc {
                            return rpg.props[i].clone();
                        }
                    }
                }
            }
        }
        PropValue::Unknown
    }

    pub fn get_csv_header(text_map: &HashMap<String, String>, max_prop_len: usize) -> String {
        let mut group_header = format!(",{},{}",
                                       text_map.get("devpropgroupdefine_name").unwrap_or(&"Name".to_string()),
                                       text_map.get("devpropgroupdefine_id").unwrap_or(&"ID".to_string()),
        );
        for _ in 0..max_prop_len {
            group_header += &format!(",{},{}",
                                     text_map.get("dev_property_id").unwrap_or(&"Property ID".to_string()),
                                     text_map.get("dev_property").unwrap_or(&"Property Value".to_string())
            );
        }
        format!(
            "{},{},{},{},{},{},{}{}",
            text_map.get("index").unwrap_or(&"Index".to_string()),
            text_map.get("dev_id").unwrap_or(&"ID".to_string()),
            text_map.get("dev_define_id").unwrap_or(&"Device Define Id".to_string()),
            text_map.get("dev_name").unwrap_or(&"Name".to_string()),
            text_map.get("dev_desc").unwrap_or(&"Description".to_string()),
            text_map.get("dev_terminal").unwrap_or(&"Terminal".to_string()),
            text_map.get("dev_container").unwrap_or(&"Container".to_string()),
            group_header
        )
    }

    pub fn to_csv_str(&self, index: usize, max_prop_len: usize,
                      prop_group_map: &HashMap<u64, RsrPropGroup>) -> String {
        let dev_terminals = &self.terminals
            .iter()
            .map(|c| c.id.to_string())
            .collect::<Vec<_>>()
            .join(";");
        let container_id = if let Some(container_id) = &self.container_id {
            container_id.to_string()
        } else {
            "".to_string()
        };

        if !self.prop_group_ids.is_empty() {
            let mut result = "".to_string();
            // 如果有属性分组，则每个属性分组一行
            for (group_index, prop_group_id) in self.prop_group_ids.iter().enumerate() {
                if let Some(prop_group) = prop_group_map.get(prop_group_id) {
                    let mut prop_group_result = format!(",{},{}", prop_group.name, prop_group.id);
                    for prop_index in 0..max_prop_len {
                        if prop_group.props.len() > prop_index {
                            let prop_id = &prop_group.defines[prop_index];
                            let prop = &prop_group.props[prop_index];
                            prop_group_result += &format!(",{}", prop_id);
                            prop_group_result += &format!(",{}", prop);
                        } else {
                            prop_group_result += ",,";
                        }
                    }
                    let sn = if group_index == 0 { index.to_string() } else { "".to_string() };
                    result += &format!("{},{},{},{},{},{},{}{}", sn, self.id, self.define_id, self.name,
                                       self.desc, dev_terminals, container_id, prop_group_result);
                    if group_index != self.prop_group_ids.len() - 1 {
                        result += "\n";
                    }
                }
            }
            result
        } else {
            // 如果没有属性分组，则需要补充空列
            let mut prop_group_result = ",,".to_string();
            for _ in 0..max_prop_len {
                prop_group_result += ",,";
            }
            format!("{},{},{},{},{},{},{}{}", index, self.id, self.define_id, self.name,
                    self.desc, dev_terminals, container_id, prop_group_result)
        }
    }
}

impl Island {
    pub fn create_dev_tree(&self) -> HashMap<u64, String> {
        let mut result = HashMap::new();
        let ids: Vec<u64> = self.resources.keys().copied().collect();
        let mut finded = vec![false; ids.len()];
        loop {
            (0..finded.len()).for_each(|i| {
                if finded[i] {
                    return;
                }
                let id = ids[i];
                let rsr = self.resources.get(&id).unwrap();
                if let Some(parent_id) = &rsr.container_id {
                    if let Some(path) = result.get(parent_id) {
                        let path = format!("{}/{}({})", path, rsr.name, id);
                        result.insert(id, path);
                        finded[i] = true;
                    } else {
                        // parent is not ready
                    }
                } else {
                    let path = format!("/{}({})", rsr.name, id);
                    result.insert(id, path);
                    finded[i] = true;
                }
            });
            if result.len() == self.resources.len() {
                break;
            }
        }
        result
    }

    pub fn create_dev_tree_with_measure(&self) -> HashMap<u64, String> {
        let mut result = self.create_dev_tree();
        for (rsr_id, meas_defs) in self.measures.iter() {
            for meas_def in meas_defs {
                if let Some(dev_path) = result.get(rsr_id) {
                    let path = format!("{}/{}", dev_path, meas_def.point_id);
                    result.insert(meas_def.point_id, path);
                }
            }
        }
        result
    }

    pub fn create_measure_tree(&self, point_names: &HashMap<u64, String>) -> HashMap<u64, String> {
        let mut result = self.create_dev_tree();
        for (rsr_id, meas_defs) in self.measures.iter() {
            for meas_def in meas_defs {
                let point_id = meas_def.point_id;
                let point_name = if let Some(name) = point_names.get(&point_id) {
                    format!("{}({})", name, point_id)
                } else {
                    format!("Not found({})", point_id)
                };
                let path = format!("{}/{}", result.get(rsr_id).unwrap(), point_name);
                result.insert(point_id, path);
            }
        }
        result
    }

    pub fn create_measure_tree2(&self, points: &HashMap<u64, Measurement>) -> HashMap<u64, String> {
        let mut result = self.create_dev_tree();
        for (rsr_id, meas_defs) in self.measures.iter() {
            for meas_def in meas_defs {
                let point_id = meas_def.point_id;
                let point_name = if let Some(p) = points.get(&point_id) {
                    format!("{}({})", p.point_name, point_id)
                } else {
                    format!("Not found({})", point_id)
                };
                let path = format!("{}/{}", result.get(rsr_id).unwrap(), point_name);
                result.insert(point_id, path);
            }
        }
        result
    }

    pub fn to_dev_csv_str(&self, text_map: &HashMap<String, String>) -> String {
        // 先循环一遍，记录最大的属性数量
        let mut max_prop_len = 0;
        for dev in self.resources.values() {
            for prop_group_id in &dev.prop_group_ids {
                if let Some(prop_group) = self.prop_groups.get(prop_group_id) {
                    if max_prop_len < prop_group.props.len() {
                        max_prop_len = prop_group.props.len();
                    }
                }
            }
        }
        // 组建表头
        let mut result = NetworkRsr::get_csv_header(text_map, max_prop_len);
        if !self.resources.is_empty() {
            result.push('\n');
        }
        // 组建表体
        let mut index = 0;
        for dev in self.resources.values() {
            index += 1;
            result += &dev.to_csv_str(index, max_prop_len, &self.prop_groups);
            if index < self.resources.len() {
                result.push('\n');
            }
        }
        result
    }

    pub fn to_meas_csv_str(&self, text_map: &HashMap<String, String>) -> String {
        // 组建表头
        let mut result = MeasureDef::get_csv_header(text_map);
        if !self.measures.is_empty() {
            result.push('\n');
        }
        // 组建表体
        let mut count = 0;
        for defs in self.measures.values() {
            count += defs.len();
        }
        let mut index = 0;
        for defs in self.measures.values() {
            for def in defs {
                index += 1;
                result.push_str(&index.to_string());
                result.push(',');
                result += &def.to_csv_str();
                if index < count {
                    result.push('\n');
                }
            }
        }
        result
    }

    pub fn to_cns_csv_str(&self, text_map: &HashMap<String, String>) -> String {
        // 组建表头
        let mut result = CN::get_csv_header(text_map);
        // 组建表体
        for cn in &self.cns {
            if !cn.terminals.is_empty() {
                result += &cn.to_csv_str();
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::model::dev::{dev_def_from_csv, dev_from_csv, meas_def_from_csv, prop_def_from_csv, PsRsrType};

    #[test]
    fn test_type_to_u16() {
        assert_eq!(1, PsRsrType::Switch as u16);
        assert_eq!(16, PsRsrType::Feeder as u16);
        assert_eq!(10000, PsRsrType::Company as u16);
        assert_eq!(30001, PsRsrType::UserDefine1 as u16);
        assert_eq!(65535, PsRsrType::Unknown as u16);
    }
    #[test]
    fn test_ningbo_parse() {
        let dev_defs = dev_def_from_csv("tests/ningbo/dev_def.csv");
        assert!(dev_defs.is_ok());
        let dev_defs = dev_defs.unwrap();
        let mut defines = HashMap::with_capacity(dev_defs.len());
        for def in &dev_defs {
            defines.insert(def.id, def.clone());
        }
        let prop_defs = prop_def_from_csv("tests/ningbo/dev_prop_def.csv");
        assert!(prop_defs.is_ok());
        let prop_defs = prop_defs.unwrap();
        let mut prop_defines = HashMap::with_capacity(prop_defs.len());
        for def in &prop_defs {
            prop_defines.insert(def.id, def.clone());
        }
        let r = dev_from_csv("tests/ningbo/devices.csv", &defines, &prop_defines);
        if let Err((row, col)) = r {
            println!("row={}, col={}", row, col);
        }
        assert!(r.is_ok());
        let r = meas_def_from_csv("tests/ningbo/measures.csv");
        assert!(r.is_ok());
    }

    #[test]
    fn test_to_string() {
        println!("{}", PsRsrType::ACline);
    }
}