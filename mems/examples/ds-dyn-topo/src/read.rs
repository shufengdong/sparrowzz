use std::collections::HashMap;
use csv::StringRecordsIter;
use mems::model::dev::PsRsrType;

pub(crate) fn read_points(records: &mut StringRecordsIter<&[u8]>) -> Result<Vec<Vec<u64>>, String> {
    let mut points = Vec::new();
    // 按行读取csv
    let mut row = 0;
    loop {
        points.push(vec![0u64; 2]);
        match records.next() {
            Some(Ok(record)) => {
                let mut col = 0;
                for str in record.iter() {
                    if let Ok(id) = str.parse() {
                        points[row][col] = id;
                    } else {
                        return Err(format!("Wrong point input, row {row} col {col}"));
                    }
                    col += 1;
                    if col == 2 {
                        break;
                    }
                }
                if col != 2 {
                    return Err(format!("Wrong point input, expected col at least 2, actual {col}"));
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

pub(crate) fn read_terminals(records: &mut StringRecordsIter<&[u8]>) -> Result<Vec<Vec<u64>>, String> {
    let mut terminals: Vec<Vec<u64>> = Vec::new();
    // 按行读取csv
    let mut row = 0;
    loop {
        terminals.push(vec![0u64; 3]);
        match records.next() {
            Some(Ok(record)) => {
                let mut col = 0;
                for str in record.iter() {
                    if let Ok(id) = str.parse() {
                        terminals[row][col] = id;
                    } else {
                        return Err(format!("Wrong terminal input, row {row} col {col}"));
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

pub(crate) fn read_edges(records: &mut StringRecordsIter<&[u8]>)
              -> Result<(Vec<Vec<u64>>, HashMap<u64, bool>), String> {
    let mut edges = Vec::new();
    let mut normal_open = HashMap::new();
    let mut row = 0;
    let swich_type = PsRsrType::Switch as u16;
    // 按行读取csv
    loop {
        edges.push(vec![0u64; 3]);
        match records.next() {
            Some(Ok(record)) => {
                let mut col = 0;
                let mut is_switch = false;
                for str in record.iter() {
                    if col < 3 {
                        if let Ok(id) = str.parse() {
                            edges[row][col] = id;
                        } else {
                            return Err(format!("Wrong static topology input, row {row} col {col}"));
                        }
                    } else if col == 3 {
                        if let Ok(type_u16) = str.parse::<u16>() {
                            is_switch = type_u16 == swich_type;
                        } else {
                            return Err(format!("Wrong static topology input, row {row} col {col}"));
                        }
                    } else if col == 4 && is_switch {
                        if let Ok(b) = str.parse::<bool>() {
                            normal_open.insert(edges[row][2], b);
                        } else {
                            return Err(format!("Wrong static topology input, row {row} col {col}"));
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
    Ok((edges, normal_open))
}