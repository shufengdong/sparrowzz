pub mod model;


// ============= 对webapp.rs中额外暴露给mems的API进行apidoc注释-开始
// ============= 因为在mems的API中过滤了webapp.rs，所以要在此处额外添加
// ============= 另一种方式是把webapp.rs拆分开两个文件，但尽量还是不动代码，就使用这种变通方式
/// public api
/**
 * @api {get} /api/v1/measures 查询历史量测
 * @apiGroup Webapp_Result
 * @apiUse HisQuery
 * @apiSuccess {PbPointValues} PbPointValues 测点值对象
 */

/**
 * @api {get} /api/v1/soes 查询SOE
 * @apiPrivate
 * @apiGroup Webapp_Result
 * @apiUse HisQuery
 * @apiSuccess {PbPointValues} PbPointValues SOE结果，结果按照时间排序
 */

/// public api
/**
 * @api {get} /api/v1/aoe_results 查询AOE执行结果
 * @apiGroup Webapp_Result
 * @apiUse HisQuery
 * @apiSuccess {PbAoeResults} PbAoeResults AOE执行结果
 */
/**
 * @api {get} /api/v1/commands 查询历史设点执行结果
 * @apiPrivate
 * @apiGroup Webapp_Result
 * @apiUse HisSetPointQuery
 * @apiSuccess {PbSetPointResults} PbSetPointResults 历史设点执行结果
 */

/// public api
/**
 * @api {get} /api/v1/alarms 查询告警
 * @apiGroup Webapp_Result
 * @apiUse HisQuery
 * @apiSuccess {PbEigAlarms} PbEigAlarms 告警结果，结果按照时间排序
 */
// ============= 对webapp.rs中额外暴露给mems的API进行apidoc注释-结束

/// 解析URL路径中带逗号,的值，返回数组
pub fn parse_path_values<T: std::str::FromStr>(path: &str) -> Vec<T> {
    let values_str: Vec<&str> = path.split(',').collect();
    let mut vec: Vec<T> = Vec::with_capacity(values_str.len());
    for value_str in values_str {
        if let Ok(v) = value_str.trim().parse() {
            vec.push(v);
        }
    }
    vec
}
