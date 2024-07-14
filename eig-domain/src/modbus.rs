use core::fmt;
use std::{collections::{BTreeMap, HashMap, HashSet}, str::FromStr};
use std::{cmp::min, convert::TryInto};
use std::fmt::{Display, Formatter};
#[cfg(target_family = "unix")]
use std::path::PathBuf;

use csv::StringRecord;
use serde::{Deserialize, Serialize};

use crate::{create_parity, csv_str, csv_string, csv_u16, csv_u32, csv_u64, csv_u8, csv_usize, DEFAULT_TCP_CLIENT_LIMIT, get_csv_str, MAX_POLLING_PERIOD, SerialPara, SerialParity, UNKNOWN_POINT_ID, UNKNOWN_TCP_PORT};
use crate::excel::{excel_bytes_to_csv_bytes, transfer_to_utf8};
use crate::prop::DataType;

// const DEFAULT_MAX_ADDR_READ_BINARY: u16 = 65535;
// const DEFAULT_MAX_ADDR_WRITE_COILS: u16 = 65535;
// const DEFAULT_MAX_ADDR_WRITE_REGISTERS: u16 = 65535;
// 这里存储地址用的是u16，所以最大值是65535

const DEFAULT_MAX_COUNT_READ_NUMERIC: u16 = 125;
const DEFAULT_MAX_COUNT_READ_BIT: u16 = 2000;
const DEFAULT_MAX_COUNT_WRITE_COILS: u16 = 1968;
const DEFAULT_MAX_COUNT_WRITE_REGISTERS: u16 = 120;

const DEFAULT_TIMEOUT_IN_MILLI: u64 = 3000;
const DEFAULT_POLLING_PERIOD_IN_MILLI: u64 = 5000;

// 默认是20ms
pub const DEFAULT_DELAY_BETWEEN_REQUESTS: u64 = 20;

/**
 * @api {枚举_注册类型} /RegisterType RegisterType
 * @apiPrivate
 * @apiGroup A_Enum
 * @apiSuccess {String} COILS COILS
 * @apiSuccess {String} DISCRETE DISCRETE
 * @apiSuccess {String} INPUT INPUT
 * @apiSuccess {String} HOLDING HOLDING
 */
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum RegisterType {
    COILS,
    DISCRETE,
    INPUT,
    HOLDING,
}

/**
 * @api {枚举_MbProtocolType} /MbProtocolType MbProtocolType
 * @apiPrivate
 * @apiGroup A_Enum
 * @apiSuccess {String} ENCAP ENCAP
 * @apiSuccess {String} XA XA
 * @apiSuccess {String} RTU RTU
 */
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum MbProtocolType {
    ENCAP,
    XA,
    RTU,
}

impl Display for MbProtocolType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for RegisterType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<&str> for MbProtocolType {
    fn from(value: &str) -> Self {
        // let value = serialized_string.to_uppercase();
        match value.to_uppercase().as_str() {
            "ENCAP" => MbProtocolType::ENCAP,
            "XA" => MbProtocolType::XA,
            "RTU" => MbProtocolType::RTU,
            _ => panic!("Error deserializing MbProtoType"),
        }
    }
}

impl FromStr for RegisterType {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "DISCRETE" => Ok(RegisterType::DISCRETE),
            "COILS" => Ok(RegisterType::COILS),
            "INPUT" => Ok(RegisterType::INPUT),
            "HOLDING" => Ok(RegisterType::HOLDING),
            _ => Err(()),
        }
    }
}

/**
 * @api {Modbus注册信息} /ModbusRegisterData ModbusRegisterData
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {RegisterType} register_type register_type
 * @apiSuccess {u16} from from
 * @apiSuccess {DataType} data_type data_type
 * @apiSuccess {bool} should_new_request 是否必须新开一个请求
 * @apiSuccess {u64} polling_period_in_milli 轮询周期，毫秒
 * @apiSuccess {u64} point_id 对应的测点Id
 */
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RegisterData {
    pub register_type: RegisterType,
    pub from: u16,
    pub data_type: DataType,
    // 是否必须新开一个请求
    pub should_new_request: bool,
    // 轮询周期
    pub polling_period_in_milli: u64,
    // 对应的测点Id
    pub point_id: u64,
}

/**
 * @api {ModbusTcpClientTp} /ModbusTcpClientTp ModbusTcpClientTp
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} id 通道id
 * @apiSuccess {String} name 通道名称
 * @apiSuccess {tuple} tcp_server 服务端的ip和port，tuple格式为(ip:String, port:u32)
 * @apiSuccess {MbConnection[]} connections connections
 */
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ModbusTcpClientTp {
    pub id: u64,
    /// 通道名称
    pub name: String,
    /// 服务端的ip和port
    pub tcp_server: (String, u32),
    pub connections: Vec<MbConnection>,
}

/**
 * @api {ModbusTcpServerTp} /ModbusTcpServerTp ModbusTcpServerTp
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} id 通道id
 * @apiSuccess {String} name 通道名称
 * @apiSuccess {u16} tcp_server_port 服务的port
 * @apiSuccess {tuple[]} connections 数组，tuple格式为(String, MbConnection)
 */
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ModbusTcpServerTp {
    pub id: u64,
    /// 通道名称
    pub name: String,
    /// 服务的port
    pub tcp_server_port: u16,
    pub connections: Vec<(String, MbConnection)>,
}

/**
 * @api {ModbusRtuClientTp} /ModbusRtuClientTp ModbusRtuClientTp
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} id 通道id
 * @apiSuccess {String} name 通道名称
 * @apiSuccess {SerialPara} para 串口参数
 * @apiSuccess {MbConnection[]} connections connections
 */
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
pub struct ModbusRtuClientTp {
    pub id: u64,
    /// 通道名称
    pub name: String,
    /// 串口参数
    pub para: SerialPara,
    pub connections: Vec<MbConnection>,
}

/**
 * @api {ModbusRtuServerTp} /ModbusRtuServerTp ModbusRtuServerTp
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} id 通道id
 * @apiSuccess {String} name 通道名称
 * @apiSuccess {SerialPara} para 串口参数
 * @apiSuccess {MbConnection} connection connection
 */
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ModbusRtuServerTp {
    pub id: u64,
    /// 通道名称
    pub name: String,
    /// 串口参数
    pub para: SerialPara,
    pub connection: MbConnection,
}

impl Default for ModbusRtuServerTp {
    fn default() -> Self {
        ModbusRtuServerTp {
            id: 0,
            name: String::new(),
            para: SerialPara::default(),
            connection: MbConnection {
                protocol_type: MbProtocolType::RTU,
                ..Default::default()
            },
        }
    }
}

/**
 * @api {Modbus通道连接信息} /MbConnection MbConnection
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u8} slave_id slave_id
 * @apiSuccess {String} name 名称
 * @apiSuccess {MbProtocolType} protocol_type protocol类型
 * @apiSuccess {u16} max_read_register_count max_read_register_count
 * @apiSuccess {u16} max_read_bit_count max_read_bit_count
 * @apiSuccess {u16} max_write_register_count max_write_register_count
 * @apiSuccess {u16} max_write_bit_count max_write_bit_count
 * @apiSuccess {u64} timeout_in_milli 超时时间_毫秒
 * @apiSuccess {u64} delay_between_requests 两条请求直接的间隔
 * @apiSuccess {u64} point_id 通道状态对应的测点号
 * @apiSuccess {u64} default_polling_period_in_milli 默认的轮询周期
 * @apiSuccess {ModbusRegisterData[]} mb_data_configure register settings
 * @apiSuccess {Map} point_id_to_rd HashMap<point_id:u64, position_of_register_data:u16>
 * @apiSuccess {Map} register_addr_to_rd HashMap<寄存器地址:u16, setting中ModbusRegisterData[]的位置:u16>
 * @apiSuccess {Map} polling_period_to_data 轮询周期不同的数据，有序Map<轮询周期_毫秒数:u64, position:u16[]>
 * @apiSuccess {u8} [coil_write_code] if write code is set, yt and yk will use this code to send
 * @apiSuccess {u8} [holding_write_code] holding_write_code
 */
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MbConnection {
    pub slave_id: u8,
    pub name: String,
    pub protocol_type: MbProtocolType,
    pub max_read_register_count: u16,
    pub max_read_bit_count: u16,
    pub max_write_register_count: u16,
    pub max_write_bit_count: u16,
    /// 超时设置
    pub timeout_in_milli: u64,
    /// 两条请求直接的间隔
    pub delay_between_requests: u64,
    /// 通道状态对应的测点号
    pub point_id: u64,
    // 默认的轮询周期
    pub default_polling_period_in_milli: u64,
    /// register settings
    pub mb_data_configure: Vec<RegisterData>,
    /// key is point id, value is position of register data
    pub point_id_to_rd: HashMap<u64, u16>,
    /// key:寄存器地址,value:setting中vec<RegisterData>的位置
    pub register_addr_to_rd: HashMap<u16, u16>,
    /// 轮询周期不同的数据, key is period in milli, value is position.
    pub polling_period_to_data: BTreeMap<u64, Vec<u16>>,
    // if write code is set, yt and yk will use this code to send
    pub coil_write_code: Option<u8>,
    pub holding_write_code: Option<u8>,
}

impl Default for MbConnection {
    fn default() -> Self {
        MbConnection {
            slave_id: 1,
            name: "new".to_string(),
            protocol_type: MbProtocolType::ENCAP,
            max_read_register_count: DEFAULT_MAX_COUNT_READ_BIT,
            max_read_bit_count: DEFAULT_MAX_COUNT_WRITE_REGISTERS,
            max_write_register_count: DEFAULT_MAX_COUNT_WRITE_REGISTERS,
            max_write_bit_count: DEFAULT_MAX_COUNT_WRITE_COILS,
            timeout_in_milli: DEFAULT_TIMEOUT_IN_MILLI,
            delay_between_requests: 0,
            point_id: 0,
            default_polling_period_in_milli: DEFAULT_POLLING_PERIOD_IN_MILLI,
            mb_data_configure: vec![],
            point_id_to_rd: HashMap::new(),
            register_addr_to_rd: HashMap::new(),
            polling_period_to_data: BTreeMap::new(),
            coil_write_code: None,
            holding_write_code: None,
        }
    }
}

pub enum MbConnectionError {
    Repeat(u64),
}

impl MbConnection {

    /// 返回的寄存器顺序是：coil, discrete, input, holding
    #[allow(clippy::type_complexity)]
    pub fn create_request(
        &self,
        period: u64,
    ) -> (Vec<(u16, u16)>, Vec<(u16, u16)>, Vec<(u16, u16)>, Vec<(u16, u16)>) {
        let max_read_bit_count = if self.max_read_bit_count > 0 {
            min(DEFAULT_MAX_COUNT_WRITE_COILS, self.max_read_bit_count)
        } else {
            DEFAULT_MAX_COUNT_WRITE_COILS
        };
        let max_register_count = if self.max_read_register_count > 0 {
            min(DEFAULT_MAX_COUNT_READ_NUMERIC, self.max_read_register_count)
        } else {
            DEFAULT_MAX_COUNT_READ_NUMERIC
        };
        return if let Some(positions) = self.polling_period_to_data.get(&period) {
            let mut registers = positions.clone();
            // 先根据寄存器地址进行排序
            registers.sort_by(|a, b| {
                self.mb_data_configure[*a as usize]
                    .from
                    .partial_cmp(&self.mb_data_configure[*b as usize].from)
                    .unwrap()
            });
            let register_type = RegisterType::COILS;
            let coils = self.get_request(&registers, register_type, max_read_bit_count);
            let register_type = RegisterType::DISCRETE;
            let discretes = self.get_request(&registers, register_type, max_read_bit_count);
            let register_type = RegisterType::INPUT;
            let inputs = self.get_request(&registers, register_type, max_register_count);
            let register_type = RegisterType::HOLDING;
            let holdings = self.get_request(&registers, register_type, max_register_count);
            (coils, discretes, inputs, holdings)
        } else {
            // 没有找到对应的寄存器列表，返回空
            (vec![], vec![], vec![], vec![])
        };
    }
    // 这里传入的register是已经按照地址排好序
    fn get_request(&self, registers: &Vec<u16>, register_type: RegisterType, limit: u16) -> Vec<(u16, u16)> {
        let mut off_set = 0u16;
        let mut num_of_registers = 0u16;
        let mut result: Vec<(u16, u16)> = Vec::with_capacity(registers.len());
        let mut last_index = None;

        for index in 0..registers.len() {
            let d = &self.mb_data_configure[registers[index] as usize];
            let current_word_num = get_register_count(d);
            // 寄存器类型发生了改变
            if d.register_type != register_type {
                continue;
            } else if last_index.is_some() {
                let last = &self.mb_data_configure[registers[last_index.unwrap()] as usize];
                let last_word_num = get_register_count(last);
                // 地址不连续
                if d.from - last.from != last_word_num
                    // 配置了必须重开一个请求的参数，且为true
                    || d.should_new_request
                    // 达到一次Request的最大地址范围
                    || num_of_registers + current_word_num > limit
                {
                    result.push((off_set, num_of_registers));
                    off_set = d.from;
                    num_of_registers = current_word_num;
                } else {
                    // 以上情况都未发生
                    num_of_registers += current_word_num;
                }
                last_index = Some(index);
            } else {
                // first found
                off_set = d.from;
                num_of_registers = current_word_num;
                last_index = Some(index);
            }
        }
        // 搜索到末尾
        if num_of_registers > 0 {
            result.push((off_set, num_of_registers));
        }
        result.shrink_to_fit();
        result
    }

    fn create_csv_row(&self, index: usize) -> String {
        let mut ch_code = ";".to_string();
        if let Some(c) = self.coil_write_code {
            ch_code = format!("{};", c);
        }
        if let Some(h) = self.holding_write_code {
            ch_code += h.to_string().as_str();
        }
        let mut timeout_str = self.timeout_in_milli.to_string();
        if self.delay_between_requests != 0 {
            timeout_str += &format!(";{}", self.delay_between_requests);
        }
        match index {
            0 => self.mb_data_configure.len().to_string(),
            1 => self.slave_id.to_string(),
            2 => self.protocol_type.to_string().to_uppercase(),
            3 => self.max_read_register_count.to_string(),
            4 => self.max_read_bit_count.to_string(),
            5 => self.max_write_register_count.to_string(),
            6 => self.max_write_bit_count.to_string(),
            7 => self.default_polling_period_in_milli.to_string(),
            8 => timeout_str,
            9 => self.point_id.to_string(),
            10 => ch_code,
            _ => "unknown".to_string(),
        }
    }

    pub fn create_data_config(&mut self) -> Result<(), (usize, usize, String)> {
        let size = self.mb_data_configure.len();
        let mut point_id_to_rd: HashMap<u64, u16> = HashMap::with_capacity(size);
        let mut register_addr_to_rd: HashMap<u16, u16> = HashMap::with_capacity(size);
        let mut polling_period_to_data: BTreeMap<u64, Vec<u16>> = BTreeMap::new();
        polling_period_to_data.insert(self.default_polling_period_in_milli, Vec::with_capacity(size));
        // 开始统计不同轮询周期的数据，同时检查地址是否超过最大范围
        let mut tmp: HashMap<u64, u16> = HashMap::with_capacity(10);
        for rd in &self.mb_data_configure {
            if let Some(ori) = tmp.get_mut(&rd.polling_period_in_milli) {
                *ori += 1;
            } else {
                tmp.insert(rd.polling_period_in_milli, 1);
            }
        }
        for (i, num) in tmp {
            // 对于不需要采集的数据可以通过设置一个很大的轮询周期
            if i >= MAX_POLLING_PERIOD {
                continue;
            }
            let mut a_v: Vec<u16> = Vec::with_capacity(num as usize);
            for (index, rd) in self.mb_data_configure.iter().enumerate() {
                if rd.polling_period_in_milli == i {
                    a_v.push(index.try_into().unwrap());
                }
            }
            polling_period_to_data.insert(i, a_v);
        }
        for (index, rd) in self.mb_data_configure.iter().enumerate() {
            // 越界
            // if index >= u16::MAX as usize {
            //     let tip = format!("Invalid register point (id :{}):\nThe number of register points is out of range", rd.point_id);
            //     return Err((index, 1, tip));
            // }
            // 测点号重复
            if point_id_to_rd.contains_key(&rd.point_id) {
                let tip = format!("Invalid register point (id :{}):\nThe point ID is already existed", rd.point_id);
                return Err((index + 1, 8, tip));
            }
            point_id_to_rd.insert(rd.point_id, index.try_into().unwrap());
            // 起始地址重复
            if register_addr_to_rd.contains_key(&rd.from) {
                let tip = format!("Invalid register point (id :{}):\nThe register address is already existed", rd.point_id);
                return Err((index + 1, 4, tip));
            }
            register_addr_to_rd.insert(rd.from, index.try_into().unwrap());
        }
        // 判断地址之间有没有互相覆盖
        let mut last_addr = u16::MIN;
        let mut keys: Vec<&u16> = register_addr_to_rd.keys().collect();
        keys.sort(); // 按照起始地址排序
        for addr in keys {
            let index = register_addr_to_rd.get(addr).unwrap();
            let rd = self.mb_data_configure.get(*index as usize).unwrap();
            // 如果开始地址在已经被使用的地址范围
            if rd.from < last_addr {
                let tip = format!("Invalid register point (id :{}):\nThe start address is in the range of addresses that are already in use", rd.point_id);
                return Err(((index + 1) as usize, 4, tip));
            }
            last_addr = rd.from + get_register_count(rd);
        }
        self.point_id_to_rd = point_id_to_rd;
        self.register_addr_to_rd = register_addr_to_rd;
        self.polling_period_to_data = polling_period_to_data;
        Ok(())
    }
}

impl ModbusTcpClientTp {
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

    pub fn from_csv(path: &str) -> Result<ModbusTcpClientTp, (usize, usize)> {
        let content = std::fs::read(path).map_err(|_| (0, 0))?;
        // if env::IS_ENCRYPT {
        //     let plain_t = decrypt_vec(content.as_slice());
        //     ModbusTcpClientTp::from_csv_bytes(plain_t.as_slice())
        // } else {
        //     ModbusTcpClientTp::from_csv_bytes(content.as_slice())
        // }
        ModbusTcpClientTp::from_csv_bytes(content.as_slice())
    }

    pub fn from_csv_bytes(content: &[u8]) -> Result<ModbusTcpClientTp, (usize, usize)> {
        let content_new = transfer_to_utf8(content.to_vec()).map_err(|_| (0, 0))?;
        let content = content_new.as_slice();
        let tp = ModbusTcpClientTp::from_csv_records(content, 0)?;
        let rc = (2usize, 1);
        // check ip address format
        tp.tcp_server.0.parse::<std::net::Ipv4Addr>().map_err(|_| rc)?;
        Ok(tp)
    }

    fn from_csv_records(
        content: &[u8],
        offset: usize,
    ) -> Result<ModbusTcpClientTp, (usize, usize)> {
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
            return Err(rc);
        }
        // 3rd line
        let rc = (2usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let tcp_server_ip = csv_string(&record, rc.1).ok_or(rc)?;
        // 4th line
        let rc = (3usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let tcp_server_port = csv_u32(&record, rc.1).ok_or(rc)?;
        // 5th line
        let rc = (4usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let slave_id = csv_u8(&record, rc.1).ok_or(rc)?;
        // 6th line
        let rc = (5usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let type_str = csv_str(&record, rc.1).ok_or(rc)?;
        let protocol_type = MbProtocolType::from(type_str);
        // 7th line
        let rc = (6usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let s = csv_str(&record, rc.1).ok_or(rc)?;
        let max_read_register_count: u16 = if s.is_empty() {
            DEFAULT_MAX_COUNT_READ_NUMERIC
        } else {
            s.parse().map_err(|_| rc)?
        };
        // 8th line
        let rc = (7usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let s = csv_str(&record, rc.1).ok_or(rc)?;
        let max_read_bit_count: u16 = if s.is_empty() {
            DEFAULT_MAX_COUNT_READ_BIT
        } else {
            s.parse().map_err(|_| rc)?
        };
        // 9th line
        let rc = (8usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let s = csv_str(&record, rc.1).ok_or(rc)?;
        let max_write_register_count: u16 = if s.is_empty() {
            DEFAULT_MAX_COUNT_WRITE_REGISTERS
        } else {
            s.parse().map_err(|_| rc)?
        };
        // 10th line
        let rc = (9usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let s = csv_str(&record, rc.1).ok_or(rc)?;
        let max_write_bit_count: u16 = if s.is_empty() {
            DEFAULT_MAX_COUNT_WRITE_COILS
        } else {
            s.parse().map_err(|_| rc)?
        };

        // 11th line
        let rc = (10usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let s = csv_str(&record, rc.1).ok_or(rc)?;
        let default_polling_period_in_milli: u64 = if s.is_empty() {
            DEFAULT_POLLING_PERIOD_IN_MILLI
        } else {
            s.parse().map_err(|_| rc)?
        };
        // 12th line
        let rc = (11usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let s = csv_str(&record, rc.1).ok_or(rc)?;
        let (timeout_in_milli, delay_between_requests) = if s.is_empty() {
            (DEFAULT_TIMEOUT_IN_MILLI, 0u64)
        } else {
            let times: Vec<&str> = s.split(';').collect();
            if times.len() == 2 {
                (
                    times[0].parse().map_err(|_| rc)?,
                    times[1].parse().map_err(|_| rc)?,
                )
            } else if times.len() == 1 {
                (times[0].parse().map_err(|_| rc)?, 0u64)
            } else {
                (DEFAULT_TIMEOUT_IN_MILLI, 0u64)
            }
        };
        // 13th line
        let rc = (12usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let s = csv_str(&record, rc.1).ok_or(rc)?;
        let point_id: u64 = if s.is_empty() {
            UNKNOWN_POINT_ID
        } else {
            s.parse().map_err(|_| rc)?
        };
        // 14th line
        let rc = (13usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let (coil_write_code, holding_write_code) = if let Some(codes) = csv_str(&record, rc.1) {
            let codes: Vec<&str> = codes.split(';').collect();
            if codes.len() == 2 {
                let c1 = if let Ok(code) = codes[0].parse::<u8>() {
                    if code != 0x05 && code != 0x0F {
                        None
                    } else {
                        Some(code)
                    }
                } else {
                    None
                };
                let c2 = if let Ok(code) = codes[1].parse::<u8>() {
                    if code != 0x06 && code != 0x10 {
                        None
                    } else {
                        Some(code)
                    }
                } else {
                    None
                };
                (c1, c2)
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };
        // 15th line ...
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(content);
        let mut records = rdr.records();
        let rc = (0, 3 + offset);
        records.next().ok_or(rc)?.map_err(|_| rc)?;
        let mut mb_data_configure: Vec<RegisterData> = Vec::with_capacity(point_num);
        for row in 1..=point_num {
            let rc = (row, 3 + offset);
            let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
            mb_data_configure.push(RegisterData::parse_register_data(&record, rc.0, rc.1)?);
        }

        let mut conn  = MbConnection {
            slave_id,
            name: name.clone(),
            protocol_type,
            max_read_register_count,
            max_read_bit_count,
            max_write_register_count,
            max_write_bit_count,
            timeout_in_milli,
            delay_between_requests,
            point_id,
            default_polling_period_in_milli,
            mb_data_configure,
            coil_write_code,
            holding_write_code,
            ..Default::default()
        };

        conn.create_data_config().map_err(|(r, c, _)|(r, c + offset))?;

        Ok(ModbusTcpClientTp {
            id: 0,
            name: name.clone(),
            tcp_server: (tcp_server_ip, tcp_server_port),
            connections: vec![conn],
        })
    }

    pub fn from_file2(path: &str) -> Result<Self, (usize, usize)> {
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
        Self::from_csv_bytes2(csv_bytes.as_slice())
    }

    pub fn from_csv2(path: &str) -> Result<ModbusTcpClientTp, (usize, usize)> {
        let content = std::fs::read(path).map_err(|_| (0, 0))?;
        // if env::IS_ENCRYPT {
        //     let plain_t = decrypt_vec(content.as_slice());
        //     ModbusTcpClientTp::from_csv_bytes2(plain_t.as_slice())
        // } else {
        //     ModbusTcpClientTp::from_csv_bytes2(content.as_slice())
        // }
        ModbusTcpClientTp::from_csv_bytes2(content.as_slice())
    }

    pub fn from_csv_bytes2(content: &[u8]) -> Result<ModbusTcpClientTp, (usize, usize)> {
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
        if conn_num == 0 {
            // 连接数不能为0
            return Err(rc);
        }
        let rc = (2usize, 1);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let tcp_server_ip = csv_string(&record, rc.1).ok_or(rc)?;
        tcp_server_ip
            .parse::<std::net::Ipv4Addr>()
            .map_err(|_| rc)?;
        let rc = (3usize, 1);
        let tcp_server_port: u32 =
            csv_u32(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        let mut connections: Vec<MbConnection> = Vec::with_capacity(conn_num);
        for i in 0..conn_num {
            let connection = ModbusTcpClientTp::from_csv_records2(content, i * 10 + 3)?;
            // 检查具有相同ip的client是否配置一样
            for conn in &connections {
                // 这里允许slave id一样，解决不同功能码地址重复的问题，可以通过设不同的连接来解决
                if conn.slave_id == connection.slave_id && conn.point_id != connection.point_id {
                    return Err((12, i * 10 + 4));
                }
                // 协议类型(xa或encap)必须一样
                if conn.protocol_type != connection.protocol_type {
                    return Err((3, i * 10 + 4));
                }
                // 测点不能一样
                let mut row = 1;
                for rd in &connection.mb_data_configure {
                    if conn.point_id_to_rd.contains_key(&rd.point_id) {
                        return Err((row, i * 10 + 11));
                    }
                    row += 1;
                }
            }
            connections.push(connection);
        }
        Ok(ModbusTcpClientTp {
            id: 0,
            name,
            tcp_server: (tcp_server_ip, tcp_server_port),
            connections,
        })
    }

    fn from_csv_records2(
        content: &[u8],
        offset: usize,
    ) -> Result<MbConnection, (usize, usize)> {
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
        if point_num > u16::MAX as usize {
            return Err(rc);
        }
        // 3th line
        let rc = (2usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let slave_id = csv_u8(&record, rc.1).ok_or(rc)?;
        // 4th line
        let rc = (3usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let type_str = csv_str(&record, rc.1).ok_or(rc)?;
        let protocol_type = MbProtocolType::from(type_str);
        // 5th line
        let rc = (4usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let s = csv_str(&record, rc.1).ok_or(rc)?;
        let max_read_register_count: u16 = if s.is_empty() {
            DEFAULT_MAX_COUNT_READ_NUMERIC
        } else {
            s.parse().map_err(|_| rc)?
        };
        // 6th line
        let rc = (5usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let s = csv_str(&record, rc.1).ok_or(rc)?;
        let max_read_bit_count: u16 = if s.is_empty() {
            DEFAULT_MAX_COUNT_READ_BIT
        } else {
            s.parse().map_err(|_| rc)?
        };
        // 7th line
        let rc = (6usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let s = csv_str(&record, rc.1).ok_or(rc)?;
        let max_write_register_count: u16 = if s.is_empty() {
            DEFAULT_MAX_COUNT_WRITE_REGISTERS
        } else {
            s.parse().map_err(|_| rc)?
        };
        // 8th line
        let rc = (7usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let s = csv_str(&record, rc.1).ok_or(rc)?;
        let max_write_bit_count: u16 = if s.is_empty() {
            DEFAULT_MAX_COUNT_WRITE_COILS
        } else {
            s.parse().map_err(|_| rc)?
        };
        // 9th line
        let rc = (8usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let s = csv_str(&record, rc.1).ok_or(rc)?;
        let default_polling_period_in_milli: u64 = if s.is_empty() {
            DEFAULT_POLLING_PERIOD_IN_MILLI
        } else {
            s.parse().map_err(|_| rc)?
        };
        // 10th line
        let rc = (9usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let s = csv_str(&record, rc.1).ok_or(rc)?;
        let (timeout_in_milli, delay_between_requests) = if s.is_empty() {
            (DEFAULT_TIMEOUT_IN_MILLI, 0u64)
        } else {
            let times: Vec<&str> = s.split(';').collect();
            if times.len() == 2 {
                (
                    times[0].parse().map_err(|_| rc)?,
                    times[1].parse().map_err(|_| rc)?,
                )
            } else if times.len() == 1 {
                (times[0].parse().map_err(|_| rc)?, 0u64)
            } else {
                (DEFAULT_TIMEOUT_IN_MILLI, 0u64)
            }
        };
        // 11th line
        let rc = (10usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let s = csv_str(&record, rc.1).ok_or(rc)?;
        let point_id: u64 = if s.is_empty() {
            UNKNOWN_POINT_ID
        } else {
            s.parse().map_err(|_| rc)?
        };
        // 12th line
        let rc = (11usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let (coil_write_code, holding_write_code) = if let Some(codes) = csv_str(&record, rc.1) {
            let codes: Vec<&str> = codes.split(';').collect();
            if codes.len() == 2 {
                let c1 = if let Ok(code) = codes[0].parse::<u8>() {
                    if code != 0x05 && code != 0x0F {
                        None
                    } else {
                        Some(code)
                    }
                } else {
                    None
                };
                let c2 = if let Ok(code) = codes[1].parse::<u8>() {
                    if code != 0x06 && code != 0x10 {
                        None
                    } else {
                        Some(code)
                    }
                } else {
                    None
                };
                (c1, c2)
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };
        // 13th line ...
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(content);
        let mut records = rdr.records();
        let rc = (0, 3 + offset);
        records.next().ok_or(rc)?.map_err(|_| rc)?;
        let mut mb_data_configure: Vec<RegisterData> = Vec::with_capacity(point_num);
        for row in 1..=point_num {
            let rc = (row, 3 + offset);
            let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
            mb_data_configure.push(RegisterData::parse_register_data(&record, rc.0, rc.1)?);
        }

        let mut conn = MbConnection {
            slave_id,
            name,
            protocol_type,
            max_read_register_count,
            max_read_bit_count,
            max_write_register_count,
            max_write_bit_count,
            timeout_in_milli,
            delay_between_requests,
            point_id,
            default_polling_period_in_milli,
            mb_data_configure,
            coil_write_code,
            holding_write_code,
            ..Default::default()
        };

        conn.create_data_config().map_err(|(r, c, _)|(r, c + offset))?;

        Ok(conn)
    }

    pub fn get_point_ids(&self) -> Vec<u64> {
        let mut size = 0;
        for conn in &self.connections {
            size += conn.mb_data_configure.len();
            size += 1;
        }
        let mut r: Vec<u64> = Vec::with_capacity(size);
        for conn in &self.connections {
            for rd in &conn.mb_data_configure {
                r.push(rd.point_id)
            }
            if conn.point_id != UNKNOWN_POINT_ID {
                r.push(conn.point_id);
            }
        }
        r
    }

    // 导出 Modbus Tcp客户端 文件内容
    pub fn export_csv(&self, text_map: &HashMap<String, String>) -> String {
        if self.connections.len() == 1 {
            // 第一排
            let mut result = format!("{},{},,",
                                     text_map.get("tp_name").unwrap_or(&"Transport Name".to_string()),
                                     get_csv_str(&self.name));
            result += &format!(
                "{},{},{},{},{},{},{},{},{},\n",
                text_map.get("conn_name").unwrap_or(&"Connection Name".to_string()),
                get_csv_str(&self.connections[0].name),
                text_map.get("index").unwrap_or(&"Index".to_string()),
                text_map.get("register_type").unwrap_or(&"Register Type".to_string()),
                text_map.get("start_addr").unwrap_or(&"Start Address".to_string()),
                text_map.get("data_type").unwrap_or(&"Data Type".to_string()),
                text_map.get("new_request_flag").unwrap_or(&"New Request".to_string()),
                text_map.get("register_period_name").unwrap_or(&"Polling Period(ms)".to_string()),
                text_map.get("tp_point_id").unwrap_or(&"Point ID".to_string()),
            );

            // 第二至十二排
            let title_conn = vec![
                text_map.get("point_number").unwrap_or(&"Point Count".to_string()).clone(),
                "Slave ID".to_string(),
                text_map.get("protocol").unwrap_or(&"Protocol".to_string()).clone(),
                text_map.get("max_rrc").unwrap_or(&"Max Read Register Count".to_string()).clone(),
                text_map.get("max_rbc").unwrap_or(&"Max Read Bit Count".to_string()).clone(),
                text_map.get("max_wrc").unwrap_or(&"Max Write Register Count".to_string()).clone(),
                text_map.get("max_wbc").unwrap_or(&"Max Write Bit Count".to_string()).clone(),
                text_map.get("register_period_name").unwrap_or(&"Polling Period(ms)".to_string()).clone(),
                text_map.get("timeout_delay_ms").unwrap_or(&"Timeout;Delay(Optional)(ms)".to_string()).clone(),
                text_map.get("status_point_id").unwrap_or(&"Status Point ID".to_string()).clone(),
                text_map.get("coil_holding_code").unwrap_or(&"Coil/Holding Code".to_string()).clone(),
            ];

            let title_tp = vec![
                format!(
                    "{},{},",
                    text_map.get("conn_num").unwrap_or(&"Connection Count".to_string()),
                    self.connections.len()
                ),
                format!(
                    "{},{},",
                    text_map.get("server_ip").unwrap_or(&"Server IP".to_string()),
                    self.tcp_server.0
                ),
                format!(
                    "{},{},",
                    text_map.get("server_port").unwrap_or(&"Server Port".to_string()),
                    self.tcp_server.1
                ),
                ",,".to_string(),
                ",,".to_string(),
                ",,".to_string(),
                ",,".to_string(),
                ",,".to_string(),
                ",,".to_string(),
                ",,".to_string(),
                ",,".to_string(),
            ];

            let conn_len = if self.connections.is_empty() {
                0
            } else {
                self.connections[0].mb_data_configure.len()
            };
            for cnt in 0..11 {
                result += &title_tp[cnt];
                if conn_len > cnt {
                    let r = &self.connections[0].mb_data_configure[cnt];
                    let content_conn = self.connections[0].create_csv_row(cnt);
                    result += &format!(
                        ",{},{},{},{},{},{},{},{},{},",
                        title_conn[cnt],
                        content_conn,
                        cnt + 1,
                        r.register_type,
                        r.from,
                        r.data_type,
                        r.should_new_request.to_string().to_uppercase(),
                        r.polling_period_in_milli,
                        r.point_id
                    );
                } else {
                    let content_conn = self.connections[0].create_csv_row(cnt);
                    result += &format!(",{},{},,,,,,,,", title_conn[cnt], content_conn);
                }
                result += "\n";
            }

            // 剩余的
            for row in 11..conn_len {
                result += ",,,";
                if conn_len > row {
                    let r = &self.connections[0].mb_data_configure[row];
                    result += &format!(
                        ",,{},{},{},{},{},{},{},",
                        row + 1,
                        r.register_type,
                        r.from,
                        r.data_type,
                        r.should_new_request.to_string().to_uppercase(),
                        r.polling_period_in_milli,
                        r.point_id
                    );
                } else {
                    result += ",,,,,,,,,,,";
                }
                result += "\n";
            }
            result
        } else {
            let len_conn = self.connections.len();
            // 第一排
            let mut result = format!("{},{},,",
                                     text_map.get("tp_name").unwrap_or(&"Transport Name".to_string()),
                                     get_csv_str(&self.name));

            for i in 0..len_conn {
                result += &format!(
                    "{},{},{},{},{},{},{},{},{}",
                    text_map.get("conn_name").unwrap_or(&"Connection Name".to_string()),
                    get_csv_str(&self.connections[i].name),
                    text_map.get("index").unwrap_or(&"Index".to_string()),
                    text_map.get("register_type").unwrap_or(&"Register Type".to_string()),
                    text_map.get("start_addr").unwrap_or(&"Start Address".to_string()),
                    text_map.get("data_type").unwrap_or(&"Data Type".to_string()),
                    text_map.get("new_request_flag").unwrap_or(&"New Request".to_string()),
                    text_map.get("register_period_name").unwrap_or(&"Polling Period(ms)".to_string()),
                    text_map.get("tp_point_id").unwrap_or(&"Point ID".to_string()),
                );
                if i != len_conn - 1 {
                    result += ",,";
                }
            }
            result += "\n";

            // 第二至十二排
            let title_conn = vec![
                text_map.get("point_number").unwrap_or(&"Point Count".to_string()).clone(),
                "Slave ID".to_string(),
                text_map.get("protocol").unwrap_or(&"Protocol".to_string()).clone(),
                text_map.get("max_rrc").unwrap_or(&"Max Read Register Count".to_string()).clone(),
                text_map.get("max_rbc").unwrap_or(&"Max Read Bit Count".to_string()).clone(),
                text_map.get("max_wrc").unwrap_or(&"Max Write Register Count".to_string()).clone(),
                text_map.get("max_wbc").unwrap_or(&"Max Write Bit Count".to_string()).clone(),
                text_map.get("register_period_name").unwrap_or(&" Polling Period(ms)".to_string()).clone(),
                text_map.get("timeout_delay_ms").unwrap_or(&"Timeout;Delay(Optional)(ms)".to_string()).clone(),
                text_map.get("status_point_id").unwrap_or(&"Status Point ID".to_string()).clone(),
                text_map.get("coil_holding_code").unwrap_or(&"Coil/Holding Code".to_string()).clone(),
            ];

            let title_tp = vec![
                format!(
                    "{},{},",
                    text_map.get("conn_num").unwrap_or(&"Connection Count".to_string()),
                    self.connections.len()
                ),
                format!(
                    "{},{},",
                    text_map.get("server_ip").unwrap_or(&"Server IP".to_string()),
                    self.tcp_server.0
                ),
                format!(
                    "{},{},",
                    text_map.get("server_port").unwrap_or(&"Server Port".to_string()),
                    self.tcp_server.1
                ),
                ",,".to_string(),
                ",,".to_string(),
                ",,".to_string(),
                ",,".to_string(),
                ",,".to_string(),
                ",,".to_string(),
                ",,".to_string(),
                ",,".to_string(),
            ];

            for cnt in 0..11 {
                result += &title_tp[cnt];
                for i in 0..len_conn {
                    if self.connections[i].mb_data_configure.len() > cnt {
                        let r = &self.connections[i].mb_data_configure[cnt];
                        let content_conn = self.connections[i].create_csv_row(cnt);
                        result += &format!(
                            ",{},{},{},{},{},{},{},{},{}",
                            title_conn[cnt],
                            content_conn,
                            cnt + 1,
                            r.register_type,
                            r.from,
                            r.data_type,
                            r.should_new_request.to_string().to_uppercase(),
                            r.polling_period_in_milli,
                            r.point_id
                        );
                        if i != len_conn - 1 {
                            result += ",";
                        }
                    } else {
                        let content_conn = self.connections[i].create_csv_row(cnt);
                        result += &format!(",{},{},,,,,,,,", title_conn[cnt], content_conn);
                    }
                }
                result += "\n";
            }

            // 剩余的
            let mut max_data_len = if self.connections.is_empty() {
                0
            } else {
                self.connections[0].mb_data_configure.len()
            };
            for c in &self.connections {
                if c.mb_data_configure.len() > max_data_len {
                    max_data_len = c.mb_data_configure.len();
                }
            }
            for row in 11..max_data_len {
                result += ",,";
                for i in 0..len_conn {
                    if self.connections[i].mb_data_configure.len() > row {
                        let r = &self.connections[i].mb_data_configure[row];
                        result += &format!(
                            ",,{},{},{},{},{},{},{}",
                            row + 1,
                            r.register_type,
                            r.from,
                            r.data_type,
                            r.should_new_request.to_string().to_uppercase(),
                            r.polling_period_in_milli,
                            r.point_id
                        );
                        if i != len_conn - 1 {
                            result += ",";
                        }
                    } else {
                        result += ",,,,,,,,,,";
                        if i != len_conn - 1 {
                            result += ",";
                        }
                    }
                }
                result += "\n";
            }
            result
        }
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
        let s = csv_str(record, rc.1).ok_or(rc)?;
        let register_type = RegisterType::from_str(s).map_err(|_|rc)?;
        let rc = (row, first_col + 1);
        let from: u16 = csv_u16(record, rc.1).ok_or(rc)?;
        let rc = (row, first_col + 2);
        let s = csv_str(record, rc.1).ok_or(rc)?;
        let data_type = DataType::from_str(s).map_err(|_| rc)?;
        // 这里判断: coils和discrete只能是binary,input和holding不能是binary
        match register_type {
            RegisterType::COILS => {
                if !matches!(data_type, DataType::Binary) {
                    return Err(rc);
                }
            }
            RegisterType::DISCRETE => {
                if !matches!(data_type, DataType::Binary) {
                    return Err(rc);
                }
            }
            RegisterType::INPUT => {
                if matches!(data_type, DataType::Binary) {
                    return Err(rc);
                }
            }
            RegisterType::HOLDING => {
                if matches!(data_type, DataType::Binary) {
                    return Err(rc);
                }
            }
        }
        // 是否必须新开一个请求
        let rc = (row, first_col + 3);
        let s = csv_str(record, rc.1).ok_or(rc)?.to_uppercase();
        let should_new_request = match s.as_str() {
            "FALSE" => false,
            "TRUE" => true,
            _ => false,
        };
        // 轮询周期
        let rc = (row, first_col + 4);
        let polling_period_in_milli = csv_u64(record, rc.1).ok_or(rc)?;
        // 对应的测点Id
        let rc = (row, first_col + 5);
        let point_id = csv_u64(record, rc.1).ok_or(rc)?;
        Ok(RegisterData {
            register_type,
            from,
            data_type,
            should_new_request,
            polling_period_in_milli,
            point_id,
        })
    }
}



impl ModbusTcpServerTp {
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

    pub fn from_csv(path: &str) -> Result<ModbusTcpServerTp, (usize, usize)> {
        let content = std::fs::read(path).map_err(|_| (0, 0))?;
        // let content = if env::IS_ENCRYPT {
        //     let plain_t = decrypt_vec(content.as_slice());
        //     plain_t
        // } else {
        //     content
        // };
        ModbusTcpServerTp::from_csv_bytes(content.as_slice())
    }

    pub fn from_csv_bytes(content: &[u8]) -> Result<ModbusTcpServerTp, (usize, usize)> {
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
        let tcp_server_port: u16 =
            csv_u16(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        let mut connections: Vec<(String, MbConnection)> = Vec::with_capacity(conn_num);
        for i in 0..conn_num {
            let mut tp = ModbusTcpClientTp::from_csv_records(content, i * 10 + 3)?;
            let (client_ip, client_port) = tp.tcp_server;
            if client_ip != "+" {
                // 如果不是通配符
                let rc = (2usize, i * 10 + 4);
                client_ip.parse::<std::net::Ipv4Addr>().map_err(|_| rc)?;
            }
            // 检查具有相同ip的client是否配置一样
            for (key, conn) in &connections {
                // 这里连接名字和通道名称是一样的
                if *key == format!("{}/{}/{}", client_ip, client_port, conn.name) {
                    // 地址配置必须一样
                    if *conn.mb_data_configure != tp.connections[0].mb_data_configure {
                        return Err((0, i * 10 + 3));
                    }
                }
            }
            let port_str = client_port.to_string();
            let port = if client_port == UNKNOWN_TCP_PORT {
                "+"
            } else {
                port_str.as_str()
            };

            // 利用point_id简化多个client配置，测点号1-100预留，不允许设置
            if tp.connections[0].point_id > 1
                && tp.connections[0].point_id <= DEFAULT_TCP_CLIENT_LIMIT as u64 {
                let count = tp.connections[0].point_id;
                for i in 1..count {
                    let key = format!("{}/{}/{}@{}", client_ip, port, tp.name, i);
                    let mut connection = tp.connections[0].clone();
                    connection.point_id = UNKNOWN_POINT_ID; // 多个通道共用一个配置，状态点号无需设置
                    connections.push((key, connection));
                }
                let key = format!("{}/{}/{}@{}", client_ip, port, tp.name, count);
                tp.connections[0].point_id = UNKNOWN_POINT_ID;
                connections.push((key, tp.connections.pop().unwrap()))
            } else {
                let key: String = format!("{}/{}/{}", client_ip, port, tp.name);
                connections.push((key, tp.connections.pop().unwrap()))
            }
        }

        Ok(ModbusTcpServerTp {
            id: 0,
            name,
            tcp_server_port,
            connections,
        })
    }

    pub fn get_point_ids(&self) -> Vec<u64> {
        let mut size = 0;

        for (_, conn) in &self.connections {
            size += conn.mb_data_configure.len()
        }
        size += self.connections.len();
        let mut r = HashSet::with_capacity(size);
        for (_, conn) in &self.connections {
            if conn.point_id != UNKNOWN_POINT_ID {
                r.insert(conn.point_id);
            }
            for rd in &conn.mb_data_configure {
                r.insert(rd.point_id);
            }
        }
        r.into_iter().collect()
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
                "{},{},{},{},{},{},{},{},{}",
                text_map.get("conn_name").unwrap_or(&"Connection Name".to_string()),
                get_csv_str(&conn.name),
                text_map.get("index").unwrap_or(&"Index".to_string()),
                text_map.get("register_type").unwrap_or(&"Register Type".to_string()),
                text_map.get("start_addr").unwrap_or(&"Start Address".to_string()),
                text_map.get("data_type").unwrap_or(&"Data Type".to_string()),
                text_map.get("new_request_flag").unwrap_or(&"New Request".to_string()),
                text_map.get("register_period_name").unwrap_or(&"Polling Period(ms)".to_string()),
                text_map.get("tp_point_id").unwrap_or(&"Point ID".to_string()),
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

        // 第二至十三排
        let title_conn = vec![
            text_map.get("point_number").unwrap_or(&"Point Count".to_string()).clone(),
            text_map.get("client_ip").unwrap_or(&"Client IP".to_string()).clone(),
            text_map.get("client_port").unwrap_or(&"Client Port".to_string()).clone(),
            "Slave ID".to_string(),
            text_map.get("protocol_type").unwrap_or(&"Protocol Type".to_string()).clone(),
            text_map.get("max_rrc").unwrap_or(&"Max Read Register Count".to_string()).clone(),
            text_map.get("max_rbc").unwrap_or(&"Max Read Bit Count".to_string()).clone(),
            text_map.get("max_wrc").unwrap_or(&"Max Write Register Count".to_string()).clone(),
            text_map.get("max_wbc").unwrap_or(&"Max Write Bit Count".to_string()).clone(),
            text_map.get("register_period_name").unwrap_or(&"Polling Period(ms)".to_string()).clone(),
            text_map.get("timeout_ms").unwrap_or(&"Timeout(ms)".to_string()).clone(),
            text_map.get("tp_point_id").unwrap_or(&"Point ID".to_string()).clone(),
        ];
        let title_tp = vec![
            format!("{},{},", text_map.get("conn_num").unwrap_or(&"Connection Count".to_string()), len_conn),
            format!(
                "{},{},",
                text_map.get("server_port").unwrap_or(&"Server Port".to_string()),
                self.tcp_server_port
            ),
            ",,".to_string(),
            ",,".to_string(),
            ",,".to_string(),
            ",,".to_string(),
            ",,".to_string(),
            ",,".to_string(),
            ",,".to_string(),
            ",,".to_string(),
            ",,".to_string(),
            ",,".to_string(),
        ];

        for cnt in 0..12 {
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
                if conn.1.mb_data_configure.len() > cnt {
                    let r = &conn.1.mb_data_configure[cnt];
                    let content_conn = if cnt == 11 && conn.0.starts_with("+/") {
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
                        Self::get_mbd_conn_csv(conn, cnt)
                    };
                    result += &format!(
                        ",{},{},{},{},{},{},{},{},{}",
                        title_conn[cnt],
                        content_conn,
                        cnt + 1,
                        r.register_type,
                        r.from,
                        r.data_type,
                        r.should_new_request.to_string().to_uppercase(),
                        r.polling_period_in_milli,
                        r.point_id
                    );
                } else {
                    let content_conn = if cnt == 11 && conn.0.starts_with("+/") {
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
                        Self::get_mbd_conn_csv(conn, cnt)
                    };
                    result += &format!(",{},{},,,,,,,", title_conn[cnt], content_conn);
                }
                if i != len_conn {
                    result += ",";
                } else {
                    break;
                }
            }
            result += "\n";
        }

        // 剩余的
        let mut max_data_len = if self.connections.is_empty() {
            0
        } else {
            self.connections[0].1.mb_data_configure.len()
        };
        for c in &self.connections {
            if c.1.mb_data_configure.len() > max_data_len {
                max_data_len = c.1.mb_data_configure.len();
            }
        }
        if max_data_len < 12 {
            result += ",,,,,,,,,,,";
        }
        for row in 12..max_data_len {
            result += ",,,,,";
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
                if conn.mb_data_configure.len() > row {
                    let r = &conn.mb_data_configure[row];
                    result += &format!(
                        "{},{},{},{},{},{},{}",
                        row + 1,
                        r.register_type,
                        r.from,
                        r.data_type,
                        r.should_new_request.to_string().to_uppercase(),
                        r.polling_period_in_milli,
                        r.point_id
                    );
                } else {
                    result += ",,,,,,,,,";
                }
                if i != len_conn {
                    result += ",";
                } else {
                    break;
                }
            }
            result += "\n";
        }
        result
    }

    fn get_mbd_conn_csv(conn: &(String, MbConnection), index: usize) -> String {
        return match index {
            0 => conn.1.mb_data_configure.len().to_string(),
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
            3 => conn.1.slave_id.to_string(),
            4 => conn.1.protocol_type.to_string(),
            5 => conn.1.max_read_register_count.to_string(),
            6 => conn.1.max_read_bit_count.to_string(),
            7 => conn.1.max_write_register_count.to_string(),
            8 => conn.1.max_write_bit_count.to_string(),
            9 => conn.1.default_polling_period_in_milli.to_string(),
            10 => conn.1.timeout_in_milli.to_string(),
            11 => conn.1.point_id.to_string(),
            _ => "".to_string(),
        };
    }
}

impl ModbusRtuClientTp {
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

    pub fn from_csv(path: &str) -> Result<ModbusRtuClientTp, (usize, usize)> {
        let content = std::fs::read(path).map_err(|_| (0, 0))?;
        // let content = if env::IS_ENCRYPT {
        //     let plain_t = decrypt_vec(content.as_slice());
        //     plain_t
        // } else {
        //     content
        // };
        ModbusRtuClientTp::from_csv_bytes(content.as_slice())
    }

    pub fn from_csv_bytes(content: &[u8]) -> Result<ModbusRtuClientTp, (usize, usize)> {
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
        let mut connections: Vec<MbConnection> = Vec::with_capacity(conn_num);
        for i in 0..conn_num {
            let mut tp = ModbusTcpClientTp::from_csv_records(content, i * 10 + 3)?;
            connections.push(tp.connections.pop().unwrap());
        }
        let para: SerialPara = SerialPara {
            file_path,
            baud_rate,
            data_bits,
            stop_bits,
            parity,
            delay_between_requests,
        };
        Ok(ModbusRtuClientTp {
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
                "{},{},{},{},{},{},{},{},{}",
                text_map.get("conn_name").unwrap_or(&"Connection Name".to_string()),
                get_csv_str(&self.connections[i].name),
                text_map.get("index").unwrap_or(&"Index".to_string()),
                text_map.get("register_type").unwrap_or(&"Register Type".to_string()),
                text_map.get("start_addr").unwrap_or(&"Start Address".to_string()),
                text_map.get("data_type").unwrap_or(&"Data Type".to_string()),
                text_map.get("new_request_flag").unwrap_or(&"New Request".to_string()),
                text_map.get("register_period_name").unwrap_or(&"Polling Period(ms)".to_string()),
                text_map.get("tp_point_id").unwrap_or(&"Point ID".to_string()),
            );
            if i != len_conn - 1 {
                result += ",,";
            }
        }
        result += "\n";

        // 第二至十四排
        let title_conn = vec![
            text_map.get("point_number").unwrap_or(&"Point Count".to_string()).clone(),
            text_map.get("desc").unwrap_or(&"Description".to_string()).clone(),
            text_map.get("priority").unwrap_or(&"Priority".to_string()).clone(),
            "Slave ID".to_string(),
            text_map.get("protocol_type").unwrap_or(&"Protocol Type".to_string()).clone(),
            text_map.get("max_rrc").unwrap_or(&"Max Read Register Count".to_string()).clone(),
            text_map.get("max_rbc").unwrap_or(&"Max Read Bit Count".to_string()).clone(),
            text_map.get("max_wrc").unwrap_or(&"Max Write Register Count".to_string()).clone(),
            text_map.get("max_wbc").unwrap_or(&"Max Write Bit Count".to_string()).clone(),
            text_map.get("register_period_name").unwrap_or(&"Polling Period(ms)".to_string()).clone(),
            text_map.get("timeout_ms").unwrap_or(&"Timeout(ms)".to_string()).clone(),
            text_map.get("tp_point_id").unwrap_or(&"Point ID".to_string()).clone(),
            text_map.get("coil_holding_code").unwrap_or(&"Coil/Holding Code".to_string()).clone(),
        ];
        let title_tp = vec![
            format!(
                "{},{},",
                text_map.get("conn_num").unwrap_or(&"Connection Count".to_string()),
                self.connections.len()
            ),
            format!(
                "{},{},",
                text_map.get("baud_rate").unwrap_or(&"Baud Rate".to_string()),
                self.para.baud_rate
            ),
            format!(
                "{},{},",
                text_map.get("file_path").unwrap_or(&"File Path".to_string()),
                self.para.file_path
            ),
            format!(
                "{},{},",
                text_map.get("data_bits_op").unwrap_or(&"Data Bits".to_string()),
                self.para.data_bits
            ),
            format!(
                "{},{},",
                text_map.get("stop_bits_op").unwrap_or(&"Stop Bits".to_string()),
                self.para.stop_bits
            ),
            format!(
                "{},{:?},",
                text_map.get("parity_op").unwrap_or(&"Parity".to_string()),
                self.para.parity
            )
                .to_uppercase(),
            format!(
                "{},{},",
                text_map.get("delay_time_op").unwrap_or(&"Delay Time(ms)".to_string()),
                self.para.delay_between_requests
            ),
            ",,".to_string(),
            ",,".to_string(),
            ",,".to_string(),
            ",,".to_string(),
            ",,".to_string(),
            ",,".to_string(),
        ];

        for cnt in 0..13 {
            result += &title_tp[cnt];
            for i in 0..len_conn {
                if self.connections[i].mb_data_configure.len() > cnt {
                    let r = &self.connections[i].mb_data_configure[cnt];
                    let content_conn = Self::get_rtu_mbc_conn_csv(&self.connections[i], cnt);
                    result += &format!(
                        ",{},{},{},{},{},{},{},{},{}",
                        title_conn[cnt],
                        content_conn,
                        cnt + 1,
                        r.register_type,
                        r.from,
                        r.data_type,
                        r.should_new_request.to_string().to_uppercase(),
                        r.polling_period_in_milli,
                        r.point_id
                    );
                    if i != len_conn - 1 {
                        result += ",";
                    }
                } else {
                    let content_conn = Self::get_rtu_mbc_conn_csv(&self.connections[i], cnt);
                    result += &format!(",{},{},,,,,,,", title_conn[cnt], content_conn);
                    if i != len_conn - 1 {
                        result += ",";
                    }
                }
            }
            result += "\n";
        }

        // 剩余的
        let mut max_data_len = if self.connections.is_empty() {
            0
        } else {
            self.connections[0].mb_data_configure.len()
        };
        for c in &self.connections {
            if c.mb_data_configure.len() > max_data_len {
                max_data_len = c.mb_data_configure.len();
            }
        }
        if max_data_len < 13 {
            result += ",,,,,,,,,,,";
        }
        for row in 13..max_data_len {
            //如果Data Type输出完了但测点寄存器还有
            result += ",,,,";
            for i in 0..len_conn {
                if self.connections[i].mb_data_configure.len() > row {
                    let r = &self.connections[i].mb_data_configure[row];
                    result += &format!(
                        ",{},{},{},{},{},{},{}",
                        row + 1,
                        r.register_type,
                        r.from,
                        r.data_type,
                        r.should_new_request.to_string().to_uppercase(),
                        r.polling_period_in_milli,
                        r.point_id
                    );
                    if i != len_conn - 1 {
                        result += ",";
                    }
                } else {
                    result += ",,,,,,,,,";
                    if i != len_conn - 1 {
                        result += ",";
                    }
                }
            }
            result += "\n";
        }
        result
    }

    fn get_rtu_mbc_conn_csv(conn: &MbConnection, index: usize) -> String {
        let mut ch_code = ";".to_string();
        if let Some(c) = conn.coil_write_code {
            ch_code = format!("{};", c);
        }
        if let Some(h) = conn.holding_write_code {
            ch_code += h.to_string().as_str();
        }
        match index {
            0 => conn.mb_data_configure.len().to_string(),
            1 => "描述".to_string(),
            2 => "1".to_string(),
            3 => conn.slave_id.to_string(),
            4 => conn.protocol_type.to_string().to_uppercase(),
            5 => conn.max_read_register_count.to_string(),
            6 => conn.max_read_bit_count.to_string(),
            7 => conn.max_write_register_count.to_string(),
            8 => conn.max_write_bit_count.to_string(),
            9 => conn.default_polling_period_in_milli.to_string(),
            10 => conn.timeout_in_milli.to_string(),
            11 => conn.point_id.to_string(),
            12 => ch_code,
            _ => "unknown".to_string(),
        }
    }

    pub fn get_point_ids(&self) -> Vec<u64> {
        let mut size = 0;
        for conn in &self.connections {
            size += conn.mb_data_configure.len()
        }
        size += self.connections.len();
        let mut r: Vec<u64> = Vec::with_capacity(size);
        for conn in &self.connections {
            for rd in &conn.mb_data_configure {
                r.push(rd.point_id)
            }
            if conn.point_id != UNKNOWN_POINT_ID {
                r.push(conn.point_id);
            }
        }
        r
    }
}

impl ModbusRtuServerTp {
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

    pub fn from_csv(path: &str) -> Result<ModbusRtuServerTp, (usize, usize)> {
        let content = std::fs::read(path).map_err(|_| (0, 0))?;
        // let content = if env::IS_ENCRYPT {
        //     let plain_t = decrypt_vec(content.as_slice());
        //     plain_t
        // } else {
        //     content
        // };
        ModbusRtuServerTp::from_csv_bytes(content.as_slice())
    }

    pub fn from_csv_bytes(content: &[u8]) -> Result<ModbusRtuServerTp, (usize, usize)> {
        let content_new = transfer_to_utf8(content.to_vec()).map_err(|_| (0, 0))?;
        let content = content_new.as_slice();
        let mut tp = ModbusTcpClientTp::from_csv_records(content, 0)?;
        // 获取串口参数
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(content);
        let mut records = rdr.records().skip(13);
        // 第14行
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
        // 第15行
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
        // 第16行
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

        let para: SerialPara = SerialPara {
            file_path: tp.tcp_server.0,
            baud_rate: tp.tcp_server.1,
            data_bits,
            stop_bits,
            parity,
            delay_between_requests: 0,
        };
        Ok(ModbusRtuServerTp {
            id: 0,
            name: tp.name,
            para,
            connection: tp.connections.pop().unwrap(),
        })
    }

    pub fn export_csv(&self, text_map: &HashMap<String, String>) -> String {
        let title = vec![
            text_map.get("point_number").unwrap_or(&"Point Count".to_string()).clone(),
            text_map.get("file_path").unwrap_or(&"File Path".to_string()).clone(),
            text_map.get("baud_rate").unwrap_or(&"Baud Rate".to_string()).clone(),
            "Slave ID".to_string(),
            text_map.get("protocol_type").unwrap_or(&"Protocol Type".to_string()).clone(),
            text_map.get("max_rrc").unwrap_or(&"Max Read Register Count".to_string()).clone(),
            text_map.get("max_rbc").unwrap_or(&"Max Read Bit Count".to_string()).clone(),
            text_map.get("max_wrc").unwrap_or(&"Max Write Register Count".to_string()).clone(),
            text_map.get("max_wbc").unwrap_or(&"Max Write Bit Count".to_string()).clone(),
            text_map.get("register_period_name").unwrap_or(&"Polling Period(ms)".to_string()).clone(),
            text_map.get("timeout_ms").unwrap_or(&"Timeout(ms)".to_string()).clone(),
            text_map.get("tp_point_id").unwrap_or(&"Point ID".to_string()).clone(),
            text_map.get("data_bits_op").unwrap_or(&"Data Bits".to_string()).clone(),
            text_map.get("stop_bits_op").unwrap_or(&"Stop Bits".to_string()).clone(),
            text_map.get("parity_op").unwrap_or(&"Parity".to_string()).clone(),
        ];
        let content = vec![
            format!("{}", self.connection.mb_data_configure.len()),
            format!("{}", self.para.file_path),
            format!("{}", self.para.baud_rate),
            format!("{}", self.connection.slave_id),
            "RTU".to_string(),
            format!("{}", self.connection.max_read_register_count),
            format!("{}", self.connection.max_read_bit_count),
            format!("{}", self.connection.max_write_register_count),
            format!("{}", self.connection.max_write_bit_count),
            format!("{}", self.connection.default_polling_period_in_milli),
            format!("{}", self.connection.timeout_in_milli),
            format!("{}", self.connection.point_id),
            format!("{}", self.para.data_bits),
            format!("{}", self.para.stop_bits),
            format!("{:?}", self.para.parity),
        ];
        let mut result = format!(
            "{},{},{},{},{},{},{},{},{}\n",
            text_map.get("conn_name").unwrap_or(&"Connection Name".to_string()),
            get_csv_str(&self.name),
            text_map.get("index").unwrap_or(&"Index".to_string()),
            text_map.get("register_type").unwrap_or(&"Register Type".to_string()),
            text_map.get("start_addr").unwrap_or(&"Start Address".to_string()),
            text_map.get("data_type").unwrap_or(&"Data Type".to_string()),
            text_map.get("new_request_flag").unwrap_or(&"New Request".to_string()),
            text_map.get("register_period_name").unwrap_or(&"Polling Period(ms)".to_string()),
            text_map.get("register_point_id").unwrap_or(&"Point ID".to_string()),
        ).to_string();

        let p = &self.connection.mb_data_configure;
        for i in 0_usize..15_usize {
            if p.len() > i {
                let bool_status = if p[i].should_new_request {
                    "TRUE"
                } else {
                    "FALSE"
                };
                result += &format!(
                    "{},{},{},{},{},{},{},{},{}\n",
                    title[i],
                    content[i],
                    i + 1,
                    p[i].register_type,
                    p[i].from,
                    p[i].data_type,
                    bool_status,
                    p[i].polling_period_in_milli,
                    p[i].point_id
                );
            } else {
                result += &format!("{},{},,,,,,,\n", title[i], content[i]);
            }
        }
        if p.len() > 15 {
            let mut index = 15_usize;
            while index < p.len() {
                let bool_status = if p[index].should_new_request {
                    "TRUE"
                } else {
                    "FALSE"
                };
                result += &format!(
                    ",,{},{},{},{},{},{},{}\n",
                    index + 1,
                    p[index].register_type,
                    p[index].from,
                    p[index].data_type,
                    bool_status,
                    p[index].polling_period_in_milli,
                    p[index].point_id
                );
                index += 1;
            }
        }

        result
    }

    pub fn get_point_ids(&self) -> Vec<u64> {
        let size = self.connection.mb_data_configure.len() + 1;
        let mut r: Vec<u64> = Vec::with_capacity(size);
        for rd in &self.connection.mb_data_configure {
            r.push(rd.point_id)
        }
        if self.connection.point_id != UNKNOWN_POINT_ID {
            r.push(self.connection.point_id);
        }
        r
    }
}

pub fn get_register_count(d: &RegisterData) -> u16 {
    let count = d.data_type.get_byte_count();
    if count > 1 {
        count / 2
    } else {
        count
    }
}



#[cfg(test)]
mod tests {
    use std::cmp::Ordering;
    use std::collections::HashMap;

    use crate::{from_csv, from_csv_bytes, Measurement, PbFile, SerialParity};
    use crate::modbus::{
        DataType, MbProtocolType, ModbusRtuClientTp, ModbusRtuServerTp, ModbusTcpClientTp,
        ModbusTcpServerTp, RegisterType,
    };

    #[test]
    fn test_cmp() {
        let t1 = RegisterType::COILS;
        let t2 = RegisterType::DISCRETE;
        assert_eq!(Some(Ordering::Less), t1.partial_cmp(&t2));
        assert!(t1 < t2);
        let mut arr = vec![t2, t1];
        arr.sort();
        assert_eq!(arr, vec![RegisterType::COILS, RegisterType::DISCRETE]);
    }

    #[test]
    fn test_protocol_to_string() {
        assert_eq!(MbProtocolType::RTU.to_string(), "RTU");
        assert_eq!(MbProtocolType::XA.to_string(), "XA");
        assert_eq!(MbProtocolType::ENCAP.to_string(), "ENCAP");
    }

    #[test]
    fn test_certify() {
        let r = ModbusTcpServerTp::from_csv("tests/xa-mbd-test1-error.csv");
        assert_eq!(r.err().unwrap(), (2, 7));
        let r = ModbusTcpClientTp::from_csv("tests/xa-mbc-test1-error.csv");
        assert_eq!(r.err().unwrap(), (4, 8));
        let r = ModbusRtuClientTp::from_csv("tests/rtu-mbc-test1-error.csv");
        assert_eq!(r.err().unwrap(), (14, 17));
    }

    #[test]
    fn test_mbc_from_csv() {
        let tp = ModbusTcpClientTp::from_csv("tests/xa-mbc-test1.csv").unwrap();
        assert_eq!(tp.name, "测试通道1");
        assert_eq!(tp.tcp_server.0, "127.0.0.1");
        assert_eq!(tp.tcp_server.1, 5502);
        assert_eq!(tp.connections[0].slave_id, 1);
        assert_eq!(tp.connections[0].protocol_type, MbProtocolType::XA);
        assert_eq!(tp.connections[0].max_read_register_count, 125);
        assert_eq!(tp.connections[0].max_read_bit_count, 2000);
        assert_eq!(tp.connections[0].default_polling_period_in_milli, 5000);
        assert_eq!(tp.connections[0].timeout_in_milli, 1000);
        assert_eq!(tp.connections[0].delay_between_requests, 20);
        assert_eq!(tp.connections[0].polling_period_to_data.len(), 1);
        assert_eq!(tp.connections[0].polling_period_to_data.get(&5000u64).unwrap().len(), 10);
        let configure = &tp.connections[0].mb_data_configure;
        assert_eq!(configure.len(), 10);
        assert_eq!(configure.first().unwrap().point_id, 4001);
        assert_eq!(configure.first().unwrap().from, 1);
        assert_eq!(configure.first().unwrap().data_type, DataType::Binary);
        assert_eq!(configure.first().unwrap().register_type, RegisterType::COILS);
        assert!(!configure.first().unwrap().should_new_request);
        assert_eq!(configure.first().unwrap().polling_period_in_milli, 5000);
        assert_eq!(configure.get(9).unwrap().point_id, 4010);
        assert_eq!(configure.get(9).unwrap().from, 10);
        assert_eq!(configure.get(9).unwrap().data_type, DataType::Binary);
        assert_eq!(configure.get(9).unwrap().register_type, RegisterType::COILS);
        assert!(!configure.get(9).unwrap().should_new_request);
        assert_eq!(configure.get(9).unwrap().polling_period_in_milli, 5000);

        assert_eq!(tp.connections[0].polling_period_to_data.len(), 1);
        assert_eq!(tp.connections[0].polling_period_to_data.get(&5000).unwrap().len(), 10);
        let (coil, discrete, input, holding)
            = tp.connections[0].create_request(5000);
        assert_eq!(coil.len(), 1);
        let (from, num) = coil.first().unwrap();
        assert_eq!(*from, 1);
        assert_eq!(*num, 10);
        assert_eq!(discrete.len(), 0);
        assert_eq!(input.len(), 0);
        assert_eq!(holding.len(), 0);
    }

    #[test]
    fn test_mbd_from_csv() {
        let tp = ModbusTcpServerTp::from_csv("tests/xa-mbd-test1.csv").unwrap();
        assert_eq!(tp.name, "server测试通道");
        assert_eq!(tp.connections.len(), 2);
        assert_eq!(tp.tcp_server_port, 5502);
        let (conn_key, conn) = tp.connections.first().unwrap();
        assert_eq!(conn_key, "127.0.0.1/11/测试通道1");
        assert_eq!(conn.name, "测试通道1");
        assert_eq!(conn.slave_id, 1);
        assert_eq!(conn.protocol_type, MbProtocolType::XA);
        assert_eq!(conn.max_read_register_count, 125);
        assert_eq!(conn.max_read_bit_count, 2000);
        assert_eq!(conn.default_polling_period_in_milli, 5000);
        assert_eq!(conn.timeout_in_milli, 1000);
        assert_eq!(conn.point_id, 4999);
        assert_eq!(conn.polling_period_to_data.len(), 1);
        assert_eq!(conn.polling_period_to_data.get(&5000u64).unwrap().len(), 10);
        assert_eq!(conn.mb_data_configure.len(), 10);

        assert_eq!(conn.mb_data_configure.first().unwrap().point_id, 4001);
        assert_eq!(conn.mb_data_configure.first().unwrap().from, 1);
        assert_eq!(conn.mb_data_configure.first().unwrap().data_type, DataType::Binary);
        assert_eq!(conn.mb_data_configure.first().unwrap().register_type, RegisterType::COILS);
        assert!(!conn.mb_data_configure.first().unwrap().should_new_request);
        assert_eq!(conn.mb_data_configure.first().unwrap().polling_period_in_milli, 5000);

        assert_eq!(conn.mb_data_configure.get(9).unwrap().point_id, 4010);
        assert_eq!(conn.mb_data_configure.get(9).unwrap().from, 10);
        assert_eq!(conn.mb_data_configure.get(9).unwrap().data_type, DataType::Binary);
        assert_eq!(conn.mb_data_configure.get(9).unwrap().register_type, RegisterType::COILS);
        assert!(!conn.mb_data_configure.get(9).unwrap().should_new_request);
        assert_eq!(conn.mb_data_configure.get(9).unwrap().polling_period_in_milli, 5000);

        let (conn2_key, connection) = tp.connections.get(1).unwrap();
        assert_eq!(conn2_key, "127.0.0.1/23/测试通道2");
        assert_eq!(connection.name, "测试通道2");
        assert_eq!(connection.mb_data_configure.len(), 20);
        assert_eq!(connection.point_id, 5999);

        let s = tp.export_csv(&HashMap::with_capacity(0));
        let tp2 = ModbusTcpServerTp::from_csv_bytes(s.as_bytes()).unwrap();
        assert_eq!(tp, tp2);
    }

    #[test]
    fn test_rtu_mbc_wanke_pcs() {
        let r = ModbusRtuClientTp::from_csv("tests/rtu-mbc-transport-PCS-20210907.csv");
        let tp = r.unwrap();
        assert_eq!(tp.connections.len(), 1);
        assert_eq!(tp.connections[0].point_id_to_rd.len(), 626);
        assert_eq!(tp.connections[0].name, "PCS逆变器");
        assert_eq!(tp.connections[0].point_id_to_rd.get(&400133462), Some(&625));

        let s = tp.export_csv(&HashMap::with_capacity(0));
        let tp2 = ModbusRtuClientTp::from_csv_bytes(s.as_bytes()).unwrap();
        assert_eq!(tp, tp2);

        let (coil, discrete, input, holding) = tp.connections[0].create_request(2000);
        assert_eq!(coil.len(), 0);
        assert_eq!(discrete.len(), 4);
        assert_eq!(input.len(), 24);
        assert_eq!(holding.len(), 15);

        let (points, _) = from_csv("tests/points-PCS-20210908.csv").unwrap();
        assert_eq!(points.len(), 626);
        assert!(points.get(&400133421).is_some());

        let ids = tp.get_point_ids();
        assert_eq!(ids.len(), 626);
        let f = create_point_file(&ids, &points);
        let (points, _) = from_csv_bytes(f.fileContent(), false).unwrap();
        assert_eq!(points.len(), 626);
    }

    pub fn create_point_file(ids: &Vec<u64>, all_points: &HashMap<u64, Measurement>) -> PbFile {
        let mut points_file = PbFile::new();
        // 每个测点估计为300个字节
        let mut s = String::with_capacity(ids.len() * 300);
        let mut index = 1;
        for id in ids {
            if let Some(m) = all_points.get(id) {
                s.push_str(format!("{},{}\n", index, m.to_string()).as_str());
                index += 1;
            } else {
                log::warn!("Not found: {}", id);
            }
        }
        points_file.set_fileContent(s.into_bytes());
        points_file
    }

    #[test]
    fn test_rtu_mbc_from_csv() {
        let tp = ModbusRtuClientTp::from_csv("tests/rtu-mbc-test1.csv").unwrap();
        assert_eq!(tp.name, "RTU测试通道");
        assert_eq!(tp.connections.len(), 2);
        assert_eq!(tp.para.baud_rate, 19200);
        assert_eq!(tp.para.file_path, "/dev/ttyUSB0");
        let conn = tp.connections.first().unwrap();
        assert_eq!(conn.name, "测试通道1");
        assert_eq!(conn.slave_id, 1);
        assert_eq!(conn.protocol_type, MbProtocolType::RTU);
        assert_eq!(conn.max_read_register_count, 125);
        assert_eq!(conn.max_read_bit_count, 2000);
        assert_eq!(conn.default_polling_period_in_milli, 5000);
        assert_eq!(conn.timeout_in_milli, 1000);
        assert_eq!(conn.polling_period_to_data.len(), 1);
        assert_eq!(conn.polling_period_to_data.get(&5000u64).unwrap().len(), 10);
        assert_eq!(conn.holding_write_code, Some(16));

        assert_eq!(conn.mb_data_configure.len(), 10);
        assert_eq!(conn.mb_data_configure.first().unwrap().point_id, 4001);
        assert_eq!(conn.mb_data_configure.first().unwrap().from, 1);
        assert_eq!(conn.mb_data_configure.first().unwrap().data_type, DataType::Binary);
        assert_eq!(conn.mb_data_configure.first().unwrap().register_type, RegisterType::COILS);
        assert!(!conn.mb_data_configure.first().unwrap().should_new_request);
        assert_eq!(conn.mb_data_configure.first().unwrap().polling_period_in_milli, 5000);

        assert_eq!(conn.mb_data_configure.get(9).unwrap().point_id, 4010);
        assert_eq!(conn.mb_data_configure.get(9).unwrap().from, 10);
        assert_eq!(conn.mb_data_configure.get(9).unwrap().data_type, DataType::Binary);
        assert_eq!(conn.mb_data_configure.get(9).unwrap().register_type, RegisterType::COILS);
        assert!(!conn.mb_data_configure.get(9).unwrap().should_new_request);
        assert_eq!(conn.mb_data_configure.get(9).unwrap().polling_period_in_milli, 5000);

        let connection = tp.connections.get(1).unwrap();
        assert_eq!(connection.name, "测试通道2");
        assert_eq!(connection.coil_write_code, None);
        assert_eq!(connection.mb_data_configure.len(), 20);

        let s = tp.export_csv(&HashMap::with_capacity(0));
        let tp2 = ModbusRtuClientTp::from_csv_bytes(s.as_bytes()).unwrap();
        assert_eq!(tp, tp2);
    }

    #[test]
    fn test_rtu_mbc_from_csv2() {
        let tp = ModbusRtuClientTp::from_csv("tests/rtu-mbc-test2.csv").unwrap();
        assert_eq!(tp.name, "RTU测试通道");
        assert_eq!(tp.connections.len(), 2);
        assert_eq!(tp.para.baud_rate, 19200);
        assert_eq!(tp.para.file_path, "/dev/ttyUSB0");
        assert_eq!(tp.para.data_bits, 10);
        assert_eq!(tp.para.stop_bits, 2);
        assert_eq!(tp.para.parity, SerialParity::Odd);
        assert_eq!(tp.para.delay_between_requests, 20);
        let conn = tp.connections.first().unwrap();
        assert_eq!(conn.name, "测试通道1");
        assert_eq!(conn.slave_id, 1);
        assert_eq!(conn.protocol_type, MbProtocolType::RTU);
        assert_eq!(conn.max_read_register_count, 125);
        assert_eq!(conn.max_read_bit_count, 2000);
        assert_eq!(conn.default_polling_period_in_milli, 5000);
        assert_eq!(conn.timeout_in_milli, 1000);
        assert_eq!(conn.polling_period_to_data.len(), 1);
        assert_eq!(conn.polling_period_to_data.get(&5000u64).unwrap().len(), 10);
        assert_eq!(conn.mb_data_configure.len(), 10);

        assert_eq!(conn.mb_data_configure.first().unwrap().point_id, 4001);
        assert_eq!(conn.mb_data_configure.first().unwrap().from, 1);
        assert_eq!(conn.mb_data_configure.first().unwrap().data_type, DataType::Binary);
        assert_eq!(conn.mb_data_configure.first().unwrap().register_type, RegisterType::COILS);
        assert!(!conn.mb_data_configure.first().unwrap().should_new_request);
        assert_eq!(conn.mb_data_configure.first().unwrap().polling_period_in_milli, 5000);

        assert_eq!(conn.mb_data_configure.get(9).unwrap().point_id, 4010);
        assert_eq!(conn.mb_data_configure.get(9).unwrap().from, 10);
        assert_eq!(conn.mb_data_configure.get(9).unwrap().data_type, DataType::Binary);
        assert_eq!(conn.mb_data_configure.get(9).unwrap().register_type, RegisterType::COILS);
        assert!(!conn.mb_data_configure.get(9).unwrap().should_new_request);
        assert_eq!(conn.mb_data_configure.get(9).unwrap().polling_period_in_milli, 5000);

        let connection = tp.connections.get(1).unwrap();
        assert_eq!(connection.name, "测试通道2");
        assert_eq!(connection.mb_data_configure.len(), 20);
        assert_eq!(connection.mb_data_configure.first().unwrap().point_id, 5001);
        assert_eq!(connection.mb_data_configure.first().unwrap().from, 1);

        let s = tp.export_csv(&HashMap::with_capacity(0));
        let tp2 = ModbusRtuClientTp::from_csv_bytes(s.as_bytes()).unwrap();
        assert_eq!(tp, tp2);

        let tp = ModbusRtuClientTp::from_csv("tests/rtu-mbc-test3.csv").unwrap();
        assert_eq!(tp.name, "RTU测试通道");
        assert_eq!(conn.name, "测试通道1");
        assert_eq!(conn.slave_id, 1);
        let connection = tp.connections.get(1).unwrap();
        assert_eq!(connection.name, "测试通道2");
        assert_eq!(connection.slave_id, 1);
        let s = tp.export_csv(&HashMap::with_capacity(0));
        let tp2 = ModbusRtuClientTp::from_csv_bytes(s.as_bytes()).unwrap();
        assert_eq!(tp, tp2);
    }

    #[test]
    fn test_rtu_mbc_from_csv3() {
        let tp = ModbusRtuClientTp::from_csv("tests/rtu-mbc-transport-shtb-20220111.csv").unwrap();
        let connection = tp.connections.first().unwrap();
        assert_eq!(connection.name, "园区负荷表");
        assert_eq!(connection.slave_id, 26);
        assert_eq!(connection.mb_data_configure.len(), 5);
        assert_eq!(connection.mb_data_configure[0].point_id, 100010033026120);
        assert_eq!(connection.mb_data_configure[4].point_id, 1000100330261001);

        let connection = tp.connections.get(1).unwrap();
        assert_eq!(connection.name, "万克结算表");
        assert_eq!(connection.mb_data_configure.len(), 17);
        assert_eq!(connection.mb_data_configure[0].point_id, 10001003300193);
        assert_eq!(connection.mb_data_configure[16].point_id, 100010033001128);

        let connection = tp.connections.get(2).unwrap();
        assert_eq!(connection.name, "温控仪");
        assert_eq!(connection.mb_data_configure.len(), 5);
        assert_eq!(connection.mb_data_configure[0].point_id, 1000100330040);
        assert_eq!(connection.mb_data_configure[4].point_id, 1000100330044);

        let s = tp.export_csv(&HashMap::with_capacity(0));
        let tp2 = ModbusRtuClientTp::from_csv_bytes(s.as_bytes()).unwrap();
        assert_eq!(tp, tp2);
    }

    #[test]
    fn test_rtu_mbd_from_csv() {
        let tp = ModbusRtuServerTp::from_csv("tests/rtu-mbd-test1.csv").unwrap();
        assert_eq!(tp.name, "测试通道1");
        assert_eq!(tp.para.baud_rate, 19200);
        assert_eq!(tp.para.file_path, "/dev/ttyUSB0");
        assert_eq!(tp.para.data_bits, 8); // 默认值
        assert_eq!(tp.para.stop_bits, 1); // 默认值
        assert_eq!(tp.para.parity, SerialParity::None); // 默认值

        let conn = &tp.connection;
        assert_eq!(conn.slave_id, 1);
        assert_eq!(conn.protocol_type, MbProtocolType::RTU);
        assert_eq!(conn.max_read_register_count, 125);
        assert_eq!(conn.max_read_bit_count, 2000);
        assert_eq!(conn.default_polling_period_in_milli, 5000);
        assert_eq!(conn.timeout_in_milli, 1000);
        assert_eq!(conn.polling_period_to_data.len(), 1);
        assert_eq!(conn.polling_period_to_data.get(&5000u64).unwrap().len(), 10);
        assert_eq!(conn.mb_data_configure.len(), 10);

        assert_eq!(conn.mb_data_configure.first().unwrap().point_id, 4001);
        assert_eq!(conn.mb_data_configure.first().unwrap().from, 1);
        assert_eq!(conn.mb_data_configure.first().unwrap().data_type, DataType::Binary);
        assert_eq!(conn.mb_data_configure.first().unwrap().register_type, RegisterType::COILS);
        assert!(!conn.mb_data_configure.first().unwrap().should_new_request);
        assert_eq!(conn.mb_data_configure.first().unwrap().polling_period_in_milli, 5000);

        assert_eq!(conn.mb_data_configure.get(9).unwrap().point_id, 4010);
        assert_eq!(conn.mb_data_configure.get(9).unwrap().from, 10);
        assert_eq!(conn.mb_data_configure.get(9).unwrap().data_type, DataType::Binary);
        assert_eq!(conn.mb_data_configure.get(9).unwrap().register_type, RegisterType::COILS);
        assert!(!conn.mb_data_configure.get(9).unwrap().should_new_request);
        assert_eq!(conn.mb_data_configure.get(9).unwrap().polling_period_in_milli, 5000);

        let s = tp.export_csv(&HashMap::with_capacity(0));
        let tp2 = ModbusRtuServerTp::from_csv_bytes(s.as_bytes()).unwrap();
        assert_eq!(tp, tp2);
    }

    #[test]
    fn test_rtu_mbd_from_csv2() {
        let tp = ModbusRtuServerTp::from_csv("tests/rtu-mbd-test2.csv").unwrap();
        assert_eq!(tp.name, "测试通道1");
        assert_eq!(tp.para.baud_rate, 19200);
        assert_eq!(tp.para.file_path, "/dev/ttyUSB0");
        assert_eq!(tp.para.data_bits, 10);
        assert_eq!(tp.para.stop_bits, 2);
        assert_eq!(tp.para.parity, SerialParity::Odd);

        let conn = &tp.connection;
        assert_eq!(conn.slave_id, 1);
        assert_eq!(conn.protocol_type, MbProtocolType::RTU);
        assert_eq!(conn.max_read_register_count, 125);
        assert_eq!(conn.max_read_bit_count, 2000);
        assert_eq!(conn.default_polling_period_in_milli, 5000);
        assert_eq!(conn.timeout_in_milli, 1000);
        assert_eq!(conn.polling_period_to_data.len(), 1);
        assert_eq!(conn.polling_period_to_data.get(&5000u64).unwrap().len(), 10);
        assert_eq!(conn.mb_data_configure.len(), 10);

        assert_eq!(conn.mb_data_configure.first().unwrap().point_id, 4001);
        assert_eq!(conn.mb_data_configure.first().unwrap().from, 1);
        assert_eq!(conn.mb_data_configure.first().unwrap().data_type, DataType::Binary);
        assert_eq!(conn.mb_data_configure.first().unwrap().register_type, RegisterType::COILS);
        assert!(!conn.mb_data_configure.first().unwrap().should_new_request);
        assert_eq!(conn.mb_data_configure.first().unwrap().polling_period_in_milli, 5000);

        assert_eq!(conn.mb_data_configure.get(9).unwrap().point_id, 4010);
        assert_eq!(conn.mb_data_configure.get(9).unwrap().from, 10);
        assert_eq!(conn.mb_data_configure.get(9).unwrap().data_type, DataType::Binary);
        assert_eq!(conn.mb_data_configure.get(9).unwrap().register_type, RegisterType::COILS);
        assert!(!conn.mb_data_configure.get(9).unwrap().should_new_request);
        assert_eq!(conn.mb_data_configure.get(9).unwrap().polling_period_in_milli, 5000);

        let s = tp.export_csv(&HashMap::with_capacity(0));
        let tp2 = ModbusRtuServerTp::from_csv_bytes(s.as_bytes()).unwrap();
        assert_eq!(tp, tp2);
    }

    #[test]
    fn test_rtu_mbd_from_csv3() {
        let tp = ModbusRtuServerTp::from_csv("tests/rtu-mbd-test3.csv").unwrap();
        assert_eq!(tp.para.baud_rate, 9600);
        assert_eq!(tp.para.file_path, "/dev/ttyUSB0");
        assert_eq!(tp.para.data_bits, 8);
        assert_eq!(tp.para.stop_bits, 1);
        assert_eq!(tp.para.parity, SerialParity::None);

        assert_eq!(tp.connection.slave_id, 1);
        assert_eq!(tp.connection.protocol_type, MbProtocolType::RTU);

        let s = tp.export_csv(&HashMap::with_capacity(0));
        let tp2 = ModbusRtuServerTp::from_csv_bytes(s.as_bytes()).unwrap();
        assert_eq!(tp, tp2);
    }

    #[test]
    fn test_mbc_encap_from_csv() {
        let tp = ModbusTcpClientTp::from_csv("tests/encap-mbc-transport-10.csv").unwrap();
        let (coil, discrete, input, holding) = tp.connections[0].create_request(5000);
        assert_eq!(coil.len(), 1);
        assert_eq!(discrete.len(), 1);
        assert_eq!(input.len(), 1);
        assert_eq!(holding.len(), 1);
        let (from, num) = coil.first().unwrap();
        assert_eq!(*from, 1);
        assert_eq!(*num, 2);
        let (from, num) = discrete.first().unwrap();
        assert_eq!(*from, 3);
        assert_eq!(*num, 2);
        let (from, num) = input.first().unwrap();
        assert_eq!(*from, 8);
        assert_eq!(*num, 3);
        let (from, num) = holding.first().unwrap();
        assert_eq!(*from, 5);
        assert_eq!(*num, 3);

        let s = tp.export_csv(&HashMap::with_capacity(0));
        let tp2 = ModbusTcpClientTp::from_csv_bytes2(s.as_bytes()).unwrap();
        assert_eq!(tp, tp2);
    }

    #[test]
    fn test_mbc_encap_from_csv_202105() {
        let tp = ModbusTcpClientTp::from_csv("tests/encap-mbc-transport-slave1.csv").unwrap();
        let (coil, discrete, input, holding) = tp.connections[0].create_request(5000);
        assert_eq!(coil.len(), 1);
        assert_eq!(discrete.len(), 1);
        assert_eq!(input.len(), 1);
        assert_eq!(holding.len(), 1);
        let (from, num) = coil.first().unwrap();
        assert_eq!(*from, 1);
        assert_eq!(*num, 10);
        let (from, num) = discrete.first().unwrap();
        assert_eq!(*from, 101);
        assert_eq!(*num, 10);
        let (from, num) = input.first().unwrap();
        assert_eq!(*from, 201);
        assert_eq!(*num, 13);
        let (from, num) = holding.first().unwrap();
        assert_eq!(*from, 301);
        assert_eq!(*num, 54);

        let s = tp.export_csv(&HashMap::with_capacity(0));
        let tp2 = ModbusTcpClientTp::from_csv_bytes2(s.as_bytes()).unwrap();
        assert_eq!(tp, tp2);
    }

    #[test]
    fn test_mbc_xa_from_csv_20211110() {
        let tp = ModbusTcpClientTp::from_csv("tests/xa-mbc-transport-py.csv").unwrap();
        let (coil, discrete, input, holding) = tp.connections[0].create_request(2000);
        assert_eq!(coil.len(), 0);
        assert_eq!(discrete.len(), 0);
        assert_eq!(input.len(), 0);
        assert_eq!(holding.len(), 7);
        let (from, num) = holding.first().unwrap();
        assert_eq!(*from, 1);
        assert_eq!(*num, 124);
        let (from, num) = holding.get(6).unwrap();
        assert_eq!(*from, 745);
        assert_eq!(*num, 64);

        let s = tp.export_csv(&HashMap::with_capacity(0));
        let tp2 = ModbusTcpClientTp::from_csv_bytes2(s.as_bytes()).unwrap();
        assert_eq!(tp, tp2);
    }

    #[test]
    fn test_tcp_mbc() {
        let tp = ModbusTcpClientTp::from_csv2("tests/tcp-mbc-test1.csv").unwrap();
        let connection = tp.connections.first().unwrap();
        assert_eq!(connection.name, "测试通道1");
        assert_eq!(connection.slave_id, 1);
        assert_eq!(connection.mb_data_configure.len(), 10);
        assert_eq!(connection.mb_data_configure[0].point_id, 4001);
        assert_eq!(connection.mb_data_configure[4].point_id, 4005);
        assert_eq!(connection.delay_between_requests, 30);

        let connection = tp.connections.get(1).unwrap();
        assert_eq!(connection.name, "测试通道2");
        assert_eq!(connection.slave_id, 2);
        assert_eq!(connection.mb_data_configure.len(), 20);
        assert_eq!(connection.mb_data_configure[0].point_id, 5001);
        assert_eq!(connection.mb_data_configure[16].point_id, 5106);
        assert_eq!(connection.delay_between_requests, 0);

        let s = tp.export_csv(&HashMap::with_capacity(0));
        let tp2 = ModbusTcpClientTp::from_csv_bytes2(s.as_bytes()).unwrap();
        assert_eq!(tp, tp2);

        let tp = ModbusTcpClientTp::from_csv2("tests/tcp-mbc-test2.csv").unwrap();
        let connection = tp.connections.first().unwrap();
        assert_eq!(connection.name, "测试通道1");
        assert_eq!(connection.slave_id, 1);

        let connection = tp.connections.get(1).unwrap();
        assert_eq!(connection.name, "测试通道2");
        assert_eq!(connection.slave_id, 1);

        let s = tp.export_csv(&HashMap::with_capacity(0));
        let tp2 = ModbusTcpClientTp::from_csv_bytes2(s.as_bytes()).unwrap();
        assert_eq!(tp, tp2);
    }

    #[test]
    fn test_encap_mbc_from_csv() {
        let tp = ModbusTcpClientTp::from_csv("tests/encap-mbc-test1.csv").unwrap();
        let connection = tp.connections.first().unwrap();
        assert_eq!(connection.name, "空调");

        let s = tp.export_csv(&HashMap::with_capacity(0));
        let tp2 = ModbusTcpClientTp::from_csv_bytes2(s.as_bytes()).unwrap();
        assert_eq!(tp, tp2);
    }

    #[test]
    fn test_export_mbd_model() {
        let tp = ModbusTcpServerTp::from_file("tests/tcp-mbd-labview.xlsx").unwrap();
        let s = tp.export_csv(&HashMap::with_capacity(0));
        let tp2 = ModbusTcpServerTp::from_csv_bytes(s.as_bytes()).unwrap();
        assert_eq!(tp, tp2);
    }

    #[test]
    fn test_export_mbc_model() {
        let tp = ModbusTcpClientTp::from_file2("tests/tcp-mbc-transport-5000-1.xlsx").unwrap();
        let s = tp.export_csv(&HashMap::with_capacity(0));
        let tp2 = ModbusTcpClientTp::from_csv_bytes2(s.as_bytes()).unwrap();
        assert_eq!(tp, tp2);
    }
    #[test]
    fn test_export_mbd_unknown_ip() {
        let tp = ModbusTcpServerTp::from_file("tests/tcp-mbd-unknown-ip.xlsx").unwrap();
        let s = tp.export_csv(&HashMap::with_capacity(0));
        let tp2 = ModbusTcpServerTp::from_csv_bytes(s.as_bytes()).unwrap();
        assert_eq!(tp, tp2);
    }
}
