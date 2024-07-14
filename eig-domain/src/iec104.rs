use std::{collections::HashMap, convert::TryInto};
use std::collections::HashSet;

use csv::StringRecord;
use serde::{Deserialize, Serialize};
use crate::excel::{excel_bytes_to_csv_bytes, transfer_to_utf8};

use super::*;


/**
 * @api {Iec104ClientTp} /Iec104ClientTp Iec104ClientTp
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} id 通道id
 * @apiSuccess {String} name 通道名称
 * @apiSuccess {tuple} tcp_server 服务端的ip和port，tuple格式为(ip:String, port:u16)
 * @apiSuccess {Iec104Connection} connection connection
 */
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Iec104ClientTp {
    /// 通道id
    pub id: u64,
    /// 通道名称
    pub name: String,
    /// 服务端的ip和port
    pub tcp_server: (String, u16),
    /// 遥信点号的数据类型
    #[serde(default)]
    pub yx_data_type: u8,
    /// 遥测点号的数据类型
    #[serde(default)]
    pub yc_data_type: u8,
    /// 连接
    pub connection: Iec104Connection,
}

/**
 * @api {Iec104ServerTp} /Iec104ServerTp Iec104ServerTp
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} id 通道id
 * @apiSuccess {String} name 通道名称
 * @apiSuccess {u16} tcp_server_port 服务的port
 * @apiSuccess {u8} yx_data_type 遥信点号的数据类型
 * @apiSuccess {u8} yc_data_type 遥测点号的数据类型
 * @apiSuccess {tuple[]} connections 连接信息，数组，tuple格式为(String, Iec104Connection)
 */
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Iec104ServerTp {
    /// 通道id
    pub id: u64,
    /// 通道名称
    pub name: String,
    /// 服务的port
    pub tcp_server_port: u16,
    /// 遥信点号的数据类型
    pub yx_data_type: u8,
    /// 遥测点号的数据类型
    pub yc_data_type: u8,
    /// 连接
    pub connections: Vec<(String, Iec104Connection)>,
}

/**
 * @api {Iec104测点信息} /Iec104Point Iec104Point
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u32} ioa 协议地址
 * @apiSuccess {u64} point_id 对应的测点Id
 * @apiSuccess {bool} is_yx 是否是遥信量
 * @apiSuccess {u32} [control_ioa] 控制点地址，若进行配置控制点地址，则说明该点可写
 */
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Iec104Point {
    /// 协议地址
    pub ioa: u32,
    /// 对应的测点Id
    pub point_id: u64,
    /// 是否是遥信量
    pub is_yx: bool,
    /// 控制点地址，若进行配置控制点地址，则说明该点可写
    pub control_ioa: Option<u32>,
}

/**
 * @api {Iec104通道连接信息} /Iec104Connection Iec104Connection
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {String} name 连接名称
 * @apiSuccess {u64} point_id 通道状态对应的测点号
 * @apiSuccess {Iec104Point[]} data_configure register settings
 * @apiSuccess {Map} point_id_to_ioa HashMap<point_id:u64, information_object_addressa:u32>
 * @apiSuccess {Map} ioa_to_pos HashMap<Point地址:u32, data_configure中的位置:u16>
 * @apiSuccess {bool} is_control_with_time 控制方向是否带时标
 * @apiSuccess {u16} common_address 公共地址
 * @apiSuccess {u8} cot_field_length 传输原因字节个数
 * @apiSuccess {u8} common_address_field_length 公共地址字节个数
 * @apiSuccess {u8} ioa_field_length 信息体地址字节个数
 * @apiSuccess {u64} max_time_no_ack_received t1
 * @apiSuccess {u64} max_time_no_ack_sent t2
 * @apiSuccess {u64} max_idle_time t3
 * @apiSuccess {u8} max_unconfirmed_apdus_sent k，发送方发送k条连续的未被确认的I格式报文，停止发送
 * @apiSuccess {u8} max_unconfirmed_apdus_received w，接收方收到w个I格式报文后发送确认
 * @apiSuccess {u64} [call_time] 总召时间间隔
 * @apiSuccess {u64} [call_counter_time] 点度量总召时间间隔
 * @apiSuccess {bool} is_client 是否为客户端
 */
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Iec104Connection {
    /// 连接名称
    pub name: String,
    /// 通道状态对应的测点号
    pub point_id: u64,
    /// register settings
    pub data_configure: Vec<Iec104Point>,
    /// key is point id, value is information object address
    pub point_id_to_ioa: HashMap<u64, u32>,
    /// key:Point地址,value:data_configure中的位置
    pub ioa_to_pos: HashMap<u32, u16>,
    /// 控制方向是否带时标
    pub is_control_with_time: bool,
    /// 遥控遥调是否为直控，默认为false
    #[serde(default)]
    pub direct_yk: bool,
    #[serde(default)]
    pub direct_yt: bool,
    /// 源发地址
    pub originator_address: u8,
    /// 公共地址
    pub common_address: u16,
    /// 传输原因字节个数
    pub cot_field_length: u8,
    /// 公共地址字节个数
    pub common_address_field_length: u8,
    /// 信息体地址字节个数
    pub ioa_field_length: u8,
    /// t1
    pub max_time_no_ack_received: u64,
    /// t2
    pub max_time_no_ack_sent: u64,
    /// t3
    pub max_idle_time: u64,
    /// k，发送方发送k条连续的未被确认的I格式报文，停止发送
    pub max_unconfirmed_apdus_sent: u8,
    /// w，接收方收到w个I格式报文后发送确认
    pub max_unconfirmed_apdus_received: u8,
    /// 总召时间间隔
    pub call_time: Option<u64>,
    /// 点度量总召时间间隔
    pub call_counter_time: Option<u64>,
    /// 是否为客户端
    pub is_client: bool,
}

impl Default for Iec104Connection {
    fn default() -> Self {
        Iec104Connection {
            name: "new".to_string(),
            point_id: 0,
            data_configure: vec![],
            point_id_to_ioa: Default::default(),
            ioa_to_pos: Default::default(),
            is_control_with_time: false,
            direct_yk: false,
            direct_yt: false,
            originator_address: 0,
            common_address: 1,
            cot_field_length: 2,
            common_address_field_length: 2,
            ioa_field_length: 3,
            max_time_no_ack_received: 15000,
            max_time_no_ack_sent: 10000,
            max_idle_time: 20000,
            max_unconfirmed_apdus_sent: 12,
            max_unconfirmed_apdus_received: 8,
            call_time: None,
            call_counter_time: None,
            is_client: false,
        }
    }
}

impl Iec104ClientTp {
    pub fn from_file(path: &str) -> Result<Self, (usize, usize)> {
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
        Self::from_csv_bytes(csv_bytes.as_slice())
    }

    pub fn from_csv(path: &str) -> Result<Iec104ClientTp, (usize, usize)> {
        let content = std::fs::read(path).map_err(|_| (0, 0))?;
        // if env::IS_ENCRYPT {
        //     let content = decrypt_vec(content.as_slice());
        //     Iec104ClientTp::from_csv_bytes(content.as_slice())
        // } else {
        //     Iec104ClientTp::from_csv_bytes(content.as_slice())
        // }
        Iec104ClientTp::from_csv_bytes(content.as_slice())
    }

    pub fn from_csv_bytes(content: &[u8]) -> Result<Iec104ClientTp, (usize, usize)> {
        let content_new = transfer_to_utf8(content.to_vec()).map_err(|_| (0, 0))?;
        let content = content_new.as_slice();
        let tp = Iec104ClientTp::from_csv_records(content, 0)?;
        let rc = (2usize, 1);
        tp.tcp_server
            .0
            .parse::<std::net::Ipv4Addr>()
            .map_err(|_| rc)?;
        Ok(tp)
    }

    fn from_csv_records(
        content: &[u8],
        offset: usize,
    ) -> Result<Iec104ClientTp, (usize, usize)> {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(content);
        let mut records = rdr.records();
        // 1st line
        let rc = (0usize, 1 + offset);
        let name = csv_string(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        // 2nd line
        let rc = (1usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let point_num = csv_usize(&record, rc.1).ok_or(rc)?;
        if point_num as u16 > u16::MAX {
            return Err(rc)
        }
        // 3rd line
        let rc = (2usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let tcp_server_ip = csv_string(&record, rc.1).ok_or(rc)?;
        // 4th line
        let rc = (3usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let tcp_server_port = csv_u16(&record, rc.1).ok_or(rc)?;
        // 5th line
        let rc = (4usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let s = csv_str(&record, rc.1).ok_or(rc)?;
        let point_id: u64 = if s.is_empty() {
            0
        } else {
            s.parse().map_err(|_| rc)?
        };
        // 6th line
        let rc = (5usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let originator_address = csv_u8(&record, rc.1).ok_or(rc)?;
        // 7th line
        let rc = (6usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let common_address = csv_u16(&record, rc.1).ok_or(rc)?;
        // 8th line
        let rc = (7usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let common_address_field_length = csv_u8(&record, rc.1).ok_or(rc)?;
        // 9th line
        let rc = (8usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let cot_field_length = csv_u8(&record, rc.1).ok_or(rc)?;
        // 10th line
        let rc = (9usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let ioa_field_length = csv_u8(&record, rc.1).ok_or(rc)?;
        // 11th line
        let rc = (10usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let max_time_no_ack_received = csv_u64(&record, rc.1).ok_or(rc)?;
        // 12th line
        let rc = (11usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let max_time_no_ack_sent = csv_u64(&record, rc.1).ok_or(rc)?;
        // 13th line
        let rc = (12usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let max_idle_time = csv_u64(&record, rc.1).ok_or(rc)?;
        // 14th line
        let rc = (13usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let max_unconfirmed_apdus_sent = csv_u8(&record, rc.1).ok_or(rc)?;
        // 15th line
        let rc = (14usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let max_unconfirmed_apdus_received = csv_u8(&record, rc.1).ok_or(rc)?;
        // 16th line
        let rc = (15usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let s = csv_string(&record, rc.1).ok_or(rc)?.trim().to_uppercase();
        let cs: Vec<&str> = s.split(';').collect();
        let is_control_with_time = if let Some(c) = cs.get(0) {
            c == &"TRUE"
        } else {
            false
        };
        let direct_yk = if let Some(c) = cs.get(1) {
            c == &"TRUE"
        } else {
            false
        };
        let direct_yt = if let Some(c) = cs.get(2) {
            c == &"TRUE"
        } else {
            false
        };
        // 17th line
        let rc = (16usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let s = csv_str(&record, rc.1).ok_or(rc)?;
        let call_time = if s.is_empty() {
            None
        } else {
            Some(s.parse::<u64>().map_err(|_| rc)?)
        };
        // 18th line
        let rc = (17usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let s = csv_string(&record, rc.1).ok_or(rc)?.trim().to_uppercase();
        let is_client = s.as_str() == "TRUE";
        // 19th line
        let rc = (18usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let s = csv_str(&record, rc.1).ok_or(rc)?;
        let call_counter_time = if s.is_empty() {
            None
        } else {
            Some(s.parse::<u64>().map_err(|_| rc)?)
        };
        // 20th line
        let rc = (19usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let s = csv_str(&record, rc.1).ok_or(rc)?;
        let yx_data_type = if s.is_empty() {
            0
        } else {
            s.parse::<u8>().map_err(|_| rc)?
        };
        // 21st line
        let rc = (20usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let s = csv_str(&record, rc.1).ok_or(rc)?;
        let yc_data_type = if s.is_empty() {
            0
        } else {
            s.parse::<u8>().map_err(|_| rc)?
        };
        // 22th ...
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(content);
        let mut records = rdr.records();
        let rc = (0, 3 + offset);
        records.next().ok_or(rc)?.map_err(|_| rc)?;
        let mut data_configure: Vec<Iec104Point> = Vec::with_capacity(point_num);
        for row in 1..=point_num {
            let rc = (row, 3 + offset);
            let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
            data_configure.push(Iec104Point::parse_register_data(&record, rc.0, rc.1)?);
        }

        let mut conn = Iec104Connection {
            name: name.clone(),
                point_id,
                data_configure,
                is_control_with_time,
                direct_yk,
                direct_yt,
                originator_address,
                common_address,
                cot_field_length,
                common_address_field_length,
                ioa_field_length,
                max_time_no_ack_received,
                max_time_no_ack_sent,
                max_idle_time,
                max_unconfirmed_apdus_sent,
                max_unconfirmed_apdus_received,
                call_time,
                call_counter_time,
                is_client,
                ..Default::default()
        };
        conn.create_data_config().map_err(|(r, c, _)|(r, c + offset))?;
        
        Ok(Iec104ClientTp {
            id: 0,
            name,
            tcp_server: (tcp_server_ip, tcp_server_port),
            yx_data_type,
            yc_data_type,
            connection: conn,
        })
    }

    pub fn export_csv(&self, text_map: &HashMap<String, String>) -> String {
        let title = vec![
            text_map.get("point_number").unwrap_or(&"Point Count".to_string()).clone(),
            text_map.get("server_ip").unwrap_or(&"Server IP".to_string()).clone(),
            text_map.get("server_port").unwrap_or(&"Server Port".to_string()).clone(),
            text_map.get("tp_point_id").unwrap_or(&"Point ID".to_string()).clone(),
            text_map.get("originator_addr").unwrap_or(&"Originator Address".to_string()).clone(),
            text_map.get("common_addr").unwrap_or(&"Common Address".to_string()).clone(),
            text_map.get("common_addr_filed_len").unwrap_or(&"Common Address Field Length".to_string()).clone(),
            text_map.get("cot_field_len").unwrap_or(&"Cot Field Length".to_string()).clone(),
            text_map.get("ioa_field_len").unwrap_or(&"Ioa Field Length".to_string()).clone(),
            text_map.get("t1_ms").unwrap_or(&"T1 (ms)".to_string()).clone(),
            text_map.get("t2_ms").unwrap_or(&"T2 (ms)".to_string()).clone(),
            text_map.get("t3_ms").unwrap_or(&"T3 (ms)".to_string()).clone(),
            text_map.get("iec104_k").unwrap_or(&"k".to_string()).clone(),
            text_map.get("iec104_w").unwrap_or(&"w".to_string()).clone(),
            text_map.get("is_control_with_time").unwrap_or(&"Is Control With Time;Direct yk;Direct yt".to_string()).clone(),
            text_map.get("call_time_ms").unwrap_or(&"Call Time (ms)".to_string()).clone(),
            text_map.get("is_client_true").unwrap_or(&"is Client(TRUE)".to_string()).clone(),
            text_map.get("call_counter_time").unwrap_or(&"Call Counter Time (ms)".to_string()).clone(),
            text_map.get("telesignaling_type_default").unwrap_or(&"Default Telesignaling Type".to_string()).clone(),
            text_map.get("telemetering_type_default").unwrap_or(&"Default Telemetering Type".to_string()).clone(),
        ];

        let c = self.connection.clone();
        let p = self.connection.data_configure.clone();
        let mut content = vec![
            format!("{}", c.data_configure.len()),
            format!("{}", self.tcp_server.0),
            format!("{}", self.tcp_server.1),
            format!("{}", c.point_id),
            format!("{}", c.originator_address),
            format!("{}", c.common_address),
            format!("{}", c.common_address_field_length),
            format!("{}", c.cot_field_length),
            format!("{}", c.ioa_field_length),
            format!("{}", c.max_time_no_ack_received),
            format!("{}", c.max_time_no_ack_sent),
            format!("{}", c.max_idle_time),
            format!("{}", c.max_unconfirmed_apdus_sent),
            format!("{}", c.max_unconfirmed_apdus_received),
            format!("{};{};{}", c.is_control_with_time, c.direct_yk, c.direct_yt).to_uppercase(),
            "".to_string(),
            //format!("{}", c.is_client).to_uppercase(),
            "TRUE".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
        ];
        if let Some(call_time) = c.call_time {
            content[15] = call_time.to_string()
        };
        if let Some(call_counter_time) = c.call_counter_time {
            content[17] = call_counter_time.to_string()
        };
        if self.yx_data_type != 0 {
            content[18] = self.yx_data_type.to_string();
        }
        if self.yc_data_type != 0 {
            content[19] = self.yc_data_type.to_string();
        }

        let mut result = format!(
            "{},{},{},{},{},{},{}\n",
            text_map.get("tp_name").unwrap_or(&"Transport Name".to_string()),
            get_csv_str(&self.name),
            text_map.get("index").unwrap_or(&"Index".to_string()),
            text_map.get("addr_monitor_direction").unwrap_or(&"Address of Monitoring Direction Information".to_string()),
            text_map.get("point_id_short").unwrap_or(&"Status Point".to_string()),
            text_map.get("is_telesignaling").unwrap_or(&"Is Telesignaling".to_string()),
            text_map.get("addr_control_direction").unwrap_or(&"Address of Control Direction Information".to_string()),
        );
        for i in 0_usize..20_usize {
            if p.len() > i {
                let yx_status = format!("{}", p[i].is_yx).to_uppercase();
                result += &format!(
                    "{},{},{},{},{},{}",
                    title[i],
                    content[i],
                    i + 1,
                    p[i].ioa,
                    p[i].point_id,
                    yx_status
                );
                if let Some(addr) = p[i].control_ioa {
                    result += &format!(",{}\n", addr);
                } else {
                    result += ", \n";
                }
            } else {
                result += &format!("{},{},,,,,\n", title[i], content[i]);
            }
        }
        if p.len() > 20 {
            let mut index = 20_usize;
            while index < p.len() {
                let yx_status = format!("{}", p[index].is_yx).to_uppercase();
                result += &format!(
                    ",,{},{},{},{}",
                    index + 1,
                    p[index].ioa,
                    p[index].point_id,
                    yx_status
                );
                if let Some(addr) = p[index].control_ioa {
                    result += &format!(",{}\n", addr);
                } else {
                    result += ", \n";
                }
                index += 1;
            }
        }

        result
    }

    pub fn get_point_ids(&self) -> Vec<u64> {
        let size = self.connection.data_configure.len() + 1;
        let mut r: Vec<u64> = Vec::with_capacity(size);
        for rd in &self.connection.data_configure {
            r.push(rd.point_id)
        }
        if self.connection.point_id != UNKNOWN_POINT_ID {
            r.push(self.connection.point_id);
        }
        r
    }
}

impl Iec104ServerTp {
    pub fn from_file(path: &str) -> Result<Self, (usize, usize)> {
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
        Self::from_csv_bytes(csv_bytes.as_slice())
    }

    pub fn from_csv(path: &str) -> Result<Iec104ServerTp, (usize, usize)> {
        let content = std::fs::read(path).map_err(|_| (0, 0))?;
        // let content = if env::IS_ENCRYPT {
        //     let content = decrypt_vec(content.as_slice());
        //     content
        // } else {
        //     content
        // };
        Iec104ServerTp::from_csv_bytes(content.as_slice())
    }

    pub fn from_csv_bytes(content: &[u8]) -> Result<Iec104ServerTp, (usize, usize)> {
        let content_new = transfer_to_utf8(content.to_vec()).map_err(|_| (0, 0))?;
        let content = content_new.as_slice();
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(content);
        // 1. 解析服务端的配置
        let mut records = rdr.records();
        let rc = (0usize, 1);
        let name = csv_string(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        let rc = (1usize, 1);
        let mut conn_num =
            csv_usize(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        let rc = (2usize, 1);
        let tcp_server_port =
            csv_u16(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        let rc = (3usize, 1);
        let yx_data_type = csv_u8(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        let rc = (4usize, 1);
        let yc_data_type = csv_u8(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        let rc = (5usize, 1);
        let s = csv_string(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        let is_client = s.as_str() == "TRUE";
        if is_client {
            conn_num = 1; //如果是客户端从站，只保留一个通道连接
        }

        // 2. 解析连接的信息
        let mut connections: Vec<(String, Iec104Connection)> = Vec::with_capacity(conn_num);
        let offset = 8;
        for i in 0..conn_num {
            let mut tp = Iec104ClientTp::from_csv_records(content, i * offset + 3)?;
            tp.connection.is_client = is_client;  // 确保104d的各个连接的client/server属性保持一致，且完全遵从通道配置
            let (client_ip, client_port) = tp.tcp_server;
            if client_ip != "+" {
                let rc = (2usize, i * offset + 4);
                client_ip.parse::<std::net::Ipv4Addr>().map_err(|_| rc)?;
            }
            // 检查具有相同ip的client是否配置一样
            for (key, conn) in &connections {
                // 这里连接名字和通道名称是一样的
                if *key == format!("{}/{}/{}", client_ip, client_port, conn.name) {
                    // 地址配置必须一样
                    if *conn.data_configure != tp.connection.data_configure {
                        return Err((0, i * offset + 3));
                    }
                }
            }
            let port_str = client_port.to_string();
            let port = if client_port as u32 == UNKNOWN_TCP_PORT {
                "+"
            } else {
                port_str.as_str()
            };

            // 利用point_id简化多个client配置，测点号1-100预留，不允许设置
            if tp.connection.point_id > 1
                && tp.connection.point_id <= DEFAULT_TCP_CLIENT_LIMIT as u64 {
                let count = tp.connection.point_id;
                for i in 1..count {
                    let key = format!("{}/{}/{}@{}", client_ip, port, tp.name, i);
                    let mut connection = tp.connection.clone();
                    connection.point_id = UNKNOWN_POINT_ID; // 多个通道共用一个配置，状态点号无需设置
                    connections.push((key, connection));
                }
                let key = format!("{}/{}/{}@{}", client_ip, port, tp.name, count);
                tp.connection.point_id = UNKNOWN_POINT_ID;
                connections.push((key, tp.connection))
            } else {
                let key = format!("{}/{}/{}", client_ip, port, tp.name);
                connections.push((key, tp.connection))
            }
        }

        Ok(Iec104ServerTp {
            id: 0,
            name,
            tcp_server_port,
            yx_data_type,
            yc_data_type,
            connections,
        })
    }

    pub fn export_csv(&self, text_map: &HashMap<String, String>) -> String {
        let mut len_conn = 0;
        let mut unkown_ip_map: HashMap<u32, usize> = HashMap::new();
        for (s, _) in &self.connections {
            if s.starts_with("+/") {
                let info: Vec<&str> = s.split('/').collect();
                if info.len() != 3 {
                    continue;
                }
                let port = if info[1] == "+" {
                    UNKNOWN_TCP_PORT
                } else {
                    info[1].parse::<u32>().unwrap_or(UNKNOWN_TCP_PORT)
                };
                if let Some(multi_count) = unkown_ip_map.get_mut(&port) {
                    *multi_count += 1;
                    continue;
                } else {
                    unkown_ip_map.insert(port, 1);
                    len_conn += 1;
                }
            } else {
                len_conn += 1;
            }
        }
        // 第一排
        let mut result = format!("{},{},,",
                                 text_map.get("tp_name").unwrap_or(&"Transport Name".to_string()),
                                 get_csv_str(&self.name));
        let mut i = 0;
        let mut multi_found = false;
        for (s, conn) in &self.connections {
            result += &format!(
                "{},{},{},{},{},{},{}",
                text_map.get("conn_name").unwrap_or(&"Connection Name".to_string()),
                get_csv_str(&conn.name),
                text_map.get("index").unwrap_or(&"Index".to_string()),
                text_map.get("addr_monitor_direction").unwrap_or(&"Address of Monitoring Direction Information".to_string()),
                text_map.get("status_point").unwrap_or(&"Status Point".to_string()),
                text_map.get("is_telesignaling").unwrap_or(&"Is Telesignaling".to_string()),
                text_map.get("addr_control_direction").unwrap_or(&"Address of Control Direction Information".to_string()),
            );
            if i != len_conn - 1 {
                result += ",,";
            } else {
                break;
            }
            if s.starts_with("+/") {
                if multi_found {
                    continue;
                } else {
                    multi_found = true;
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
        result += "\n";

        // 第二至十九排
        let title_conn = vec![
            text_map.get("point_number").unwrap_or(&"Point Count".to_string()).clone(),
            text_map.get("client_ip").unwrap_or(&"Client IP".to_string()).clone(),
            text_map.get("client_port").unwrap_or(&"Client Port".to_string()).clone(),
            text_map.get("tp_point_id").unwrap_or(&"Point ID".to_string()).clone(),
            text_map.get("originator_addr").unwrap_or(&"Originator Address".to_string()).clone(),
            text_map.get("common_addr").unwrap_or(&"Common Address".to_string()).clone(),
            text_map.get("common_addr_filed_len").unwrap_or(&"Common Address Field Length".to_string()).clone(),
            text_map.get("cot_field_len").unwrap_or(&"Cot Field Length".to_string()).clone(),
            text_map.get("ioa_field_len").unwrap_or(&"Ioa Field Length".to_string()).clone(),
            text_map.get("t1_ms").unwrap_or(&"T1 (ms)".to_string()).clone(),
            text_map.get("t2_ms").unwrap_or(&"T2 (ms)".to_string()).clone(),
            text_map.get("t3_ms").unwrap_or(&"T3 (ms)".to_string()).clone(),
            text_map.get("iec104_k").unwrap_or(&"k".to_string()).clone(),
            text_map.get("iec104_w").unwrap_or(&"w".to_string()).clone(),
            text_map.get("is_control_with_time").unwrap_or(&"Is Control With Time;Direct yk;Direct yt".to_string()).clone(),
            text_map.get("call_time_ms").unwrap_or(&"Call Time (ms)".to_string()).clone(),
            text_map.get("is_client").unwrap_or(&"is Client".to_string()).clone(),
            text_map.get("call_counter_time").unwrap_or(&"Call Counter Time (ms)".to_string()).clone(),
        ];
        let title_tp = vec![
            format!(
                "{},{}",
                text_map.get("conn_num").unwrap_or(&"Connection Count".to_string()),
                len_conn
            ),
            format!(
                "{},{}",
                text_map.get("server_port").unwrap_or(&"Server Port".to_string()),
                self.tcp_server_port
            ),
            format!(
                "{},{}",
                text_map.get("telesignaling_type").unwrap_or(&"Telesignaling Type".to_string()),
                self.yx_data_type
            ),
            format!(
                "{},{}",
                text_map.get("telemetering_type").unwrap_or(&"Telemetering Type".to_string()),
                self.yc_data_type
            ),
            format!("{},FALSE", text_map.get("is_client").unwrap_or(&"Is Client".to_string())),
            text_map.get("type_id_name").unwrap_or(&"Type ID,Type Name".to_string()).clone(),
            IEC104D_INFO[0].to_string(),
            IEC104D_INFO[1].to_string(),
            IEC104D_INFO[2].to_string(),
            IEC104D_INFO[3].to_string(),
            IEC104D_INFO[4].to_string(),
            IEC104D_INFO[5].to_string(),
            IEC104D_INFO[6].to_string(),
            IEC104D_INFO[7].to_string(),
            IEC104D_INFO[8].to_string(),
            IEC104D_INFO[9].to_string(),
            IEC104D_INFO[10].to_string(),
            IEC104D_INFO[11].to_string(),
        ];

        for cnt in 0..18 {
            result += &title_tp[cnt];
            let mut i = 0;
            let mut multi_found = HashSet::with_capacity(unkown_ip_map.len());
            for conn in &self.connections {
                if conn.0.starts_with("+/") {
                    let info: Vec<&str> = conn.0.split('/').collect();
                    if info.len() != 3 {
                        continue;
                    }
                    let port = if info[1] == "+" {
                        UNKNOWN_TCP_PORT
                    } else {
                        info[1].parse::<u32>().unwrap_or(UNKNOWN_TCP_PORT)
                    };
                    if multi_found.contains(&port) {
                        continue;
                    } else {
                        multi_found.insert(port);
                        i += 1;
                    }
                } else {
                    i += 1;
                }
                if conn.1.data_configure.len() > cnt {
                    let p = &conn.1.data_configure[cnt];
                    let content_conn = if cnt == 3 && conn.0.starts_with("+/") {
                        let info: Vec<&str> = conn.0.split('/').collect();
                        if info.len() != 3 {
                            continue;
                        }
                        let port = if info[1] == "+" {
                            UNKNOWN_TCP_PORT
                        } else {
                            info[1].parse::<u32>().unwrap_or(UNKNOWN_TCP_PORT)
                        };
                        unkown_ip_map.get(&port).unwrap_or(&0).to_string()
                    } else {
                        Self::get_iec104d_conn_csv(conn, cnt)
                    };
                    result += &format!(
                        ",,{},{},{},{},{},{}",
                        title_conn[cnt],
                        content_conn,
                        cnt + 1,
                        p.ioa,
                        p.point_id,
                        p.is_yx.to_string().to_uppercase()
                    );
                    if let Some(addr) = p.control_ioa {
                        result += &format!(",{}", addr);
                    } else {
                        result += ",";
                    }
                } else {
                    let content_conn = Self::get_iec104d_conn_csv(conn, cnt);
                    result += &format!(",,{},{},,,,,", title_conn[cnt], content_conn);
                }
                if i == len_conn {
                    break;
                }
            }
            result += "\n";
        }

        // 剩余的
        let mut max_data_len = if self.connections.is_empty() {
            0
        } else {
            self.connections[0].1.data_configure.len()
        };
        for c in &self.connections {
            if c.1.data_configure.len() > max_data_len {
                max_data_len = c.1.data_configure.len();
            }
        }
        for row in 18..max_data_len {
            if row < 27 {
                result += &format!("{},", IEC104D_INFO[row - 6]);
            } else {
                // 如果Data Type输出完了但测点寄存器还有
                result += ",,";
            }
            let mut i = 0;
            let mut multi_found = HashSet::with_capacity(unkown_ip_map.len());
            for (s, conn) in &self.connections {
                if s.starts_with("+/") {
                    let info: Vec<&str> = s.split('/').collect();
                    if info.len() != 3 {
                        continue;
                    }
                    let port = if info[1] == "+" {
                        UNKNOWN_TCP_PORT
                    } else {
                        info[1].parse::<u32>().unwrap_or(UNKNOWN_TCP_PORT)
                    };
                    if multi_found.contains(&port) {
                        continue;
                    } else {
                        multi_found.insert(port);
                        i += 1;
                    }
                } else {
                    i += 1;
                }
                if conn.data_configure.len() > row {
                    let p = &conn.data_configure[row];
                    result += &format!(
                        ",,,{},{},{},{}",
                        row + 1,
                        p.ioa,
                        p.point_id,
                        p.is_yx.to_string().to_uppercase()
                    );
                    if let Some(addr) = p.control_ioa {
                        result += &format!(",{}", addr);
                    } else {
                        result += ",";
                    }
                } else {
                    result += ",,,,,";
                }
                if i != len_conn {
                    result += ",";
                } else {
                    break;
                }
            }
            result += "\n";
        }
        let row = usize::max(max_data_len, 19);
        if (8..28).contains(&row) {
            //如果测点寄存器输出完了但Data Type还有
            // 8个逗号重复n遍
            let comma  = ",,,,,,,,".repeat(len_conn);
            for row in row..28 {
                result += &format!("{}{}\n", IEC104D_INFO[row - 8], comma);
            }
        }
        result
    }
    fn get_iec104d_conn_csv(conn: &(String, Iec104Connection), index: usize) -> String {
        return match index {
            0 => conn.1.data_configure.len().to_string(),
            1 => {
                let info: Vec<&str> = conn.0.split('/').collect();
                if info.len() != 3 {
                    return "".to_string();
                }
                info[0].to_string()
            }
            2 => {
                let info: Vec<&str> = conn.0.split('/').collect();
                if info.len() != 3 {
                    return "".to_string();
                }
                if info[1] == "+" {
                    UNKNOWN_TCP_PORT.to_string()
                } else {
                    info[1].to_string()
                }
            }
            3 => conn.1.point_id.to_string(),
            4 => conn.1.originator_address.to_string(),
            5 => conn.1.common_address.to_string(),
            6 => conn.1.common_address_field_length.to_string(),
            7 => conn.1.cot_field_length.to_string(),
            8 => conn.1.ioa_field_length.to_string(),
            9 => conn.1.max_time_no_ack_received.to_string(),
            10 => conn.1.max_time_no_ack_sent.to_string(),
            11 => conn.1.max_idle_time.to_string(),
            12 => conn.1.max_unconfirmed_apdus_sent.to_string(),
            13 => conn.1.max_unconfirmed_apdus_received.to_string(),
            14 => format!("{};{};{}", conn.1.is_control_with_time, conn.1.direct_yk, conn.1.direct_yt).to_uppercase(),
            15 => {
                if let Some(t) = conn.1.call_time {
                    t.to_string()
                } else {
                    "".to_string()
                }
            }
            16 => conn.1.is_client.to_string().to_uppercase(),
            17 => {
                if let Some(t) = conn.1.call_counter_time {
                    t.to_string()
                } else {
                    "".to_string()
                }
            }
            _ => "unknown".to_string(),
        };
    }

    pub fn get_point_ids(&self) -> Vec<u64> {
        let mut size = 0;
        for (_, conn) in &self.connections {
            size += conn.data_configure.len()
        }
        size += self.connections.len();
        let mut r = HashSet::with_capacity(size);
        for (_, conn) in &self.connections {
            if conn.point_id != UNKNOWN_POINT_ID {
                r.insert(conn.point_id);
            }
            for rd in &conn.data_configure {
                r.insert(rd.point_id);
            }
        }
        r.into_iter().collect()
    }
}

impl Iec104Connection {
    pub fn create_data_config(&mut self) -> Result<(),(usize, usize, String)> {
        let size = self.data_configure.len();
        let mut point_id_to_ioa: HashMap<u64, u32> = HashMap::with_capacity(size);
        // key:寄存器地址,value:setting中vec<RegisterData>的位置
        let mut ioa_to_pos: HashMap<u32, u16> = HashMap::with_capacity(size);
        let mut control_ioa_to_measure_ioa: HashMap<u32, u32> = HashMap::with_capacity(size);
        for (index, rd) in self.data_configure.iter().enumerate() {
            // 测点号重复
            if point_id_to_ioa.contains_key(&rd.point_id) {
                let tip = format!("Invalid register point (id :{}):\nThe point ID is already existed", rd.point_id);
                return Err((index + 1, 4, tip)); // 测点号的位置
            }
            point_id_to_ioa.insert(rd.point_id, rd.ioa);
            // 起始地址重复
            if ioa_to_pos.contains_key(&rd.ioa) {
                let tip = format!("Invalid register point (ioa :{}):\nThe ioa is already existed", rd.ioa);
                return Err((index + 1, 3, tip)); // 监视地址的位置
            }
            ioa_to_pos.insert(rd.ioa, index.try_into().unwrap());
            // 有控制点的情况下：判断控制点地址是否重复，控制点地址与测量点地址是否重复
            if let Some(control_ioa) = &rd.control_ioa {
                if control_ioa_to_measure_ioa.contains_key(control_ioa) {
                    let tip = format!("Invalid register point (control ioa :{}):\nThe control ioa is already existed", control_ioa);
                    return Err((index + 1, 6, tip)); // 控制地址的位置
                }
                if ioa_to_pos.contains_key(control_ioa) {
                    let tip = format!("Invalid register point (control ioa :{}):\nThe control ioa and another ioa are repeated", control_ioa);
                    return Err((index + 1, 6, tip)); // 监视地址的位置
                }
                control_ioa_to_measure_ioa.insert(*control_ioa, rd.ioa);
            }
        }
        self.point_id_to_ioa = point_id_to_ioa;
        self.ioa_to_pos = ioa_to_pos;
        Ok(())
    }
}

impl Iec104Point {
    fn parse_register_data(
        record: &StringRecord,
        row: usize,
        first_col: usize,
    ) -> Result<Self, (usize, usize)> {
        //let start: usize = 3 + offset;
        let rc = (row, first_col);
        let ioa = csv_u32(record, rc.1).ok_or(rc)?;
        // 对应的测点Id
        let rc = (row, first_col + 1);
        let point_id = csv_u64(record, rc.1).ok_or(rc)?;
        let rc = (row, first_col + 2);
        let s = csv_string(record, rc.1).ok_or(rc)?.trim().to_uppercase();
        let is_yx = s.as_str() == "TRUE";
        let rc = (row, first_col + 3);
        let s = csv_str(record, rc.1).ok_or(rc)?;
        let control_ioa = if s.is_empty() {
            None
        } else {
            Some(s.parse::<u32>().map_err(|_| rc)?)
        };
        Ok(Iec104Point {
            ioa,
            point_id,
            is_yx,
            control_ioa,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::iec104::Iec104ClientTp;
    use crate::Iec104ServerTp;

    #[test]
    fn test_iec_client_from_csv() {
        let tp = Iec104ClientTp::from_csv("tests/iec104c-test1.csv").unwrap();
        assert_eq!(tp.name, "iec104测试通道1");
        assert_eq!(tp.tcp_server.0, "127.0.0.1");
        assert_eq!(tp.tcp_server.1, 2404);
        assert_eq!(tp.yx_data_type, 1);
        assert_eq!(tp.yc_data_type, 13);
        assert_eq!(tp.connection.originator_address, 0);
        assert_eq!(tp.connection.common_address, 1);
        assert_eq!(tp.connection.point_id, 109001);
        assert_eq!(tp.connection.cot_field_length, 2);
        assert_eq!(tp.connection.common_address_field_length, 2);
        assert_eq!(tp.connection.max_idle_time, 20000);
        assert_eq!(tp.connection.max_time_no_ack_received, 15000);
        assert_eq!(tp.connection.max_time_no_ack_sent, 10000);
        assert_eq!(tp.connection.data_configure.len(), 10);
        assert!(!tp.connection.is_control_with_time);
        assert!(!tp.connection.direct_yk);
        assert!(!tp.connection.direct_yt);
        assert_eq!(tp.connection.data_configure[0].ioa, 1);
        assert_eq!(tp.connection.data_configure[0].point_id, 104001);
        assert!(!tp.connection.data_configure[0].is_yx);
        assert_eq!(tp.connection.data_configure[0].control_ioa, Some(6001));
        assert_eq!(tp.connection.call_time, Some(10000));
        assert_eq!(tp.connection.call_counter_time, Some(10000));
        assert!(tp.connection.is_client);

        let s = tp.export_csv(&HashMap::with_capacity(0));
        let tp2 = Iec104ClientTp::from_csv_bytes(s.as_bytes()).unwrap();
        assert_eq!(tp, tp2);
    }

    #[test]
    fn test_iec_server_from_csv() {
        let tp = Iec104ServerTp::from_csv("tests/iec104d-test1.csv").unwrap();
        assert_eq!(tp.name, "iec104服务通道1");
        assert_eq!(tp.tcp_server_port, 2404);
        assert_eq!(tp.yx_data_type, 1);
        assert_eq!(tp.yc_data_type, 13);
        assert_eq!(tp.connections.len(), 2);
        assert_eq!(tp.connections[0].1.originator_address, 0);
        assert_eq!(tp.connections[0].1.common_address, 1);
        assert_eq!(tp.connections[0].1.point_id, 102404);
        assert_eq!(tp.connections[0].1.cot_field_length, 2);
        assert_eq!(tp.connections[0].1.common_address_field_length, 2);
        assert_eq!(tp.connections[0].1.max_idle_time, 20000);
        assert_eq!(tp.connections[0].1.max_time_no_ack_received, 15000);
        assert_eq!(tp.connections[0].1.max_time_no_ack_sent, 10000);
        assert_eq!(tp.connections[0].1.data_configure.len(), 10);
        assert!(!tp.connections[0].1.is_control_with_time);
        assert!(!tp.connections[0].1.direct_yk);
        assert!(!tp.connections[0].1.direct_yt);
        assert!(!tp.connections[0].1.is_client);
        assert!(!tp.connections[0].1.data_configure[0].is_yx);
        assert!(tp.connections[0].1.data_configure[0].control_ioa.is_some());
        assert_eq!(tp.connections[0].1.call_time, Some(10000));
        assert_eq!(tp.connections[0].1.call_counter_time, Some(10000));
        assert!(tp.connections[1].1.data_configure[0].control_ioa.is_none());

        let s = tp.export_csv(&HashMap::with_capacity(0));
        let tp2 = Iec104ServerTp::from_csv_bytes(s.as_bytes()).unwrap();
        assert_eq!(tp, tp2);
    }
    #[test]
    fn test_export_iec_server_model() {
        let tp = Iec104ServerTp::from_file("tests/iec104d-transport-example.xlsx").unwrap();
        let s = tp.export_csv(&HashMap::with_capacity(0));
        let tp2 = Iec104ServerTp::from_csv_bytes(s.as_bytes()).unwrap();
        assert_eq!(tp, tp2);
    }

    #[test]
    fn test_export_iec_server_model2() {
        let tp = Iec104ServerTp::from_file("tests/iec104d-unknown-ip.xlsx").unwrap();
        let s = tp.export_csv(&HashMap::with_capacity(0));
        let tp2 = Iec104ServerTp::from_csv_bytes(s.as_bytes()).unwrap();
        assert_eq!(tp, tp2);
    }
}
