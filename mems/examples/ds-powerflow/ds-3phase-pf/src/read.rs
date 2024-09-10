use std::collections::HashMap;

use csv::StringRecordsIter;
use ndarray::Array2;

use eig_domain::prop::DataUnit;

const MAT_SIZE: usize = 18;

pub(crate) fn read_dev_ohm(records: &mut StringRecordsIter<&[u8]>)
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