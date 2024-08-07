use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use eig_domain::{prop::DataUnit, MeasureValue};

use crate::model::dev::{Island, PropDefine, RsrDefine};

pub mod dev;
pub mod plan;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum ModelType {
    Island,
    Meas,
    File(Vec<String>),
    Outgoing(Vec<String>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PluginInput {
    pub model: Vec<ModelType>,
    pub model_len: Vec<u32>,
    pub dfs: Vec<String>,
    pub dfs_len: Vec<u32>,
    pub bytes: Vec<u8>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PluginOutput {
    pub error_msg: Option<String>,
    pub schema: Option<Vec<arrow_schema::Schema>>,
    pub csv_bytes: Vec<(String, Vec<u8>)>,
}

pub fn get_island_from_plugin_input(input: &PluginInput) -> Result<(Island, Vec<PropDefine>, HashMap<u64, RsrDefine>), String> {
    let mut from = 0;
    let mut index = 0;
    for i in 0..input.model.len() {
        match input.model[i] {
            ModelType::Island => {
                if input.model_len.len() <= index {
                    return Err("model_len length error".to_string());
                }
                let size = input.model_len[index] as usize;
                let end = from + size;
                let r = ciborium::from_reader(&input.bytes[from..end]);
                if r.is_err() {
                    return Err(format!("{:?}", r));
                }
                from += size;
                let island = r.unwrap();
                index += 1;
                if input.model_len.len() <= index {
                    return Err("model_len length error".to_string());
                }
                let size = input.model_len[index] as usize;
                let end = from + size;
                let r = ciborium::from_reader(&input.bytes[from..end]);
                if r.is_err() {
                    return Err(format!("{:?}", r));
                }
                from += size;
                let defines = r.unwrap();
                index += 1;
                if input.model_len.len() <= index {
                    return Err("model_len length error".to_string());
                }
                let size = input.model_len[index] as usize;
                let end = from + size;
                let r = ciborium::from_reader(&input.bytes[from..end]);
                if r.is_err() {
                    return Err(format!("{:?}", r));
                }
                let prop_defs = r.unwrap();
                return Ok((island, prop_defs, defines));
            }
            ModelType::Meas => {
                if input.model_len.len() <= index + 2 {
                    return Err("model_len length error".to_string());
                }
                let size1 = input.model_len[index] as usize;
                let size2 = input.model_len[index + 1] as usize;
                from += size1;
                from += size2;
                index += 2;
            }
            _ => {}
        }
    }
    Err("Island not found in plugin input".to_string())
}

pub fn get_meas_from_plugin_input(input: &PluginInput) -> Result<(Vec<MeasureValue>, HashMap<u64, DataUnit>), String> {
    let mut from = 0;
    let mut index = 0;
    for i in 0..input.model.len() {
        match input.model[i] {
            ModelType::Meas => {
                if input.model_len.len() < index {
                    return Err("model_len length error".to_string());
                }
                let size = input.model_len[index] as usize;
                let end = from + size;
                let r = ciborium::from_reader(&input.bytes[from..end]);
                if r.is_err() {
                    return Err(format!("{:?}", r));
                }
                from += size;
                let meas = r.unwrap();
                index += 1;
                if input.model_len.len() <= index {
                    return Err("model_len length error".to_string());
                }
                let size = input.model_len[index] as usize;
                let end = from + size;
                let r = ciborium::from_reader(&input.bytes[from..end]);
                if r.is_err() {
                    return Err(format!("{:?}", r));
                }
                let units = r.unwrap();
                return Ok((meas, units));
            }
            ModelType::Island => {
                if input.model_len.len() < index + 3 {
                    return Err("model_len length error".to_string());
                }
                let size1 = input.model_len[index] as usize;
                let size2 = input.model_len[index + 1] as usize;
                let size3 = input.model_len[index + 2] as usize;
                from += size1;
                from += size2;
                from += size3;
                index += 3;
            }
            _ => {}
        }
    }
    Err("Measure not found in plugin input".to_string())
}

pub fn get_df_from_in_plugin(input: &PluginInput) -> Result<usize, String> {
    let mut from = 0;
    let mut index = 0;
    for i in 0..input.model.len() {
        match input.model[i] {
            ModelType::Meas => {
                if input.model_len.len() < index + 2 {
                    return Err(format!("model_len length error, expect more then {}, actual {}",
                                       index + 2, input.model_len.len()));
                }
                let size1 = input.model_len[index] as usize;
                let size2 = input.model_len[index + 1] as usize;
                from += size1;
                from += size2;
                index += 2;
            }
            ModelType::Island => {
                if input.model_len.len() < index + 3 {
                    return Err(format!("model_len length error, expect more then {}, actual {}",
                                       index + 3, input.model_len.len()));
                }
                let size1 = input.model_len[index] as usize;
                let size2 = input.model_len[index + 1] as usize;
                let size3 = input.model_len[index + 2] as usize;
                from += size1;
                from += size2;
                from += size3;
                index += 3;
            }
            _ => {}
        }
    }
    Ok(from)
}

// #[inline]
// pub fn get_wasm_result(output: PluginOutput) -> u64 {
//     // 下面的unwrap是必要的，否则输出的字节无法解析
//     let mut v = Vec::new();
//     ciborium::into_writer(&output, &mut v).unwrap();
//     v.shrink_to_fit();
//     let offset = v.as_ptr() as i32;
//     let len = v.len() as u32;
//     let mut bytes = BytesMut::with_capacity(8);
//     bytes.put_i32(offset);
//     bytes.put_u32(len);
//     return bytes.get_u64();
// }

#[inline]
pub fn get_csv_str(s: &str) -> String {
    if s.contains(',') || s.contains('\n') || s.contains('"')
        || s.starts_with(' ') || s.ends_with(' ') {
        format!("\"{}\"", s.replace('\"', "\"\""))
    } else {
        s.to_string()
    }
}



