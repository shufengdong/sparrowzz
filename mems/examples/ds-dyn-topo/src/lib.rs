use std::collections::HashMap;

use arrow_schema::{DataType, Field, Schema};

use ds_common::{DEV_TOPO_DF_NAME, DYN_TOPO_DF_NAME, POINT_DF_NAME, STATIC_TOPO_DF_NAME, TERMINAL_DF_NAME};
use ds_common::static_topo::{read_edges, read_points, read_terminals};
use eig_domain::DataUnit;
use mems::model::{get_df_from_in_plugin, get_meas_from_plugin_input, get_wasm_result, ModelType, PluginInput, PluginOutput};

#[no_mangle]
pub unsafe fn run(ptr: i32, len: u32) -> u64 {
    // 从内存中获取字符串
    let input = unsafe {
        let slice = std::slice::from_raw_parts(ptr as _, len as _);
        let input: PluginInput = serde_cbor::from_slice(slice).unwrap();
        input
    };
    let mut error = None;
    let r1 = get_meas_from_plugin_input(&input);
    if let Err(s) = &r1 {
        error = Some(s.clone());
    }
    let r2 = get_df_from_in_plugin(&input);
    // source, target, dev
    let mut edges: Vec<Vec<u64>> = vec![];
    // switch id to normal open
    let mut normal_open: HashMap<u64, bool> = HashMap::with_capacity(0);
    // terminal, cn, dev
    let mut terminals: Vec<Vec<u64>> = vec![];
    // point, terminal
    let mut points: Vec<Vec<u64>> = vec![];
    if error.is_none() {
        if let Err(s) = &r2 {
            error = Some(s.clone());
        } else {
            let mut from = r2.unwrap();
            for i in 0..input.dfs_len.len() {
                let size = input.dfs_len[i] as usize;
                let end = from + size;
                let mut rdr = csv::ReaderBuilder::new().has_headers(true).from_reader(&input.bytes[from..end]);
                let mut records = rdr.records();
                // 开始读取输入的static topology DataFrame
                if input.dfs[i] == STATIC_TOPO_DF_NAME {
                    match read_edges(&mut records) {
                        Ok(v) => (edges, normal_open) = v,
                        Err(s) => error = Some(s),
                    }
                } else if input.dfs[i] == TERMINAL_DF_NAME {
                    match read_terminals(&mut records) {
                        Ok(v) => terminals = v,
                        Err(s) => error = Some(s),
                    }
                } else if input.dfs[i] == POINT_DF_NAME {
                    match read_points(&mut records) {
                        Ok(v) => points = v,
                        Err(s) => error = Some(s),
                    }
                }
                from += size;
            }
        }
    }
    if error.is_none() {
        let (meas, units) = r1.unwrap();
        let mut point_map: HashMap<u64, u64> = HashMap::with_capacity(points.len());
        let mut terminal_cn: HashMap<u64, u64> = HashMap::with_capacity(points.len());
        let mut terminal_dev: HashMap<u64, u64> = HashMap::with_capacity(points.len());
        for ids in points {
            point_map.insert(ids[0], ids[1]);
        }
        for ids in terminals {
            terminal_cn.insert(ids[0], ids[1]);
            terminal_dev.insert(ids[0], ids[2]);
        }
        // 开始构建
        let mut closed_switch_to_cn: HashMap<u64, u64> = HashMap::with_capacity(terminal_cn.len() / 2);
        // 开始处理开关量
        for m in meas {
            if let Some(unit) = units.get(&m.point_id) {
                if DataUnit::OnOrOff == *unit {
                    if let Some(terminal_id) = point_map.get(&m.point_id) {
                        if let Some(cn_id) = terminal_cn.get(terminal_id) {
                            if let Some(dev_id) = terminal_dev.get(terminal_id) {
                                if m.get_value2() > 0 {
                                    closed_switch_to_cn.insert(*dev_id, *cn_id);
                                }
                            }
                        }
                    }
                }
            }
        }
        // build tns
        let mut cn_tn: HashMap<u64, usize> = HashMap::with_capacity(terminal_cn.len() / 2);
        let mut not_dealed = Vec::new();
        for v in edges {
            let cn1 = v[0];
            let cn2 = v[1];
            let dev_id = v[2];
            // switch with measure
            if let Some(cn) = closed_switch_to_cn.get(&dev_id) {
                if *cn == cn1 {
                    if let Some(tn) = cn_tn.get(cn) {
                        cn_tn.insert(cn2, *tn);
                    } else {
                        let tn = cn_tn.len() + 1;
                        cn_tn.insert(cn1, tn);
                        cn_tn.insert(cn2, tn);
                    }
                } else if *cn == cn2 {
                    if let Some(tn) = cn_tn.get(cn) {
                        cn_tn.insert(cn1, *tn);
                    } else {
                        let tn = cn_tn.len() + 1;
                        cn_tn.insert(cn1, tn);
                        cn_tn.insert(cn2, tn);
                    }
                }
            }
            // this is a closed switch with no measure
            else if let Some(false) = normal_open.get(&dev_id) {
                if let Some(tn) = cn_tn.get(&cn1) {
                    cn_tn.insert(cn2, *tn);
                } else if let Some(tn) = cn_tn.get(&cn2) {
                    cn_tn.insert(cn1, *tn);
                } else {
                    let tn = cn_tn.len() + 1;
                    cn_tn.insert(cn1, tn);
                    cn_tn.insert(cn2, tn);
                }
            }
            // this is open switch or not switch
            else {
                if !cn_tn.contains_key(&cn1) {
                    not_dealed.push(cn1);
                }
                if !cn_tn.contains_key(&cn2) {
                    not_dealed.push(cn2);
                }
            }
        }
        for cn in not_dealed {
            if !cn_tn.contains_key(&cn) {
                let tn = cn_tn.len() + 1;
                cn_tn.insert(cn, tn);
            }
        }
        // get outgoing edges
        let mut outgoing = vec![];
        for model_input in &input.model {
            match model_input {
                ModelType::Outgoing(edge_name) => {
                    outgoing = edge_name.clone();
                }
                _ => {}
            }
        }
        let mut csv_bytes = Vec::with_capacity(2);
        let mut schema = Vec::with_capacity(2);
        if outgoing.is_empty() || outgoing.contains(&DYN_TOPO_DF_NAME.to_string()) ||
            (!outgoing.contains(&DYN_TOPO_DF_NAME.to_string()) && !outgoing.contains(&DEV_TOPO_DF_NAME.to_string())) {
            // build topology
            let mut topo_csv = String::from("cn,tn\n");
            for (cn, tn) in &cn_tn {
                topo_csv.push_str(&format!("{cn},{tn}\n"));
            }
            // build topology schema
            let topo_schema = Schema::new(vec![
                Field::new("cn", DataType::UInt64, false),
                Field::new("tn", DataType::UInt64, false),
            ]);
            csv_bytes.push((DYN_TOPO_DF_NAME.to_string(), topo_csv.into_bytes()));
            schema.push(topo_schema);
        }
        if outgoing.contains(&DEV_TOPO_DF_NAME.to_string()) {
            // build dev connection
            let mut dev_csv = String::from("terminal,cn,tn,dev\n");
            for (terminal, dev) in terminal_dev {
                if closed_switch_to_cn.contains_key(&dev) {
                    continue;
                }
                if let Some(cn) = terminal_cn.get(&terminal) {
                    if let Some(tn) = cn_tn.get(cn) {
                        dev_csv.push_str(&format!("{terminal},{cn},{tn},{dev}\n"));
                    }
                }
            }
            // build dev connection schema
            let dev_schema = Schema::new(vec![
                Field::new("terminal", DataType::UInt64, false),
                Field::new("cn", DataType::UInt64, false),
                Field::new("tn", DataType::UInt64, false),
                Field::new("dev", DataType::UInt64, false),
            ]);
            csv_bytes.push((DEV_TOPO_DF_NAME.to_string(), dev_csv.into_bytes()));
            schema.push(dev_schema);
        }
        let output = PluginOutput {
            error_msg: None,
            schema: Some(schema),
            csv_bytes,
        };
        get_wasm_result(output)
    } else {
        let output = PluginOutput {
            error_msg: error,
            schema: None,
            csv_bytes: vec![],
        };
        get_wasm_result(output)
    }
}