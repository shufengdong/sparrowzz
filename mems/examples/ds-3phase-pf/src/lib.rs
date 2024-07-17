#![allow(non_snake_case)]

use std::collections::HashMap;

use arrow_schema::{DataType, Field, Schema};
use ndarray::Array2;

use ds_common::dyn_topo::{read_dev_topo, read_dyn_topo};
use ds_common::{DEV_CONDUCTOR_DF_NAME, DEV_TOPO_DF_NAME, DS_PF_NLP_CONS, DS_PF_NLP_OBJ, DYN_TOPO_DF_NAME, TN_INPUT_DF_NAME};
use ds_common::tn_input::read_tn_input;
use mems::model::{get_wasm_result, PluginInput, PluginOutput};

use crate::read::read_dev_ohm;

mod read;
mod nlp;

#[no_mangle]
pub unsafe fn run(ptr: i32, len: u32) -> u64 {
    // 从内存中获取字符串
    let input = unsafe {
        let slice = std::slice::from_raw_parts(ptr as _, len as _);
        let input: PluginInput = serde_cbor::from_slice(slice).unwrap();
        input
    };
    let from = 0;
    let mut error = None;
    // get from dynamic topology wasm node
    // cn, tn
    let mut dyn_topo: Vec<Vec<u64>>;
    // terminal, cn, tn, dev
    let mut dev_topo: Vec<Vec<u64>>;

    // dev id, conductor matrix, get from conductor impedance cal wasm node
    let mut dev_conductor: HashMap<u64, Vec<Array2<f64>>>;
    // tn id with input
    let mut input_tns;
    // input pos
    let mut input_phases;
    // input types
    let mut input_types;
    // input values
    let mut input_values;
    for i in 0..input.dfs_len.len() {
        let size = input.dfs_len[i] as usize;
        let end = from + size;
        let mut rdr = csv::ReaderBuilder::new().has_headers(true).from_reader(&input.bytes[from..end]);
        let mut records = rdr.records();
        // 对第i个边输入该节点的 dataframe 进行处理
        if input.dfs[i] == DYN_TOPO_DF_NAME {
            match read_dyn_topo(&mut records) {
                Ok(v) => dyn_topo = v,
                Err(s) => {
                    error = Some(s);
                    break;
                }
            }
        } else if input.dfs[i] == DEV_TOPO_DF_NAME {
            match read_dev_topo(&mut records) {
                Ok(v) => dev_topo = v,
                Err(s) => {
                    error = Some(s);
                    break;
                }
            }
        } else if input.dfs[i] == DEV_CONDUCTOR_DF_NAME {
            match read_dev_ohm(&mut records) {
                Ok(v) => dev_conductor = v,
                Err(s) => {
                    error = Some(s);
                    break;
                }
            }
        } else if input.dfs[i] == TN_INPUT_DF_NAME {
            match read_tn_input(&mut records) {
                Ok(v) => (input_tns, input_phases, input_types, input_values) = v,
                Err(s) => {
                    error = Some(s);
                    break;
                }
            }
        }
    }
    if error.is_some() {
        let output = PluginOutput {
            error_msg: error,
            schema: None,
            csv_bytes: vec![],
        };
        get_wasm_result(output)
    } else {
        let mut obj_csv_str = String::from("cn,tn\n");
        // build schema
        let obj_schema = Schema::new(vec![
            Field::new("cn", DataType::UInt64, false),
            Field::new("tn", DataType::UInt64, false),
        ]);
        let mut cons_csv_str = String::from("cn,tn\n");
        // build schema
        let cons_schema = Schema::new(vec![
            Field::new("cn", DataType::UInt64, false),
            Field::new("tn", DataType::UInt64, false),
        ]);
        let csv_bytes = vec![
            (DS_PF_NLP_OBJ.to_string(), obj_csv_str.into_bytes()),
            (DS_PF_NLP_CONS.to_string(), cons_csv_str.into_bytes()),
        ];
        let output = PluginOutput {
            error_msg: None,
            schema: Some(vec![obj_schema, cons_schema]),
            csv_bytes,
        };
        get_wasm_result(output)
    }
}