use std::collections::{HashMap, HashSet};
use arrow_schema::{DataType, Field, Schema};

use ds_common::{DEV_TOPO_DF_NAME, DYN_TOPO_DF_NAME, POINT_DF_NAME, STATIC_TOPO_DF_NAME, TERMINAL_DF_NAME};
use ds_common::dyn_topo::{read_dev_topo, read_dyn_topo};
use ds_common::static_topo::{read_point_terminal, read_static_topo, read_terminal_cn_dev};
use eig_domain::DataUnit;
use mems::model::{get_meas_from_plugin_input, get_wasm_result, PluginInput, PluginOutput};
use mems::model::dev::{MeasPhase, PsRsrType};

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
    // static topo
    // point, terminal
    let mut points: Vec<Vec<u64>> = vec![];
    let mut meas_phase: Vec<MeasPhase> = vec![];
    // terminal, cn, dev
    let mut terminals: Vec<Vec<u64>> = vec![];
    // dev id to device type
    let mut dev_type: HashMap<u64, u16> = HashMap::new();
    // dynamic topo
    let mut dyn_topo: Vec<Vec<u64>>;
    // terminal, cn, tn, dev
    let mut dyn_dev_topo: Vec<Vec<u64>> = vec![];
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
                Ok(v) => dyn_dev_topo = v,
                Err(s) => {
                    error = Some(s);
                    break;
                }
            }
        } else if input.dfs[i] == STATIC_TOPO_DF_NAME {
            match read_static_topo(&mut records, None, Some(&mut dev_type)) {
                Ok(_) => {},
                Err(s) => error = Some(s),
            }
        } else if input.dfs[i] == TERMINAL_DF_NAME {
            match read_terminal_cn_dev(&mut records) {
                Ok(v) => terminals = v,
                Err(s) => error = Some(s),
            }
        } else if input.dfs[i] == POINT_DF_NAME {
            match read_point_terminal(&mut records, Some(&mut meas_phase)) {
                Ok(v) => points = v,
                Err(s) => error = Some(s),
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
        let type1 = PsRsrType::SyncGenerator as u16;
        let type2 = PsRsrType::Load as u16;
        let type3 = PsRsrType::ShuntCompensator as u16;
        let shunt_types = [type1, type2, type3];
        let mut terminal_with_shunt_dev = HashSet::new();
        for v in terminals {
            let terminal = v[0];
            let dev_id = v[2];
            if let Some(dev_type) = dev_type.get(&dev_id) {
                if shunt_types.contains(&dev_type) {
                    terminal_with_shunt_dev.insert(terminal);
                }
            }
        }
        let mut point_terminal = HashMap::with_capacity(points.len());
        for i in 0..points.len() {
            let point_id = points[i][0];
            let terminal = points[i][1];
            if terminal_with_shunt_dev.contains(&terminal) {
                let phase = meas_phase[i].to_string();
                csv_str.push_str(&format!("{point_id},{terminal},{phase}\n"));
                point_terminal.insert(point_id, (terminal, meas_phase[i].clone()));
            }
        }

        let (meas, units) = r1.unwrap();
        for v in dyn_dev_topo {
            let terminal = v[0];
            let tn = v[2];
            let dev = v[3];
            let dev_type = v[4] as u16;
        }
        // 开始处理开关量
        for m in meas {
            if let Some((terminal, phase)) = point_terminal.get(&m.point_id) {
                if terminal_with_shunt_dev.contains(terminal) {
                    if let Some(unit) = units.get(&m.point_id) {
                        match unit {
                            DataUnit::A => {}
                            DataUnit::V => {}
                            DataUnit::kV => {}
                            DataUnit::W => {}
                            DataUnit::kW => {}
                            DataUnit::MW => {}
                            DataUnit::Var => {}
                            DataUnit::kVar => {}
                            DataUnit::MVar => {}
                            _ => {}
                        }
                    }
                }
            }
        }
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