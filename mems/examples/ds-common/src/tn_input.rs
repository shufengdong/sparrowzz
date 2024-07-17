use std::collections::HashMap;

use csv::StringRecordsIter;
use eig_domain::DataUnit;
use mems::model::dev::MeasPhase;

pub fn read_shunt_measures(records: &mut StringRecordsIter<&[u8]>)
                           -> Result<HashMap<u64, (u64, MeasPhase)>, String> {
    let mut meas = HashMap::new();
    // 按行读取csv
    let mut row = 0;
    loop {
        match records.next() {
            Some(Ok(record)) => {
                let mut col = 0;
                let mut point = 0u64;
                let mut terminal = 0u64;
                for str in record.iter() {
                    if col == 0 {
                        if let Ok(id) = str.parse() {
                            point = id;
                        } else {
                            return Err(format!("Wrong shunt measure input, row {row} col {col}"));
                        }
                    } else if col == 1 {
                        if let Ok(id) = str.parse() {
                            terminal = id;
                        } else {
                            return Err(format!("Wrong shunt measure input, row {row} col {col}"));
                        }
                    } else if col == 2 {
                        meas.insert(point, (terminal, MeasPhase::from(str)));
                    }
                    col += 1;
                    if col == 3 {
                        break;
                    }
                }
                if col != 3 {
                    return Err(format!("Wrong shunt measure input, expected col at least 3, actual {col}"));
                }
            }
            Some(Err(e)) => {
                return Err(format!("Wrong shunt measure input, err: {:?}", e));
            }
            None => {
                break;
            }
        }
        row += 1;
    }
    Ok(meas)
}

pub fn read_tn_input(records: &mut StringRecordsIter<&[u8]>)
                            -> Result<(Vec<u64>, Vec<MeasPhase>, Vec<DataUnit>, Vec<f64>), String> {
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