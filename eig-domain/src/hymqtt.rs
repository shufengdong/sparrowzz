use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::excel::{excel_bytes_to_csv_bytes, transfer_to_utf8};
use crate::{csv_str, csv_string, csv_u16, csv_u32, csv_u64, csv_usize};

pub const SKIP_TYPE_PREFIX: &str = "SKIP_";
/**
 * @api {华云Mqtt通道信息} /HYMqttTransport HYMqttTransport
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u8} id 通道id
 * @apiSuccess {String} name 通道名称
 * @apiSuccess {tuple} mqtt_broker 服务端的ip和port，tuple格式为(ip:String, port:u16)
 * @apiSuccess {u64} point_id 通道状态对应的测点号
 * @apiSuccess {String} read_topic 读测点的主题
 * @apiSuccess {String} write_topic 写测点的主题
 * @apiSuccess {String} [user_name] 用户名，可选
 * @apiSuccess {String} [user_password] 密码，可选
 * @apiSuccess {u64} poll_time 轮询周期，单位毫秒
 * @apiSuccess {bool} is_poll is_poll
 * @apiSuccess {bool} is_new 版本，false是配电物联2020版本，true是2021版本，该参数会导致topic不同
 * @apiSuccess {String} app_name APP的名称，用于生成topic
 * @apiSuccess {Map} point_id_to_pos HashMap<point_id:u64, data_configure的索引:usize>
 * @apiSuccess {HYPoint[]} data_configure 测点列表
 * @apiSuccess {Map} model_to_pos HashMap<model:String, 测点索引:usize[]>
 * @apiSuccess {Map} device_configure HashMap<device_id:u64, 设备的信息:HYDevice>
 */
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct HYMqttTransport {
    pub id: u64,
    /// 通道名称
    pub name: String,
    /// 服务端的ip和por
    pub mqtt_broker: (String, u16),
    /// 通道状态对应的测点号
    pub point_id: u64,
    /// 读测点的主题
    pub read_topic: String,
    /// 写测点的主题
    pub write_topic: String,
    /// 用户名，可选
    pub user_name: Option<String>,
    /// 用户密码，可选
    pub user_password: Option<String>,
    /// 轮询周期，单位毫秒
    pub poll_time: u64,
    pub is_poll: bool,
    /// 版本，false是配电物联2020版本，true是2021版本，该参数会导致topic不同
    pub is_new: bool,
    /// APP的名称，用于生成topic
    pub app_name: String,
    /// key is point id, value is information object address(data_configure的索引)
    pub point_id_to_pos: HashMap<u64, usize>,
    /// 测点列表
    pub data_configure: Vec<HYPoint>,
    /// 模型列表key is model, value is 测点索引
    pub model_to_pos: HashMap<String, Vec<usize>>,
    /// 设备key is 设备序号, value is (dev,设备的信息)
    pub device_configure: HashMap<u32, HYDevice>,
}

/**
 * @api {华云台区智能融合终端模型} /HYDevice HYDevice
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u32} device_id device_id
 * @apiSuccess {bool} need_register need_register
 * @apiSuccess {String} [dev] dev
 * @apiSuccess {usize[]} points_pos points_pos
 * @apiSuccess {u64} poll_period poll_period
 * @apiSuccess {HYDeviceInfo} device_info device_info
 */
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[allow(non_snake_case)]
pub struct HYDevice {
    pub device_id: u32,
    pub need_register: bool,
    pub dev_uuid: Option<String>,
    pub points_pos: Vec<usize>,
    pub poll_period: u64,
    pub device_info: HYDeviceInfo,
}
/**
 * @api {华云台区智能融合终端模型信息} /HYDeviceInfo HYDeviceInfo
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {String} model 模型名称
 * @apiSuccess {String} port RS485-1、RS485-2、RS485-3、RS485-4、PLC、UMW
 * @apiSuccess {String} addr 地址
 * @apiSuccess {String} desc 描述
 * @apiSuccess {String} manuID 厂商ID 1234 名
 * @apiSuccess {String} isReport 上报标志 0不需要上报，1需要上报
 * @apiSuccess {String} manuName 厂商名称
 * @apiSuccess {String} ProType 协议类型
 * @apiSuccess {String} deviceType 设备型号
 * @apiSuccess {String} nodeID 节点ID
 * @apiSuccess {String} productID 产品ID
 */
// 华云-台区智能融合终端模型
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[allow(non_snake_case)]
pub struct HYDeviceInfo {
    /// 模型名称
    pub model: String,
    /// RS485-1、RS485-2、RS485-3、RS485-4、PLC、UMW
    pub port: String,
    pub addr: String,
    pub desc: String,
    pub manuID: String,   // 厂商ID 1234 名
    pub isReport: String, // 上报标志 0不需要上报，1需要上报
    // 以下是新版本中添加的
    pub manuName: String,   // 厂商名称
    pub ProType: String,    // 协议类型
    pub deviceType: String, // 设备型号
    pub nodeID: String,     // 节点ID
    pub productID: String,  // 产品ID
}

impl HYDeviceInfo {
    pub fn new_undefine() -> Self {
        HYDeviceInfo {
            model: "".to_string(),
            port: "".to_string(),
            addr: "".to_string(),
            desc: "".to_string(),
            manuID: "".to_string(),
            isReport: "".to_string(),
            manuName: "".to_string(),
            ProType: "".to_string(),
            deviceType: "".to_string(),
            nodeID: "".to_string(),
            productID: "".to_string(),
        }
    }
}

/**
 * @api {华云台区智能融合终端测点} /HYPoint HYPoint
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {bool} is_writable 是否可写（暂时无用）
 * @apiSuccess {u64} device_id 测点归属的设备序号
 * @apiSuccess {u64} point_id 对应的测点Id
 * @apiSuccess {HYPointInfo} point_info 测点信息
 */
// 华云-台区智能融合终端测点
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct HYPoint {
    /// 暂时无用
    pub not_realtime: bool,
    /// 测点归属的设备序号
    pub device_id: u32,
    /// 对应的测点Id
    pub point_id: u64,
    pub point_info: HYPointInfo,
}

/**
 * @api {华云台区智能融合终端测点信息} /HYPointInfo HYPointInfo
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {String} name name
 * @apiSuccess {String} type type
 * @apiSuccess {String} unit unit
 * @apiSuccess {String} deadzone deadzone
 * @apiSuccess {String} ratio ratio
 * @apiSuccess {String} isReport isReport
 * @apiSuccess {String} userdefine userdefine
 */
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[allow(non_snake_case)]
pub struct HYPointInfo {
    pub name: String,
    pub r#type: String,
    pub unit: String,
    pub deadzone: String,
    pub ratio: String,
    pub isReport: String,
    // 名字不能改！！！
    pub userdefine: String,
}

impl HYMqttTransport {
    pub fn from_file(path: &str) -> Result<Self, (usize, usize)> {
        let content = std::fs::read(path).map_err(|_| (0, 0))?;
        // let content = if env::IS_ENCRYPT {
        //     env::decrypt_vec(content.as_slice())
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

    pub fn from_csv(path: &str) -> Result<HYMqttTransport, (usize, usize)> {
        let content = std::fs::read(path).map_err(|_| (0, 0))?;
        // let content = if env::IS_ENCRYPT {
        //     let plain_t = decrypt_vec(content.as_slice());
        //     plain_t
        // } else {
        //     content
        // };
        HYMqttTransport::from_csv_bytes(content.as_slice())
    }

    pub fn from_csv_bytes(content: &[u8]) -> Result<HYMqttTransport, (usize, usize)> {
        let content_new = transfer_to_utf8(content.to_vec()).map_err(|_| (0, 0))?;
        let content = content_new.as_slice();
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(content);
        let mut records = rdr.records();
        let rc = (0usize, 1);
        let name = csv_string(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        let rc = (1usize, 1);
        let broker_ip = csv_string(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        let rc = (2usize, 1);
        let broker_port = csv_u16(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        let mqtt_broker = (broker_ip, broker_port);
        let rc = (3usize, 1);
        let point_num = csv_usize(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        let rc = (4usize, 1);
        let point_id = csv_u64(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        let rc = (5usize, 1);
        let read_topic = csv_string(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        let rc = (6usize, 1);
        let write_topic =
            csv_string(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        // 下面用户名和密码是可选的
        let mut user_name = None;
        let mut user_password = None;
        let rc = (7usize, 1);
        if let Some(Ok(line)) = records.next() {
            user_name = csv_string(&line, rc.1);
        }
        let rc = (8usize, 1);
        if let Some(Ok(line)) = records.next() {
            user_password = csv_string(&line, rc.1);
        }
        // 轮询时间可选，需大于100ms
        let rc = (9usize, 1);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let s = csv_str(&record, rc.1).ok_or(rc)?;
        let (poll_time, is_poll) = if s.is_empty() {
            (10000, false)
        } else {
            let time = s.parse::<u64>().map_err(|_| rc)?;
            if time < 100 {
                return Err(rc);
            }
            (time, true)
        };
        let rc = (10usize, 1);
        let app_name = csv_string(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        let rc = (11usize, 1);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let s = csv_str(&record, rc.1).ok_or(rc)?.to_uppercase();
        let is_new = match s.as_str() {
            "FALSE" => false,
            "TRUE" => true,
            _ => false,
        };

        // 开启读取测点信息
        let mut point_id_to_pos: HashMap<u64, usize> = HashMap::with_capacity(point_num);
        let mut data_configure: Vec<HYPoint> = Vec::with_capacity(point_num);
        let mut model_to_pos: HashMap<String, Vec<usize>> = HashMap::new();
        let mut device_configure: HashMap<u32, HYDevice> = HashMap::new();
        // 从新加载
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(content);
        let mut records = rdr.records();
        let mut tmp = HashSet::with_capacity(point_num);
        for i in 0..point_num {
            let rc = (i, 3);
            let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
            let id = csv_u64(&record, rc.1).ok_or(rc)?;
            let rc = (i, 4);
            let s = csv_str(&record, rc.1).ok_or(rc)?.to_uppercase();
            let is_writable = match s.as_str() {
                "FALSE" => false,
                "TRUE" => true,
                _ => false,
            };
            let rc = (i, 5);
            let name = csv_string(&record, rc.1).ok_or(rc)?;
            let rc = (i, 6);
            let point_type = csv_string(&record, rc.1).ok_or(rc)?;
            let rc = (i, 7);
            let point_information = csv_string(&record, rc.1).ok_or(rc)?;
            let pi: Vec<&str> = point_information.split(';').collect();
            if pi.len() < 5 {
                return Err(rc);
            }
            let rc = (i, 8);
            let model_name = csv_string(&record, rc.1).ok_or(rc)?;
            let rc = (i, 9);
            let device_id = csv_u32(&record, rc.1).ok_or(rc)?;
            let rc = (i, 10);
            let device_information = csv_string(&record, rc.1).ok_or(rc)?;
            let de: Vec<&str> = device_information.split(';').collect();

            match device_configure.get_mut(&device_id) {
                Some(d) => {
                    // 已经有这个设备
                    if d.device_info.model != model_name {
                        // 同一个设备的模型不一致
                        return Err((i, 9));
                    }
                    d.points_pos.push(i);
                }
                None => {
                    // 没有这个设备就添加新设备
                    if de.len() < 5 {
                        return Err(rc);
                    }
                    let rc = (i, 11);
                    let poll_period = csv_u64(&record, rc.1).ok_or(rc)?;
                    let rc = (i, 12);
                    let s = csv_str(&record, rc.1).ok_or(rc)?.to_uppercase();
                    let need_register = match s.as_str() {
                        "FALSE" => false,
                        "TRUE" => true,
                        _ => false,
                    };
                    device_configure.insert(
                        device_id,
                        HYDevice {
                            device_id,
                            need_register,
                            dev_uuid: None,
                            points_pos: vec![i],
                            poll_period,
                            device_info: HYDeviceInfo {
                                model: model_name.clone(),
                                port: de[0].to_string(),
                                addr: de[1].to_string(),
                                desc: de[2].to_string(),
                                manuID: de[3].to_string(),
                                isReport: de[4].to_string(),
                                // 以下如果没有就取默认值
                                manuName: de.get(5).unwrap_or(&"xxx").to_string(),
                                ProType: de.get(6).unwrap_or(&"xxx").to_string(),
                                deviceType: de.get(7).unwrap_or(&"1234").to_string(),
                                nodeID: de.get(8).unwrap_or(&"XXXX").to_string(),
                                productID: de.get(9).unwrap_or(&"1111XXXX").to_string(),
                            },
                        },
                    );
                }
            }
            data_configure.push(HYPoint {
                not_realtime: is_writable,
                device_id,
                point_id: id,
                point_info: HYPointInfo {
                    name: name.clone(),
                    r#type: point_type.clone(),
                    unit: pi[0].to_string(),
                    deadzone: pi[1].to_string(),
                    ratio: pi[2].to_string(),
                    isReport: pi[3].to_string(),
                    userdefine: pi[4].to_string(),
                },
            });
            match model_to_pos.get_mut(&model_name) {
                Some(v) => {
                    // 记录名字不重复的量测点
                    let mut is_exist = false;
                    for pos in &mut *v {
                        if data_configure[*pos].point_info.name == name {
                            is_exist = true;
                            break;
                        }
                    }
                    if !is_exist {
                        v.push(i);
                    }
                }
                None => {
                    model_to_pos.insert(model_name, vec![i]);
                }
            }
            // 测点不能重复
            if tmp.contains(&id) {
                return Err(rc);
            }
            point_id_to_pos.insert(id, i);
            let point_type_upper = point_type.to_uppercase();
            let types = point_type_upper.split(";").collect::<Vec<_>>();
            if types.len() > 1 {
                let mut current_id = id;
                for t in types {
                    if !t.starts_with(SKIP_TYPE_PREFIX) {
                        tmp.insert(current_id);
                        current_id += 1;
                    }
                }
            } else {
                tmp.insert(id);
            }
        }
        Ok(HYMqttTransport {
            id: 0,
            name,
            mqtt_broker,
            point_id,
            read_topic,
            write_topic,
            user_name,
            user_password,
            poll_time,
            is_poll,
            point_id_to_pos,
            data_configure,
            model_to_pos,
            device_configure,
            app_name,
            is_new
        })
    }

    pub fn get_point_ids(&self) -> Vec<u64> {
        let mut result = Vec::with_capacity(self.point_id_to_pos.len());
        for (id, index) in &self.point_id_to_pos {
            let type_upper = self.data_configure[*index].point_info.r#type.to_uppercase();
            let types: Vec<&str> = type_upper.split(";").collect();
            if types.len() > 1 {
                let mut current_id = *id;
                for t in types {
                    if !t.starts_with(SKIP_TYPE_PREFIX) {
                        result.push(current_id);
                        current_id += 1;
                    }
                }
            } else {
                result.push(*id);
            }
        }
        result
    }
}