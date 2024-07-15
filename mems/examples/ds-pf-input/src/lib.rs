use arrow_schema::{DataType, Field, Schema};

use ds_common::{DEV_TOPO_DF_NAME, DYN_TOPO_DF_NAME};
use ds_common::dyn_topo::{read_dev_topo, read_dyn_topo};
use mems::model::{get_meas_from_plugin_input, get_wasm_result, PluginInput, PluginOutput};

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
    let r1 = get_meas_from_plugin_input(&input);
    if let Err(s) = &r1 {
        error = Some(s.clone());
    }
    let mut dyn_topo: Vec<Vec<u64>>;
    // terminal, cn, tn, dev
    let mut dev_topo: Vec<Vec<u64>>;
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
        }
    }
    if error.is_none() {
        let output = PluginOutput {
            error_msg: error,
            schema: None,
            csv_bytes: vec![],
        };
        get_wasm_result(output)
    } else {
        let mut csv_str = String::from("cn,tn\n");
        // build schema
        let schema = Schema::new(vec![
            Field::new("cn", DataType::UInt64, false),
            Field::new("tn", DataType::UInt64, false),
        ]);
        let csv_bytes = vec![("".to_string(), csv_str.into_bytes())];
        let output = PluginOutput {
            error_msg: None,
            schema: Some(vec![schema]),
            csv_bytes,
        };
        get_wasm_result(output)
    }
}