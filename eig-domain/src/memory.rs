use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;
use std::str::FromStr;

use csv::StringRecord;
use serde::{Deserialize, Serialize};

use crate::{csv_i32, csv_str, csv_string, csv_u64, csv_usize, MAX_POLLING_PERIOD};
use crate::excel::{excel_bytes_to_csv_bytes, transfer_to_utf8};
use crate::prop::DataType;

const DEFAULT_POLLING_PERIOD_IN_MILLI: u64 = 5000;

#[derive(Serialize, Deserialize, PartialEq, Debug, Default, Clone, Copy)]
pub enum MemLock {
    #[default]
    None,
    Mutex(usize),
    Semaphore,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MemConnection {
    pub name: String,
    pub base_addr: usize,
    // 取决于计算机位数，如果溢出，应该报错。
    pub total_size: Option<usize>,
    pub point_to_pos: HashMap<u64, usize>,
    pub data: Vec<MemData>,
    pub default_polling_period_in_milli: u64,
    /// key:寄存器地址,value:setting中vec<MemData>的位置
    pub mem_addr_to_pos: HashMap<usize, usize>,
    /// 轮询周期不同的数据, key is period in milli, value is position.
    pub polling_period_to_data: BTreeMap<u64, Vec<usize>>,
    pub lock_method: MemLock,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MemoryPosixTp {
    pub id: u64,
    pub name: String,
    pub is_transfer: bool,
    /// 通道名称
    pub path: Option<String>,
    pub connections: Vec<MemConnection>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
pub struct MemorySystemVTp {
    pub id: u64,
    pub name: String,
    pub is_transfer: bool,
    /// 通道名称
    pub path: String,
    pub identifier: i32,
    pub connection: MemConnection,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MemData {
    pub is_writable: bool,
    pub from: usize,
    // 数据类型
    pub data_type: DataType,
    // 对应的测点Id
    pub point_id: u64,
    pub polling_period_in_milli: u64,
}

impl Default for MemConnection {
    fn default() -> Self {
        MemConnection {
            name: "new".to_string(),
            base_addr: 0,
            total_size: None,
            point_to_pos: Default::default(),
            data: vec![],
            default_polling_period_in_milli: DEFAULT_POLLING_PERIOD_IN_MILLI,
            mem_addr_to_pos: Default::default(),
            polling_period_to_data: Default::default(),
            lock_method: MemLock::None,
        }
    }
}

impl MemConnection {
    pub fn read_config(&self) -> HashMap<usize, MemData> {
        let mut r = HashMap::with_capacity(self.data.len());
        for pdi in &self.data {
            if !pdi.is_writable {
                r.insert(pdi.from, pdi.clone());
            }
        }
        r.shrink_to_fit();
        r
    }

    pub fn write_config(&self) -> HashMap<usize, MemData> {
        let mut r = HashMap::new();
        for pdi in &self.data {
            if pdi.is_writable {
                r.insert(pdi.from, pdi.clone());
            }
        }
        r.shrink_to_fit();
        r
    }

    fn from_csv_records_posix(
        content: &[u8],
        offset: usize,
    ) -> Result<Self, (usize, usize)> {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(content);
        let mut records = rdr.records();
        // 1th line
        let rc = (0usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let name = csv_string(&record, rc.1).ok_or(rc)?;
        // 2th line
        let rc = (1usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let point_num = csv_usize(&record, rc.1).ok_or(rc)?;
        // 3th line
        let rc = (2usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let base_addr = csv_usize(&record, rc.1).ok_or(rc)?;
        // 4th line
        let rc = (3usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let total_size_tmp = csv_string(&record, rc.1).ok_or(rc)?;
        let total_size = if total_size_tmp.is_empty() {
            None
        } else {
            Some(total_size_tmp.parse().map_err(|_| rc)?)
        };
        // 5th line
        let rc = (4usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let s = csv_str(&record, rc.1).ok_or(rc)?;
        let default_polling_period_in_milli: u64 = if s.is_empty() {
            DEFAULT_POLLING_PERIOD_IN_MILLI
        } else {
            s.parse().map_err(|_| rc)?
        };

        // 6th line
        let rc = (5usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let s = csv_str(&record, rc.1).ok_or(rc).unwrap_or_default();
        let lock_method = match s.trim().to_uppercase().as_str() {
            "NONE" => MemLock::None,
            "SEMAPHORE" => MemLock::Semaphore,
            "MUTEX" => {
                // 7th line
                let rc = (6usize, 1 + offset);
                let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
                let mutex_num = csv_usize(&record, rc.1).ok_or(rc)?;
                MemLock::Mutex(mutex_num)
            }
            _ => MemLock::None,
        };

        // 9th..
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(content);
        let mut records = rdr.records();

        let mut mem_data = Vec::with_capacity(point_num);
        let rc = (0, 3 + offset);
        records.next().ok_or(rc)?.map_err(|_| rc)?;
        for row in 1..=point_num {
            let rc = (row, 3 + offset);
            let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
            let data = MemData::parse_mem_data(&record, rc.0, rc.1)?;
            mem_data.push(data);
        }

        let mut conn = MemConnection {
            name,
            base_addr,
            total_size,
            default_polling_period_in_milli,
            data: mem_data,
            lock_method,
            ..Default::default()
        };
        conn.create_data_config().map_err(|(r, c, _)| (r, c + offset))?;
        Ok(conn)
    }

    pub fn create_data_config(&mut self) -> Result<(), (usize, usize, String)> {
        let size = self.data.len();
        let mut point_to_pos = HashMap::with_capacity(size);
        for (index, rd) in self.data.iter().enumerate() {
            point_to_pos.insert(rd.point_id, index);
        }
        let mut point_exist: HashSet<u64> = HashSet::with_capacity(size);
        let mut register_addr_to_rd: HashMap<usize, usize> = HashMap::with_capacity(size);
        let mut polling_period_to_data: BTreeMap<u64, Vec<usize>> = BTreeMap::new();
        polling_period_to_data.insert(self.default_polling_period_in_milli, Vec::with_capacity(size));
        let mut tmp: HashMap<u64, usize> = HashMap::with_capacity(10);
        for data in &self.data {
            if let Some(ori) = tmp.get_mut(&data.polling_period_in_milli) {
                *ori += 1;
            } else {
                tmp.insert(data.polling_period_in_milli, 1);
            }
        }
        for (i, num) in tmp {
            // 对于不需要采集的数据可以通过设置一个很大的轮询周期
            if i >= MAX_POLLING_PERIOD {
                continue;
            }
            let mut a_v: Vec<usize> = Vec::with_capacity(num);
            for (index, rd) in self.data.iter().enumerate() {
                if rd.polling_period_in_milli == i {
                    a_v.push(index.try_into().unwrap());
                }
            }
            polling_period_to_data.insert(i, a_v);
        }
        for (index, data) in self.data.iter().enumerate() {
            // 测点号重复
            if point_exist.contains(&data.point_id) {
                let tip = format!("Invalid point (id :{}):\nThe point ID is already existed", data.point_id);
                return Err((index + 1, 8, tip));
            }
            point_exist.insert(data.point_id);
            // 起始地址重复
            if register_addr_to_rd.contains_key(&data.from) {
                let tip = format!("Invalid point (id :{}):\nThe register address is already existed", data.point_id);
                return Err((index + 1, 4, tip));
            }
            register_addr_to_rd.insert(data.from, index);
        }
        // 判断地址之间有没有互相覆盖
        let mut last_addr = usize::MIN;
        let mut keys: Vec<&usize> = register_addr_to_rd.keys().collect();
        keys.sort(); // 按照起始地址排序
        for addr in keys {
            let index = register_addr_to_rd.get(addr).unwrap();
            let rd = self.data.get(*index).unwrap();
            // 如果开始地址在已经被使用的地址范围
            if rd.from < last_addr {
                let tip = format!("Invalid register point (id :{}):\nThe start address is in the range of addresses that are already in use", rd.point_id);
                return Err((index + 1, 4, tip));
            }
            last_addr = rd.from + rd.data_type.get_byte_count() as usize;
        }
        self.point_to_pos = point_to_pos;
        self.mem_addr_to_pos = register_addr_to_rd;
        self.polling_period_to_data = polling_period_to_data;
        Ok(())
    }
    // 这里传入的register是已经按照地址排好序
    pub fn create_request(&self, period: u64, is_transfer: bool) -> Vec<(usize, usize)> {
        return if let Some(positions) = self.polling_period_to_data.get(&period) {
            let mut off_set = 0usize;
            let mut num_of_registers = 0usize;
            let mut result = Vec::with_capacity(positions.len());
            let mut last_index: Option<usize> = None;

            for index in 0..positions.len() {
                let d = &self.data[positions[index]];
                if is_transfer && !d.is_writable {
                    continue;
                }
                let current_byte_num = d.data_type.get_byte_count() as usize;
                if last_index.is_some() {
                    let last = &self.data[positions[last_index.unwrap()]];
                    let last_byte_num = last.data_type.get_byte_count() as usize;
                    // 地址不连续
                    if d.from - last.from != last_byte_num {
                        result.push((off_set, num_of_registers));
                        off_set = d.from;
                        num_of_registers = current_byte_num;
                    } else {
                        // 以上情况都未发生
                        num_of_registers += current_byte_num;
                    }
                    last_index = Some(index);
                } else {
                    // first found
                    off_set = d.from;
                    num_of_registers = current_byte_num;
                    last_index = Some(index);
                }
            }
            // 搜索到末尾
            if num_of_registers > 0 {
                result.push((off_set, num_of_registers));
            }
            result.shrink_to_fit();
            result
        } else {
            vec![]
        };
    }
}

impl MemoryPosixTp {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, (usize, usize)> {
        let content = std::fs::read(&path).map_err(|_| (0, 0))?;
        // let content = if env::IS_ENCRYPT {
        //     decrypt_vec(content.as_slice())
        // } else {
        //     content
        // };
        let extension = path.as_ref().extension();
        let csv_bytes = if let Some(suffix) = extension {
            if suffix.eq("xlsx") || suffix.eq("xls") {
                let mut r = excel_bytes_to_csv_bytes(content.as_slice()).unwrap_or_default();
                if r.is_empty() {
                    return Err((0, 0));
                }
                r.pop().unwrap()
            } else {
                content
            }
        } else {
            content
        };
        Self::from_csv_bytes(csv_bytes.as_slice())
    }

    pub fn from_csv(path: &str) -> Result<Self, (usize, usize)> {
        let content = std::fs::read(path).map_err(|_| (0, 0))?;
        // if env::IS_ENCRYPT {
        //     let plain_t = decrypt_vec(content.as_slice());
        //     ModbusTcpClientTp::from_csv_bytes(plain_t.as_slice())
        // } else {
        //     ModbusTcpClientTp::from_csv_bytes(content.as_slice())
        // }
        Self::from_csv_bytes(content.as_slice())
    }

    pub fn from_csv_bytes(content: &[u8]) -> Result<Self, (usize, usize)> {
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
        let path_tmp =
            csv_string(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        let path = if path_tmp.trim().is_empty() {
            None
        } else {
            Some(path_tmp)
        };
        let rc = (3usize, 1);
        let is_transfer =
            csv_string(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).unwrap_or("FALSE".to_string());
        let is_transfer = if is_transfer.to_uppercase() == "TRUE" {
            true
        } else {
            false
        };
        let mut connections = Vec::with_capacity(conn_num);
        for i in 0..conn_num {
            let connection = MemConnection::from_csv_records_posix(content, i * 9 + 3)?;
            connections.push(connection);
        }
        Ok(MemoryPosixTp {
            id: 0,
            name,
            path,
            is_transfer,
            connections,
        })
    }

    pub fn get_point_ids(&self) -> Vec<u64> {
        let mut size = 0;
        for conn in &self.connections {
            size += conn.data.len();
            size += 1;
        }
        let mut r: Vec<u64> = Vec::with_capacity(size);
        for conn in &self.connections {
            for rd in &conn.data {
                r.push(rd.point_id)
            }
        }
        r
    }

    // 导出CSV文件内容
    pub fn export_csv(&self, text_map: &HashMap<String, String>) -> String {
        "".to_string()
    }
}

impl MemorySystemVTp {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, (usize, usize)> {
        let content = std::fs::read(&path).map_err(|_| (0, 0))?;
        // let content = if env::IS_ENCRYPT {
        //     decrypt_vec(content.as_slice())
        // } else {
        //     content
        // };
        let extension = path.as_ref().extension();
        let csv_bytes = if let Some(suffix) = extension {
            if suffix.eq("xlsx") || suffix.eq("xls") {
                let mut r = excel_bytes_to_csv_bytes(content.as_slice()).unwrap_or_default();
                if r.is_empty() {
                    return Err((0, 0));
                }
                r.pop().unwrap()
            } else {
                content
            }
        } else {
            content
        };
        Self::from_csv_bytes(csv_bytes.as_slice())
    }

    pub fn from_csv(path: &str) -> Result<Self, (usize, usize)> {
        let content = std::fs::read(path).map_err(|_| (0, 0))?;
        // if env::IS_ENCRYPT {
        //     let plain_t = decrypt_vec(content.as_slice());
        //     ModbusTcpClientTp::from_csv_bytes(plain_t.as_slice())
        // } else {
        //     ModbusTcpClientTp::from_csv_bytes(content.as_slice())
        // }
        Self::from_csv_bytes(content.as_slice())
    }

    pub fn from_csv_bytes(content: &[u8]) -> Result<Self, (usize, usize)> {
        let content_new = transfer_to_utf8(content.to_vec()).map_err(|_| (0, 0))?;
        let content = content_new.as_slice();
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(content);
        let mut records = rdr.records();
        let rc = (0usize, 1);
        let name = csv_string(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        let rc = (1usize, 1);
        let point_num: usize =
            csv_usize(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        let rc = (2usize, 1);
        let path = csv_string(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        let rc = (3usize, 1);
        let identifier = csv_i32(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        // 5th line
        let rc = (4usize, 1);
        let total_size = csv_usize(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        // 6th line
        let rc = (5usize, 1);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let s = csv_str(&record, rc.1).ok_or(rc)?;
        let default_polling_period_in_milli: u64 = if s.is_empty() {
            DEFAULT_POLLING_PERIOD_IN_MILLI
        } else {
            s.parse().map_err(|_| rc)?
        };
        // 7th line 
        let rc = (6usize, 1);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let s = csv_str(&record, rc.1).ok_or(rc).unwrap_or_default();
        let lock_method = match s.trim().to_uppercase().as_str() {
            "NONE" => MemLock::None,
            "SEMAPHORE" => MemLock::Semaphore,
            "MUTEX" => {
                // 8th line
                let rc = (7usize, 1);
                let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
                let mutex_num = csv_usize(&record, rc.1).ok_or(rc)?;
                MemLock::Mutex(mutex_num)
            }
            _ => MemLock::None,
        };
        // 9th line
        let rc = (6usize, 1);
        let is_transfer =
            csv_string(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).unwrap_or("FALSE".to_string());
        let is_transfer = if is_transfer.to_uppercase() == "TRUE" {
            true
        } else {
            false
        };

        let mut mem_data = Vec::with_capacity(point_num);

        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(content);
        let mut records = rdr.records();
        let rc = (0, 3);
        records.next().ok_or(rc)?.map_err(|_| rc)?;
        for row in 1..=point_num {
            let rc = (row, 3);
            let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
            let data = MemData::parse_mem_data(&record, rc.0, rc.1)?;
            mem_data.push(data);
        }

        let mut connection = MemConnection {
            name: name.clone(),
            base_addr: 0,
            total_size: Some(total_size),
            default_polling_period_in_milli,
            data: mem_data,
            lock_method,
            ..Default::default()
        };
        connection.create_data_config().map_err(|(r, c, _)| (r, c))?;

        Ok(MemorySystemVTp {
            id: 0,
            name,
            path,
            identifier,
            is_transfer,
            connection,
        })
    }

    pub fn get_point_ids(&self) -> Vec<u64> {
        self.connection.data.iter().map(|d| d.point_id).collect()
    }

    // 导出CSV文件内容
    pub fn export_csv(&self, text_map: &HashMap<String, String>) -> String {
        "".to_string()
    }
}

impl MemData {
    fn parse_mem_data(
        record: &StringRecord,
        row: usize,
        first_col: usize,
    ) -> Result<Self, (usize, usize)> {
        let rc = (row, first_col);
        let s = csv_str(record, rc.1).ok_or(rc)?;
        let is_writable = match s {
            "FALSE" => false,
            "TRUE" => true,
            _ => false,
        };
        let rc = (row, first_col + 1);
        let from = csv_usize(record, rc.1).ok_or(rc)?;
        let rc = (row, first_col + 2);
        let s = csv_str(record, rc.1).ok_or(rc)?;
        let data_type = DataType::from_str(s).map_err(|_| rc)?;
        let rc = (row, first_col + 3);
        let polling_period_in_milli = csv_u64(record, rc.1).ok_or(rc)?;
        // 对应的测点Id
        let rc = (row, first_col + 4);
        let point_id = csv_u64(record, rc.1).ok_or(rc)?;
        Ok(MemData {
            is_writable,
            from,
            data_type,
            polling_period_in_milli,
            point_id,
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_file() {
        let file = "tests/posix-memory-transport-test1.xlsx";
        let tp = MemoryPosixTp::from_file(file);
        assert!(tp.is_ok());
        let tp = tp.unwrap();
        assert_eq!("posix-memory-transport-file", tp.path.unwrap());
    }
    

    #[test]
    fn parse_file_systemv() {
        let file = "tests/systemv-memory-transport-test1.xlsx";
        let tp = MemorySystemVTp::from_file(file);
        let tp = tp.unwrap();
        assert_eq!("/", tp.path);
    }

}