static EIG_IN: &str = "GwIn";
static EIG_OUT: &str = "GwOut";

// --------------------- from users to gateway -------------------

/// 控制指令下发，内容是　PbSetPoints
pub fn set_points(bee_id: &str) -> String {
    format!("{EIG_IN}/C/{bee_id}")
}
/// aoe调度指令，内容是 PbAoeOperation
pub fn aoe_control(bee_id: &str) -> String {
    format!("{EIG_IN}/Aoe/{bee_id}")
}

/// 重置，无内容
pub fn reset(bee_id: &str) -> String {
    format!("{EIG_IN}/Reset/{bee_id}")
}

/// recover，无内容
pub fn recover(bee_id: &str) -> String {
    format!("{EIG_IN}/Recover/{bee_id}")
}

/// 重置AOE文件，内容是  PbFile
pub fn reload_aoe_file(bee_id: &str) -> String {
    format!("{EIG_IN}/AoeFile/{bee_id}")
}

/// 重置通道文件, 内容是 PbFile
pub fn reload_tp_file(bee_id: &str) -> String {
    format!("{EIG_IN}/TpFile/{bee_id}")
}

/// 重置测点文件，内容是  PbFile
pub fn reload_point_file(bee_id: &str) -> String {
    format!("{EIG_IN}/PtFile/{bee_id}")
}

/// 重置配置文件，内容是  PbFile
pub fn reload_config_file(bee_id: &str) -> String {
    format!("{EIG_IN}/conf/{bee_id}")
}

/// 重置svg文件，内容是  PbFile
pub fn reload_svg_file(bee_id: &str) -> String {
    format!("{EIG_IN}/SvgFile/{bee_id}")
}

/// 查询当前所有数据，内容为空
pub fn call_all(bee_id: &str) -> String {
    format!("{EIG_IN}/AM/{bee_id}")
}

/// 查询网关Ping消息，内容为空
pub fn gw_ping_req() -> String {
    format!("{EIG_IN}/PING/REQ")
}

// --------------------- from gateway to users ------------------------

/// 测量值变化数据上传，内容是 PbPointValues
pub fn measure_changed(bee_id: &str) -> String {
    format!("{EIG_OUT}/SM_/{bee_id}")
}

/// 所有当前所有量测值的命令，内容是 PbPointValues
pub fn call_alled(bee_id: &str) -> String {
    format!("{EIG_OUT}/AM_/{bee_id}")
}

/// 网关通道、测点、svg三类文件的概况信息，内容是 PbEigProfile
pub fn gw_peeked(bee_id: &str) -> String {
    format!("{EIG_OUT}/GP_/{bee_id}")
}

/// 网关的概况，内容是 pbEigPing
pub fn gw_ping_res() -> String {
    format!("{EIG_OUT}/PING/RES")
}

/// 网关里的文件，内容是 PbFile
pub fn gw_file_res(file_url: &str) -> String {
    format!("{EIG_OUT}/FR_/{file_url}")
}

/// 内容是 PbEigAlarms
pub fn gw_alarmed(bee_id: &str) -> String {
    format!("{EIG_OUT}/ALARM_/{bee_id}")
}

pub fn gw_loged(file_url: &str) -> String {
    format!("{EIG_OUT}/LOG_/{file_url}")
}

/// 设点结果，内容是 PbSetPointResults
pub fn set_points_result(bee_id: &str) -> String {
    format!("{EIG_OUT}/C_/{bee_id}")
}

/// aoe运行结果，内容是 PbAoeResult
pub fn aoe_executed(bee_id: &str) -> String {
    format!("{EIG_OUT}/AH_/{bee_id}")
}

pub fn standby_topic(bee_id: &str) -> String {
    format!("standby/{}", bee_id)
}