use std::collections::HashMap;

use csv::StringRecordsIter;

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
                            return Err(format!("Wrong terminal input, row {row} col {col}"));
                        }
                    } else if col == 1 {
                        if let Ok(id) = str.parse() {
                            terminal = id;
                        } else {
                            return Err(format!("Wrong terminal input, row {row} col {col}"));
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
    Ok(meas)
}