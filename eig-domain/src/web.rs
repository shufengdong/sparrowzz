use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::PbEigPingRes;

/**
 * @api {Eig配置对象} /EigConfig EigConfig
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {Map} properties HashMap<String, String>
 */
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct EigConfig {
    pub properties: HashMap<String, String>,
    pub properties2: HashMap<String, String>,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigMsg {
    pub code: u8,
    pub config: Option<EigConfig>,
    pub ping: Option<PbEigPingRes>,
}