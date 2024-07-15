use std::collections::HashMap;

use csv::StringRecordsIter;
use ndarray::Array2;

use eig_domain::DataUnit;
use mems::model::dev::MeasPhase;

const MAT_SIZE: usize = 2 * 54;
pub(crate) fn read_dyn_topo(records: &mut StringRecordsIter<&[u8]>)
                            -> Result<Vec<Vec<u64>>, String> {
    let mut dyn_topo = Vec::new();
    // 按行读取csv
    let mut row = 0;
    loop {
        match records.next() {
            Some(Ok(record)) => {
                let mut col = 0;
                dyn_topo.push(vec![0u64; 2]);
                for str in record.iter() {
                    if let Ok(id) = str.parse() {
                        dyn_topo[row][col] = id;
                    } else {
                        return Err(format!("Wrong dynamic topology input, row {row} col {col}"));
                    }
                    col += 1;
                    if col == 2 {
                        break;
                    }
                }
                if col != 2 {
                    return Err(format!("Wrong dynamic topology input, expected col 2, actual {col}"));
                }
            }
            Some(Err(e)) => {
                return Err(format!("Wrong dynamic topology input, err: {:?}", e));
            }
            None => {
                break;
            }
        }
        row += 1;
    }
    Ok(dyn_topo)
}
pub(crate) fn read_dev_topo(records: &mut StringRecordsIter<&[u8]>)
                            -> Result<Vec<Vec<u64>>, String> {
    let mut dev_topo = Vec::new();
    // 按行读取csv
    let mut row = 0;
    loop {
        match records.next() {
            Some(Ok(record)) => {
                let mut col = 0;
                dev_topo.push(vec![0u64; 4]);
                for str in record.iter() {
                    if let Ok(id) = str.parse() {
                        dev_topo[row][col] = id;
                    } else {
                        return Err(format!("Wrong device topology, row {row} col {col}"));
                    }
                    col += 1;
                    if col == 4 {
                        break;
                    }
                }
                if col != 4 {
                    return Err(format!("Wrong device topology input, expected col 4, actual {col}"));
                }
            }
            Some(Err(e)) => {
                return Err(format!("Wrong device topology input, err: {:?}", e));
            }
            None => {
                break;
            }
        }
        row += 1;
    }
    Ok(dev_topo)
}

pub(crate) fn read_dev_matrix(records: &mut StringRecordsIter<&[u8]>)
                              -> Result<HashMap<u64, Vec<Array2<f64>>>, String> {
    let mut map = HashMap::new();
    let mut dev_id = 0u64;
    let mut matrix: Vec<f64> = Vec::with_capacity(MAT_SIZE);
    let mut row = 0;
    loop {
        match records.next() {
            Some(Ok(record)) => {
                let mut col = 0;
                for str in record.iter() {
                    if col == 0 {
                        if let Ok(id) = str.parse() {
                            if dev_id != id {
                                if dev_id != 0 {
                                    if matrix.len() != MAT_SIZE {
                                        return Err(format!("matrix len must be {MAT_SIZE}"));
                                    } else {
                                        let v = create_rx(&matrix);
                                        map.insert(dev_id, v);
                                    }
                                }
                                dev_id = id;
                                matrix.clear();
                            }
                        } else {
                            return Err(format!("Wrong dev matrix, row {row} col {col}"));
                        }
                    } else {
                        if let Ok(f) = str.parse() {
                            matrix.push(f);
                        } else {
                            return Err(format!("Wrong dev matrix, row {row} col {col}"));
                        }
                    }
                    col += 1;
                }
            }
            Some(Err(e)) => {
                return Err(format!("Wrong dev matrix, err: {:?}", e));
            }
            None => {
                break;
            }
        }
        row += 1;
    }
    if dev_id != 0 {
        if matrix.len() != MAT_SIZE {
            return Err(format!("matrix len must be {MAT_SIZE}"));
        } else {
            let v = create_rx(&matrix);
            map.insert(dev_id, v);
        }
    }
    Ok(map)
}

pub(crate) fn read_tn_input(records: &mut StringRecordsIter<&[u8]>)
                            -> Result<(Vec<u64>, Vec<Vec<MeasPhase>>, Vec<Vec<DataUnit>>, Vec<Vec<f64>>), String> {
    let mut tn = Vec::new();
    let mut input_type = Vec::new();
    let mut input_phase = Vec::new();
    let mut value = Vec::new();
    // 按行读取csv
    let mut row = 0;
    loop {
        match records.next() {
            Some(Ok(record)) => {
                let mut col = 0;
                for str in record.iter() {
                    if col == 0 {
                        if let Ok(v) = str.parse() {
                            tn.push(v);
                        } else {
                            return Err(format!("Wrong bus input, row {row} col {col}"));
                        }
                    } else if col == 1 {
                        if let Ok(v) = serde_json::from_str(str) {
                            input_phase.push(v);
                        } else {
                            return Err(format!("Wrong bus input, row {row} col {col}"));
                        }
                    } else if col == 2 {
                        if let Ok(v) = serde_json::from_str(str) {
                            input_type.push(v);
                        } else {
                            return Err(format!("Wrong bus input, row {row} col {col}"));
                        }
                    } else if col == 3 {
                        if let Ok(v) = serde_json::from_str(str) {
                            value.push(v);
                        } else {
                            return Err(format!("Wrong bus input, row {row} col {col}"));
                        }
                    }
                    col += 1;
                    if col == 4 {
                        break;
                    }
                }
                if col != 4 {
                    return Err(format!("Wrong bus input, expected col 4, actual {col}"));
                }
            }
            Some(Err(e)) => {
                return Err(format!("Wrong bus input, err: {:?}", e));
            }
            None => {
                break;
            }
        }
        row += 1;
    }
    Ok((tn, input_phase, input_type, value))
}


fn create_rx(matrix: &[f64]) -> Vec<Array2<f64>> {
    let r = Array2::from_shape_vec((3, 3), matrix[0..9].to_vec()).unwrap();
    let x = Array2::from_shape_vec((3, 3), matrix[9..18].to_vec()).unwrap();
    let v = vec![r, x];
    v
}

#[test]
fn test_unit_parse() {
    let units = vec![DataUnit::kVar, DataUnit::kW];
    let s = serde_json::to_string(&units).unwrap();
    assert_eq!("[\"kVar\",\"kW\"]", s);
    let r: Vec<DataUnit> = serde_json::from_str("[\"kVar\", \"kW\"]").unwrap();
    assert_eq!(r, units);
}