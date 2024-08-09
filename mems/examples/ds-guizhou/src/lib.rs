use arrow_schema::{DataType, TimeUnit, Field, Schema};
use bytes::{Buf, BufMut, BytesMut};
use log::info;
use mems::model::{PluginInput, PluginOutput};
use chrono::{Days, Duration, Timelike, Utc};
use chrono_tz::Asia::Shanghai;

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
    let schema = Schema::new(vec![
        Field::new("datetime", DataType::Timestamp(TimeUnit::Millisecond, Some("Asia/Shanghai".into())), false),
        Field::new("value", DataType::Float64, false),
    ]);

    let mut csv_str = String::from("datetime,value\n");

    let mut rdr = csv::ReaderBuilder::new().has_headers(true).from_reader(&*input.bytes);
    let records = rdr.records();
    let mut values = vec![];
    for (i, record) in records.enumerate() {
        if let Ok(f) = record {
            let s = f.get(0).unwrap().trim();
            let value = s.parse::<f64>().unwrap();
            values.push(value);
        }
    }

    let now = Utc::now().with_timezone(&Shanghai);
    let today = now.date_naive();
    let starttime = if values.len() >= 24 {
        let startday = if now.hour() < 1 {
            today
        } else {
            today.checked_add_days(Days::new(1)).unwrap()
        };
        startday.and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(Shanghai)
        .unwrap()
    } else {
        let minutes = now.minute() / 15 * 15;
        let time0 = today.and_hms_opt(now.hour(), minutes, 0)
            .unwrap()
            .and_local_timezone(Shanghai)
            .unwrap();
        time0 + Duration::minutes(15)
    };

    for (i, value) in values.into_iter().enumerate() {
        let date = starttime + Duration::minutes((15 * i) as i64);
        let date_str = date.format("%Y-%m-%d %H:%M:%S");
        csv_str.push_str(&format!("{date_str}, {value}\n"));
    }

    let csv_bytes = vec![("".to_string(), csv_str.into_bytes())];
    let output = PluginOutput {
            error_msg: None,
            schema: Some(vec![schema]),
            csv_bytes,
        };
    #[allow(static_mut_refs)]
    ciborium::into_writer(&output, &mut OUTPUT).unwrap();
    let offset = OUTPUT.as_ptr() as i32;
    let len = OUTPUT.len() as u32;
    let mut bytes = BytesMut::with_capacity(8);
    bytes.put_i32(offset);
    bytes.put_u32(len);
    return bytes.get_u64();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let now = Utc::now().with_timezone(&Shanghai);
        let today = now.date_naive();
        let startday = if now.hour() < 1 {
            today
        } else {
            today.checked_add_days(chrono::Days::new(1)).unwrap()
        };
        let a = startday.and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(Shanghai)
        .unwrap();
        println!("{:?}", a);
    }
}