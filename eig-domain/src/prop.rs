use std::{fmt, str::FromStr};
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use crate::DataUnitError;

/**
 * @api {枚举_采集数据类型} /DataType DataType
 * @apiPrivate
 * @apiGroup A_Enum
 * @apiSuccess {String} Binary Binary
 * @apiSuccess {String} OneByteIntSigned OneByteIntSigned
 * @apiSuccess {String} OneByteIntSignedLower OneByteIntSignedLower
 * @apiSuccess {String} OneByteIntSignedUpper OneByteIntSignedUpper
 * @apiSuccess {String} OneByteIntUnsigned OneByteIntUnsigned
 * @apiSuccess {String} OneByteIntUnsignedLower OneByteIntUnsignedLower
 * @apiSuccess {String} OneByteIntUnsignedUpper OneByteIntUnsignedUpper
 * @apiSuccess {String} TwoByteIntUnsigned TwoByteIntUnsigned
 * @apiSuccess {String} TwoByteIntUnsignedSwapped TwoByteIntUnsignedSwapped
 * @apiSuccess {String} TwoByteIntSigned TwoByteIntSigned
 * @apiSuccess {String} TwoByteIntSignedSwapped TwoByteIntSignedSwapped
 * @apiSuccess {String} TwoByteBcd TwoByteBcd
 * @apiSuccess {String} FourByteIntUnsigned FourByteIntUnsigned
 * @apiSuccess {String} FourByteIntSigned FourByteIntSigned
 * @apiSuccess {String} FourByteIntUnsignedSwapped FourByteIntUnsignedSwapped
 * @apiSuccess {String} FourByteIntSignedSwapped FourByteIntSignedSwapped
 * @apiSuccess {String} FourByteIntUnsignedSwappedSwapped FourByteIntUnsignedSwappedSwapped
 * @apiSuccess {String} FourByteIntSignedSwappedSwapped FourByteIntSignedSwappedSwapped
 * @apiSuccess {String} FourByteFloat FourByteFloat
 * @apiSuccess {String} FourByteFloatSwapped FourByteFloatSwapped
 * @apiSuccess {String} FourByteFloatSwappedSwapped FourByteFloatSwappedSwapped
 * @apiSuccess {String} FourByteBcd FourByteBcd
 * @apiSuccess {String} FourByteBcdSwapped FourByteBcdSwapped
 * @apiSuccess {String} FourByteMod10k FourByteMod10k
 * @apiSuccess {String} FourByteMod10kSwapped FourByteMod10kSwapped
 * @apiSuccess {String} SixByteMod10k SixByteMod10k
 * @apiSuccess {String} SixByteMod10kSwapped SixByteMod10kSwapped
 * @apiSuccess {String} EightByteIntUnsigned EightByteIntUnsigned
 * @apiSuccess {String} EightByteIntSigned EightByteIntSigned
 * @apiSuccess {String} EightByteIntUnsignedSwapped EightByteIntUnsignedSwapped
 * @apiSuccess {String} EightByteIntSignedSwapped EightByteIntSignedSwapped
 * @apiSuccess {String} EightByteIntUnsignedSwappedSwapped EightByteIntUnsignedSwappedSwapped
 * @apiSuccess {String} EightByteIntSignedSwappedSwapped EightByteIntSignedSwappedSwapped
 * @apiSuccess {String} EightByteFloat EightByteFloat
 * @apiSuccess {String} EightByteFloatSwapped EightByteFloatSwapped
 * @apiSuccess {String} EightByteFloatSwappedSwapped EightByteFloatSwappedSwapped
 * @apiSuccess {String} EightByteMod10kSwapped EightByteMod10kSwapped
 * @apiSuccess {String} EightByteMod10k EightByteMod10k
 */
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum DataType {
    Binary,
    OneByteIntSigned,
    OneByteIntSignedLower,
    OneByteIntSignedUpper,
    OneByteIntUnsigned,
    OneByteIntUnsignedLower,
    OneByteIntUnsignedUpper,

    TwoByteIntUnsigned,
    TwoByteIntUnsignedSwapped,
    TwoByteIntSigned,
    TwoByteIntSignedSwapped,
    TwoByteBcd,

    FourByteIntUnsigned,
    FourByteIntSigned,
    FourByteIntUnsignedSwapped,
    FourByteIntSignedSwapped,
    FourByteIntUnsignedSwappedSwapped,
    FourByteIntSignedSwappedSwapped,
    FourByteFloat,
    FourByteFloatSwapped,
    FourByteFloatSwappedSwapped,
    FourByteBcd,
    FourByteBcdSwapped,
    FourByteMod10k,
    FourByteMod10kSwapped,

    SixByteMod10k,
    SixByteMod10kSwapped,

    EightByteIntUnsigned,
    EightByteIntSigned,
    EightByteIntUnsignedSwapped,
    EightByteIntSignedSwapped,
    EightByteIntUnsignedSwappedSwapped,
    EightByteIntSignedSwappedSwapped,
    EightByteFloat,
    EightByteFloatSwapped,
    EightByteFloatSwappedSwapped,
    EightByteMod10kSwapped,
    EightByteMod10k,
}

impl FromStr for DataType {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let r = match value {
            "Binary" => DataType::Binary,
            "OneByteIntSigned" => DataType::OneByteIntSigned,
            "OneByteIntSignedLower" => DataType::OneByteIntSignedLower,
            "OneByteIntSignedUpper" => DataType::OneByteIntSignedUpper,
            "OneByteIntUnsigned" => DataType::OneByteIntUnsigned,
            "OneByteIntUnsignedLower" => DataType::OneByteIntUnsignedLower,
            "OneByteIntUnsignedUpper" => DataType::OneByteIntUnsignedUpper,

            "TwoByteIntUnsigned" => DataType::TwoByteIntUnsigned,
            "TwoByteIntSigned" => DataType::TwoByteIntSigned,
            "TwoByteIntSignedSwapped" => DataType::TwoByteIntSignedSwapped,
            "TwoByteBcd" => DataType::TwoByteBcd,
            "TwoByteIntUnsignedSwapped" => DataType::TwoByteIntUnsignedSwapped,

            "FourByteIntUnsigned" => DataType::FourByteIntUnsigned,
            "FourByteIntSigned" => DataType::FourByteIntSigned,
            "FourByteIntUnsignedSwapped" => DataType::FourByteIntUnsignedSwapped,
            "FourByteIntSignedSwapped" => DataType::FourByteIntSignedSwapped,
            "FourByteIntUnsignedSwappedSwapped" => DataType::FourByteIntUnsignedSwappedSwapped,
            "FourByteIntSignedSwappedSwapped" => DataType::FourByteIntSignedSwappedSwapped,
            "FourByteFloat" => DataType::FourByteFloat,
            "FourByteFloatSwapped" => DataType::FourByteFloatSwapped,
            "FourByteFloatSwappedSwapped" => DataType::FourByteFloatSwappedSwapped,
            "FourByteBcd" => DataType::FourByteBcd,
            "FourByteBcdSwapped" => DataType::FourByteBcdSwapped,
            "FourByteMod10k" => DataType::FourByteMod10k,
            "FourByteMod10kSwapped" => DataType::FourByteMod10kSwapped,

            "SixByteMod10k" => DataType::SixByteMod10k,
            "SixByteMod10kSwapped" => DataType::SixByteMod10kSwapped,

            "EightByteIntUnsigned" => DataType::EightByteIntUnsigned,
            "EightByteIntSigned" => DataType::EightByteIntSigned,
            "EightByteIntUnsignedSwapped" => DataType::EightByteIntUnsignedSwapped,
            "EightByteIntSignedSwapped" => DataType::EightByteIntSignedSwapped,
            "EightByteFloat" => DataType::EightByteFloat,
            "EightByteFloatSwapped" => DataType::EightByteFloatSwapped,
            "EightByteMod10kSwapped" => DataType::EightByteMod10kSwapped,
            "EightByteMod10k" => DataType::EightByteMod10k,
            _ => return Err(()),
        };
        Ok(r)
    }
}


impl DataType {
    pub fn get_byte_count(&self) -> u16 {
        match self {
            DataType::Binary => 1,
            DataType::OneByteIntSigned => 1,
            DataType::OneByteIntSignedLower => 1,
            DataType::OneByteIntSignedUpper => 1,
            DataType::OneByteIntUnsigned => 1,
            DataType::OneByteIntUnsignedLower => 1,
            DataType::OneByteIntUnsignedUpper => 1,
            DataType::TwoByteIntUnsigned => 2,
            DataType::TwoByteIntSigned => 2,
            DataType::TwoByteIntSignedSwapped => 2,
            DataType::TwoByteBcd => 2,
            DataType::TwoByteIntUnsignedSwapped => 2,
            DataType::FourByteIntUnsigned => 4,
            DataType::FourByteIntSigned => 4,
            DataType::FourByteIntUnsignedSwapped => 4,
            DataType::FourByteIntSignedSwapped => 4,
            DataType::FourByteIntUnsignedSwappedSwapped => 4,
            DataType::FourByteIntSignedSwappedSwapped => 4,
            DataType::FourByteFloat => 4,
            DataType::FourByteFloatSwapped => 4,
            DataType::FourByteFloatSwappedSwapped => 4,
            DataType::FourByteBcd => 4,
            DataType::FourByteBcdSwapped => 4,
            DataType::FourByteMod10k => 4,
            DataType::FourByteMod10kSwapped => 4,
            DataType::SixByteMod10k => 6,
            DataType::SixByteMod10kSwapped => 6,
            DataType::EightByteIntUnsigned => 8,
            DataType::EightByteIntSigned => 8,
            DataType::EightByteIntUnsignedSwapped => 8,
            DataType::EightByteIntSignedSwapped => 8,
            DataType::EightByteIntUnsignedSwappedSwapped => 8,
            DataType::EightByteIntSignedSwappedSwapped => 8,
            DataType::EightByteFloat => 8,
            DataType::EightByteFloatSwapped => 8,
            DataType::EightByteFloatSwappedSwapped => 8,
            DataType::EightByteMod10kSwapped => 8,
            DataType::EightByteMod10k => 8,
        }
    }
}

impl Display for DataType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/**
 * @api {枚举_属性类型} /PropType PropType
 * @apiPrivate
 * @apiGroup A_Enum
 * @apiSuccess {String} U8 u8
 * @apiSuccess {String} U16 u16
 * @apiSuccess {String} U32 u32
 * @apiSuccess {String} U64 u64
 * @apiSuccess {String} I8 i8
 * @apiSuccess {String} I16 i16
 * @apiSuccess {String} I32 i32
 * @apiSuccess {String} I64 i64
 * @apiSuccess {String} F32 f32
 * @apiSuccess {String} F64 f64
 * @apiSuccess {String} Str str
 * @apiSuccess {String} ComplexF32 f32类型复数
 * @apiSuccess {String} ComplexF64 f64类型复数
 * @apiSuccess {String} TensorF32 f32类型向量
 * @apiSuccess {String} TensorF64 f64类型向量
 * @apiSuccess {String} TensorComplexF32 f32类型复数向量
 * @apiSuccess {String} TensorComplexF64 f64类型复数向量
 * @apiSuccess {String} Unknown 未知
 */
/// 属性类型枚举
#[repr(u8)]
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Copy, Default)]
pub enum PropType {
    U8 = 1,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Str,
    // 复数
    ComplexF32,
    ComplexF64,
    // 向量
    TensorF32,
    TensorF64,
    TensorComplexF32,
    TensorComplexF64,
    // unknown type
    #[default]
    Unknown = 255,
}

/**
 * @api {枚举_属性值} /PropValue PropValue
 * @apiPrivate
 * @apiGroup A_Enum
 * @apiSuccess {Object} U8 {"U8": u8}
 * @apiSuccess {Object} U16 {"U16": u16}
 * @apiSuccess {Object} U32 {"U32": u32}
 * @apiSuccess {Object} U64 {"U64": u64}
 * @apiSuccess {Object} I8 {"I8": i8}
 * @apiSuccess {Object} I16 {"I16": i16}
 * @apiSuccess {Object} I32 {"I32": i32}
 * @apiSuccess {Object} I64 {"I64": i64}
 * @apiSuccess {Object} F32 {"F32": f32}
 * @apiSuccess {Object} F64 {"F64": f64}
 * @apiSuccess {Object} Str {"Str": String}
 * @apiSuccess {Object} ComplexF32 f32类型复数，{"ComplexF32": tuple(f32, f32)}
 * @apiSuccess {Object} ComplexF64 f64类型复数，{"ComplexF64": tuple(f64, f64)}
 * @apiSuccess {Object} TensorF32 f32类型向量，{"TensorF32": tuple(usize[], f32[])}
 * @apiSuccess {Object} TensorF64 f64类型向量，{"TensorF64": tuple(usize[], f64[])}
 * @apiSuccess {Object} TensorComplexF32 f32类型复数向量，{"TensorComplexF32": tuple(usize[], tuple(f32, f32)[])}
 * @apiSuccess {Object} TensorComplexF64 f64类型复数向量，{"TensorComplexF64": tuple(usize[], tuple(f64, f64)[])}
 * @apiSuccess {String} Unknown 未知
 */
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum PropValue {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Str(String),
    // 复数
    ComplexF32(f32, f32),
    ComplexF64(f64, f64),
    // 向量
    TensorF32(Vec<usize>, Vec<f32>),
    TensorF64(Vec<usize>, Vec<f64>),
    TensorComplexF32(Vec<usize>, Vec<(f32, f32)>),
    TensorComplexF64(Vec<usize>, Vec<(f64, f64)>),
    Unknown,
}

/// 将枚举转换成字符串，调用to_string()方法
impl Display for PropValue {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            PropValue::U8(s) => write!(f, "{}", s),
            PropValue::U16(s) => write!(f, "{}", s),
            PropValue::U32(s) => write!(f, "{}", s),
            PropValue::U64(s) => write!(f, "{}", s),
            PropValue::I8(s) => write!(f, "{}", s),
            PropValue::I16(s) => write!(f, "{}", s),
            PropValue::I32(s) => write!(f, "{}", s),
            PropValue::I64(s) => write!(f, "{}", s),
            PropValue::F32(s) => write!(f, "{}", s),
            PropValue::F64(s) => write!(f, "{}", s),
            PropValue::Str(s) => write!(f, "{}", s),
            PropValue::ComplexF32(_, _) => write!(f, ""),
            PropValue::ComplexF64(_, _) => write!(f, ""),
            PropValue::TensorF32(_, _) => write!(f, ""),
            PropValue::TensorF64(_, _) => write!(f, ""),
            PropValue::TensorComplexF32(_, _) => write!(f, ""),
            PropValue::TensorComplexF64(_, _) => write!(f, ""),
            PropValue::Unknown => write!(f, ""),
        }
    }
}

impl PropValue {
    pub fn from_str(t: PropType, s: &str) -> Option<Self> {
        let v = match t {
            PropType::U8 => PropValue::U8(s.parse().ok()?),
            PropType::U16 => PropValue::U16(s.parse().ok()?),
            PropType::U32 => PropValue::U32(s.parse().ok()?),
            PropType::U64 => PropValue::U64(s.parse().ok()?),
            PropType::I8 => PropValue::I8(s.parse().ok()?),
            PropType::I16 => PropValue::I16(s.parse().ok()?),
            PropType::I32 => PropValue::I32(s.parse().ok()?),
            PropType::I64 => PropValue::I64(s.parse().ok()?),
            PropType::F32 => PropValue::F32(s.parse().ok()?),
            PropType::F64 => PropValue::F64(s.parse().ok()?),
            PropType::Str => PropValue::Str(s.parse().ok()?),
            PropType::ComplexF32 => PropValue::Unknown,
            PropType::ComplexF64 => PropValue::Unknown,
            PropType::TensorF32 => PropValue::Unknown,
            PropType::TensorF64 => PropValue::Unknown,
            PropType::TensorComplexF32 => PropValue::Unknown,
            PropType::TensorComplexF64 => PropValue::Unknown,
            PropType::Unknown => PropValue::Unknown,
        };
        Some(v)
    }
    pub fn from_f64(t: PropType, f: f64) -> Option<Self> {
        let v = match t {
            PropType::U8 => PropValue::U8(f as u8),
            PropType::U16 => PropValue::U16(f as u16),
            PropType::U32 => PropValue::U32(f as u32),
            PropType::U64 => PropValue::U64(f as u64),
            PropType::I8 => PropValue::I8(f as i8),
            PropType::I16 => PropValue::I16(f as i16),
            PropType::I32 => PropValue::I32(f as i32),
            PropType::I64 => PropValue::I64(f as i64),
            PropType::F32 => PropValue::F32(f as f32),
            PropType::F64 => PropValue::F64(f),
            PropType::Str => PropValue::Str(f.to_string()),
            PropType::ComplexF32 => PropValue::Unknown,
            PropType::ComplexF64 => PropValue::Unknown,
            PropType::TensorF32 => PropValue::Unknown,
            PropType::TensorF64 => PropValue::Unknown,
            PropType::TensorComplexF32 => PropValue::Unknown,
            PropType::TensorComplexF64 => PropValue::Unknown,
            PropType::Unknown => PropValue::Unknown,
        };
        Some(v)
    }
    pub fn get_f64(&self) -> Option<f64> {
        match self {
            PropValue::U8(s) => Some(*s as f64),
            PropValue::U16(s) => Some(*s as f64),
            PropValue::U32(s) => Some(*s as f64),
            PropValue::U64(s) => Some(*s as f64),
            PropValue::I8(s) => Some(*s as f64),
            PropValue::I16(s) => Some(*s as f64),
            PropValue::I32(s) => Some(*s as f64),
            PropValue::I64(s) => Some(*s as f64),
            PropValue::F32(s) => Some(*s as f64),
            PropValue::F64(s) => Some(*s),
            PropValue::Str(s) => s.parse().ok(),
            _ => None,
        }
    }

    pub fn get_bool(&self) -> Option<bool> {
        match self {
            PropValue::U8(s) => Some(*s > 0),
            PropValue::U16(s) => Some(*s > 0),
            PropValue::U32(s) => Some(*s > 0),
            PropValue::U64(s) => Some(*s > 0),
            PropValue::I8(s) => Some(*s > 0),
            PropValue::I16(s) => Some(*s > 0),
            PropValue::I32(s) => Some(*s > 0),
            PropValue::I64(s) => Some(*s > 0),
            PropValue::F32(s) => Some(*s > 0.),
            PropValue::F64(s) => Some(*s > 0.),
            PropValue::Str(s) => {
                match s.to_uppercase().as_str() {
                    "TRUE" | "YES" | "T" | "Y" => Some(true),
                    "FALSE" | "NO" | "F" | "N" => Some(false),
                    _ => None
                }
            }
            _ => None,
        }
    }
}

impl PropType {
    /// 用于遍历所有属性类型列表
    pub const PS_PROP_TYPE: [PropType; 18] = [
        PropType::U8,
        PropType::U16,
        PropType::U32,
        PropType::U64,
        PropType::I8,
        PropType::I16,
        PropType::I32,
        PropType::I64,
        PropType::F32,
        PropType::F64,
        PropType::Str,
        PropType::ComplexF32,
        PropType::ComplexF64,
        PropType::TensorF32,
        PropType::TensorF64,
        PropType::TensorComplexF32,
        PropType::TensorComplexF64,
        PropType::Unknown,
    ];
}

impl FromStr for PropType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let p = match s.to_uppercase().as_str() {
            "U8" => PropType::U8,
            "U16" => PropType::U16,
            "U32" => PropType::U32,
            "U64" => PropType::U64,
            "I8" => PropType::I8,
            "I16" => PropType::I16,
            "I32" => PropType::I32,
            "I64" => PropType::I64,
            "F32" => PropType::F32,
            "F64" => PropType::F64,
            "STR" => PropType::Str,
            "COMPLEXF32" => PropType::ComplexF32,
            "COMPLEXF64" => PropType::ComplexF64,
            "TENSORF32" => PropType::TensorF32,
            "TENSORF64" => PropType::TensorF64,
            "TENSORCOMPLEXF32" => PropType::TensorComplexF32,
            "TENSORCOMPLEXF64" => PropType::TensorComplexF64,
            _ => PropType::Unknown,
        };
        Ok(p)
    }
}

/// 将枚举转换成字符串，调用to_string()方法
impl Display for PropType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}


/**
 * @api {枚举_数据单位} /DataUnit DataUnit
 * @apiPrivate
 * @apiGroup A_Enum
 * @apiSuccess {String} A 安培
 * @apiSuccess {String} V 伏特
 * @apiSuccess {String} kV 千伏
 * @apiSuccess {String} W 瓦特
 * @apiSuccess {String} kW 千瓦
 * @apiSuccess {String} MW 兆瓦
 * @apiSuccess {String} H 亨利
 * @apiSuccess {String} mH 毫亨
 * @apiSuccess {String} Ah 安时
 * @apiSuccess {String} mAh 毫安时
 * @apiSuccess {String} kWh 千瓦时
 * @apiSuccess {String} Celsius 摄氏度
 * @apiSuccess {String} feet feet
 * @apiSuccess {String} km kilometer
 * @apiSuccess {String} meter meter
 * @apiSuccess {String} UnitOne 无单位
 * @apiSuccess {String} Percent 百分比
 * @apiSuccess {String} Unknown 其他未知单位
 */
#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Copy, Clone, Hash, Default)]
pub enum DataUnit {
    /// switch on of off
    OnOrOff,
    /// 安培
    A,
    /// 伏特
    V,
    /// 千伏
    kV,
    /// 瓦特
    W,
    /// 千瓦
    kW,
    /// 兆瓦
    MW,
    /// Var
    Var,
    /// kVar
    kVar,
    /// MVar
    MVar,
    // VA
    VA,
    // kVA
    kVA,
    // MVA
    MVA,
    /// 亨利
    H,
    /// 毫亨
    mH,
    /// 安时
    Ah,
    /// 毫安时
    mAh,
    /// 千瓦时
    kWh,
    /// 摄氏度
    Celsius,
    /// feet
    feet,
    /// kilometer
    km,
    /// meter
    meter,
    /// Square millimeter
    mm2,
    /// 无单位
    UnitOne,
    /// 百分比
    Percent,
    /// 大小单位
    bit,
    /// Byte
    B,
    /// KB
    kB,
    /// MB
    MB,
    /// GB
    GB,
    /// TB
    TB,
    /// PB
    PB,
    /// 其他未知单位
    #[default]
    Unknown,
}

impl DataUnit {
    /// 用于遍历所有数据单位列表
    pub const DATA_UNIT: [DataUnit; 33] = [
        DataUnit::OnOrOff,
        DataUnit::A,
        DataUnit::V,
        DataUnit::kV,
        DataUnit::W,
        DataUnit::kW,
        DataUnit::MW,
        DataUnit::Var,
        DataUnit::kVar,
        DataUnit::MVar,
        DataUnit::VA,
        DataUnit::kVA,
        DataUnit::MVA,
        DataUnit::H,
        DataUnit::mH,
        DataUnit::Ah,
        DataUnit::mAh,
        DataUnit::kWh,
        DataUnit::Celsius,
        DataUnit::feet,
        DataUnit::km,
        DataUnit::meter,
        DataUnit::mm2,
        DataUnit::UnitOne,
        DataUnit::Percent,
        DataUnit::bit,
        DataUnit::B,
        DataUnit::kB,
        DataUnit::MB,
        DataUnit::GB,
        DataUnit::TB,
        DataUnit::PB,
        DataUnit::Unknown,
    ];
}


impl FromStr for DataUnit {
    type Err = DataUnitError;

    fn from_str(s: &str) -> Result<DataUnit, DataUnitError> {
        match s.trim().to_uppercase().as_str() {
            "ON_OR_OFF" => Ok(DataUnit::OnOrOff),
            "ONOROFF" => Ok(DataUnit::OnOrOff),
            "ONOFF" => Ok(DataUnit::OnOrOff),
            "ON/OFF" => Ok(DataUnit::OnOrOff),
            "A" => Ok(DataUnit::A),
            "安" => Ok(DataUnit::A),
            "安培" => Ok(DataUnit::A),
            "V" => Ok(DataUnit::V),
            "伏" => Ok(DataUnit::V),
            "伏特" => Ok(DataUnit::kV),
            "KV" => Ok(DataUnit::kV),
            "千伏" => Ok(DataUnit::kV),
            "W" => Ok(DataUnit::W),
            "瓦" => Ok(DataUnit::W),
            "瓦特" => Ok(DataUnit::W),
            "KW" => Ok(DataUnit::kW),
            "千瓦" => Ok(DataUnit::kW),
            "MW" => Ok(DataUnit::MW),
            "兆瓦" => Ok(DataUnit::MW),
            "VA" => Ok(DataUnit::VA),
            "伏安" => Ok(DataUnit::VA),
            "KVA" => Ok(DataUnit::kVA),
            "千伏安" => Ok(DataUnit::kVA),
            "MVA" => Ok(DataUnit::MVA),
            "兆伏安" => Ok(DataUnit::MVA),
            "VAR" => Ok(DataUnit::Var),
            "乏" => Ok(DataUnit::Var),
            "KVAR" => Ok(DataUnit::kVar),
            "千乏" => Ok(DataUnit::kVar),
            "MVAR" => Ok(DataUnit::MVar),
            "兆乏" => Ok(DataUnit::MVar),
            "H" => Ok(DataUnit::H),
            "亨" => Ok(DataUnit::H),
            "亨利" => Ok(DataUnit::H),
            "MH" => Ok(DataUnit::mH),
            "毫亨" => Ok(DataUnit::mH),
            "AH" => Ok(DataUnit::Ah),
            "安时" => Ok(DataUnit::Ah),
            "MAH" => Ok(DataUnit::mAh),
            "毫安时" => Ok(DataUnit::mAh),
            "KWH" => Ok(DataUnit::kWh),
            "千瓦时" => Ok(DataUnit::kWh),
            "度" => Ok(DataUnit::kWh),
            "℃" => Ok(DataUnit::Celsius), //℃ 字符代码 2103 中文摄氏度 一个字符
            "°C" => Ok(DataUnit::Celsius), // °C 英文摄氏度  两个字符 字符代码00B0 +大写字母C
            "CELSIUS" => Ok(DataUnit::Celsius),
            "摄氏度" => Ok(DataUnit::Celsius),
            "FEET" => Ok(DataUnit::feet),
            "KM" => Ok(DataUnit::km),
            "M" => Ok(DataUnit::meter),
            "METER" => Ok(DataUnit::meter),
            "MM2" => Ok(DataUnit::mm2),
            "%" => Ok(DataUnit::Percent),
            "PERCENT" => Ok(DataUnit::Percent),
            "BIT" => Ok(DataUnit::bit),
            "BYTE" => Ok(DataUnit::B),
            "B" => Ok(DataUnit::B),
            "KB" => Ok(DataUnit::kB),
            "MB" => Ok(DataUnit::MB),
            "GB" => Ok(DataUnit::GB),
            "TB" => Ok(DataUnit::TB),
            "PB" => Ok(DataUnit::PB),
            "UNITONE" => Ok(DataUnit::UnitOne),
            //todo：finish it
            _ => Ok(DataUnit::Unknown),
        }
    }
}

impl Display for DataUnit {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            DataUnit::OnOrOff => write!(f, "on_or_off"),
            DataUnit::feet => write!(f, "ft"),
            DataUnit::UnitOne => write!(f, ""),
            DataUnit::Percent => write!(f, "%"),
            DataUnit::Celsius => write!(f, "°C"),
            DataUnit::Unknown => write!(f, ""),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::DataUnit;

    #[test]
    fn test() {
        let a = DataUnit::Celsius;
        let b = a.to_string();
        println!("{}", b);
    }
}