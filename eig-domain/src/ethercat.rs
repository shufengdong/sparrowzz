use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use csv::StringRecord;

use crate::{csv_str, csv_string, csv_u16, csv_u64, csv_usize, get_csv_str};
use crate::excel::{excel_bytes_to_csv_bytes, transfer_to_utf8};
use crate::UNKNOWN_POINT_ID;
use crate::prop::DataType;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct EcConnection {
    pub name: String,
    pub module_name: String,
    pub index: usize,
    pub point_id: u64,
    pub data: Vec<PdiData>,
    pub point_to_pos: HashMap<u64, usize>,
    pub cycle_time_in_micro: u64,
    pub watchdog_pdi: Option<u16>,  // 1/25M*(multi_watchdog+2)*pdi_watchdog
    pub watchdog_sm: Option<u16>,   // 1/25M*(multi_watchdog+2)*sm_watchdog, defaukt to 1000
    pub watchdog_multi: Option<u16>,    // defaukt to 2498
}
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct EcMasterTp {
    pub id: u64,
    /// 通道名称
    pub name: String,
    pub eth: String,
    pub connections: Vec<EcConnection>
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PdiData {
    pub is_writable: bool,
    pub from: u16,
    // 数据类型
    pub data_type: DataType,
    // 对应的测点Id
    pub point_id: u64,
}

impl Default for EcConnection {
    fn default() -> Self {
        EcConnection {
            name: "new".to_string(),
            module_name: "new_module".to_string(),
            index: 0,
            point_id: 0,
            data: Vec::new(),
            point_to_pos: HashMap::new(),
            cycle_time_in_micro: 0,
            watchdog_pdi: None,
            watchdog_sm: None,
            watchdog_multi: None,
        }
    }
}

impl EcConnection {
    pub fn read_config(&self) -> HashMap<usize, PdiData> {
        let mut r = HashMap::with_capacity(self.data.len());
        for pdi in &self.data {
            if !pdi.is_writable {
                r.insert(pdi.from as usize, pdi.clone());
            }
        }
        r.shrink_to_fit();
        r
    }

    pub fn write_config(&self) -> HashMap<usize, PdiData> {
        let mut r = HashMap::new();
        for pdi in &self.data {
            if pdi.is_writable {
                r.insert(pdi.from as usize, pdi.clone());
            }
        }
        r.shrink_to_fit();
        r
    }
}

impl EcMasterTp {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, (usize, usize)> {
        let content = std::fs::read(&path).map_err(|_| (0, 0))?;
        // let content = if env::IS_ENCRYPT {
        //     decrypt_vec(content.as_slice())
        // } else {
        //     content
        // };
        let csv_bytes = if path.as_ref().ends_with(".xlsx") || path.as_ref().ends_with(".xls") {
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
        let eth =
            csv_string(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        let mut connections = Vec::with_capacity(conn_num);
        for i in 0..conn_num {
            let connection = EcConnection::from_csv_records(content, i * 8 + 3)?;
            connections.push(connection);
        }
        Ok(EcMasterTp {
            id: 0,
            name,
            eth,
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
            if conn.point_id != UNKNOWN_POINT_ID {
                r.push(conn.point_id);
            }
        }
        r
    }

    // 导出CSV文件内容
    pub fn export_csv(&self, text_map: &HashMap<String, String>) -> String {
        let len_conn = self.connections.len();

        // 第一排
        let mut result = format!("{},{},,",
            text_map.get("tp_name").unwrap_or(&"Transport Name".to_string()),
            get_csv_str(&self.name));
        let mut i = 0;
        for conn in &self.connections {
            i += 1;
            result += &format!("{},{},{},{},{},{},{}",
                text_map.get("conn_name").unwrap_or(&"Connection Name".to_string()),
                get_csv_str(&conn.name),
                text_map.get("index").unwrap_or(&"Index".to_string()),
                text_map.get("is_writable").unwrap_or(&"Writable".to_string()),
                text_map.get("start_addr").unwrap_or(&"Start Address".to_string()),
                text_map.get("data_type").unwrap_or(&"Data Type".to_string()),
                text_map.get("point_id").unwrap_or(&"Point ID".to_string()),
            );
            if i != len_conn {
                result += ",,";
            } else {
                break;
            }
        }
        result += "\n";

        let title_conn = vec![
            text_map.get("point_number").unwrap_or(&"Point Count".to_string()).clone(),
            text_map.get("module_name").unwrap_or(&"Module Name".to_string()).clone(),
            text_map.get("period").unwrap_or(&"Period(ms)".to_string()).clone(),
            text_map.get("index_num").unwrap_or(&"Index Num".to_string()).clone(),
            text_map.get("watchdog_multi").unwrap_or(&"WD Multi".to_string()).clone(),
            text_map.get("watchdog_sm").unwrap_or(&"WD SM".to_string()).clone(),
            text_map.get("watchdog_pdi").unwrap_or(&"WD Pdi".to_string()).clone(),
        ];
        let title_tp = vec![
            format!("{},{}",
                    text_map.get("conn_num").unwrap_or(&"Connection Count".to_string()),
                    len_conn
            ),
            format!("{},{}",
                    text_map.get("eth_name").unwrap_or(&"NIC Name".to_string()),
                    get_csv_str(&self.eth)
            ),
        ];

        // 第二至三排
        for row in 0..2 {
            result += &title_tp[row];
            let mut i = 0;
            for conn in &self.connections {
                i += 1;
                let content_conn = match row {
                    0 => conn.data.len().to_string(),
                    1 => conn.module_name.clone(),
                    _ => "".to_string(),
                };
                if conn.data.len() > row {
                    let p = &conn.data[row];
                    result += &format!(
                        ",,{},{},{},{},{},{},{}",
                        title_conn[row],
                        content_conn,
                        row + 1,
                        p.is_writable.to_string().to_uppercase(),
                        p.from.to_string(),
                        p.data_type.to_string(),
                        p.point_id.to_string(),
                    );
                } else {
                    result += &format!(",,{},{},,,,,", title_conn[row], content_conn);
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
            self.connections[0].data.len()
        };
        for conn in &self.connections {
            if conn.data.len() > max_data_len {
                max_data_len = conn.data.len();
            }
        }
        if max_data_len < 7 {
            for row in 2..max_data_len {
                result += ",";
                let mut i = 0;
                for conn in &self.connections {
                    i += 1;
                    let content_conn = match row {
                        2 => conn.cycle_time_in_micro.to_string(),
                        3 => conn.index.to_string(),
                        4 => if let Some(s) = conn.watchdog_multi { s.to_string() } else { "".to_string() }
                        5 => if let Some(s) = conn.watchdog_sm { s.to_string() } else { "".to_string() }
                        6 => if let Some(s) = conn.watchdog_pdi { s.to_string() } else { "".to_string() }
                        _ => "".to_string(),
                    };
                    if conn.data.len() > row {
                        let p = &conn.data[row];
                        result += &format!(
                            ",,{},{},{},{},{},{},{}",
                            title_conn[row],
                            content_conn,
                            row + 1,
                            p.is_writable.to_string().to_uppercase(),
                            p.from.to_string(),
                            p.data_type.to_string(),
                            p.point_id.to_string(),
                        );
                    } else {
                        result += &format!(",,{},{},,,,,", title_conn[row], content_conn);
                    }
                    if i == len_conn {
                        break;
                    }
                }
                result += "\n";
            }

            for row in max_data_len..7 {
                result += ",";
                let mut i = 0;
                for conn in &self.connections {
                    i += 1;
                    let content_conn = match row {
                        2 => conn.cycle_time_in_micro.to_string(),
                        3 => conn.index.to_string(),
                        4 => if let Some(s) = conn.watchdog_multi { s.to_string() } else { "".to_string() }
                        5 => if let Some(s) = conn.watchdog_sm { s.to_string() } else { "".to_string() }
                        6 => if let Some(s) = conn.watchdog_pdi { s.to_string() } else { "".to_string() }
                        _ => "".to_string(),
                    };
                    result += &format!(",,{},{},,,,,", title_conn[row], content_conn);
                    if i == len_conn {
                        break;
                    }
                }
                result += "\n";
            }
        } else {
            for row in 2..7 {
                result += ",";
                let mut i = 0;
                for conn in &self.connections {
                    i += 1;
                    let content_conn = match row {
                        2 => conn.cycle_time_in_micro.to_string(),
                        3 => conn.index.to_string(),
                        4 => if let Some(s) = conn.watchdog_multi { s.to_string() } else { "".to_string() }
                        5 => if let Some(s) = conn.watchdog_sm { s.to_string() } else { "".to_string() }
                        6 => if let Some(s) = conn.watchdog_pdi { s.to_string() } else { "".to_string() }
                        _ => "".to_string(),
                    };
                    if conn.data.len() > row {
                        let p = &conn.data[row];
                        result += &format!(
                            ",,{},{},{},{},{},{},{}",
                            title_conn[row],
                            content_conn,
                            row + 1,
                            p.is_writable.to_string().to_uppercase(),
                            p.from.to_string(),
                            p.data_type.to_string(),
                            p.point_id.to_string(),
                        );
                    } else {
                        result += &format!(",,{},{},,,,,", title_conn[row], content_conn);
                    }
                    if i == len_conn {
                        break;
                    }
                }
                result += "\n";
            }

            for row in 7..max_data_len {
                result += ",";
                let mut i = 0;
                for conn in &self.connections {
                    i += 1;
                    if conn.data.len() > row {
                        let p = &conn.data[row];
                        result += &format!(
                            ",,,{},{},{},{},{}",
                            row + 1,
                            p.is_writable.to_string().to_uppercase(),
                            p.from.to_string(),
                            p.data_type.to_string(),
                            p.point_id.to_string(),
                        );
                    } else {
                        result += ",,,,,,,,,";
                    }
                    if i == len_conn {
                        break;
                    }
                }
                result += "\n";
            }
        }




        result
    }
}

impl EcConnection {
    fn from_csv_records (
        content: &[u8],
        offset: usize,
    ) -> Result<EcConnection, (usize, usize)> {
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
        let module_name = csv_string(&record, rc.1).ok_or(rc)?;
        // 4th line
        let rc = (3usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let cycle_time_in_micro = csv_u64(&record, rc.1).ok_or(rc)?;
        // 5th line
        let rc = (4usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let index = csv_usize(&record, rc.1).ok_or(rc)?;
        // 6th line
        let rc = (5usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let watchdog_multi = csv_u16(&record, rc.1);
        // 7th line
        let rc = (6usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let watchdog_sm = csv_u16(&record, rc.1);
        // 8th line
        let rc = (7usize, 1 + offset);
        let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
        let watchdog_pdi = csv_u16(&record, rc.1);

        // 9th..
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(content);
        let mut records = rdr.records();
        let mut pdi_data = Vec::with_capacity(point_num);
        let rc = (0, 3 + offset);
        records.next().ok_or(rc)?.map_err(|_| rc)?;
        for row in 1..=point_num {
            let rc = (row, 3 + offset);
            let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
            let data = PdiData::parse_pdi_data(&record, rc.0, rc.1)?;
            pdi_data.push(data);
        }
        let mut conn = EcConnection {
            name,
            module_name,
            point_id: UNKNOWN_POINT_ID,
            data: pdi_data,
            index,
            cycle_time_in_micro,
            watchdog_multi,
            watchdog_pdi,
            watchdog_sm,
            ..Default::default()
        };
        conn.create_data_config().map_err(|(r, c, _)|(r, c + offset))?;
        Ok(conn)
    }

    pub fn create_data_config(&mut self) -> Result<(),(usize, usize, String)> {
        let size = self.data.len();
        let mut point_to_pos = HashMap::with_capacity(size);
        for (index, rd) in self.data.iter().enumerate() {
            point_to_pos.insert(rd.point_id, index);
        }

        let mut keys = self.data.clone();
        keys.sort_by(|a, b| a.from.cmp(&b.from));
        // 判断地址之间有没有互相覆盖
        let mut last_addr = u16::MIN;
        for (index, rd) in keys.iter().enumerate() {
            let tip = format!("Invalid register point (id :{}):\nThe register address is already existed", rd.point_id);
            // 如果开始地址在已经被使用的地址范围
            if rd.from < last_addr {
                return Err((index + 1, 4, tip)); // 地址的位置
            }
            last_addr = rd.from + rd.data_type.get_byte_count();
        }
        self.point_to_pos = point_to_pos;
        Ok(())
    }
}

impl PdiData {
    fn parse_pdi_data(
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
        let from = csv_u16(record, rc.1).ok_or(rc)?;
        let rc = (row, first_col + 2);
        let s = csv_str(record, rc.1).ok_or(rc)?;
        let data_type = DataType::from_str(s).map_err(|_|rc)?;
        // 对应的测点Id
        let rc = (row, first_col + 3);
        let point_id = csv_u64(record, rc.1).ok_or(rc)?;
        Ok(PdiData {
            is_writable,
            from,
            data_type,
            point_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_file() {
        let file = "tests/ethercat-transport-test1.csv";
        let tp = EcMasterTp::from_file(file);
        assert!(tp.is_ok());
        let tp = tp.unwrap();
        assert_eq!("enp5s0", tp.eth);
    }
}