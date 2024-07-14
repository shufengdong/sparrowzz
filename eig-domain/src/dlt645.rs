use std::collections::{BTreeMap, HashMap};
use std::convert::TryInto;
use std::net::SocketAddr;
#[cfg(target_family = "unix")]
use std::path::PathBuf;

use csv::StringRecord;
use serde::{Deserialize, Serialize};

use crate::excel::{excel_bytes_to_csv_bytes, transfer_to_utf8};
use crate::{create_parity, csv_str, csv_string, csv_u32, csv_u64, csv_u8, csv_usize, get_csv_str, SerialPara, SerialParity, UNKNOWN_POINT_ID};

const DEFAULT_POLLING_PERIOD_IN_MILLI: u64 = 5000;
const DEFAULT_TIMEOUT_IN_MILLI: u64 = 3000;
// 默认是20ms
pub const DEFAULT_DELAY_BETWEEN_REQUESTS: u64 = 20;

/**
 * @api {枚举_Dlt645参数} /Dlt645Para Dlt645Para
 * @apiPrivate
 * @apiGroup A_Enum
 * @apiSuccess {Object} Serial {"Serial": SerialPara}
 * @apiSuccess {Object} Socket {"Socket": tuple(String, u16)}
 */
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum Dlt645Para {
    Serial(SerialPara),
    Socket(String, u16),
}

/**
 * @api {Dlt645ClientTp} /Dlt645ClientTp Dlt645ClientTp
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} id 通道id
 * @apiSuccess {String} name 通道名称
 * @apiSuccess {Dlt645Para} para 参数
 * @apiSuccess {Dlt645Connection[]} connections connections
 */
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Dlt645ClientTp {
    pub id: u64,
    pub name: String,
    pub para: Dlt645Para,
    pub connections: Vec<Dlt645Connection>,
}

/**
 * @api {Dlt645连接信息} /Dlt645Connection Dlt645Connection
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u8} slave_id slave_id
 * @apiSuccess {String} name 连接名称
 * @apiSuccess {u64} timeout_in_milli 超时时间_毫秒
 * @apiSuccess {u64} point_id 通道状态对应的测点号
 * @apiSuccess {u64} default_polling_period_in_milli 默认的轮询周期
 * @apiSuccess {Dlt645RegisterData[]} data_configure register settings
 * @apiSuccess {Map} point_id_to_rd HashMap<point_id:u64, position_of_register_data:u16>
 * @apiSuccess {Map} data_id_to_rd HashMap<寄存器地址:u16, setting中Dlt645RegisterData[]的位置:u16>
 * @apiSuccess {Map} polling_period_to_data 轮询周期不同的数据，有序Map<轮询周期_毫秒数:u64, position:u16[]>
 */
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Dlt645Connection {
    pub slave_id: u64,
    // 连接名称
    pub name: String,
    // 超时设置
    pub timeout_in_milli: u64,
    // 通道状态对应的测点号
    pub point_id: u64,
    // 默认的轮询周期
    pub default_polling_period_in_milli: u64,
    // register settings
    pub data_configure: Vec<RegisterData>,
    // key is point id, value is position of register data
    pub point_id_to_rd: HashMap<u64, u16>,
    // key:寄存器地址,value:setting中vec<RegisterData>的位置
    pub data_id_to_rd: HashMap<u32, u16>,
    // 轮询周期不同的数据, key is period in milli, value is position.
    pub polling_period_to_data: BTreeMap<u64, Vec<u16>>,
}

impl Default for Dlt645Connection {
    fn default() -> Self {
        Dlt645Connection {
            slave_id: 1,
            name: "new".to_string(),
            timeout_in_milli: DEFAULT_TIMEOUT_IN_MILLI,
            point_id: 0,
            default_polling_period_in_milli: DEFAULT_POLLING_PERIOD_IN_MILLI,
            data_configure: Vec::new(),
            point_id_to_rd: HashMap::new(),
            data_id_to_rd: HashMap::new(),
            polling_period_to_data: BTreeMap::new(),
        }
    }
}

/**
 * @api {Dlt645注册信息} /Dlt645RegisterData Dlt645RegisterData
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u32} data_id 数据标识
 * @apiSuccess {u64} polling_period_in_milli 轮询周期，毫秒
 * @apiSuccess {u64[]} point_ids 对应的测点Id
 */
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RegisterData {
    // 数据标识
    pub data_id: u32,
    // 轮询周期
    pub polling_period_in_milli: u64,
    // 对应的测点Id
    pub point_ids: Vec<u64>,
}

impl Dlt645Connection {
    fn from_csv_records(
        content: &[u8],
        offset: usize,
    ) -> Result<Self, (usize, usize)> {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(content);
        let mut records = rdr.records();
        // 1th line
        let rc = (0usize, 1 + offset);
        let name = csv_string(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        // 2th line
        let rc = (1usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let point_num = csv_usize(&record, rc.1).ok_or(rc)?;
        if point_num as u16 > u16::MAX {
            return Err(rc)
        }
        // 3th line
        let rc = (2usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let slave_id = csv_u64(&record, rc.1).ok_or(rc)?;
        // 4th line
        let rc = (3usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let s = csv_str(&record, rc.1).ok_or(rc)?;
        let default_polling_period_in_milli: u64 = if s.is_empty() {
            DEFAULT_POLLING_PERIOD_IN_MILLI
        } else {
            s.parse().map_err(|_| rc)?
        };
        // 5th line
        let rc = (4usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let s = csv_str(&record, rc.1).ok_or(rc)?;
        let timeout_in_milli: u64 = if s.is_empty() {
            DEFAULT_TIMEOUT_IN_MILLI
        } else {
            s.parse().map_err(|_| rc)?
        };
        // 6th line
        let rc = (5usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let s = csv_str(&record, rc.1).ok_or(rc)?;
        let point_id: u64 = if s.is_empty() {
            0
        } else {
            s.parse().map_err(|_| rc)?
        };
        // 9th line ...
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(content);
        let mut records = rdr.records();
        let rc = (0, 3 + offset);
        records.next().ok_or(rc)?.map_err(|_| rc)?;
        let mut data_configure: Vec<RegisterData> = Vec::with_capacity(point_num);
        for row in 1..=point_num {
            let rc = (row + 8, 3 + offset);
            let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
            data_configure.push(RegisterData::parse_register_data(&record, rc.0, rc.1)?);
        }
        let mut conn = Dlt645Connection {
            slave_id,
            name,
            timeout_in_milli,
            point_id,
            default_polling_period_in_milli,
            data_configure,
            ..Default::default()
        };
        conn.create_data_config().map_err(|(r, c, _)|(r, c + offset))?;
        Ok(conn)
    }

    /// 得到某一个采样周期的data id集合
    pub fn create_request(&self, period: u64) -> Vec<u32> {
        return if let Some(positions) = self.polling_period_to_data.get(&period) {
            let mut result = Vec::with_capacity(positions.len());
            for pos in positions {
                result.push(self.data_configure[*pos as usize].data_id);
            }
            result
        } else {
            // 没有找到对应的寄存器列表，返回空
            vec![]
        };
    }

    pub fn create_data_config(&mut self) -> Result<(),(usize, usize, String)> {
        let size = self.data_configure.len();
        let mut point_id_to_rd: HashMap<u64, u16> = HashMap::with_capacity(size);
        // key:寄存器地址,value:setting中vec<RegisterData>的位置
        let mut data_id_to_rd: HashMap<u32, u16> = HashMap::with_capacity(size);
        // 轮询周期不同的数据, key is period in milli, value is position.
        let mut polling_period_to_data: BTreeMap<u64, Vec<u16>> = BTreeMap::new();
        polling_period_to_data.insert(self.default_polling_period_in_milli, Vec::with_capacity(size));
        // 开始统计不同轮询周期的数据
        let mut tmp: HashMap<u64, u32> = HashMap::with_capacity(10);
        for rd in &self.data_configure {
            if let Some(ori) = tmp.get_mut(&rd.polling_period_in_milli) {
                *ori += 1;
            } else {
                tmp.insert(rd.polling_period_in_milli, 1);
            }
        }
        for (i, num) in tmp {
            let mut a_v: Vec<u16> = Vec::with_capacity(num as usize);
            for (index, rd) in self.data_configure.iter().enumerate() {
                if rd.polling_period_in_milli == i {
                    a_v.push(index.try_into().unwrap());
                }
            }
            polling_period_to_data.insert(i, a_v);
        }
        for (index, rd) in self.data_configure.iter().enumerate() {
            // 测点号重复
            for point_id in &rd.point_ids {
                if point_id_to_rd.contains_key(point_id) {
                    let tip = format!("Invalid register point (id :{}):\nThe point ID is already existed", point_id);
                    return Err((index + 1, 8, tip)); // 测点号的位置
                }
            }
            for point_id in &rd.point_ids {
                point_id_to_rd.insert(*point_id, index.try_into().unwrap());
            }
            // 起始地址重复
            if data_id_to_rd.contains_key(&rd.data_id) {
                let tip = format!("Invalid register point (id :{}):\nThe register address is already existed", rd.data_id);
                return Err((index + 1, 4, tip));  // 地址的位置
            }
            data_id_to_rd.insert(rd.data_id, index.try_into().unwrap());
        }
        self.data_id_to_rd = data_id_to_rd;
        self.point_id_to_rd = point_id_to_rd;
        self.polling_period_to_data = polling_period_to_data;
        Ok(())
    }
}

impl Dlt645ClientTp {
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

    pub fn from_csv(path: &str) -> Result<Dlt645ClientTp, (usize, usize)> {
        let content = std::fs::read(path).map_err(|_| (0, 0))?;
        // let content = if env::IS_ENCRYPT {
        //     let plain_t = decrypt_vec(content.as_slice());
        //     plain_t
        // } else {
        //     content
        // };
        Dlt645ClientTp::from_csv_bytes(content.as_slice())
    }

    pub fn from_csv_bytes(content: &[u8]) -> Result<Dlt645ClientTp, (usize, usize)> {
        let content_new = transfer_to_utf8(content.to_vec()).map_err(|_| (0, 0))?;
        let content = content_new.as_slice();
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(content);
        let mut records = rdr.records();
        let rc = (0usize, 1);
        let name = csv_string(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        let rc = (1usize, 1);
        let conn_num: usize =
            csv_usize(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        let rc = (2usize, 1);
        let baud_rate = csv_u32(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        let rc = (3usize, 1);
        let file_path = csv_string(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        // 如果能解析成socket
        let para = if let Ok(addres) = file_path.parse::<SocketAddr>() {
            Dlt645Para::Socket(addres.ip().to_string(), addres.port())
        } else {
            #[cfg(target_family = "unix")]
                let file_path = if PathBuf::from(file_path.clone()).is_relative() {
                "/dev/".to_string() + file_path.as_str()
            } else {
                file_path
            };
            // 下面三行是可选的
            // 第5行
            let record = records.next();
            let data_bits = if let Some(Ok(tmp)) = record {
                if let Some(v) = csv_u8(&tmp, 1) {
                    v
                } else {
                    8 // 默认是8
                }
            } else {
                8
            };
            // 第6行
            let record = records.next();
            let stop_bits = if let Some(Ok(tmp)) = record {
                if let Some(v) = csv_u8(&tmp, 1) {
                    v
                } else {
                    1 // 默认是1
                }
            } else {
                1
            };
            // 第7行
            let record = records.next();
            let parity = if let Some(Ok(tmp)) = record {
                if let Some(v) = csv_str(&tmp, 1) {
                    create_parity(v)
                } else {
                    SerialParity::None
                }
            } else {
                SerialParity::None
            };
            // 第8行
            // 3.5个字符，每个字符 1起始+8+1校验（或0个）+1结尾 38.5个字符
            let mut delay_between_requests = (38500. / (baud_rate as f64)).ceil() as u64;
            if delay_between_requests == 0 {
                delay_between_requests = DEFAULT_DELAY_BETWEEN_REQUESTS;
            }
            let record = records.next();
            if let Some(Ok(tmp)) = record {
                if let Some(v) = csv_u64(&tmp, 1) {
                    if v > 0 {
                        delay_between_requests = v;
                    }
                }
            }
            Dlt645Para::Serial(SerialPara {
                file_path,
                baud_rate,
                data_bits,
                stop_bits,
                parity,
                delay_between_requests,
            })
        };

        let mut connections: Vec<Dlt645Connection> = Vec::with_capacity(conn_num);
        for i in 0..conn_num {
            let connection = Dlt645Connection::from_csv_records(content, i * 7 + 3)?;
            connections.push(connection);
        }
        Ok(Dlt645ClientTp {
            id: 0,
            name,
            para,
            connections,
        })
    }

    pub fn export_csv(&self, text_map: &HashMap<String, String>) -> String {
        let len_conn = self.connections.len();

        // 第一排
        let mut result = format!("{},{},,",
                                 text_map.get("tp_name").unwrap_or(&"Transport Name".to_string()),
                                 get_csv_str(&self.name));

        for i in 0..len_conn {
            result += &format!(
                "{},{},{},{},{},{}",
                text_map.get("conn_name").unwrap_or(&"Connection Name".to_string()),
                get_csv_str(&self.connections[i].name),
                text_map.get("index").unwrap_or(&"Index".to_string()),
                text_map.get("dlt645_data_id").unwrap_or(&"Data ID".to_string()),
                text_map.get("register_period_name").unwrap_or(&"Polling Period(ms)".to_string()),
                text_map.get("status_point").unwrap_or(&"Status Point".to_string()),
            );
            if i != len_conn - 1 {
                result += ",,";
            }
        }
        result += "\n";

        // 第二至八排
        let title_conn = [text_map.get("point_number").unwrap_or(&"Point Count".to_string()).clone(),
            text_map.get("slave_id").unwrap_or(&"Slave ID".to_string()).clone(),
            text_map.get("register_period_name").unwrap_or(&"Polling Period(ms)".to_string()).clone(),
            text_map.get("timeout_ms").unwrap_or(&"Timeout(ms)".to_string()).clone(),
            text_map.get("tp_point_id").unwrap_or(&"Point ID".to_string()).clone(),
            text_map.get("status_point").unwrap_or(&"Status Point".to_string()).clone(),
            "".to_string(),
            "".to_string()];
        let title_tp = match &self.para {
            Dlt645Para::Serial(para) => {
                let parity = match &para.parity {
                    SerialParity::None => "NONE",
                    SerialParity::Odd => "ODD",
                    SerialParity::Even => "EVEN",
                    SerialParity::Mark => "MARK",
                    SerialParity::Space => "SPACE",
                };
                vec![
                    format!("{},{},",
                        text_map.get("conn_num").unwrap_or(&"Connection Count".to_string()),
                        self.connections.len()),
                    format!("{},{},",
                            text_map.get("baud_rate").unwrap_or(&"Baud Rate".to_string()),
                            para.baud_rate),
                    format!("{},{},",
                            text_map.get("file_path").unwrap_or(&"File Path".to_string()),
                            para.file_path),
                    format!("{},{},",
                            text_map.get("data_bits").unwrap_or(&"Data Bits".to_string()),
                            para.data_bits),
                    format!("{},{},",
                            text_map.get("stop_bits").unwrap_or(&"Stop Bits".to_string()),
                            para.stop_bits),
                    format!("{},{},",
                            text_map.get("parity").unwrap_or(&"Parity".to_string()),
                            parity),
                    format!("{},{},",
                        text_map.get("serial_para_delay_ms_tip").unwrap_or(&"Delay Between Requests (ms)".to_string()),
                        para.delay_between_requests),
                ]
            }
            Dlt645Para::Socket(ip, port) => {
                vec![
                    format!("{},{},",
                        text_map.get("conn_num").unwrap_or(&"Connection Count".to_string()),
                        self.connections.len()),
                    format!("{},19200,", text_map.get("baud_rate").unwrap_or(&"Baud Rate".to_string())),
                    format!("{},{}:{},", text_map.get("file_path").unwrap_or(&"File Path".to_string()), ip, port),
                    ",,".to_string(),
                    ",,".to_string(),
                    ",,".to_string(),
                    ",,".to_string(),
                ]
            }
        };

        for cnt in 0..7 {
            result += &title_tp[cnt];
            for i in 0..len_conn {
                if self.connections[i].data_configure.len() > cnt {
                    let r = &self.connections[i].data_configure[cnt];
                    let content_conn = Self::get_dlt_conn_csv(&self.connections[i], cnt);
                    result += &format!(
                        ",{},{},{},{:#010X},{},",
                        title_conn[cnt],
                        content_conn,
                        cnt + 1,
                        r.data_id,
                        r.polling_period_in_milli
                    );
                    for i in 0..r.point_ids.len() {
                        result += &format!("{}", r.point_ids[i]);
                        if i != r.point_ids.len() - 1 {
                            result += ";";
                        }
                    }
                    if i != len_conn - 1 {
                        result += ",";
                    }
                } else {
                    let content_conn = Self::get_dlt_conn_csv(&self.connections[i], cnt);
                    result += &format!(",{},{},,,,", title_conn[cnt], content_conn);
                }
            }
            result += "\n";
        }

        // 剩余的
        let mut max_data_len = self.connections[0].data_configure.len();
        for c in &self.connections {
            if c.data_configure.len() > max_data_len {
                max_data_len = c.data_configure.len();
            }
        }
        for row in 7..max_data_len {
            result += ",,";
            for i in 0..len_conn {
                if self.connections[i].data_configure.len() > row {
                    let r = &self.connections[i].data_configure[row];
                    result += &format!(
                        ",,,{},{:#010X},{},",
                        row + 1,
                        r.data_id,
                        r.polling_period_in_milli
                    );
                    for i in 0..r.point_ids.len() {
                        result += &format!("{}", r.point_ids[i]);
                        if i != r.point_ids.len() - 1 {
                            result += ";";
                        }
                    }
                    if i != len_conn - 1 {
                        result += ",";
                    }
                } else {
                    result += ",,,,,,,";
                }
            }
            result += "\n";
        }

        result
    }

    fn get_dlt_conn_csv(conn: &Dlt645Connection, index: usize) -> String {
        match index {
            0 => conn.data_configure.len().to_string(),
            1 => conn.slave_id.to_string(),
            2 => conn.default_polling_period_in_milli.to_string(),
            3 => conn.timeout_in_milli.to_string(),
            4 => conn.point_id.to_string(),
            _ => "".to_string(),
        }
    }

    pub fn get_point_ids(&self) -> Vec<u64> {
        let mut size = 0;
        for conn in &self.connections {
            size += conn.data_configure.len()
        }
        size += self.connections.len();
        let mut r: Vec<u64> = Vec::with_capacity(size);
        for conn in &self.connections {
            for rd in &conn.data_configure {
                for point_id in &rd.point_ids {
                    r.push(*point_id);
                }
            }
            if conn.point_id != UNKNOWN_POINT_ID {
                r.push(conn.point_id);
            }
        }
        r
    }
}

impl RegisterData {
    fn parse_register_data(
        record: &StringRecord,
        row: usize,
        first_col: usize,
    ) -> Result<Self, (usize, usize)> {
        //let start: usize = 3 + offset;
        let rc = (row, first_col);
        let data_id = csv_u32(record, rc.1).ok_or(rc)?;
        // 轮询周期
        let rc = (row, first_col + 1);
        let polling_period_in_milli = csv_u64(record, rc.1).ok_or(rc)?;
        // 对应的测点Id
        let rc = (row, first_col + 2);
        let points = csv_str(record, rc.1).ok_or(rc)?;
        let ids: Vec<&str> = points.split(';').collect();
        let mut point_ids: Vec<u64> = Vec::with_capacity(ids.len());
        for id in ids {
            let point_id: u64 = id.parse().map_err(|_| rc)?;
            point_ids.push(point_id);
        }
        Ok(RegisterData {
            data_id,
            polling_period_in_milli,
            point_ids,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::dlt645::{Dlt645ClientTp, Dlt645Para};
    use crate::{SerialPara, SerialParity};
    use crate::env::Env;

    #[test]
    fn test_parse_dlt_csv() {
        Env::init("");
        Env::set_vitrual_env();
        let tp = Dlt645ClientTp::from_csv("tests/dlt645-test1.csv").unwrap();
        assert_eq!(tp.name, "DLT测试通道");
        assert_eq!(tp.connections.len(), 2);
        assert_eq!(
            tp.para,
            Dlt645Para::Serial(SerialPara {
                file_path: "/dev/ttyUSB0".to_string(),
                baud_rate: 19200,
                data_bits: 10,
                stop_bits: 2,
                parity: SerialParity::Odd,
                delay_between_requests: 20,
            })
        );
        let connection = tp.connections.first().unwrap();
        assert_eq!(connection.name, "测试通道1");
        assert_eq!(connection.slave_id, 0x00_00_00_34_03_10_98_67);
        assert_eq!(connection.default_polling_period_in_milli, 5000);
        assert_eq!(connection.timeout_in_milli, 1000);
        assert_eq!(connection.polling_period_to_data.len(), 1);
        assert_eq!(
            connection
                .polling_period_to_data
                .get(&5000u64)
                .unwrap()
                .len(),
            10
        );
        assert_eq!(connection.data_configure.len(), 10);

        assert_eq!(
            connection.data_configure.first().unwrap().data_id,
            0x00_00_00_00
        );
        assert_eq!(connection.data_configure.first().unwrap().point_ids.len(), 1);
        assert_eq!(connection.data_configure.first().unwrap().point_ids[0], 4001);
        assert_eq!(
            connection
                .data_configure.first()
                .unwrap()
                .polling_period_in_milli,
            5000
        );

        assert_eq!(
            connection.data_configure.get(6).unwrap().data_id,
            0x01_01_00_00
        );
        assert_eq!(connection.data_configure.get(6).unwrap().point_ids.len(), 2);
        assert_eq!(connection.data_configure.get(6).unwrap().point_ids[0], 4007);
        assert_eq!(connection.data_configure.get(6).unwrap().point_ids[1], 4011);
        assert_eq!(
            connection
                .data_configure
                .get(6)
                .unwrap()
                .polling_period_in_milli,
            5000
        );

        assert_eq!(connection.data_configure.get(9).unwrap().point_ids[0], 4010);
        assert_eq!(
            connection.data_configure.get(9).unwrap().data_id,
            0x02_80_00_0A
        );
        assert_eq!(
            connection
                .data_configure
                .get(9)
                .unwrap()
                .polling_period_in_milli,
            5000
        );

        let connection = tp.connections.get(1).unwrap();
        assert_eq!(connection.name, "测试通道2");
        assert_eq!(connection.data_configure.len(), 20);

        let s = tp.export_csv(&HashMap::with_capacity(0));
        let tp2 = Dlt645ClientTp::from_csv_bytes(s.as_bytes()).unwrap();
        assert_eq!(tp, tp2);

    }

    #[test]
    fn test_parse_dlt_csv2() {
        let tp = Dlt645ClientTp::from_csv("tests/dlt645-test2.csv").unwrap();
        assert_eq!(tp.name, "DLT测试通道");
        assert_eq!(tp.connections.len(), 2);
        assert_eq!(tp.para, Dlt645Para::Socket("127.0.0.1".to_string(), 2300));
        let connection = tp.connections.first().unwrap();
        assert_eq!(connection.name, "测试通道1");
        assert_eq!(connection.slave_id, 0x00_00_00_34_03_10_98_67);
        assert_eq!(connection.default_polling_period_in_milli, 5000);
        assert_eq!(connection.timeout_in_milli, 1000);
        assert_eq!(connection.polling_period_to_data.len(), 1);
        assert_eq!(
            connection
                .polling_period_to_data
                .get(&5000u64)
                .unwrap()
                .len(),
            10
        );
        assert_eq!(connection.data_configure.len(), 10);

        assert_eq!(
            connection.data_configure.first().unwrap().data_id,
            0x00_00_00_00
        );
        assert_eq!(connection.data_configure.first().unwrap().point_ids.len(), 1);
        assert_eq!(connection.data_configure.first().unwrap().point_ids[0], 4001);
        assert_eq!(
            connection
                .data_configure.first()
                .unwrap()
                .polling_period_in_milli,
            5000
        );

        assert_eq!(
            connection.data_configure.get(6).unwrap().data_id,
            0x01_01_00_00
        );
        assert_eq!(connection.data_configure.get(6).unwrap().point_ids.len(), 2);
        assert_eq!(connection.data_configure.get(6).unwrap().point_ids[0], 4007);
        assert_eq!(connection.data_configure.get(6).unwrap().point_ids[1], 4011);
        assert_eq!(
            connection
                .data_configure
                .get(6)
                .unwrap()
                .polling_period_in_milli,
            5000
        );

        assert_eq!(connection.data_configure.get(9).unwrap().point_ids[0], 4010);
        assert_eq!(
            connection.data_configure.get(9).unwrap().data_id,
            0x02_80_00_0A
        );
        assert_eq!(
            connection
                .data_configure
                .get(9)
                .unwrap()
                .polling_period_in_milli,
            5000
        );

        let connection = tp.connections.get(1).unwrap();
        assert_eq!(connection.name, "测试通道2");
        assert_eq!(connection.data_configure.len(), 20);

        let s = tp.export_csv(&HashMap::with_capacity(0));
        let tp2 = Dlt645ClientTp::from_csv_bytes(s.as_bytes()).unwrap();
        assert_eq!(tp, tp2);
    }

    // #[test]
    // fn test_export_dlt_model() {
    //     let tp1 = Dlt645ClientTp::from_csv("tests/dlt645-test1.csv").unwrap();
    //     let tp1_export = tp1.export_model();
    //     let tp1_parse_from_export = Dlt645ClientTp::from_csv_bytes(tp1_export.as_bytes());
    //     assert_eq!(tp1_parse_from_export, Ok(tp1));

    //     let tp2 = Dlt645ClientTp::from_csv("tests/dlt645-test2.csv").unwrap();
    //     let tp2_export = tp2.export_model();
    //     let tp2_parse_from_export = Dlt645ClientTp::from_csv_bytes(tp2_export.as_bytes());
    //     assert_eq!(tp2_parse_from_export, Ok(tp2));
    // }
}
