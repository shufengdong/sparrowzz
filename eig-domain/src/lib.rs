use std::fmt;
use std::marker::PhantomData;
use csv::StringRecord;
use protobuf::{EnumFull, EnumOrUnknown};
use serde::{Deserialize, Serialize};

use eig_expr::Expr;

use crate::prop::DataUnit;

pub mod prop;
pub mod proto;
pub mod web;
pub mod excel;

/**
 * @api {Measurement} /Measurement Measurement
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} point_id 测点id
 * @apiSuccess {String} point_name 测点名
 * @apiSuccess {String} alias_id 字符串id
 * @apiSuccess {bool} is_discrete 是否是离散量
 * @apiSuccess {bool} is_computing_point 是否是计算点
 * @apiSuccess {String} expression 如果是计算点，这是表达式
 * @apiSuccess {String} trans_expr 变换公式
 * @apiSuccess {String} inv_trans_expr 逆变换公式
 * @apiSuccess {String} change_expr 判断是否"变化"的公式，用于变化上传或储存
 * @apiSuccess {String} zero_expr 判断是否为0值的公式
 * @apiSuccess {String} data_unit 单位
 * @apiSuccess {f64} upper_limit 上限，用于坏数据辨识
 * @apiSuccess {f64} lower_limit 下限，用于坏数据辨识
 * @apiSuccess {String} alarm_level1_expr 告警级别1的表达式
 * @apiSuccess {String} alarm_level2_expr 告警级别2的表达式
 * @apiSuccess {bool} is_realtime 如是，则不判断是否"变化"，均上传
 * @apiSuccess {bool} is_soe 是否是soe点
 * @apiSuccess {u64} init_value 默认值存储在8个字节，需要根据is_discrete来转换成具体的值
 */
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Measurement {
    /// 唯一的id
    pub point_id: u64,
    /// 测点名
    pub point_name: String,
    /// 字符串id
    pub alias_id: String,
    /// 是否是离散量
    pub is_discrete: bool,
    /// 是否是计算点
    pub is_computing_point: bool,
    /// 如果是计算点，这是表达式
    pub expression: String,
    /// 变换公式
    pub trans_expr: String,
    /// 逆变换公式
    pub inv_trans_expr: String,
    /// 判断是否"变化"的公式，用于变化上传或储存
    pub change_expr: String,
    /// 判断是否为0值的公式
    pub zero_expr: String,
    /// 单位
    pub data_unit: String,
    #[serde(skip)]
    pub unit: DataUnit,
    /// 上限，用于坏数据辨识
    pub upper_limit: f64,
    /// 下限，用于坏数据辨识
    pub lower_limit: f64,
    /// 告警级别1的表达式
    pub alarm_level1_expr: String,
    #[serde(skip)]
    pub alarm_level1: Option<Expr>,
    /// 告警级别2的表达式
    pub alarm_level2_expr: String,
    #[serde(skip)]
    pub alarm_level2: Option<Expr>,
    /// 如是，则不判断是否"变化"，均上传
    pub is_realtime: bool,
    /// 是否是soe点
    pub is_soe: bool,
    /// 默认值存储在8个字节，需要根据is_discrete来转换成具体的值
    pub init_value: u64,
    /// Description
    pub desc: String,
    /// 标识该测点是否是采集点，在运行时根据测点是否属于通道来判断
    #[serde(skip)]
    pub is_remote: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MeasureValue {
    /// 对应的测点
    pub point_id: u64,
    /// 是否离散量
    pub is_discrete: bool,
    /// 时间戳
    pub timestamp: u64,
    /// 模拟量值
    pub analog_value: f64,
    /// 离散量值
    pub discrete_value: i64,
    /// 是否已经变换
    pub is_transformed: bool,
    /// 变换后的模拟量值
    pub transformed_analog: f64,
    /// 变换后的离散量值
    pub transformed_discrete: i64,
}


impl MeasureValue {

    pub fn init_discrete_with_time(
        point_id: u64,
        discrete_value: i64,
        timestamp: u64,
    ) -> MeasureValue {
        MeasureValue {
            point_id,
            is_discrete: true,
            timestamp,
            analog_value: 0.0,
            discrete_value,
            is_transformed: false,
            transformed_analog: 0.0,
            transformed_discrete: 0,
        }
    }

    /// 生成浮点数存储的测点值对象
    pub fn init_analog_with_time(point_id: u64, analog_value: f64, timestamp: u64) -> MeasureValue {
        MeasureValue {
            point_id,
            is_discrete: false,
            timestamp,
            analog_value,
            discrete_value: 0,
            is_transformed: false,
            transformed_analog: 0.0,
            transformed_discrete: 0,
        }
    }

    /// 生成bool型测点值对象
    pub fn create_bool_measure(
        point_id: u64,
        b: bool,
        timestamp: u64,
        is_discrete: bool,
    ) -> MeasureValue {
        let discrete_value = if is_discrete {
            if b { 1 } else { 0 }
        } else {
            0
        };
        let analog_value = if !is_discrete {
            if b { 1.0 } else { 0.0 }
        } else {
            0.0
        };
        MeasureValue {
            point_id,
            is_discrete,
            timestamp,
            analog_value,
            discrete_value,
            is_transformed: false,
            transformed_analog: 0.0,
            transformed_discrete: 0,
        }
    }

    /// 取测点的值，如果经过了变换则返回变换后的值
    pub fn get_value(&self) -> f64 {
        if self.is_discrete {
            if self.is_transformed {
                self.transformed_discrete as f64
            } else {
                self.discrete_value as f64
            }
        } else if self.is_transformed {
            self.transformed_analog
        } else {
            self.analog_value
        }
    }

    pub fn get_value2(&self) -> i64 {
        if self.is_discrete {
            if self.is_transformed {
                self.transformed_discrete
            } else {
                self.discrete_value
            }
        } else if self.is_transformed {
            self.transformed_analog as i64
        } else {
            self.analog_value as i64
        }
    }

    /// 计算偏差
    pub fn get_error(&self, new_m: &MeasureValue) -> f64 {
        new_m.get_value() - self.get_value()
    }

    pub fn update_time(&mut self, t: u64) {
        self.timestamp = t;
    }

    /// 更新测点值
    pub fn update(&mut self, new_m: &MeasureValue) {
        // 如果已经修改了类型，不再更新
        if self.is_discrete != new_m.is_discrete {
            return;
        }
        if self.is_discrete {
            self.discrete_value = new_m.discrete_value;
            if new_m.is_transformed {
                self.transformed_discrete = new_m.transformed_discrete;
            }
        } else {
            self.analog_value = new_m.analog_value;
            if new_m.is_transformed {
                self.transformed_analog = new_m.transformed_analog;
            }
        }
        self.is_transformed = new_m.is_transformed;
        self.timestamp = new_m.timestamp;
    }

    pub fn is_same_value(&self, new_m: &MeasureValue) -> bool {
        if new_m.is_discrete {
            new_m.discrete_value == self.discrete_value
        } else if new_m.is_transformed {
            // 比较变换之后的值
            new_m.transformed_analog == self.transformed_analog
        } else {
            new_m.analog_value == self.analog_value
        }
    }
}

fn serialize_enum_or_unknown<E: EnumFull, S: serde::Serializer>(
    e: &Option<EnumOrUnknown<E>>,
    s: S,
) -> Result<S::Ok, S::Error> {
    if let Some(e) = e {
        match e.enum_value() {
            Ok(v) => s.serialize_str(v.descriptor().name()),
            Err(v) => s.serialize_i32(v),
        }
    } else {
        s.serialize_unit()
    }
}


/**
 * @api {整型指令数据} /SetIntValue SetIntValue
 * @apiGroup A_Object
 * @apiSuccess {u64} sender_id sender_id
 * @apiSuccess {u64} point_id point_id
 * @apiSuccess {i64} yk_command yk_command
 * @apiSuccess {u64} timestamp timestamp
 */
/// 指令数据
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetIntValue {
    pub sender_id: u64,
    pub point_id: u64,
    pub yk_command: i64,
    pub timestamp: u64,
}

/**
 * @api {浮点型指令数据} /SetFloatValue SetFloatValue
 * @apiGroup A_Object
 * @apiSuccess {u64} sender_id sender_id
 * @apiSuccess {u64} point_id point_id
 * @apiSuccess {f64} yt_command yt_command
 * @apiSuccess {u64} timestamp timestamp
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetFloatValue {
    pub sender_id: u64,
    pub point_id: u64,
    pub yt_command: f64,
    pub timestamp: u64,
}

/**
 * @api {公式型指令数据} /SetPointValue SetPointValue
 * @apiGroup A_Object
 * @apiSuccess {u64} sender_id sender_id
 * @apiSuccess {u64} point_id point_id
 * @apiSuccess {expr} command command
 * @apiSuccess {u64} timestamp timestamp
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetPointValue {
    pub sender_id: u64,
    pub point_id: u64,
    pub command: Expr,
    pub timestamp: u64,
}

fn deserialize_enum_or_unknown<'de, E: EnumFull, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Option<EnumOrUnknown<E>>, D::Error> {
    struct DeserializeEnumVisitor<E: EnumFull>(PhantomData<E>);

    impl<'de, E: EnumFull> serde::de::Visitor<'de> for DeserializeEnumVisitor<E> {
        type Value = Option<EnumOrUnknown<E>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "a string, an integer or none")
        }

        fn visit_i32<R>(self, v: i32) -> Result<Self::Value, R>
        where
            R: serde::de::Error,
        {
            Ok(Some(EnumOrUnknown::from_i32(v)))
        }

        fn visit_str<R>(self, v: &str) -> Result<Self::Value, R>
        where
            R: serde::de::Error,
        {
            match E::enum_descriptor().value_by_name(v) {
                Some(v) => Ok(Some(EnumOrUnknown::from_i32(v.value()))),
                None => Err(serde::de::Error::custom(format!(
                    "unknown enum value: {}",
                    v
                ))),
            }
        }

        fn visit_unit<R>(self) -> Result<Self::Value, R>
        where
            R: serde::de::Error,
        {
            Ok(None)
        }
    }

    d.deserialize_any(DeserializeEnumVisitor(PhantomData))
}

pub fn csv_str(record: &StringRecord, col: usize) -> Option<&str> {
    Some(record.get(col)?.trim())
}

pub fn csv_string(record: &StringRecord, col: usize) -> Option<String> {
    Some(record.get(col)?.trim().to_string())
}

pub fn csv_usize(record: &StringRecord, col: usize) -> Option<usize> {
    let s = record.get(col)?.to_string();
    let r = s.parse().ok()?;
    Some(r)
}

pub fn csv_u8(record: &StringRecord, col: usize) -> Option<u8> {
    let s = record.get(col)?.trim();
    let r = if s.starts_with("0x") {
        u8::from_str_radix(s.trim_start_matches("0x"), 16).ok()?
    } else {
        s.parse().ok()?
    };
    Some(r)
}

pub fn csv_u16(record: &StringRecord, col: usize) -> Option<u16> {
    let s = record.get(col)?.trim();
    let r = if s.starts_with("0x") {
        u16::from_str_radix(s.trim_start_matches("0x"), 16).ok()?
    } else {
        s.parse().ok()?
    };
    Some(r)
}

pub fn csv_u32(record: &StringRecord, col: usize) -> Option<u32> {
    let s = record.get(col)?.trim();
    let r = if s.starts_with("0x") {
        u32::from_str_radix(s.trim_start_matches("0x"), 16).ok()?
    } else {
        s.parse().ok()?
    };
    Some(r)
}

pub fn csv_u64(record: &StringRecord, col: usize) -> Option<u64> {
    let s = record.get(col)?.trim();
    let r = if s.starts_with("0x") {
        u64::from_str_radix(s.trim_start_matches("0x"), 16).ok()?
    } else {
        s.parse().ok()?
    };
    Some(r)
}

pub fn csv_i8(record: &StringRecord, col: usize) -> Option<i8> {
    let s = record.get(col)?.trim();
    let r = if s.starts_with("0x") {
        i8::from_str_radix(s.trim_start_matches("0x"), 16).ok()?
    } else {
        s.parse().ok()?
    };
    Some(r)
}

pub fn csv_i16(record: &StringRecord, col: usize) -> Option<i16> {
    let s = record.get(col)?.trim();
    let r = if s.starts_with("0x") {
        i16::from_str_radix(s.trim_start_matches("0x"), 16).ok()?
    } else {
        s.parse().ok()?
    };
    Some(r)
}

pub fn csv_i32(record: &StringRecord, col: usize) -> Option<i32> {
    let s = record.get(col)?.trim();
    let r = if s.starts_with("0x") {
        i32::from_str_radix(s.trim_start_matches("0x"), 16).ok()?
    } else {
        s.parse().ok()?
    };
    Some(r)
}

pub fn csv_i64(record: &StringRecord, col: usize) -> Option<i64> {
    let s = record.get(col)?.trim();
    let r = if s.starts_with("0x") {
        i64::from_str_radix(s.trim_start_matches("0x"), 16).ok()?
    } else {
        s.parse().ok()?
    };
    Some(r)
}

pub fn csv_f64(record: &StringRecord, col: usize) -> Option<f64> {
    let s = record.get(col)?.trim();
    let r = s.parse().ok()?;
    Some(r)
}

pub fn csv_f32(record: &StringRecord, col: usize) -> Option<f32> {
    let s = record.get(col)?.trim();
    let r = s.parse().ok()?;
    Some(r)
}