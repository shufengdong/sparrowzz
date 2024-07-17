use std::collections::HashMap;
use csv::StringRecordsIter;
use mems::model::dev::{MeasPhase, PsRsrType};

pub fn read_point_terminal(records: &mut StringRecordsIter<&[u8]>,
                           mut meas_phase: Option<&mut Vec<MeasPhase>>) -> Result<Vec<Vec<u64>>, String> {
    let mut points = Vec::new();
    // 按行读取csv
    let mut row = 0;
    loop {
        match records.next() {
            Some(Ok(record)) => {
                points.push(vec![0u64; 2]);
                let mut col = 0;
                for str in record.iter() {
                    if col < 2 {
                        if let Ok(id) = str.parse() {
                            points[row][col] = id;
                        } else {
                            return Err(format!("Wrong point input, row {row} col {col}"));
                        }

                    } else if meas_phase.is_some() {
                        meas_phase.as_mut().unwrap().push(MeasPhase::from(str))
                    }
                    col += 1;
                    if col == 3 {
                        break;
                    }
                }
                if col != 3 {
                    return Err(format!("Wrong point input, expected col at least 3, actual {col}"));
                }
            }
            Some(Err(e)) => {
                return Err(format!("Wrong point input, err: {:?}", e));
            }
            None => {
                break;
            }
        }
        row += 1;
    }
    Ok(points)
}

pub fn read_terminal_cn_dev(records: &mut StringRecordsIter<&[u8]>) -> Result<Vec<Vec<u64>>, String> {
    let mut terminals: Vec<Vec<u64>> = Vec::new();
    // 按行读取csv
    let mut row = 0;
    loop {
        match records.next() {
            Some(Ok(record)) => {
                terminals.push(vec![0u64; 3]);
                let mut col = 0;
                for str in record.iter() {
                    if let Ok(id) = str.parse() {
                        terminals[row][col] = id;
                    } else {
                        return Err(format!("Wrong terminal input, row {row} col {col}: {str}"));
                    }
                    col += 1;
                    if col == 3 {
                        break;
                    }
                }
                if col != 3 {
                    return Err(format!("Wrong terminal input, expected col at least 3, actual {col}"));
                }
            }
            Some(Err(e)) => {
                return Err(format!("Wrong terminal input, err: {:?}", e));
            }
            None => {
                break;
            }
        }
        row += 1;
    }
    Ok(terminals)
}

pub fn read_static_topo(records: &mut StringRecordsIter<&[u8]>,
                        mut normal_open: Option<&mut HashMap<u64, bool>>,
                        mut dev_type: Option<&mut HashMap<u64, u16>>)
                        -> Result<Vec<Vec<u64>>, String> {
    let mut edges = Vec::new();
    let mut row = 0;
    let swich_type = PsRsrType::Switch as u16;
    // 按行读取csv
    loop {
        match records.next() {
            Some(Ok(record)) => {
                edges.push(vec![0u64; 3]);
                let mut col = 0;
                let mut is_switch = false;
                for str in record.iter() {
                    if col < 3 {
                        if let Ok(id) = str.parse() {
                            edges[row][col] = id;
                        } else {
                            return Err(format!("Wrong static topology input, row {row} col {col}: {str}"));
                        }
                    } else if col == 3 {
                        if let Ok(type_u16) = str.parse::<u16>() {
                            is_switch = type_u16 == swich_type;
                            if dev_type.is_some() {
                                dev_type.as_mut().unwrap().insert(edges[row][2], type_u16);
                            }
                        } else {
                            return Err(format!("Wrong static topology input, row {row} col {col}: {str}"));
                        }
                    } else if col == 4 && is_switch && normal_open.is_some() {
                        if let Ok(b) = str.parse::<bool>() {
                            normal_open.as_mut().unwrap().insert(edges[row][2], b);
                        } else {
                            return Err(format!("Wrong static topology input, row {row} col {col}: {str}"));
                        }
                    }
                    col += 1;
                    if col == 5 {
                        break;
                    }
                }
                if col != 5 {
                    return Err(format!("Wrong static topology input, expected col at least 5, actual {col}"));
                }
            }
            Some(Err(e)) => {
                return Err(format!("Wrong  static topology input, err: {:?}", e));
            }
            None => {
                break;
            }
        }
        row += 1;
    }
    Ok(edges)
}