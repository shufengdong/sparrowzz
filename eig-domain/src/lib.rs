extern crate core;

use std::{fmt, io};
use std::collections::{BTreeMap, HashMap};
use std::fmt::Display;
use std::str::FromStr;
use std::marker::PhantomData;

use byteorder::{BigEndian, ByteOrder};
// use bytes::BytesMut;
use csv::{Reader, StringRecord};
// use protobuf::CodedInputStream;
// use protobuf::error::WireError;
use serde::{Deserialize, Serialize};

use protobuf::EnumFull;
use protobuf::EnumOrUnknown;

use eig_expr::Expr;
use eig_expr::Token;

pub use crate::prop::*;
use crate::excel::{excel_bytes_to_csv_bytes, transfer_to_utf8};
use crate::dlt645::Dlt645ClientTp;
use crate::ethercat::EcMasterTp;
pub use crate::hymqtt::{HYMqttTransport, HYPointInfo};
pub use crate::iec104::{Iec104ClientTp, Iec104ServerTp, Iec104Connection, Iec104Point};
use crate::memory::{MemoryPosixTp, MemorySystemVTp};
pub use crate::modbus::{ModbusRtuClientTp, ModbusTcpClientTp, ModbusRtuServerTp, ModbusTcpServerTp, MbConnection, RegisterType};
pub use crate::mqtt::MqttTransport;
pub use crate::proto::eig::*;
pub use crate::proto::eig::pb_alarm_define::AlarmLevel as PbAlarmDefine_AlarmLevel;
pub use crate::proto::eig::pb_eig_alarm::AlarmType as PbEigAlarm_AlarmType;
pub use crate::proto::eig::pb_eig_alarm::AlarmStatus as PbEigAlarm_AlarmStatus;
pub use crate::proto::eig::pb_set_point_result::SetPointStatus as PbSetPointResult_SetPointStatus;
pub use crate::proto::eig::pb_file::FileOperation as PbFile_FileOperation;
pub use crate::proto::eig::pb_request::RequestType as PbRequest_RequestType;

pub mod excel;
pub mod hymqtt;
pub mod iec104;
pub mod dlt645;
pub mod modbus;
pub mod mqtt;
pub mod ethercat;
pub mod memory;
pub mod prop;
pub mod proto;
pub mod topics;
pub mod web;

pub const UNKNOWN_TCP_PORT: u32 = 9999;
pub const UNKNOWN_POINT_ID: u64 = 0;
pub const MINIMUM_POINT_ID: u64 = 100001;
// 增加aoe id最小值的限制，这是为了避免在注册监听量测变化的时候id，从文件加载的通道id是系统自动生成的，从1开始
// aoe和transport都会监听，而各自的id是监听注册时的id，发生冲突的时候，早注册的那个会收不到量测变化通知
// user id type is u16, so aoe id will not conflict with user id
pub const MINIMUM_AOE_ID: u64 = u16::MAX as u64 + 1;
pub const DEFAULT_TCP_CLIENT_LIMIT: u8 = 100;
pub const MAX_POLLING_PERIOD: u64 = 1_000_000_000;
pub const DATA_INFO: [&str; 38] = [
    "DATA TYPE,NOTE",
    "Binary,bool",
    "OneByteIntSigned,byte",
    "OneByteIntSignedLower,byte",
    "OneByteIntSignedUpper,byte",
    "OneByteIntUnsigned,byte",
    "OneByteIntUnsignedLower,byte",
    "OneByteIntUnsignedUpper,byte",
    "TwoByteIntUnsigned,u16",
    "TwoByteIntSigned,i16",
    "TwoByteIntSignedSwapped,",
    "TwoByteBcd,",
    "TwoByteIntUnsignedSwapped,",
    "FourByteIntUnsigned,",
    "FourByteIntSigned,u32",
    "FourByteIntUnsignedSwapped,",
    "FourByteIntSignedSwapped,",
    "FourByteIntUnsignedSwappedSwapped,",
    "FourByteIntSignedSwappedSwapped,",
    "FourByteFloat,",
    "FourByteFloatSwapped,",
    "FourByteBcd,",
    "FourByteBcdSwapped,",
    "FourByteMod10k,",
    "FourByteMod10kSwapped,",
    "SixByteMod10k,",
    "SixByteMod10kSwapped,",
    "EightByteIntUnsigned,",
    "EightByteIntSigned,",
    "EightByteIntUnsignedSwapped,",
    "EightByteIntSignedSwapped,",
    "EightByteIntUnsignedSwappedSwapped,",
    "EightByteIntSignedSwappedSwapped,",
    "EightByteFloat,",
    "EightByteFloatSwapped,",
    "EightByteFloatSwappedSwapped,",
    "EightByteMod10kSwapped,",
    "EightByteMod10k,",
];

// pub const IEC104D_INFO: [&str; 21] = ["1, 单点遥信", "3,双点遥信", "5,步位置遥信", "30,单点遥信（带时标）", "31,双点遥信（带时标）",
//     "32,步位置遥信（带时标）", "7,32比特串遥测", "9,规一化遥测值", "11,标度化遥测值", "13,短浮点遥测值", "15,累计量遥测", "33,32比特串遥测（带时标）",
//     "34,规一化遥测值（带时标）", "35,标度化遥测值（带时标）", "36,短浮点遥测值（带时标）", "37,累计量遥测（带时标）", "20,成组单点遥信",
//     "21,规一化遥测值", "38,继电保护装置事件", "39,继电保护装置成组启动事件", "40, 继电保护装置成组出口信息"];

pub const IEC104D_INFO: [&str; 21] = [
    "1,Single Command",
    "3,Double Command",
    "5,Step Command",
    "30,Single Command (with time)",
    "31,Double Command (with time)",
    "32,Step Command (with time)",
    "7,32-Bitstring Command",
    "9,Normalized Command Value",
    "11,Scaled Command Value",
    "13,Short Float Command Value",
    "15,Cumulant Command Value",
    "33,32-Bitstring Command (with time)",
    "34,Normalized Command Value (with time)",
    "35,Scaled Command Value (with time)",
    "36,Short Float Command Value (with time)",
    "37,Cumulant Command Value(with time)",
    "20,Group Single Command",
    "21,Normalized Command Value",
    "38,Relay Protection Device Event",
    "39,Group Startup of Relay Protection Device Event",
    "40, Group Export Information of Relay Protection Device",
];

/// public api
/**
 * @api {枚举_通道类型} /TransportType TransportType
 * @apiGroup A_Enum
 * @apiSuccess {String} ModbusTcpClient ModbusTcp客户端
 * @apiSuccess {String} ModbusTcpServer ModbusTcp服务端
 * @apiSuccess {String} ModbusRtuClient ModbusRtu客户端
 * @apiSuccess {String} ModbusRtuServer ModbusRtu服务端
 * @apiSuccess {String} DLT645Client DLT645客户端
 * @apiSuccess {String} Mqtt Mqtt
 * @apiSuccess {String} Iec104Client Iec104客户端
 * @apiSuccess {String} Iec104Server Iec104服务端
 * @apiSuccess {String} HYMqtt HYMqtt
 * @apiSuccess {String} Unknown 未知
 */
#[derive(Serialize, Deserialize, Copy, Debug, Clone, PartialEq)]
pub enum TransportType {
    ModbusTcpClient = 1,
    ModbusTcpServer,
    ModbusRtuClient,
    ModbusRtuServer,
    // dlt
    DLT645Client,
    // mqtt
    Mqtt,
    // iec 104
    Iec104Client = 11,
    Iec104Server,
    // 融合终端mqtt
    HYMqtt,
    // etherCAT
    EtherCAT,
    // memory
    MemoryPosix,
    MemorySystemV,
    Unknown = 100,
}

impl Display for TransportType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TransportType {
    pub fn to_header(&self) -> String {
        match self {
            TransportType::ModbusTcpClient => String::from("tcp-mbc"),
            TransportType::ModbusTcpServer => String::from("tcp-mbd"),
            TransportType::ModbusRtuClient => String::from("rtu-mbc"),
            TransportType::ModbusRtuServer => String::from("rtu-mbd"),
            TransportType::DLT645Client => String::from("dlt645"),
            TransportType::Mqtt => String::from("mqtt"),
            TransportType::Iec104Client => String::from("iec104c"),
            TransportType::Iec104Server => String::from("iec104d"),
            TransportType::EtherCAT => String::from("ethercat"),
            TransportType::MemoryPosix => String::from("posix-memory"),
            TransportType::MemorySystemV => String::from("symtemv-memory"),
            _ => String::from("unknown"),
        }
    }
}

impl From<&str> for TransportType {
    fn from(value: &str) -> Self {
        match value {
            "ModbusTcpClient" => TransportType::ModbusTcpClient,
            "ModbusTcpServer" => TransportType::ModbusTcpServer,
            "ModbusRtuClient" => TransportType::ModbusRtuClient,
            "ModbusRtuServer" => TransportType::ModbusRtuServer,
            "DLT645Client" => TransportType::DLT645Client,
            "Mqtt" => TransportType::Mqtt,
            "Iec104Client" => TransportType::Iec104Client,
            "Iec104Server" => TransportType::Iec104Server,
            "EtherCAT" => TransportType::EtherCAT,
            "MemoryPosix" => TransportType::MemoryPosix,
            "MemorySystemV" => TransportType::MemorySystemV,
            _ => TransportType::Unknown,
        }
    }
}

impl From<String> for TransportType {
    fn from(value: String) -> Self {
        TransportType::from(value.as_str())
    }
}

/**
 * @api {枚举_通道对象} /Transport Transport
 * @apiPrivate
 * @apiGroup A_Enum
 * @apiSuccess {Object} MbcTcp {"MbcTcp": ModbusTcpClientTp}
 * @apiSuccess {Object} MbdTcp {"MbdTcp": ModbusTcpServerTp}
 * @apiSuccess {Object} MbcRtu {"MbcRtu": ModbusRtuClientTp}
 * @apiSuccess {Object} MbdRtu {"MbdRtu": ModbusRtuServerTp}
 * @apiSuccess {Object} DLT645c {"DLT645c": Dlt645ClientTp}
 * @apiSuccess {Object} Mqtt {"Mqtt": MqttTransport}
 * @apiSuccess {Object} Iec104c {"Iec104c": Iec104ClientTp}
 * @apiSuccess {Object} Iec104d {"Iec104d": Iec104ServerTp}
 * @apiSuccess {Object} HYMqtt {"HYMqtt": HYMqttTransport}
 */
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum Transport {
    MbcTcp(ModbusTcpClientTp),
    MbdTcp(ModbusTcpServerTp),
    MbcRtu(ModbusRtuClientTp),
    MbdRtu(ModbusRtuServerTp),
    DLT645c(Dlt645ClientTp),
    Mqtt(MqttTransport),
    Iec104c(Iec104ClientTp),
    Iec104d(Iec104ServerTp),
    HYMqtt(HYMqttTransport),
    EtherCAT(EcMasterTp),
    MemoryPosix(MemoryPosixTp),
    MemorySystemV(MemorySystemVTp),
}

impl Transport {
    pub fn id(&self) -> u64 {
        match self {
            Transport::MbcTcp(t) => t.id,
            Transport::MbdTcp(t) => t.id,
            Transport::MbcRtu(t) => t.id,
            Transport::MbdRtu(t) => t.id,
            Transport::DLT645c(t) => t.id,
            Transport::Mqtt(t) => t.id,
            Transport::Iec104c(t) => t.id,
            Transport::Iec104d(t) => t.id,
            Transport::HYMqtt(t) => t.id,
            Transport::EtherCAT(t) => t.id,
            Transport::MemoryPosix(t) => t.id,
            Transport::MemorySystemV(t) => t.id,
        }
    }

    pub fn name(&self) -> String {
        match self {
            Transport::MbcTcp(t) => t.name.clone(),
            Transport::MbdTcp(t) => t.name.clone(),
            Transport::MbcRtu(t) => t.name.clone(),
            Transport::MbdRtu(t) => t.name.clone(),
            Transport::DLT645c(t) => t.name.clone(),
            Transport::Mqtt(t) => t.name.clone(),
            Transport::Iec104c(t) => t.name.clone(),
            Transport::Iec104d(t) => t.name.clone(),
            Transport::HYMqtt(t) => t.name.clone(),
            Transport::EtherCAT(t) => t.name.clone(),
            Transport::MemoryPosix(t) => t.name.clone(),
            Transport::MemorySystemV(t) => t.name.clone(),
        }
    }

    pub fn get_type(&self) -> TransportType {
        match self {
            Transport::MbcTcp(_) => TransportType::ModbusTcpClient,
            Transport::MbdTcp(_) => TransportType::ModbusTcpServer,
            Transport::MbcRtu(_) => TransportType::ModbusRtuClient,
            Transport::MbdRtu(_) => TransportType::ModbusRtuServer,
            Transport::DLT645c(_) => TransportType::DLT645Client,
            Transport::Mqtt(_) => TransportType::Mqtt,
            Transport::Iec104c(_) => TransportType::Iec104Client,
            Transport::Iec104d(_) => TransportType::Iec104Server,
            Transport::HYMqtt(_) => TransportType::HYMqtt,
            Transport::EtherCAT(_) => TransportType::EtherCAT,
            Transport::MemoryPosix(_) => TransportType::MemoryPosix,
            Transport::MemorySystemV(_) => TransportType::MemorySystemV,
        }
    }

    pub fn set_id(&mut self, id: u64) {
        match self {
            Transport::MbcTcp(t) => t.id = id,
            Transport::MbdTcp(t) => t.id = id,
            Transport::MbcRtu(t) => t.id = id,
            Transport::MbdRtu(t) => t.id = id,
            Transport::DLT645c(t) => t.id = id,
            Transport::Mqtt(t) => t.id = id,
            Transport::Iec104c(t) => t.id = id,
            Transport::Iec104d(t) => t.id = id,
            Transport::HYMqtt(t) => t.id = id,
            Transport::EtherCAT(t) => t.id = id,
            Transport::MemoryPosix(t) => t.id = id,
            Transport::MemorySystemV(t) => t.id = id,
        }
    }

    pub fn set_name(&mut self, name: String) {
        match self {
            Transport::MbcTcp(t) => t.name = name,
            Transport::MbdTcp(t) => t.name = name,
            Transport::MbcRtu(t) => t.name = name,
            Transport::MbdRtu(t) => t.name = name,
            Transport::DLT645c(t) => t.name = name,
            Transport::Mqtt(t) => t.name = name,
            Transport::Iec104c(t) => t.name = name,
            Transport::Iec104d(t) => t.name = name,
            Transport::HYMqtt(t) => t.name = name,
            Transport::EtherCAT(t) => t.name = name,
            Transport::MemoryPosix(t) => t.name = name,
            Transport::MemorySystemV(t) => t.name = name,
        }
    }

    pub fn get_point_ids(&self) -> Vec<u64> {
        match self {
            Transport::MbcTcp(t) => t.get_point_ids(),
            Transport::MbdTcp(t) => t.get_point_ids(),
            Transport::MbcRtu(t) => t.get_point_ids(),
            Transport::MbdRtu(t) => t.get_point_ids(),
            Transport::DLT645c(t) => t.get_point_ids(),
            Transport::Mqtt(t) => t.get_point_ids(),
            Transport::Iec104c(t) => t.get_point_ids(),
            Transport::Iec104d(t) => t.get_point_ids(),
            Transport::HYMqtt(t) => t.get_point_ids(),
            Transport::EtherCAT(t) => t.get_point_ids(),
            Transport::MemoryPosix(t) => t.get_point_ids(),
            Transport::MemorySystemV(t) => t.get_point_ids(),
        }
    }

    pub fn get_connection_count(&self) -> usize {
        match self {
            Transport::MbcTcp(t) => t.connections.len(),
            Transport::MbdTcp(t) => t.connections.len(),
            Transport::MbcRtu(t) => t.connections.len(),
            Transport::DLT645c(t) => t.connections.len(),
            Transport::Iec104d(t) => t.connections.len(),
            Transport::EtherCAT(t) => t.connections.len(),
            Transport::MemoryPosix(t) => t.connections.len(),
            _ => 1,
        }
    }

    pub fn is_remote(&self) -> bool {
        match self {
            Transport::MbcTcp(_) => true,
            Transport::MbdTcp(_) => false,
            Transport::MbcRtu(_) => true,
            Transport::MbdRtu(_) => false,
            Transport::DLT645c(_) => true,
            Transport::Mqtt(t) => !t.is_transfer,
            Transport::Iec104c(_) => true,
            Transport::Iec104d(_) => false,
            Transport::HYMqtt(_) => true,
            Transport::EtherCAT(_) => true,
            Transport::MemoryPosix(t) => !t.is_transfer,
            Transport::MemorySystemV(t) => !t.is_transfer,
        }
    }

    pub fn get_file_name(&self) -> String {
        let id = self.id();
        match self {
            Transport::MbcTcp(_) => format!("tcp-mbc-{}.csv", id),
            Transport::MbdTcp(_) => format!("tcp-mbd-{}.csv", id),
            Transport::MbcRtu(_) => format!("rtu-mbc-{}.csv", id),
            Transport::MbdRtu(_) => format!("rut-mbd-{}.csv", id),
            Transport::DLT645c(_) => format!("dlt645-{}.csv", id),
            Transport::Mqtt(_) => format!("mqtt-{}.csv", id),
            Transport::Iec104c(_) => format!("iec104c-{}.csv", id),
            Transport::Iec104d(_) => format!("iec104d-{}.csv", id),
            Transport::HYMqtt(_) => format!("ttu-{}.csv", id),
            Transport::EtherCAT(_) => format!("ethercat-{}.csv", id),
            Transport::MemoryPosix(_) => format!("posix-memory-{}.csv", id),
            Transport::MemorySystemV(_) => format!("systemv-memory-{}.csv", id),
        }
    }

    pub fn from_bytes(file_name: &str, file_content: &[u8]) -> Vec<Result<Self, (usize, usize)>> {
        let mut result_vec = Vec::new();
        let file_name = file_name.to_lowercase();
        let content_vec = if file_name.ends_with(".xlsx") || file_name.ends_with(".xls") || file_name.is_empty() {
            excel_bytes_to_csv_bytes(file_content).unwrap_or_default()
        } else if file_name.ends_with(".csv") {
            vec![file_content.to_owned()]
        } else {
            return vec![Err((0, 0))];
        };
        for content in content_vec {
            let tp_result = if file_name.contains("xa-mbc") || file_name.contains("encap-mbc") {
                match ModbusTcpClientTp::from_csv_bytes(content.as_slice()) {
                    Ok(t) => {
                        Ok(Transport::MbcTcp(t))
                    }
                    Err((r, c)) => Err((r + 1, c + 1))
                }
            } else if file_name.contains("tcp-mbc") {
                match ModbusTcpClientTp::from_csv_bytes2(content.as_slice()) {
                    Ok(t) => {
                        Ok(Transport::MbcTcp(t))
                    }
                    Err((r, c)) => Err((r + 1, c + 1))
                }
            } else if file_name.contains("tcp-mbd") {
                match ModbusTcpServerTp::from_csv_bytes(content.as_slice()) {
                    Ok(t) => {
                        Ok(Transport::MbdTcp(t))
                    }
                    Err((r, c)) => Err((r + 1, c + 1))
                }
            } else if file_name.contains("rtu-mbc") {
                match ModbusRtuClientTp::from_csv_bytes(content.as_slice()) {
                    Ok(t) => {
                        Ok(Transport::MbcRtu(t))
                    }
                    Err((r, c)) => Err((r + 1, c + 1))
                }
            } else if file_name.contains("rtu-mbd") {
                match ModbusRtuServerTp::from_csv_bytes(content.as_slice()) {
                    Ok(t) => {
                        Ok(Transport::MbdRtu(t))
                    }
                    Err((r, c)) => Err((r + 1, c + 1))
                }
            } else if file_name.contains("dlt645") {
                match Dlt645ClientTp::from_csv_bytes(content.as_slice()) {
                    Ok(t) => {
                        Ok(Transport::DLT645c(t))
                    }
                    Err((r, c)) => Err((r + 1, c + 1))
                }
            } else if file_name.contains("mqtt") {
                match MqttTransport::from_csv_bytes(content.as_slice()) {
                    Ok(t) => {
                        Ok(Transport::Mqtt(t))
                    }
                    Err((r, c)) => Err((r + 1, c + 1))
                }
            } else if file_name.contains("iec104c") {
                match Iec104ClientTp::from_csv_bytes(content.as_slice()) {
                    Ok(t) => {
                        Ok(Transport::Iec104c(t))
                    }
                    Err((r, c)) => Err((r + 1, c + 1))
                }
            } else if file_name.contains("iec104d") {
                match Iec104ServerTp::from_csv_bytes(content.as_slice()) {
                    Ok(t) => {
                        Ok(Transport::Iec104d(t))
                    }
                    Err((r, c)) => Err((r + 1, c + 1))
                }
            } else if file_name.contains("ttu") {
                match HYMqttTransport::from_csv_bytes(content.as_slice()) {
                    Ok(t) => {
                        Ok(Transport::HYMqtt(t))
                    }
                    Err((r, c)) => Err((r + 1, c + 1))
                }
            } else if file_name.contains("ethercat") {
                match EcMasterTp::from_csv_bytes(content.as_slice()) {
                    Ok(t) => {
                        Ok(Transport::EtherCAT(t))
                    }
                    Err((r, c)) => Err((r + 1, c + 1))
                }
            } else if file_name.contains("posix-memory") {
                match MemoryPosixTp::from_csv_bytes(content.as_slice()) {
                    Ok(t) => {
                        Ok(Transport::MemoryPosix(t))
                    }
                    Err((r, c)) => Err((r + 1, c + 1))
                }
            } else if file_name.contains("systemv-memory") {
                match MemorySystemVTp::from_csv_bytes(content.as_slice()) {
                    Ok(t) => {
                        Ok(Transport::MemorySystemV(t))
                    }
                    Err((r, c)) => Err((r + 1, c + 1))
                }
            } else {
                Err((0, 0))
            };
            result_vec.push(tp_result);
        }
        result_vec
    }

    pub fn from_file(path: &str) -> Vec<Result<Self, (usize, usize)>> {
        let f_result = std::fs::read(path);
        if f_result.is_err() {
            return vec![Err((0, 0))];
        }
        let content = f_result.unwrap();
        // let content = if env::IS_ENCRYPT {
        //     decrypt_vec(content.as_slice())
        // } else {
        //     content
        // };

        Self::from_bytes(path, content.as_slice())
    }

    pub fn export_csv(&self, text_map: &HashMap<String, String>) -> String {
        match self {
            Transport::MbcTcp(t) => t.export_csv(text_map),
            Transport::MbdTcp(t) => t.export_csv(text_map),
            Transport::MbcRtu(t) => t.export_csv(text_map),
            Transport::MbdRtu(t) => t.export_csv(text_map),
            Transport::DLT645c(t) => t.export_csv(text_map),
            Transport::Mqtt(t) => t.export_csv(text_map),
            Transport::Iec104c(t) => t.export_csv(text_map),
            Transport::Iec104d(t) => t.export_csv(text_map),
            Transport::EtherCAT(t) => t.export_csv(text_map),
            Transport::MemoryPosix(t) => t.export_csv(text_map),
            _ => "".to_string(),
        }
    }
}

/**
 * @api {Measurement} /Measurement Measurement
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} point_id 测点id
 * @apiSuccess {String} point_name 测点名
 * @apiSuccess {String} alias_id 字符串id
 * @apiSuccess {bool} is_discrete 是否是离散量
 * @apiSuccess {bool} is_computing_point 是否是计算点
 * @apiSuccess {String} expression 如果是计算点，这是表达式
 * @apiSuccess {String} trans_expr 变换公式
 * @apiSuccess {String} inv_trans_expr 逆变换公式
 * @apiSuccess {String} change_expr 判断是否"变化"的公式，用于变化上传或储存
 * @apiSuccess {String} zero_expr 判断是否为0值的公式
 * @apiSuccess {String} data_unit 单位
 * @apiSuccess {f64} upper_limit 上限，用于坏数据辨识
 * @apiSuccess {f64} lower_limit 下限，用于坏数据辨识
 * @apiSuccess {String} alarm_level1_expr 告警级别1的表达式
 * @apiSuccess {String} alarm_level2_expr 告警级别2的表达式
 * @apiSuccess {bool} is_realtime 如是，则不判断是否"变化"，均上传
 * @apiSuccess {bool} is_soe 是否是soe点
 * @apiSuccess {u64} init_value 默认值存储在8个字节，需要根据is_discrete来转换成具体的值
 */
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Measurement {
    /// 唯一的id
    pub point_id: u64,
    /// 测点名
    pub point_name: String,
    /// 字符串id
    pub alias_id: String,
    /// 是否是离散量
    pub is_discrete: bool,
    /// 是否是计算点
    pub is_computing_point: bool,
    /// 如果是计算点，这是表达式
    pub expression: String,
    /// 变换公式
    pub trans_expr: String,
    /// 逆变换公式
    pub inv_trans_expr: String,
    /// 判断是否"变化"的公式，用于变化上传或储存
    pub change_expr: String,
    /// 判断是否为0值的公式
    pub zero_expr: String,
    /// 单位
    pub data_unit: String,
    #[serde(skip)]
    pub unit: DataUnit,
    /// 上限，用于坏数据辨识
    pub upper_limit: f64,
    /// 下限，用于坏数据辨识
    pub lower_limit: f64,
    /// 告警级别1的表达式
    pub alarm_level1_expr: String,
    #[serde(skip)]
    pub alarm_level1: Option<Expr>,
    /// 告警级别2的表达式
    pub alarm_level2_expr: String,
    #[serde(skip)]
    pub alarm_level2: Option<Expr>,
    /// 如是，则不判断是否"变化"，均上传
    pub is_realtime: bool,
    /// 是否是soe点
    pub is_soe: bool,
    /// 默认值存储在8个字节，需要根据is_discrete来转换成具体的值
    pub init_value: u64,
    /// Description
    pub desc: String,
    /// 标识该测点是否是采集点，在运行时根据测点是否属于通道来判断
    #[serde(skip)]
    pub is_remote: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MeasureValue {
    /// 对应的测点
    pub point_id: u64,
    /// 是否离散量
    pub is_discrete: bool,
    /// 时间戳
    pub timestamp: u64,
    /// 模拟量值
    pub analog_value: f64,
    /// 离散量值
    pub discrete_value: i64,
    /// 是否已经变换
    pub is_transformed: bool,
    /// 变换后的模拟量值
    pub transformed_analog: f64,
    /// 变换后的离散量值
    pub transformed_discrete: i64,
}

/**
 * @api {SerialParity} /SerialParity SerialParity
 * @apiPrivate
 * @apiGroup A_Enum
 * @apiSuccess {String} None None
 * @apiSuccess {String} Odd Odd
 * @apiSuccess {String} Even Even
 * @apiSuccess {String} Mark Mark
 * @apiSuccess {String} Space Space
 */
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub enum SerialParity {
    #[default]
    None = 0,
    Odd = 1,
    Even = 2,
    Mark = 3,
    Space = 4,
}

/**
 * @api {串口通道参数} /SerialPara SerialPara
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {String} file_path file_path
 * @apiSuccess {u32} baud_rate baud_rate
 * @apiSuccess {u8} data_bits data_bits
 * @apiSuccess {u8} stop_bits stop_bits
 * @apiSuccess {SerialParity} parity parity
 * @apiSuccess {u64} delay_between_requests delay_between_requests
 */
/// 串口通道
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct SerialPara {
    pub file_path: String,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub stop_bits: u8,
    pub parity: SerialParity,
    pub delay_between_requests: u64,
}

impl Default for SerialPara {
    fn default() -> Self {
        SerialPara {
            file_path: Default::default(),
            baud_rate: 9600,
            data_bits: 8,
            stop_bits: 1,
            parity: Default::default(),
            delay_between_requests: Default::default(),
        }
    }
}

/**
 * @api {整型指令数据} /SetIntValue SetIntValue
 * @apiGroup A_Object
 * @apiSuccess {u64} sender_id sender_id
 * @apiSuccess {u64} point_id point_id
 * @apiSuccess {i64} yk_command yk_command
 * @apiSuccess {u64} timestamp timestamp
 */
/// 指令数据
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetIntValue {
    pub sender_id: u64,
    pub point_id: u64,
    pub yk_command: i64,
    pub timestamp: u64,
}

/**
 * @api {整型指令数据} /SetIntValue2 SetIntValue2
 * @apiGroup A_Object
 * @apiSuccess {u64} sender_id sender_id
 * @apiSuccess {u64} point_alias point_alias
 * @apiSuccess {i64} yk_command yk_command
 * @apiSuccess {u64} timestamp timestamp
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetIntValue2 {
    pub sender_id: u64,
    pub point_alias: String,
    pub yk_command: i64,
    pub timestamp: u64,
}

/**
 * @api {浮点型指令数据} /SetFloatValue SetFloatValue
 * @apiGroup A_Object
 * @apiSuccess {u64} sender_id sender_id
 * @apiSuccess {u64} point_id point_id
 * @apiSuccess {f64} yt_command yt_command
 * @apiSuccess {u64} timestamp timestamp
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetFloatValue {
    pub sender_id: u64,
    pub point_id: u64,
    pub yt_command: f64,
    pub timestamp: u64,
}

/**
 * @api {浮点型指令数据} /SetFloatValue2 SetFloatValue2
 * @apiGroup A_Object
 * @apiSuccess {u64} sender_id sender_id
 * @apiSuccess {str} point_alias point_alias
 * @apiSuccess {f64} yt_command yt_command
 * @apiSuccess {u64} timestamp timestamp
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetFloatValue2 {
    pub sender_id: u64,
    pub point_alias: String,
    pub yt_command: f64,
    pub timestamp: u64,
}

/**
 * @api {公式型指令数据} /SetPointValue SetPointValue
 * @apiGroup A_Object
 * @apiSuccess {u64} sender_id sender_id
 * @apiSuccess {u64} point_id point_id
 * @apiSuccess {expr} command command
 * @apiSuccess {u64} timestamp timestamp
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetPointValue {
    pub sender_id: u64,
    pub point_id: u64,
    pub command: Expr,
    pub timestamp: u64,
}

#[repr(i8)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum SetPointStatus {
    YkCreated = 0,
    YtCreated,
    YkSuccess,
    YtSuccess,
    YkFailTimeout,
    YtFailTimeout,
    YkFailTooBusy,
    YtFailTooBusy,
    YkFailProtocol,
    YtFailProtocol,
}

pub fn is_yk(r: &PbSetPointResult) -> bool {
    matches!(
        r.status(),
        PbSetPointResult_SetPointStatus::YkCreated
            | PbSetPointResult_SetPointStatus::YkSuccess
            | PbSetPointResult_SetPointStatus::YkFailTimeout
            | PbSetPointResult_SetPointStatus::YkFailTooBusy
            | PbSetPointResult_SetPointStatus::YkFailProtocol
    )
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SetPointResult {
    pub sender_id: u64,
    pub point_id: u64,
    pub create_time: u64,
    pub finish_time: u64,
    pub command: u64,
    pub status: SetPointStatus,
}

impl SetPointResult {
    pub fn is_yk(&self) -> bool {
        matches!(
            self.status,
            SetPointStatus::YkCreated
                | SetPointStatus::YkFailTimeout
                | SetPointStatus::YkFailTooBusy
                | SetPointStatus::YkFailProtocol
                | SetPointStatus::YkSuccess
        )
    }

    pub fn get_float_command(&self) -> f64 {
        byteorder::BigEndian::read_f64(&self.command.to_be_bytes())
    }

    pub fn get_int_command(&self) -> i64 {
        byteorder::BigEndian::read_i64(&self.command.to_be_bytes())
    }

    pub fn create_pb_result(&self) -> PbSetPointResult {
        let mut result = PbSetPointResult::new();
        result.set_sender_id(self.sender_id);
        result.set_command(self.command);
        result.set_create_time(self.create_time);
        result.set_finish_time(self.finish_time);
        result.set_point_id(self.point_id);
        match &self.status {
            SetPointStatus::YkCreated => {
                result.set_status(PbSetPointResult_SetPointStatus::YkCreated)
            }
            SetPointStatus::YtCreated => {
                result.set_status(PbSetPointResult_SetPointStatus::YtCreated)
            }
            SetPointStatus::YkSuccess => {
                result.set_status(PbSetPointResult_SetPointStatus::YkSuccess)
            }
            SetPointStatus::YtSuccess => {
                result.set_status(PbSetPointResult_SetPointStatus::YtSuccess)
            }
            SetPointStatus::YkFailTimeout => {
                result.set_status(PbSetPointResult_SetPointStatus::YkFailTimeout)
            }
            SetPointStatus::YtFailTimeout => {
                result.set_status(PbSetPointResult_SetPointStatus::YtFailTimeout)
            }
            SetPointStatus::YkFailTooBusy => {
                result.set_status(PbSetPointResult_SetPointStatus::YkFailTooBusy)
            }
            SetPointStatus::YtFailTooBusy => {
                result.set_status(PbSetPointResult_SetPointStatus::YtFailTooBusy)
            }
            SetPointStatus::YkFailProtocol => {
                result.set_status(PbSetPointResult_SetPointStatus::YkFailProtocol)
            }
            SetPointStatus::YtFailProtocol => {
                result.set_status(PbSetPointResult_SetPointStatus::YtFailProtocol)
            }
        }
        result
    }

    pub fn from(r: PbSetPointResult) -> Self {
        let status = match r.status() {
            PbSetPointResult_SetPointStatus::YkCreated => SetPointStatus::YkCreated,
            PbSetPointResult_SetPointStatus::YtCreated => SetPointStatus::YtCreated,
            PbSetPointResult_SetPointStatus::YkSuccess => SetPointStatus::YkSuccess,
            PbSetPointResult_SetPointStatus::YtSuccess => SetPointStatus::YtSuccess,
            PbSetPointResult_SetPointStatus::YkFailTimeout => SetPointStatus::YkFailTimeout,
            PbSetPointResult_SetPointStatus::YtFailTimeout => SetPointStatus::YtFailTimeout,
            PbSetPointResult_SetPointStatus::YkFailTooBusy => SetPointStatus::YkFailTooBusy,
            PbSetPointResult_SetPointStatus::YtFailTooBusy => SetPointStatus::YtFailTooBusy,
            PbSetPointResult_SetPointStatus::YkFailProtocol => SetPointStatus::YkFailProtocol,
            PbSetPointResult_SetPointStatus::YtFailProtocol => SetPointStatus::YtFailProtocol,
        };
        Self {
            sender_id: r.sender_id(),
            point_id: r.point_id(),
            create_time: r.create_time(),
            finish_time: r.finish_time(),
            command: r.command(),
            status,
        }
    }
}

impl Display for Measurement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //point_id, point_name, dev_id, is_discrete, is_computing_point,
        //expression, trans_expression,inv_trans_expression,change_expression,zero_expression
        //data_unit, upper_limit, lower_limit, max_gradient, min_gradient
        //is_realtime,is_soe,init_value,
        let init_v = if self.is_discrete {
            get_i64_str(self.get_init_discrete())
        } else {
            get_f64_str(self.get_init_analog())
        };
        let upper_limit = if self.upper_limit == f64::MAX {
            "".to_string()
        } else {
            self.upper_limit.to_string()
        };
        let lower_limit = if self.lower_limit == f64::MIN {
            "".to_string()
        } else {
            self.lower_limit.to_string()
        };
        let str = format!(
            "{},{},{},{},{},{},{},{},{},{},{},{:?},{:?},{},{},{},{},{}",
            self.point_id,
            get_csv_str(&self.point_name),
            get_csv_str(&self.alias_id),
            if self.is_discrete { "TRUE" } else { "FALSE" },
            if self.is_computing_point { "TRUE" } else { "FALSE" },
            get_csv_str(&self.expression),
            get_csv_str(&self.trans_expr),
            get_csv_str(&self.inv_trans_expr),
            get_csv_str(&self.change_expr),
            get_csv_str(&self.zero_expr),
            self.data_unit,
            upper_limit,
            lower_limit,
            get_csv_str(&self.alarm_level1_expr),
            get_csv_str(&self.alarm_level2_expr),
            if self.is_realtime { "TRUE" } else { "FALSE" },
            if self.is_soe { "TRUE" } else { "FALSE" },
            init_v,
        );
        write!(f, "{}", str)
    }
}

impl MeasureValue {

    pub fn init_discrete_with_time(
        point_id: u64,
        discrete_value: i64,
        timestamp: u64,
    ) -> MeasureValue {
        MeasureValue {
            point_id,
            is_discrete: true,
            timestamp,
            analog_value: 0.0,
            discrete_value,
            is_transformed: false,
            transformed_analog: 0.0,
            transformed_discrete: 0,
        }
    }

    /// 生成浮点数存储的测点值对象
    pub fn init_analog_with_time(point_id: u64, analog_value: f64, timestamp: u64) -> MeasureValue {
        MeasureValue {
            point_id,
            is_discrete: false,
            timestamp,
            analog_value,
            discrete_value: 0,
            is_transformed: false,
            transformed_analog: 0.0,
            transformed_discrete: 0,
        }
    }

    /// 生成bool型测点值对象
    pub fn create_bool_measure(
        point_id: u64,
        b: bool,
        timestamp: u64,
        is_discrete: bool,
    ) -> MeasureValue {
        let discrete_value = if is_discrete {
            if b { 1 } else { 0 }
        } else {
            0
        };
        let analog_value = if !is_discrete {
            if b { 1.0 } else { 0.0 }
        } else {
            0.0
        };
        MeasureValue {
            point_id,
            is_discrete,
            timestamp,
            analog_value,
            discrete_value,
            is_transformed: false,
            transformed_analog: 0.0,
            transformed_discrete: 0,
        }
    }

    /// 取测点的值，如果经过了变换则返回变换后的值
    pub fn get_value(&self) -> f64 {
        if self.is_discrete {
            if self.is_transformed {
                self.transformed_discrete as f64
            } else {
                self.discrete_value as f64
            }
        } else if self.is_transformed {
            self.transformed_analog
        } else {
            self.analog_value
        }
    }

    pub fn get_value2(&self) -> i64 {
        if self.is_discrete {
            if self.is_transformed {
                self.transformed_discrete
            } else {
                self.discrete_value
            }
        } else if self.is_transformed {
            self.transformed_analog as i64
        } else {
            self.analog_value as i64
        }
    }

    /// 计算偏差
    pub fn get_error(&self, new_m: &MeasureValue) -> f64 {
        new_m.get_value() - self.get_value()
    }

    pub fn update_time(&mut self, t: u64) {
        self.timestamp = t;
    }

    /// 更新测点值
    pub fn update(&mut self, new_m: &MeasureValue) {
        // 如果已经修改了类型，不再更新
        if self.is_discrete != new_m.is_discrete {
            return;
        }
        if self.is_discrete {
            self.discrete_value = new_m.discrete_value;
            if new_m.is_transformed {
                self.transformed_discrete = new_m.transformed_discrete;
            }
        } else {
            self.analog_value = new_m.analog_value;
            if new_m.is_transformed {
                self.transformed_analog = new_m.transformed_analog;
            }
        }
        self.is_transformed = new_m.is_transformed;
        self.timestamp = new_m.timestamp;
    }

    pub fn is_same_value(&self, new_m: &MeasureValue) -> bool {
        if new_m.is_discrete {
            new_m.discrete_value == self.discrete_value
        } else if new_m.is_transformed {
            // 比较变换之后的值
            new_m.transformed_analog == self.transformed_analog
        } else {
            new_m.analog_value == self.analog_value
        }
    }
}

impl SetIntValue {
    /// 生成设定指令
    pub fn from_bool(sender_id: u64, point_id: u64, b: bool, timestamp: u64) -> SetIntValue {
        let discrete_value = if b { 1 } else { 0 };
        SetIntValue {
            sender_id,
            point_id,
            yk_command: discrete_value,
            timestamp,
        }
    }
}

pub fn from_csv_to_map<R: io::Read>(
    mut rdr: Reader<R>,
    start_row: usize,
    map: &mut HashMap<u64, Measurement>,
    id_check: bool,
) -> Result<Vec<(String, Vec<u64>)>, (usize, usize)> {
    let mut records = rdr.records();
    let offset: usize = 1;
    let mut row: usize = start_row;
    let mut assist_map: BTreeMap<String, Vec<u64>> = BTreeMap::new();
    while let Some(Ok(record)) = records.next() {
        let rc = (row, offset);
        let point_id = csv_u64(&record, rc.1).ok_or(rc)?;
        if id_check && point_id < MINIMUM_POINT_ID {
            return Err(rc);
        }
        let rc = (row, offset + 1);
        let point_name = csv_string(&record, rc.1).ok_or(rc)?;
        let rc = (row, offset + 2);
        // check alias_id
        let mut alias_id = csv_string(&record, rc.1).ok_or(rc)?;
        if !alias_id.is_empty() {
            if let Ok(alias_expr) = alias_id.parse::<Expr>() {
                for token in &alias_expr.rpn {
                    match token {
                        Token::Var(n) => alias_id = n.clone(),
                        _ => return Err(rc),
                    }
                }
            } else {
                return Err(rc);
            }
        }
        let rc = (row, offset + 3);
        let s = csv_str(&record, rc.1).ok_or(rc)?;
        let is_discrete = match s.to_uppercase().as_str() {
            "FALSE" => false,
            "TRUE" => true,
            _ => false,
        };
        let rc = (row, offset + 4);
        let s = csv_str(&record, rc.1).ok_or(rc)?;
        let is_computing_point = match s.to_uppercase().as_str() {
            "FALSE" => false,
            "TRUE" => true,
            _ => false,
        };
        let rc = (row, offset + 5);
        let expression = csv_string(&record, rc.1).ok_or(rc)?;
        if !expression.is_empty() {
            expression.parse::<Expr>().map_err(|_| rc)?;
        }
        let rc = (row, offset + 6);
        let trans_expr = csv_string(&record, rc.1).ok_or(rc)?;
        if !trans_expr.is_empty() {
            trans_expr.parse::<Expr>().map_err(|_| rc)?;
        }
        let rc = (row, offset + 7);
        let inv_trans_expr = csv_string(&record, rc.1).ok_or(rc)?;
        if !inv_trans_expr.is_empty() {
            inv_trans_expr.parse::<Expr>().map_err(|_| rc)?;
        }
        let rc = (row, offset + 8);
        let change_expr = csv_string(&record, rc.1).ok_or(rc)?;
        if !change_expr.is_empty() {
            change_expr.parse::<Expr>().map_err(|_| rc)?;
        }
        let rc = (row, offset + 9);
        let zero_expr = csv_string(&record, rc.1).ok_or(rc)?;
        if !zero_expr.is_empty() {
            zero_expr.parse::<Expr>().map_err(|_| rc)?;
        }
        let rc = (row, offset + 10);
        let data_unit = csv_string(&record, rc.1).ok_or(rc)?;
        let unit = DataUnit::from_str(&data_unit).or(Err(rc))?;
        let rc = (row, offset + 11);
        let upper_limit_str = csv_string(&record, rc.1).ok_or(rc)?;
        let mut upper_limit = f64::MAX;
        if !upper_limit_str.is_empty() {
            upper_limit = upper_limit_str.parse::<f64>().map_err(|_| rc)?;
        }
        let rc = (row, offset + 12);
        let lower_limit_str = csv_string(&record, rc.1).ok_or(rc)?;
        let mut lower_limit = f64::MIN;
        if !lower_limit_str.is_empty() {
            lower_limit = lower_limit_str.parse::<f64>().map_err(|_| rc)?;
        }
        if lower_limit > upper_limit {
            return Err(rc);
        }
        let rc = (row, offset + 13);
        let alarm_level1_expr = csv_string(&record, rc.1).ok_or(rc)?;
        let alarm_level1 = if !alarm_level1_expr.is_empty() {
            Some(alarm_level1_expr.parse::<Expr>().map_err(|_| rc)?)
        } else {
            None
        };
        let rc = (row, offset + 14);
        let alarm_level2_expr = csv_string(&record, rc.1).ok_or(rc)?;
        let alarm_level2 = if !alarm_level2_expr.is_empty() {
            Some(alarm_level2_expr.parse::<Expr>().map_err(|_| rc)?)
        } else {
            None
        };
        let rc = (row, offset + 15);
        let s = csv_str(&record, rc.1).ok_or(rc)?;
        let is_realtime = match s.to_uppercase().as_str() {
            "FALSE" => false,
            "TRUE" => true,
            _ => false,
        };
        let rc = (row, offset + 16);
        let s = csv_str(&record, rc.1).ok_or(rc)?;
        let is_soe = match s.to_uppercase().as_str() {
            "FALSE" => false,
            "TRUE" => true,
            _ => false,
        };
        let init_value = if is_discrete {
            let rc = (row, offset + 17);
            let v = csv_i64(&record, rc.1).ok_or(rc)?;
            let bytes = v.to_be_bytes();
            BigEndian::read_u64(&bytes)
        } else {
            let rc = (row, offset + 17);
            let v = csv_f64(&record, rc.1).ok_or(rc)?;
            let bytes = v.to_be_bytes();
            BigEndian::read_u64(&bytes)
        };
        // tags
        if let Some(tags) = csv_str(&record, offset + 18) {
            let tag_vec: Vec<&str> = tags.split(';').collect();
            for tag in tag_vec {
                let tag = tag.trim();
                if tag.is_empty() {
                    continue;
                }
                if let Some(v) = assist_map.get_mut(tag) {
                    v.push(point_id);
                } else {
                    assist_map.insert(tag.to_string(), vec![point_id]);
                }
            }
        }
        // desc
        let desc = if let Some(desc) = csv_string(&record, offset + 19) {
            desc
        } else {
            "".to_string()
        };
        map.insert(
            point_id,
            Measurement {
                point_id,
                point_name,
                alias_id,
                is_discrete,
                is_computing_point,
                expression,
                trans_expr,
                inv_trans_expr,
                change_expr,
                zero_expr,
                data_unit,
                upper_limit,
                lower_limit,
                alarm_level1_expr,
                alarm_level1,
                alarm_level2_expr,
                alarm_level2,
                is_realtime,
                is_soe,
                init_value,
                unit,
                desc,
                is_remote: false,
            },
        );
        row += 1;
    }
    let mut result = Vec::with_capacity(assist_map.len());
    for (k, v) in assist_map {
        result.push((k, v));
    }
    Ok(result)
}

pub fn from_file(path: &str) -> Result<(HashMap<u64, Measurement>, Vec<(String, Vec<u64>)>), (usize, usize)> {
    let content = std::fs::read(path).map_err(|_| (0, 0))?;
    // let content = if env::IS_ENCRYPT {
    //     decrypt_vec(content.as_slice())
    // } else {
    //     content
    // };
    let csv_bytes = if path.ends_with(".xlsx") || path.ends_with(".xls") {
        let r = excel_bytes_to_csv_bytes(content.as_slice()).unwrap_or_default();
        if r.is_empty() {
            return Err((0, 0));
        }
        r[0].clone()
    } else {
        content
    };

    from_csv_bytes(csv_bytes.as_slice(), true)
}

pub fn from_file2(path: &str, id_check: bool) -> Result<(HashMap<u64, Measurement>, Vec<(String, Vec<u64>)>), (usize, usize)> {
    let content = std::fs::read(path).map_err(|_| (0, 0))?;
    // let content = if env::IS_ENCRYPT {
    //     decrypt_vec(content.as_slice())
    // } else {
    //     content
    // };
    let csv_bytes = if path.ends_with(".xlsx") || path.ends_with(".xls") {
        let r = excel_bytes_to_csv_bytes(content.as_slice()).unwrap_or_default();
        if r.is_empty() {
            return Err((0, 0));
        }
        r[0].clone()
    } else {
        content
    };

    from_csv_bytes2(csv_bytes.as_slice(), true, id_check)
}

pub fn from_csv(path: &str) -> Result<(HashMap<u64, Measurement>, Vec<(String, Vec<u64>)>), (usize, usize)> {
    let content = std::fs::read(path).map_err(|_| (0, 0))?;
    // if env::IS_ENCRYPT {
    //     let content = decrypt_vec(content.as_slice());
    //     from_csv_bytes(content.as_slice(), true)
    // } else {
    //     from_csv_bytes(content.as_slice(), true)
    // }
    from_csv_bytes(content.as_slice(), true)
}

pub fn from_csv2(path: &str, id_check: bool) -> Result<(HashMap<u64, Measurement>, Vec<(String, Vec<u64>)>), (usize, usize)> {
    let content = std::fs::read(path).map_err(|_| (0, 0))?;
    // if env::IS_ENCRYPT {
    //     let content = decrypt_vec(content.as_slice());
    //     from_csv_bytes2(content.as_slice(), true, id_check)
    // } else {
    //     from_csv_bytes2(content.as_slice(), true, id_check)
    // }
    from_csv_bytes2(content.as_slice(), true, id_check)
}

pub fn from_csv_bytes(
    content: &[u8],
    has_headers: bool,
) -> Result<(HashMap<u64, Measurement>, Vec<(String, Vec<u64>)>), (usize, usize)> {
    let content_new = transfer_to_utf8(content.to_vec()).map_err(|_| (0, 0))?;
    let content = content_new.as_slice();
    let rdr = csv::ReaderBuilder::new()
        .has_headers(has_headers)
        .from_reader(content);
    let start_row = if has_headers { 1 } else { 0 };
    let mut map: HashMap<u64, Measurement> = HashMap::new();
    let tags = from_csv_to_map(rdr, start_row, &mut map, true)?;
    map.shrink_to_fit();
    Ok((map, tags))
}

pub fn from_csv_bytes2(
    content: &[u8],
    has_headers: bool,
    id_check: bool,
) -> Result<(HashMap<u64, Measurement>, Vec<(String, Vec<u64>)>), (usize, usize)> {
    let content_new = transfer_to_utf8(content.to_vec()).map_err(|_| (0, 0))?;
        let content = content_new.as_slice();
    let rdr = csv::ReaderBuilder::new()
        .has_headers(has_headers)
        .from_reader(content);
    let start_row = if has_headers { 1 } else { 0 };
    let mut map: HashMap<u64, Measurement> = HashMap::new();
    let tags = from_csv_to_map(rdr, start_row, &mut map, id_check)?;
    map.shrink_to_fit();
    Ok((map, tags))
}

pub fn init_discrete_point(point_id: u64, init_v: i64) -> Measurement {
    let bytes = init_v.to_be_bytes();
    let init_value = BigEndian::read_u64(&bytes);
    Measurement {
        point_id,
        point_name: "".to_string(),
        alias_id: "".to_string(),
        is_discrete: true,
        is_computing_point: false,
        expression: "".to_string(),
        trans_expr: "".to_string(),
        inv_trans_expr: "".to_string(),
        change_expr: "".to_string(),
        zero_expr: "".to_string(),
        data_unit: "".to_string(),
        unit: DataUnit::UnitOne,
        upper_limit: f64::MAX,
        lower_limit: f64::MIN,
        alarm_level1_expr: "".to_string(),
        alarm_level1: None,
        alarm_level2_expr: "".to_string(),
        alarm_level2: None,
        is_realtime: false,
        is_soe: false,
        is_remote: false,
        init_value,
        desc: "".to_string(),
    }
}

pub fn init_analog_point(point_id: u64, init_v: f64) -> Measurement {
    let bytes = init_v.to_be_bytes();
    let init_value = BigEndian::read_u64(&bytes);
    Measurement {
        point_id,
        point_name: "".to_string(),
        alias_id: "".to_string(),
        is_discrete: false,
        is_computing_point: false,
        expression: "".to_string(),
        trans_expr: "".to_string(),
        inv_trans_expr: "".to_string(),
        change_expr: "".to_string(),
        zero_expr: "".to_string(),
        data_unit: "".to_string(),
        unit: DataUnit::UnitOne,
        upper_limit: f64::MAX,
        lower_limit: f64::MIN,
        alarm_level1_expr: "".to_string(),
        alarm_level1: None,
        alarm_level2_expr: "".to_string(),
        alarm_level2: None,
        is_realtime: false,
        is_soe: false,
        is_remote: false,
        init_value,
        desc: "".to_string(),
    }
}

pub fn init_computing_point(point_id: u64, expr: &str, is_discrete: bool) -> Measurement {
    Measurement {
        point_id,
        point_name: "".to_string(),
        alias_id: "".to_string(),
        is_discrete,
        is_computing_point: true,
        expression: expr.to_string(),
        trans_expr: "".to_string(),
        inv_trans_expr: "".to_string(),
        change_expr: "".to_string(),
        zero_expr: "".to_string(),
        data_unit: "".to_string(),
        unit: DataUnit::UnitOne,
        upper_limit: f64::MAX,
        lower_limit: f64::MIN,
        alarm_level1_expr: "".to_string(),
        alarm_level1: None,
        alarm_level2_expr: "".to_string(),
        alarm_level2: None,
        is_realtime: false,
        is_soe: false,
        is_remote: false,
        init_value: 0,
        desc: "".to_string(),
    }
}

impl Measurement {
    pub fn get_init_discrete(&self) -> i64 {
        let buf = self.init_value.to_be_bytes();
        BigEndian::read_i64(&buf)
    }

    pub fn get_init_analog(&self) -> f64 {
        let buf = self.init_value.to_be_bytes();
        BigEndian::read_f64(&buf)
    }
}

pub fn csv_str(record: &StringRecord, col: usize) -> Option<&str> {
    Some(record.get(col)?.trim())
}

pub fn csv_string(record: &StringRecord, col: usize) -> Option<String> {
    Some(record.get(col)?.trim().to_string())
}

pub fn csv_usize(record: &StringRecord, col: usize) -> Option<usize> {
    let s = record.get(col)?.to_string();
    let r = s.parse().ok()?;
    Some(r)
}

pub fn csv_u8(record: &StringRecord, col: usize) -> Option<u8> {
    let s = record.get(col)?.trim();
    let r = if s.starts_with("0x") {
        u8::from_str_radix(s.trim_start_matches("0x"), 16).ok()?
    } else {
        s.parse().ok()?
    };
    Some(r)
}

pub fn csv_u16(record: &StringRecord, col: usize) -> Option<u16> {
    let s = record.get(col)?.trim();
    let r = if s.starts_with("0x") {
        u16::from_str_radix(s.trim_start_matches("0x"), 16).ok()?
    } else {
        s.parse().ok()?
    };
    Some(r)
}

pub fn csv_u32(record: &StringRecord, col: usize) -> Option<u32> {
    let s = record.get(col)?.trim();
    let r = if s.starts_with("0x") {
        u32::from_str_radix(s.trim_start_matches("0x"), 16).ok()?
    } else {
        s.parse().ok()?
    };
    Some(r)
}

pub fn csv_u64(record: &StringRecord, col: usize) -> Option<u64> {
    let s = record.get(col)?.trim();
    let r = if s.starts_with("0x") {
        u64::from_str_radix(s.trim_start_matches("0x"), 16).ok()?
    } else {
        s.parse().ok()?
    };
    Some(r)
}

pub fn csv_i8(record: &StringRecord, col: usize) -> Option<i8> {
    let s = record.get(col)?.trim();
    let r = if s.starts_with("0x") {
        i8::from_str_radix(s.trim_start_matches("0x"), 16).ok()?
    } else {
        s.parse().ok()?
    };
    Some(r)
}

pub fn csv_i16(record: &StringRecord, col: usize) -> Option<i16> {
    let s = record.get(col)?.trim();
    let r = if s.starts_with("0x") {
        i16::from_str_radix(s.trim_start_matches("0x"), 16).ok()?
    } else {
        s.parse().ok()?
    };
    Some(r)
}

pub fn csv_i32(record: &StringRecord, col: usize) -> Option<i32> {
    let s = record.get(col)?.trim();
    let r = if s.starts_with("0x") {
        i32::from_str_radix(s.trim_start_matches("0x"), 16).ok()?
    } else {
        s.parse().ok()?
    };
    Some(r)
}

pub fn csv_i64(record: &StringRecord, col: usize) -> Option<i64> {
    let s = record.get(col)?.trim();
    let r = if s.starts_with("0x") {
        i64::from_str_radix(s.trim_start_matches("0x"), 16).ok()?
    } else {
        s.parse().ok()?
    };
    Some(r)
}

pub fn csv_f64(record: &StringRecord, col: usize) -> Option<f64> {
    let s = record.get(col)?.trim();
    let r = s.parse().ok()?;
    Some(r)
}

pub fn csv_f32(record: &StringRecord, col: usize) -> Option<f32> {
    let s = record.get(col)?.trim();
    let r = s.parse().ok()?;
    Some(r)
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataUnitError {
    UnknownDataUnit(String),
}

pub fn create_parity(v: &str) -> SerialParity {
    if v.to_uppercase() == "NONE" {
        SerialParity::None
    } else if v.to_uppercase() == "ODD" {
        SerialParity::Odd
    } else if v.to_uppercase() == "EVEN" {
        SerialParity::Even
    } else if v.to_uppercase() == "MARK" {
        SerialParity::Mark
    } else if v.to_uppercase() == "SPACE" {
        SerialParity::Space
    } else {
        SerialParity::None
    }
}

pub fn export_points_header(text_map: &HashMap<String, String>) -> String {
    format!(
        "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
        text_map.get("index").unwrap_or(&"Index".to_string()),
        text_map.get("point_id").unwrap_or(&"ID".to_string()),
        text_map.get("name").unwrap_or(&"Name".to_string()),
        text_map.get("alias").unwrap_or(&"Alias".to_string()),
        text_map.get("is_discrete").unwrap_or(&"Is Discrete".to_string()),
        text_map.get("is_computing").unwrap_or(&"Is Computing".to_string()),
        text_map.get("point_expression").unwrap_or(&"Expression".to_string()),
        text_map.get("trans_expr").unwrap_or(&"Trans Expr".to_string()),
        text_map.get("inv_trans_expr").unwrap_or(&"Inv Trans Expr".to_string()),
        text_map.get("change_expr").unwrap_or(&"Change Expr".to_string()),
        text_map.get("zero_expr").unwrap_or(&"Zero Expr".to_string()),
        text_map.get("unit").unwrap_or(&"Unit".to_string()),
        text_map.get("upper_limit").unwrap_or(&"Upper Limit".to_string()),
        text_map.get("lower_limit").unwrap_or(&"Lower Limit".to_string()),
        text_map.get("alarm_level1").unwrap_or(&"Alarm 1".to_string()),
        text_map.get("alarm_level2").unwrap_or(&"Alarm 2".to_string()),
        text_map.get("is_realtime").unwrap_or(&"Is Realtime".to_string()),
        text_map.get("is_soe").unwrap_or(&"Is SOE".to_string()),
        text_map.get("point_init").unwrap_or(&"Initial".to_string()),
        text_map.get("tags").unwrap_or(&"Tags".to_string()),
        text_map.get("desc").unwrap_or(&"Description".to_string()),
    )
}

pub fn export_points_csv(points: &[Measurement], tags_map: &HashMap<u64, String>,
                        text_map: &HashMap<String, String>) -> String {
    let mut points_csv = export_points_header(text_map);
    if !points.is_empty() {
        points_csv.push('\n');
    }
    for i in 0..points.len() {
        points_csv.push_str((i + 1).to_string().as_str());
        points_csv.push(',');
        points_csv += &points[i].to_string();
        points_csv.push(',');
        if let Some(s) = tags_map.get(&points[i].point_id) {
            points_csv += &get_csv_str(s.as_str());
        }
        points_csv.push(',');
        points_csv += &get_csv_str(&points[i].desc);
        if i != points.len() - 1 {
            points_csv += "\n";
        }
    }
    points_csv
}

pub fn parse_set_points_csv( content: &[u8],
                             has_headers: bool,
                             points: &[Measurement],
) -> Result<(Vec<SetIntValue>,Vec<SetFloatValue>), (usize, usize)> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(has_headers)
        .from_reader(content);
    let mut alias_to_pos = HashMap::new();
    let mut id_to_pos = HashMap::with_capacity(points.len());
    for (i, point) in points.iter().enumerate() {
        alias_to_pos.insert(point.alias_id.clone(), i);
        id_to_pos.insert(point.point_id, i);
    }
    let start_row = if has_headers { 1 } else { 0 };
    let mut records = rdr.records();
    let offset: usize = 1;
    let mut row: usize = start_row;
    let mut set_int_values = Vec::new();
    let mut set_float_values = Vec::new();
    while let Some(Ok(record)) = records.next() {
        let rc = (row, offset);
        let index = if let Some(point_id) = csv_u64(&record, rc.1) {
            *id_to_pos.get(&point_id).ok_or(rc)?
        } else {
            let alias = csv_str(&record, rc.1).ok_or(rc)?;
            *alias_to_pos.get(alias).ok_or(rc)?
        };
        let point_id= points[index].point_id;
        if points[index].is_discrete {
            let rc = (row, offset + 1);
            let v = csv_i64(&record, rc.1).ok_or(rc)?;
            set_int_values.push(SetIntValue {
                sender_id: 0,
                point_id,
                yk_command: v,
                timestamp: 0,
            });
        } else {
            let rc = (row, offset + 1);
            let v = csv_f64(&record, rc.1).ok_or(rc)?;
            set_float_values.push(SetFloatValue {
                sender_id: 0,
                point_id,
                yt_command: v,
                timestamp: 0,
            });
        }
        row += 1;
    }
    Ok((set_int_values, set_float_values))
}

pub fn get_csv_str(s : &str) -> String {
    if s.contains(',') || s.contains('\n') || s.contains('"')
        || s.starts_with(' ') || s.ends_with(' ') {
        format!("\"{}\"", s.replace('\"', "\"\""))
    } else {
        s.to_string()
    }
}

//至少保留1位小数，且最多8位有效数字
pub fn get_f64_str(num: f64) -> String {
    let length = 8;
    let num_integer = count_integer_places(num);
    let num_decimal = count_decimal_places(num);
    if (num_integer > length - 1)
        || ((num != 0.0) && (num.abs() < 0.1f64.powi((length - 1) as i32))) {
        format!("{:.dec$e}", num, dec = length - 1)
    } else {
        let remain = length - num_integer;
        format!("{:.dec$}", num, dec = 1.max(num_decimal.min(remain)))
    }
}


/// 限制显示数值的长度，离散值
pub fn get_i64_str(num: i64) -> String {
    if num.abs() > 1000000i64 {
        format!("{:.6e}", num)
    } else {
        format!("{}", num)
    }
}

/// 计算整数位数
fn count_integer_places(num: f64) -> usize {
    let remainder = num.abs() as i64;
    remainder.to_string().len()
}

/// 计算小数位数, 17 means it > 17
fn count_decimal_places(num: f64) -> usize {
    let tmp = format!("{:?}", num);
    if tmp.contains('e') {
        return 17; // magic number, 做成常量比较好
    };
    if tmp.contains('.') {
        tmp.split('.').collect::<Vec<&str>>()[1].len()
    } else {
        0
    }
}

fn serialize_enum_or_unknown<E: EnumFull, S: serde::Serializer>(
    e: &Option<EnumOrUnknown<E>>,
    s: S,
) -> Result<S::Ok, S::Error> {
    if let Some(e) = e {
        match e.enum_value() {
            Ok(v) => s.serialize_str(v.descriptor().name()),
            Err(v) => s.serialize_i32(v),
        }
    } else {
        s.serialize_unit()
    }
}

fn deserialize_enum_or_unknown<'de, E: EnumFull, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Option<EnumOrUnknown<E>>, D::Error> {
    struct DeserializeEnumVisitor<E: EnumFull>(PhantomData<E>);

    impl<'de, E: EnumFull> serde::de::Visitor<'de> for DeserializeEnumVisitor<E> {
        type Value = Option<EnumOrUnknown<E>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "a string, an integer or none")
        }

        fn visit_i32<R>(self, v: i32) -> Result<Self::Value, R>
        where
            R: serde::de::Error,
        {
            Ok(Some(EnumOrUnknown::from_i32(v)))
        }

        fn visit_str<R>(self, v: &str) -> Result<Self::Value, R>
        where
            R: serde::de::Error,
        {
            match E::enum_descriptor().value_by_name(v) {
                Some(v) => Ok(Some(EnumOrUnknown::from_i32(v.value()))),
                None => Err(serde::de::Error::custom(format!(
                    "unknown enum value: {}",
                    v
                ))),
            }
        }

        fn visit_unit<R>(self) -> Result<Self::Value, R>
        where
            R: serde::de::Error,
        {
            Ok(None)
        }
    }

    d.deserialize_any(DeserializeEnumVisitor(PhantomData))
}

#[cfg(test)]
mod tests {
    // use bytes::{Buf, BufMut, BytesMut};
    // use protobuf::Message;
    // use protobuf::error::WireError;
    // use crate::PbMessage;

    use crate::from_csv2;
    use crate::{from_csv_bytes2, Measurement, PbSetIntPoint, TransportType};

    // fn parse_pb_message(buf: &BytesMut) -> ProtobufResult<(usize, PbMessage)> {
    //     let mut is = CodedInputStream::from_bytes(buf);
    //     let mut msg = PbMessage::new();
    //     while !is.eof()? {
    //         let (field_number, _) = is.read_tag_unpack()?;
    //         match field_number {
    //             1 => {
    //                 let mut topic = String::new();
    //                 is.read_string_into(&mut topic)?;
    //                 msg.set_topic(topic);
    //             }
    //             2 => {
    //                 let mut content = Vec::new();
    //                 is.read_bytes_into(&mut content)?;
    //                 msg.set_content(content);
    //                 break;
    //             }
    //             _ => {
    //                 return Err(WireError(WireError::IncorrectTag(
    //                     field_number,
    //                 )));
    //             }
    //         };
    //     }
    //     Ok((is.pos() as usize, msg))
    // }

    // #[test]
    // fn test_protobuf_parse() {
    //     let mut msg1 = PbMessage::new();
    //     msg1.set_topic("t1".to_string());
    //     msg1.set_content(b"abcdefg".to_vec());
    //     let mut msg2 = PbMessage::new();
    //     msg2.set_topic("t2".to_string());
    //     msg2.set_content(b"123abcdefg".to_vec());
    //     let bytes1 = msg1.write_to_bytes().unwrap();
    //     let bytes2 = msg2.write_to_bytes().unwrap();
    //     let mut buf = BytesMut::with_capacity(bytes1.len() + bytes2.len() + 2);
    //     buf.extend_from_slice(bytes1.as_slice());
    //     // 加入一些干扰的数据
    //     buf.extend_from_slice(b"11");
    //     let len = bytes2.len();
    //     // 第二个对象数据不全
    //     buf.extend_from_slice(&bytes2[0..len - 1]);
    //     let mut result = Vec::with_capacity(2);
    //     while !buf.is_empty() {
    //         match parse_pb_message(&buf) {
    //             Ok((needed, msg)) => {
    //                 buf.advance(needed);
    //                 result.push(msg);
    //             }
    //             Err(WireError(WireError::UnexpectedEof)) => {
    //                 break;
    //             }
    //             Err(e) => {
    //                 println!("{:?}", e);
    //                 buf.advance(1);
    //             }
    //         }
    //     }
    //     // 把第二个对象的数据补全
    //     buf.put_u8(bytes2[len - 1]);
    //     while !buf.is_empty() {
    //         match parse_pb_message(&buf) {
    //             Ok((needed, msg)) => {
    //                 buf.advance(needed);
    //                 result.push(msg);
    //             }
    //             Err(WireError(WireError::UnexpectedEof)) => {
    //                 break;
    //             }
    //             Err(e) => {
    //                 println!("{:?}", e);
    //                 buf.advance(1);
    //             }
    //         }
    //     }
    //     assert_eq!(result.len(), 2);
    //     assert_eq!(result[0], msg1);
    //     assert_eq!(result[1], msg2);
    // }

    #[test]
    fn test_proto_serde() {
        let mut o = PbSetIntPoint::new();
        o.set_pointId(1);
        let s = serde_json::to_string(&o).unwrap();
        assert!(!s.is_empty());
    }

    #[test]
    fn test_point_from_csv() {
        let (points, _) = from_csv2("tests/points-test1.csv", false).unwrap();
        assert_eq!(points.len(), 10);
        for i in 100001..100009 {
            let p = points.get(&i).unwrap();
            assert_eq!(p.get_init_discrete(), (i - 100000) as i64);
        }
        let p = points.get(&100009).unwrap();
        assert_eq!(p.get_init_discrete(), -1);
        let p = points.get(&100010).unwrap();
        assert_eq!(p.get_init_analog(), 10.99);

        let mut csv_str = String::new();
        let mut i = 1;
        for p in points.values() {
            csv_str.push_str(i.to_string().as_str());
            csv_str.push(',');
            csv_str.push_str(p.to_string().as_str());
            csv_str.push('\n');
            i += 1;
        }
        let (points2, _) = from_csv_bytes2(csv_str.as_bytes(), false, false).unwrap();
        assert_eq!(points, points2);
    }

    #[test]
    fn test_from_csv_bytes() {
        let mut p = Measurement {
            point_id: 1000,
            point_name: "abc".to_string(),
            alias_id: "abcd".to_string(),
            is_discrete: false,
            is_computing_point: false,
            expression: "".to_string(),
            trans_expr: "".to_string(),
            inv_trans_expr: "".to_string(),
            change_expr: "".to_string(),
            zero_expr: "".to_string(),
            data_unit: "".to_string(),
            unit: crate::DataUnit::UnitOne,
            upper_limit: f64::MAX,
            lower_limit: f64::MIN,
            alarm_level1_expr: "".to_string(),
            alarm_level1: None,
            alarm_level2_expr: "".to_string(),
            alarm_level2: None,
            is_realtime: false,
            is_soe: false,
            is_remote: false,
            init_value: 0,
            desc: "".to_string(),
        };
        let mut s = format!("1,{}", p.to_string());
        s.push('\n');
        p.point_id = 2000;
        p.data_unit = "kV".to_string();
        s.push_str(format!("2,{}", p.to_string()).as_str());
        s.push('\n');
        println!("{}", s);
        let (r, _) = from_csv_bytes2(s.into_bytes().as_slice(), false, false).unwrap();
        assert_eq!(2, r.len());
        assert_eq!(1000, r.get(&1000).unwrap().point_id);
        assert_eq!(2000, r.get(&2000).unwrap().point_id);
        assert_eq!("kV", r.get(&2000).unwrap().data_unit);
    }

    #[test]
    fn test_parse_radix() {
        let s = "0x00010000";
        let r = u32::from_str_radix(s.trim_start_matches("0x"), 16).unwrap();
        assert_eq!(0x00_01_00_00, r);

        let s = "0x00C2000C";
        let r = u32::from_str_radix(s.trim_start_matches("0x"), 16).unwrap();
        assert_eq!(0x00_C2_00_0C, r);
        let bytes = r.to_be_bytes();
        println!("{:02x?}", bytes);
    }

    #[test]
    fn test_transport_type() {
        assert_eq!(
            TransportType::ModbusTcpClient,
            TransportType::from("ModbusTcpClient")
        );
        assert_eq!(
            TransportType::ModbusTcpServer,
            TransportType::from("ModbusTcpServer")
        );
        assert_eq!(
            TransportType::Iec104Client,
            TransportType::from("Iec104Client")
        );
        assert_eq!(
            TransportType::Iec104Server,
            TransportType::from("Iec104Server")
        );
        assert_eq!(TransportType::Unknown, TransportType::from("xx"));
        assert_eq!(1, TransportType::ModbusTcpClient as u16);
        assert_eq!(2, TransportType::ModbusTcpServer as u16);
        assert_eq!(11, TransportType::Iec104Client as u16);
        assert_eq!(12, TransportType::Iec104Server as u16);
        assert_eq!(100, TransportType::Unknown as u16);
    }

    #[test]
    #[allow(unused_variables)]
    fn serde_test() {
        // let m = init_discrete_point(100001,0);
        // let byte = serde_json::to_vec(&vec![m]).unwrap();
        // let m_d = serde_json::from_slice::<Vec<Measurement>>(&byte).unwrap();

        let num_f64 = 1.;
        let byte = serde_json::to_vec(&num_f64).unwrap();
        let num_f64_d = serde_json::from_slice::<f64>(&byte).unwrap();

        // let num_f64 = f64::NAN;
        // let byte = serde_json::to_vec(&num_f64).unwrap();
        // let num_f64_d = serde_json::from_slice::<f64>(&byte).unwrap();

        let num_f64 = f64::INFINITY;
        let byte = serde_cbor::to_vec(&num_f64).unwrap();
        let num_f64_d = serde_cbor::from_slice::<f64>(&byte).unwrap();
    }

    #[test]
    fn test_parse_exp() {
        let a = "0.000000e0".to_string();
        let b = a.parse::<f64>().unwrap();
        println!("{:?}", b);
    }
}
