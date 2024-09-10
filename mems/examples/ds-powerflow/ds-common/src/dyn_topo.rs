use csv::StringRecordsIter;

pub fn read_dyn_topo(records: &mut StringRecordsIter<&[u8]>)
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
pub fn read_dev_topo(records: &mut StringRecordsIter<&[u8]>)
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
