use std::collections::HashMap;
use arrow_schema::{DataType, Field, Schema};
use csv::StringRecordsIter;
use log::{info, warn};
use ndarray::{Array2, ArrayBase, Ix2, OwnedRepr};
use eig_domain::PropValue;
use mems::model::{get_csv_str, get_df_from_in_plugin, get_island_from_plugin_input, get_wasm_result, PluginInput, PluginOutput};
use mems::model::dev::PsRsrType;

#[no_mangle]
pub unsafe fn run(ptr: i32, len: u32) -> u64 {
    info!("Read plugin input firstly");
    // 从内存中获取字符串
    let input = unsafe {
        let slice = std::slice::from_raw_parts(ptr as _, len as _);
        let input: PluginInput = serde_cbor::from_slice(slice).unwrap();
        input
    };
    let mut error = None;
    let r = get_island_from_plugin_input(&input);
    if let Err(s) = &r {
        error = Some(s.clone());
    }
    let r2 = get_df_from_in_plugin(&input);
    if let Err(s) = &r2 {
        error = Some(s.clone());
    }
    let mut config= HashMap::with_capacity(0);
    let mut csv_str = String::from("dev_id,ohm\n");
    if error.is_none() {
        let (island, prop_defs, defines) = r.unwrap();
        let from = r2.unwrap();
        info!("input dataframe num from edges is {}", input.dfs.len());
        for i in 0..input.dfs_len.len() {
            let size = input.dfs_len[i] as usize;
            let end = from + size;
            let mut rdr = csv::ReaderBuilder::new().has_headers(true).from_reader(&input.bytes[from..end]);
            let mut records = rdr.records();
            match read_config( &mut records) {
                Ok(v) => config = v,
                Err(s) => error = Some(s),
            }
            break;
        }
        if error.is_none() {
            let mut prop_defines = HashMap::with_capacity(prop_defs.len());
            for def in prop_defs.into_iter() {
                prop_defines.insert(def.id, def);
            }
            for (_, rsr) in &island.resources {
                if let Some(def) = defines.get(&rsr.define_id) {
                    if def.rsr_type == PsRsrType::ACline {
                        let dev_id = rsr.id;
                        let line_conf = rsr.get_prop_value("model", &island.prop_groups, &prop_defines);
                        let length = rsr.get_prop_value("length", &island.prop_groups, &prop_defines);
                        if let PropValue::Str(s) = line_conf {
                            if let Some((mat_re, mat_im)) = config.get(&s) {
                                if let Some(f) = length.get_f64() {
                                    let ratio = f / 1000.0;
                                    let mut v1 = (mat_re * ratio).into_raw_vec();
                                    let v2 = (mat_im * ratio).into_raw_vec();
                                    v1.extend(v2);
                                    let s = get_csv_str(&serde_json::to_string(&v1).unwrap());
                                    csv_str.push_str(&format!("{dev_id},{s}\n"));
                                } else {
                                    warn!("Length is not set for acline {}", rsr.name);
                                    continue;
                                }
                            } else {
                                warn!("!!Failed to find matrix for line_conf: {s}");
                            }
                        }
                    }
                    // todo: add other types: transformer\regulator
                }
            }
        }
    }
    if error.is_none() {
        // build schema
        let schema = Schema::new(vec![
            Field::new("dev_id", DataType::UInt64, false),
            Field::new("ohm", DataType::Utf8, false),
        ]);
        let csv_bytes = vec![("".to_string(), csv_str.into_bytes())];
        let output = PluginOutput {
            error_msg: None,
            schema: Some(vec![schema]),
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

fn read_config(records: &mut StringRecordsIter<&[u8]>)
    -> Result<HashMap<String, (ArrayBase<OwnedRepr<f64>, Ix2>, ArrayBase<OwnedRepr<f64>, Ix2>)>, String>{
    let mut config: HashMap<String, (ArrayBase<OwnedRepr<f64>, Ix2>, ArrayBase<OwnedRepr<f64>, Ix2>)> = HashMap::new();
    // 按行读取csv
    loop {
        match records.next() {
            Some(Ok(record)) => {
                let mut col = 0;
                let mut name = "".to_string();
                let mut ohm_per_km = "".to_string();
                for str in record.iter() {
                    if col == 0 {
                        name = str.to_string();
                    } else {
                        ohm_per_km = str.to_string();
                    }
                    col += 1;
                    if col == 2 {
                        break;
                    }
                }
                if col != 2 {
                    return Err(format!("Wrong config input, expected col more than 2, actual {col}"));
                }
                match serde_json::from_str::<[f64; 18]>(&ohm_per_km) {
                    Ok(ohm) => {
                        let mat_re = Array2::from_shape_vec((3, 3), ohm[0..9].to_vec()).unwrap();
                        let mat_im = Array2::from_shape_vec((3, 3), ohm[9..18].to_vec()).unwrap();
                        config.insert(name, (mat_re, mat_im));
                    }
                    Err(e) => {
                        return Err(format!("Failed to parse matrix from {ohm_per_km}, err: {:?}", e));
                    }
                }
            }
            Some(Err(e)) => {
                return Err(format!("{:?}", e));
            }
            None => {
                break;
            }
        }
    }
    Ok(config)
}