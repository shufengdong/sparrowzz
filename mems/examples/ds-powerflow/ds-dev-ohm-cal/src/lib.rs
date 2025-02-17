use std::collections::HashMap;

use arrow_schema::{DataType, Field, Schema};
use bytes::{Buf, BufMut, BytesMut};
use csv::StringRecordsIter;
use log::{info, warn};
use ndarray::{Array2, ArrayBase, Ix2, OwnedRepr};

use eig_domain::prop::PropValue;
use mems::model::{get_csv_str, get_df_from_in_plugin, get_island_from_plugin_input, PluginInput, PluginOutput};
use mems::model::dev::PsRsrType;

static mut OUTPUT: Vec<u8> = vec![];
#[no_mangle]
pub unsafe fn run(ptr: i32, len: u32) -> u64 {
    info!("Read plugin input firstly");
    // 从内存中获取字符串
    let input = unsafe {
        let slice = std::slice::from_raw_parts(ptr as _, len as _);
        let input: PluginInput = ciborium::from_reader(slice).unwrap();
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
                                    let u_re = vec![1., 0., 0., 0., 1., 0., 0., 0., 1.];
                                    let u_im = vec![0., 0., 0., 0., 0., 0., 0., 0., 0.];
                                    let (z_re, _) = (mat_re * ratio).into_raw_vec_and_offset();
                                    let (z_im, _) = (mat_im * ratio).into_raw_vec_and_offset();
                                    let u_re_json = get_csv_str(&serde_json::to_string(&u_re).unwrap());
                                    let u_im_json = get_csv_str(&serde_json::to_string(&u_im).unwrap());
                                    let z_re_json = get_csv_str(&serde_json::to_string(&z_re).unwrap());
                                    let z_im_json = get_csv_str(&serde_json::to_string(&z_im).unwrap());
                                    csv_str.push_str(&format!("{dev_id},{u_re_json},{u_im_json},{z_re_json},{z_im_json},{u_im_json},{u_im_json},{z_re_json},{z_im_json}\n"));
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
    let output = if error.is_none() {
        // build schema
        let schema = Schema::new(vec![
            Field::new("dev_id", DataType::UInt64, false),
            Field::new("a_re", DataType::Utf8, false),
            Field::new("a_im", DataType::Utf8, false),
            Field::new("b_re", DataType::Utf8, false),
            Field::new("b_im", DataType::Utf8, false),
            Field::new("c_re", DataType::Utf8, false),
            Field::new("c_im", DataType::Utf8, false),
            Field::new("d_re", DataType::Utf8, false),
            Field::new("d_im", DataType::Utf8, false),
        ]);
        let csv_bytes = vec![("".to_string(), csv_str.into_bytes())];
        PluginOutput {
            error_msg: None,
            schema: Some(vec![schema]),
            csv_bytes,
        }
    } else {
        PluginOutput {
            error_msg: error,
            schema: None,
            csv_bytes: vec![],
        }
    };
    ciborium::into_writer(&output, &mut OUTPUT).unwrap();
    let offset = OUTPUT.as_ptr() as i32;
    let len = OUTPUT.len() as u32;
    let mut bytes = BytesMut::with_capacity(8);
    bytes.put_i32(offset);
    bytes.put_u32(len);
    bytes.get_u64()
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