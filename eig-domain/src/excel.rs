use std::collections::HashMap;
use std::io::{Cursor, Write};
use std::path::Path;

use calamine::{open_workbook_auto_from_rs, Data, Reader, Sheets, Xlsx, open_workbook_from_rs};

pub fn excel_to_csv_bytes<P: AsRef<Path>>(path: P) -> Option<Vec<Vec<u8>>> {
    let bytes = std::fs::read(path).ok()?;
    excel_bytes_to_csv_bytes(bytes.as_slice())
}

pub fn get_first_sheet_merged_cells(bytes: Vec<u8>) -> Option<(u32, u32, HashMap<(u32,u32), (u32, u32)>)> {
    let c = Cursor::new(bytes);
    let mut excel: Xlsx<_> = open_workbook_from_rs(c).ok()?;
    excel.load_merged_regions().ok()?;
    let sheet_names = excel.sheet_names();
    let mut max_col = 0;
    if sheet_names.len() > 0 {
        let v = excel.merged_regions_by_sheet(&sheet_names[0]);
        let mut merged_cells = HashMap::with_capacity(v.len());
        for (_, _, c) in v {
            merged_cells.insert(c.start, c.end);
            if c.end.1 > max_col {
                max_col = c.end.1;
            }
        }
        let range = excel.worksheet_range_ref(&sheet_names[0]).ok()?;
        let (m, w) = range.get_size();
        let n = if w as u32 > max_col + 1 { w as u32 } else { max_col + 1 };
        return Some((m as u32, n, merged_cells));
    }
    None
}

pub fn excel_bytes_to_csv_bytes(bytes: &[u8]) -> Option<Vec<Vec<u8>>> {
    let c = Cursor::new(bytes.to_vec());
    if let Ok(mut xl) = open_workbook_auto_from_rs(c) {
        let mut sheet_names = xl.sheet_names();
        sheet_names.retain(|name| !name.starts_with('_'));
        sheets_to_csv(&mut xl, sheet_names)
    } else {
        let is_csv = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(bytes)
        .records()
        .next().is_some_and(|x| x.is_ok());
        if is_csv {
            Some(vec![bytes.to_vec()])
        } else {
            None
        }
    }
}

pub fn excel_bytes_to_csv_bytes_by_sheet_names(
    bytes: &[u8],
    names: Vec<String>,
) -> Option<Vec<Vec<u8>>> {
    let c = Cursor::new(bytes.to_vec());
    let mut xl = open_workbook_auto_from_rs(c).ok()?;
    sheets_to_csv(&mut xl, names)
}

fn sheets_to_csv<T>(xl: &mut Sheets<T>, names: Vec<String>) -> Option<Vec<Vec<u8>>>
where
    T: std::io::Read + std::io::Seek,
{
    let mut result = Vec::with_capacity(names.len());
    for name in names {
        let range = xl.worksheet_range(name.as_str()).ok()?;
        let n = range.get_size().1 - 1;
        let mut dest = Vec::new();
        for r in range.rows() {
            for (i, c) in r.iter().enumerate() {
                match *c {
                    Data::Empty => Ok(()),
                    Data::String(ref s) => {
                        if s.contains(',')
                            || s.contains('\r')
                            || s.contains('\n')
                            || s.contains('"')
                        {
                            let new_s = s.replace('\"', "\"\"");
                            write!(dest, "\"{new_s}\"")
                        } else {
                            write!(dest, "{s}")
                        }
                    }
                    Data::Float(ref f) => write!(dest, "{f}"),
                    Data::DateTime(ref data) => write!(dest, "{data}"),
                    Data::DurationIso(ref s) | Data::DateTimeIso(ref s) => write!(dest, "{s}"),
                    Data::Int(ref i) => write!(dest, "{i}"),
                    Data::Error(ref e) => write!(dest, "{:?}", e),
                    Data::Bool(ref b) => write!(dest, "{b}"),
                }
                .ok()?;
                if i != n {
                    write!(dest, ",").ok()?;
                }
            }
            write!(dest, "\r\n").ok()?;
        }
        if !dest.is_empty() {
            result.push(dest);
        }
    }
    Some(result)
}


#[derive(Debug, PartialEq)]
enum FileEncode {
    UTF8,
    UTF16LE,
    UTF16BE,
    GBK,
}

pub fn transfer_to_utf8(data: Vec<u8>) -> Result<Vec<u8>,()> {
    let encode = get_encoding(data.as_slice());
    // encoding_rs::max_utf8_buffer_length
    let mut decoder = match encode {
        FileEncode::UTF8 => encoding_rs::UTF_8.new_decoder(),
        FileEncode::UTF16LE => {
            encoding_rs::UTF_16LE.new_decoder()
        }
        FileEncode::UTF16BE => {
            encoding_rs::UTF_16BE.new_decoder()
        }
        FileEncode::GBK => {
            encoding_rs::GBK.new_decoder()
        }
    };

    let mut result = Vec::with_capacity(
        decoder.max_utf8_buffer_length(data.len()).unwrap()
    );
    result.resize(result.capacity(), 0u8);

    let (_, _, written, has_errors) = decoder.decode_to_utf8(data.as_slice(), &mut result, true);
    if has_errors {
        Err(())
    } else {
        result.truncate(written);
        Ok(result)
    }
}

fn get_encoding(data: &[u8]) -> FileEncode {
    // let data: Vec<u8> = vec![0xFF, 0xFE, 0x41, 0x00, 0x42, 0x00];
    // let data = data.to_owned();

    // let data_clone = data.to_owned();
    let len = data.len();
    if len > 2 && data[0] == 0xFF && data[1] == 0xFE {
        return FileEncode::UTF16LE;
    } else if len > 2 && data[0] == 0xFE && data[1] == 0xFF {
        return FileEncode::UTF16BE;
    } else if len > 3 && data[0] == 0xEF && data[1] == 0xBB && data[2] == 0xBF {
        // UTF8-BOM
        return FileEncode::UTF8;
    } else {
        // 根据编码规则判断编码格式是GBK/UTF-8

        //无文件头根据编码规律来判断编码格式
        //UTF-8的编码规则很简单，只有二条：
        //1）对于单字节的符号，字节的第一位设为0，后面7位为这个符号的unicode码。因此对于英语字母，UTF - 8编码和ASCII码是相同的。
        //2）对于n字节的符号（n>1），第一个字节的前n位都设为1，第n + 1位设为0，后面字节的前两位一律设为10。剩下的没有提及的二进制位，全部为这个符号的unicode码。
        // let mut byte_number = 0;
        let mut utf8_number = 0;
        let mut index = 0;
        while index < len {
            //取第一个字节判断第一位是否为1，以及获取第一位为1时后面位连续为1的数量
            let mut byte_number = 0;
            for i in 0..8 {
                if data[index] & (0b10000000 >> i) != 0 {
                    byte_number += 1;
                } else {
                    break;
                }
            }

            //若byte等于0，则非中文，中文数量清零
            if byte_number == 0 {
                utf8_number = 0;
                index += 1;
            } else if byte_number == 1 || byte_number > 4 {
                return FileEncode::GBK;
            } else {
                //如果该字节开头几位连续为1，且数量byte超过1，则判断d该自己后面byte-1个字节是否符合UTF-8编码规则, 即10开头；
                for i in 1..byte_number {
                    if data[index + i] & 0b11000000 != 0b10000000 {
                        return FileEncode::GBK;
                    }
                }
                //即使满足UTF-8，仍可能为GBK
                //如果连续的UTF-8编码的中文数量超过3个，则判断为utf-8
                utf8_number += 1;
                index += byte_number;

                if utf8_number >= 3 {
                    return FileEncode::UTF8;
                }
            }
        }
    }
    return FileEncode::UTF8;

}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read, Write};

    use super::*;

    #[test]
    fn test_parse() {
        let bytes = std::fs::read("../eig-domain/tests/points-test1.xlsx").unwrap();
        let mut c = Cursor::new(Vec::new());
        c.write_all(bytes.as_slice()).unwrap();
        let mut xl = open_workbook_auto_from_rs(c).unwrap();

        let names = xl.sheet_names().to_owned();
        let bytes = sheets_to_csv(&mut xl, names).unwrap()[0].clone();
        let mut buf = String::new();
        bytes.as_slice().read_to_string(&mut buf).unwrap();
        println!("{}", buf);
    }

    #[test]
    fn test_parse_points() {
        use crate::{from_csv, from_csv_bytes2};

        let path1 = "tests/points-test1.xlsx";
        let path2 = "tests/points-test1.csv";

        let bytes = excel_to_csv_bytes(path1).unwrap()[0].clone();
        let (map1, tags1) = from_csv_bytes2(bytes.as_slice(), true, false).unwrap();

        let (map2, tags2) = from_csv(path2).unwrap();
        for key in map2.keys() {
            assert_eq!(map1.get(key), map2.get(key));
        }
        assert_eq!(tags1, tags2);
    }

    #[test]
    fn test_parse_points2() {
        use crate::{from_csv, from_csv_bytes2};

        let path1 = "tests/points-test1.csv";
        let path2 = "tests/points-test1.csv";

        let bytes = excel_to_csv_bytes(path1).unwrap()[0].clone();
        let (map1, tags1) = from_csv_bytes2(bytes.as_slice(), true, false).unwrap();

        let (map2, tags2) = from_csv(path2).unwrap();
        for key in map2.keys() {
            assert_eq!(map1.get(key), map2.get(key));
        }
        assert_eq!(tags1, tags2);
    }

    // #[test]
    // fn test_parse_aoes() {
    //     use eig_aoe::aoe::parse::{from_csv_bytes, from_file};

    //     let path = "../eig-aoe/tests/test_zq/aoe-test1.xlsx";
    //     let path2 = "../eig-aoe/tests/test_zq/aoe-test1.csv";

    //     let bytes = excel_to_csv_bytes(path).unwrap()[0].clone();
    //     let aoes1 = from_csv_bytes(bytes.as_slice()).unwrap();

    //     let aoes2 = from_file(path2).unwrap();
    //     assert_eq!(3, aoes2.len());
    //     for i in 0..aoes2.len() {
    //         assert_eq!(aoes1[i].model, aoes2[i].model);
    //     }
    // }

    #[test]
    fn test_parse_mbtcps() {
        use crate::modbus::ModbusTcpClientTp;

        let path = "tests/tcp-mbc-test1.xlsx";
        let path2 = "tests/tcp-mbc-test1.csv";

        let bytes = excel_to_csv_bytes(path).unwrap()[0].clone();
        let mut buf = String::new();
        bytes.as_slice().read_to_string(&mut buf).unwrap();
        println!("{}", buf);
        let tps1 = ModbusTcpClientTp::from_csv_bytes2(bytes.as_slice()).unwrap();
        let tps2 = ModbusTcpClientTp::from_csv2(path2).unwrap();

        assert_eq!(tps1, tps2);
    }

    #[test]
    fn test_parse_kanban() {
        let xlsx_bytes = std::fs::read("tests/kanban1.xlsx").unwrap();
        let (m, n, merged_cells) = get_first_sheet_merged_cells(xlsx_bytes).unwrap();
        assert_eq!(m, 6);
        assert_eq!(n, 12);
        let mut is_dealt = vec![false; (m * n) as usize];
        let mut cells = Vec::new();
        for i in 0..m {
            for j in 0..n {
                let index = (i * n + j) as usize;
                if is_dealt[index] {
                    continue;
                }
                let mut class_str = "cell".to_string();
                let coordinate = (i, j);
                if let Some((end_row, end_col)) = merged_cells.get(&coordinate) {
                    let row_span = *end_row - i + 1;
                    let col_span = *end_col - j + 1;
                    if row_span > 1 {
                        class_str.push_str(&format!(" is-row-span-{row_span}"))
                    }
                    if col_span > 1 {
                        class_str.push_str(&format!(" is-col-span-{col_span}"))
                    }
                    cells.push(class_str);
                    for row in i..=*end_row {
                        for col in j..=*end_col {
                            let pos = (row * n + col) as usize;
                            is_dealt[pos] = true;
                        }
                    }
                } else {
                    cells.push(class_str);
                }
            }
        }
        assert_eq!(23, cells.len());
    }

    #[test]
    fn test_parse_kanban2() {
        let xlsx_bytes = std::fs::read("tests/kanban2.xlsx").unwrap();
        let (m, n, _) = get_first_sheet_merged_cells(xlsx_bytes).unwrap();
        assert_eq!(m, 7);
        assert_eq!(n, 12);
        let xlsx_bytes = std::fs::read("tests/kanban3.xlsx").unwrap();
        let (m, n, _) = get_first_sheet_merged_cells(xlsx_bytes).unwrap();
        assert_eq!(m, 7);
        assert_eq!(n, 12);
    }

    #[test]
    fn test_parse_file() {
        let content = std::fs::read("tests/GBK.txt").unwrap();
        let encoding = get_encoding(content.as_slice());
        assert_eq!(encoding, FileEncode::GBK);
        let gbk = transfer_to_utf8(content).unwrap();

        let content = std::fs::read("tests/UTF8.txt").unwrap();
        let encoding = get_encoding(content.as_slice());
        assert_eq!(encoding, FileEncode::UTF8);
        let utf8 = transfer_to_utf8(content).unwrap();
        assert_eq!(gbk, utf8);
    }
}
