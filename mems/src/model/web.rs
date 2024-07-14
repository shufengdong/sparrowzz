use serde::{Deserialize, Serialize};
use std::fmt::Display;

use eig_db::{AoeControl, LccDevice, PointControl};

// used in lcc manager
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LccOp {
    PutLcc(LccDevice),
    DelLccs(Vec<String>),
}

/**
 * @api {枚举_Lcc操作} /LccControl LccControl
 * @apiPrivate
 * @apiGroup A_Enum
 * @apiSuccess {String} Reset 重启
 * @apiSuccess {String} Recover 重置，recover as new, all data and configs will be deleted
 * @apiSuccess {Object} AoeControl 控制AOE启动，停止或更新，{"AoeControl": AoeControl}
 * @apiSuccess {Object} PointControl 设置测点，{"PointControl": PointControl}
 * @apiSuccess {Object} PointInitControl 设置测点 and init，{"PointInitControl": PointControl}
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LccControl {
    /// 强制退出
    QuitForce,
    /// 重启
    Reset,
    // recover as new, all data and configs will be deleted
    Recover,
    /// 控制AOE启动，停止或更新
    AoeControl(AoeControl),
    /// 设置测点
    PointControl(PointControl),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AoeQuery {
    pub version: u32,
    pub id: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UiPosition {
    // plcc's UI
    Plcc,
    // plcc UI from MEMS proxy
    PlccProxy,
    // MEMS's UI
    Mems,
    // mirror
    Mirror,
    // plcc UI from MEMS proxy
    PlccProxyMirror(String),
}

impl Display for UiPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UiPosition::Plcc => write!(f, "plcc"),
            UiPosition::PlccProxy => write!(f, "plcc_proxy"),
            UiPosition::Mems => write!(f, "mems"),
            UiPosition::Mirror => write!(f, "mirror"),
            UiPosition::PlccProxyMirror(s) => write!(f, "plcc_proxy_mirror_{}", s),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PscpuInfo {
    pub is_start: bool,
    pub island_info: Option<(u32, usize, String)>,
    pub point_info: Option<(u32, usize, String)>,
    pub aoe_info: Option<(u32, usize, String)>,
}

/**
 * @api {WebPlugin} /WebPlugin WebPlugin
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} id id
 * @apiSuccess {String} path 文件树中的路径
 * @apiSuccess {String} name 在浏览模式下显示的名称
 * @apiSuccess {bool} is_file_uploaded 文件是否已经上传
 * @apiSuccess {bool} is_js 是否是JavaScript文件
 */
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WebPlugin {
    pub id: u64,
    // 文件树中的路径
    pub path: String,
    // 在浏览模式下显示的名称
    pub name: String,
    // wasm或js文件的名称
    pub model_name: String,
    // 文件是否已经上传
    pub is_file_uploaded: bool,
}

/**
 * @api {WebPluginFile} /WebPluginFile WebPluginFile
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} plugin_id id
 * @apiSuccess {u8[]} sevenz_file 内容
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebPluginFile {
    pub plugin_id: u64,
    pub sevenz_file: Vec<u8>,
}

//文件树的操作类型
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum FileTreeOp {
    Query,
    //查询
    Add,
    //增加
    Delete,
    //删除
    Change,
    //改变
    Apply,
    //版本应用
    QueryApply, //查询应用的版本
}

//文件树的上传结构
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileTreeNote {
    pub op: FileTreeOp,
    pub tree_id: String,
    pub version: Option<u32>,
    pub path: Option<String>,
    pub op_paths: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QueryWithId {
    pub id: Option<u64>,
}

impl QueryWithId {
    pub fn query_str(&self) -> String {
        let mut query = String::new();
        if let Some(id) = self.id {
            query.push_str(&format!("?id={}", id));
        }
        query
    }
}

#[cfg(test)]
mod tests {
    use eig_db::HisQuery;

    #[test]
    fn test_query_condition() {
        let query = HisQuery {
            id: Some("1,2".to_string()),
            start: Some(0),
            end: None,
            date: None,
            source: None,
            last_only: None,
            with_init: None,
        };
        assert_eq!(query.query_str(), "?id=1,2&start=0")
    }
}
