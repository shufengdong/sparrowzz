use std::collections::{BTreeMap, HashMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Number, Value};

use crate::excel::{excel_bytes_to_csv_bytes, transfer_to_utf8};
use crate::{csv_str, csv_string, csv_u16, csv_u64, csv_usize, get_csv_str};

/**
 * @api {Mqtt通道信息} /MqttTransport MqttTransport
 * @apiPrivate
 * @apiGroup A_Object
 * @apiSuccess {u64} id 通道id
 * @apiSuccess {String} name 通道名称
 * @apiSuccess {tuple} mqtt_broker 服务端的ip和por，tuple格式为(ip:String, port:u16)
 * @apiSuccess {u64} point_id 通道状态对应的测点号
 * @apiSuccess {tuple[]} point_ids 通过mqtt读写的测点，数组，tuple格式为(u64, bool)
 * @apiSuccess {String} read_topic 读测点的主题
 * @apiSuccess {String} write_topic 写测点的主题
 * @apiSuccess {bool} is_json 是否是json编码格式，默认是false，表示protobuf格式
 * @apiSuccess {String} [user_name] 用户名，可选
 * @apiSuccess {String} [user_password] 密码，可选
 */
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
pub struct MqttTransport {
    pub id: u64,
    /// 通道名称
    pub name: String,
    /// 服务端的ip和por
    pub mqtt_broker: (String, u16),
    /// 通道状态对应的测点号
    pub point_id: u64,
    /// 通过mqtt读写的测点
    pub point_ids: Vec<(u64, bool)>,
    /// 读测点的主题
    pub read_topic: String,
    /// 写测点的主题
    pub write_topic: String,
    /// 编码格式，默认是protobuf
    pub is_json: bool,
    /// 是否转发通道
    pub is_transfer: bool,
    /// 心跳时间
    pub keep_alive: Option<u16>,
    /// 用户名，可选
    pub user_name: Option<String>,
    /// 用户密码，可选
    pub user_password: Option<String>,
    /// json格式过滤器
    pub json_filters: Option<Vec<Vec<String>>>,
    /// json测点对应的数据标识, key是过滤器对应Array的json字符串，value是标识以及测点的索引
    pub json_tags: Option<HashMap<String, HashMap<String, usize>>>,
    /// json写测点模板
    pub json_write_template: Option<HashMap<u64, String>>,
    /// json写测点模板
    pub json_write_tag: Option<HashMap<u64, String>>,
}

impl MqttTransport {
    pub fn from_file(path: &str) -> Result<Self, (usize, usize)> {
        let content = std::fs::read(path).map_err(|_| (0, 0))?;
        // let content = if env::IS_ENCRYPT {
        //     decrypt_vec(content.as_slice())
        // } else {
        //     content
        // };
        let csv_bytes = if path.ends_with(".xlsx") || path.ends_with(".xls") {
            let r = excel_bytes_to_csv_bytes(content.as_slice()).unwrap_or_default();
            if r.is_empty() {
                return Err((0, 0));
            }
            r[0].clone()
        } else {
            content
        };
        Self::from_csv_bytes(csv_bytes.as_slice())
    }

    pub fn from_csv(path: &str) -> Result<MqttTransport, (usize, usize)> {
        let content = std::fs::read(path).map_err(|_| (0, 0))?;
        // let content = if env::IS_ENCRYPT {
        //     let content = decrypt_vec(content.as_slice());
        //     content
        // } else {
        //     content
        // };
        MqttTransport::from_csv_bytes(content.as_slice())
    }

    pub fn from_csv_bytes(content: &[u8]) -> Result<MqttTransport, (usize, usize)> {
        let content_new = transfer_to_utf8(content.to_vec()).map_err(|_| (0, 0))?;
        let content = content_new.as_slice();
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(content);
        let mut records = rdr.records();
        let rc = (0usize, 1);
        let name = csv_string(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        let rc = (1usize, 1);
        let broker_ip = csv_string(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        let rc = (2usize, 1);
        let broker_port = csv_u16(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        let mqtt_broker = (broker_ip, broker_port);
        let rc = (3usize, 1);
        let point_num = csv_usize(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        let rc = (4usize, 1);
        let point_id = csv_u64(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        let rc = (5usize, 1);
        let read_topic = csv_string(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        let rc = (6usize, 1);
        let write_topic = csv_string(&records.next().ok_or(rc)?.map_err(|_| rc)?, rc.1).ok_or(rc)?;
        // 下面5个是可选的
        let rc = (7usize, 1);
        let mut user_name = None;
        let mut user_password = None;
        let mut is_json = false;
        let mut is_transfer = false;
        let mut keep_alive = None;
        if let Some(Ok(line)) = records.next() {
            user_name = csv_string(&line, rc.1);
            if Some("".to_string()) == user_name {
                user_name = None;
            }
            let rc = (8usize, 1);
            if let Some(Ok(line)) = records.next() {
                user_password = csv_string(&line, rc.1);
                if Some("".to_string()) == user_password {
                    user_password = None;
                }
                let record = records.next();
                if let Some(Ok(tmp)) = record {
                    if let Some(v) = csv_string(&tmp, 1) {
                        if v.to_uppercase() == "TRUE" {
                            is_json = true;
                        }
                    }
                }
                let record = records.next();
                if let Some(Ok(tmp)) = record {
                    if let Some(v) = csv_string(&tmp, 1) {
                        if v.to_uppercase() == "TRUE" {
                            is_transfer = true;
                        }
                    }
                }
                let record = records.next();
                if let Some(Ok(tmp)) = record {
                    if let Some(v) = csv_u16(&tmp, 1) {
                        keep_alive = Some(v);
                    }
                }
            }
        }
        // 开启读取测点信息
        let mut point_ids = Vec::with_capacity(point_num);
        // 从新加载
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(content);
        let mut records = rdr.records();
        let mut tmp = HashSet::with_capacity(point_num);
        let mut json_filters = Vec::new();
        let mut json_tags: HashMap<String, HashMap<String, usize>> = HashMap::new();
        let mut write_templates: HashMap<u64, String> = HashMap::new();
        let mut write_tags: HashMap<u64, String> = HashMap::new();
        let mut is_custom = false;
        for i in 0..point_num {
            let rc = (i + 1, 3);
            let record = records.next().ok_or(rc)?.map_err(|_| rc)?;
            if i == 0 {
                is_custom = is_json && record.get(6).is_some() && !record.get(6).as_ref().unwrap().is_empty();
            }
            let id = csv_u64(&record, rc.1).ok_or(rc)?;
            // 测点不能重复
            if tmp.contains(&id) {
                return Err(rc);
            }
            let rc = (i + 1, 4);
            let s = csv_str(&record, rc.1).ok_or(rc)?.to_uppercase();
            let is_writable = match s.as_str() {
                "FALSE" => false,
                "TRUE" => true,
                _ => false,
            };
            point_ids.push((id, is_writable));
            tmp.insert(id);
            if !is_custom {
                continue;
            }
            // 下面这列是可选的
            let rc = (i + 1, 5);
            if let Some(s) = csv_str(&record, rc.1) {
                let filter = if s.is_empty() {
                    Value::Object(Map::with_capacity(0))
                } else {
                    serde_json::from_str::<Value>(s).map_err(|_| rc)?
                };
                let map = split_json(&filter);
                let mut k = Vec::with_capacity(map.len() + 1);
                let mut keys = Vec::with_capacity(map.len());
                for (key, v) in map {
                    k.push(v);
                    keys.push(key);
                }
                let mut is_exist = false;
                for i in 0..json_filters.len() {
                    if json_filters[i] == keys {
                        is_exist = true;
                        k.push(Value::Number(Number::from(i)));
                        break;
                    }
                }
                if !is_exist {
                    k.push(Value::Number(Number::from(json_filters.len())));
                    json_filters.push(keys);
                }
                let rc = (i + 1, 6);
                let tag = csv_string(&record, rc.1).ok_or(rc)?;
                let k_json = Value::Array(k).to_string();
                if let Some(m) = json_tags.get_mut(&k_json) {
                    if m.contains_key(&tag) {
                        return Err(rc);
                    }
                    m.insert(tag, i);
                } else {
                    let mut m = HashMap::new();
                    m.insert(tag, i);
                    json_tags.insert(k_json, m);
                }
                let rc = (i + 1, 7);
                if let Some(template) = csv_string(&record, rc.1) {
                    write_templates.insert(id, template);
                    let rc = (i + 1, 8);
                    let tag = csv_string(&record, rc.1).ok_or(rc)?;
                    write_tags.insert(id, tag);
                }
            }
        }
        let json_filters = if !json_filters.is_empty() {
            Some(json_filters)
        } else {
            None
        };
        let json_tags = if !json_tags.is_empty() {
            Some(json_tags)
        } else {
            None
        };
        let json_write_template = if !write_templates.is_empty() {
            Some(write_templates)
        } else {
            None
        };
        let json_write_tag = if !write_tags.is_empty() {
            Some(write_tags)
        } else {
            None
        };
        Ok(MqttTransport {
            id: 0,
            name,
            mqtt_broker,
            point_id,
            point_ids,
            read_topic,
            write_topic,
            is_json,
            is_transfer,
            user_name,
            user_password,
            keep_alive,
            json_filters,
            json_tags,
            json_write_template,
            json_write_tag,
        })
    }

    pub fn export_csv(&self, text_map: &HashMap<String, String>) -> String {
        let title = vec![
            text_map.get("broker_ip").unwrap_or(&"Broker Ip".to_string()).clone(),
            text_map.get("broker_port").unwrap_or(&"Broker Port".to_string()).clone(),
            text_map.get("point_number").unwrap_or(&"Point Count".to_string()).clone(),
            text_map.get("status_point").unwrap_or(&"Status Point".to_string()).clone(),
            text_map.get("mqtt_r_topic").unwrap_or(&"Read Topic".to_string()).clone(),
            text_map.get("mqtt_w_topic").unwrap_or(&"Write Topic".to_string()).clone(),
            text_map.get("user_name").unwrap_or(&"User Name".to_string()).clone(),
            text_map.get("user_password").unwrap_or(&"User Password".to_string()).clone(),
            text_map.get("json_format").unwrap_or(&"JSON Format".to_string()).clone(),
            text_map.get("transfer").unwrap_or(&"Transfer".to_string()).clone(),
        ];
        let mut content = vec![
            format!("{}", self.mqtt_broker.0),
            format!("{}", self.mqtt_broker.1),
            format!("{}", self.point_ids.len()),
            format!("{}", self.point_id),
            format!("{}", self.read_topic),
            format!("{}", self.write_topic),
        ];

        if let Some(user_name) = &self.user_name {
            content.push(get_csv_str(user_name.as_str()));
        } else {
            content.push("".to_string());
        };
        if let Some(user_password) = &self.user_password {
            content.push(get_csv_str(user_password.as_str()));
        } else {
            content.push("".to_string());
        };
        content.push(self.is_json.to_string().to_uppercase());
        content.push(self.is_transfer.to_string().to_uppercase());

        let mut result = format!(
            "{},{},{},{},{}",
            text_map.get("tp_name").unwrap_or(&"Transport Name".to_string()),
            get_csv_str(&self.name),
            text_map.get("index").unwrap_or(&"Index".to_string()),
            text_map.get("status_point").unwrap_or(&"Status Point".to_string()).clone(),
            text_map.get("is_writable").unwrap_or(&"Is Writable".to_string()).clone(),
        ).to_string();
        if self.json_filters.is_some() {
            let s = format!(
                ",{},{}",
                text_map.get("json_read_filter").unwrap_or(&"Json Filter".to_string()).clone(),
                text_map.get("json_read_tag").unwrap_or(&"Json Tag".to_string()).clone(),
            );
            result += &s;
            if self.json_write_template.is_some() {
                let s = format!(
                    ",{},{}\n",
                    text_map.get("json_wirte_template").unwrap_or(&"Json Write Template".to_string()).clone(),
                    text_map.get("json_write_tag").unwrap_or(&"Json Write Tag".to_string()).clone(),
                );
                result += &s;
            } else {
                result.push('\n');
            }
        } else {
            result.push('\n');
        }
        let mut pos_to_str = HashMap::new();
        if let Some(map) = &self.json_tags {
            for (k, v) in map {
                let mut values = vec![];
                if let Ok(k_json) = serde_json::from_str::<Value>(k.as_str()) {
                    if let Value::Array(obj) = k_json {
                        values = obj;
                    }
                }
                if values.is_empty() {
                    continue;
                }
                let index: usize;
                if let Value::Number(n) = &values[values.len() - 1] {
                    if let Some(i) = n.as_u64() {
                        index = i as usize;
                    } else {
                        continue;
                    }
                } else {
                    continue;
                }
                if let Some(filters) = &self.json_filters {
                    let value = merge_json(&filters[index], &values[0..values.len() - 1]);
                    for (tag, i) in v {
                        let json_str = value.to_string();
                        pos_to_str.insert(*i, format!("{},{}", get_csv_str(&json_str), tag));
                    }
                }
            }
        }
        let p = self.point_ids.clone();
        for i in 0_usize..10_usize {
            if p.len() > i {
                let w_status = if p[i].1 { "TRUE" } else { "FALSE" };
                result += &format!(
                    "{},{},{},{},{}",
                    title[i],
                    content[i],
                    i + 1,
                    p[i].0,
                    w_status
                );
                if let Some(s) = pos_to_str.get(&i) {
                    result.push(',');
                    result += &s.as_str();
                } else if self.json_filters.is_some() {
                    result += ",,";
                    if let Some(templates) = &self.json_write_template {
                        result += ",";
                        if let Some(s) = templates.get(&p[i].0) {
                            result += &get_csv_str(s.as_str());
                        } else {
                            result += ",";
                        }
                        if let Some(write_tags) = &self.json_write_tag {
                            if let Some(s) = write_tags.get(&p[i].0) {
                                result += &get_csv_str(s.as_str());
                            } else {
                                result += ",";
                            }
                        } else {
                            result += ",";
                        }
                    }
                }
                result.push('\n');
            } else {
                result += &format!("{},{},,,", title[i], content[i]);
                if self.json_filters.is_some() {
                    result += ",,";
                    if self.json_write_template.is_some() {
                        result += ",,";
                    }
                }
                result.push('\n');
            }
        }
        if p.len() > 10 {
            let mut index = 10_usize;
            while index < p.len() {
                let w_status = if p[index].1 { "TRUE" } else { "FALSE" };
                result += &format!(" ,,{},{},{}", index + 1, p[index].0, w_status);
                if let Some(s) = pos_to_str.get(&index) {
                    result.push(',');
                    result += &s.as_str();
                } else if self.json_filters.is_some() {
                    result += ",,";
                    if let Some(templates) = &self.json_write_template {
                        result += ",";
                        if let Some(s) = templates.get(&p[index].0) {
                            result += &get_csv_str(s.as_str());
                        } else {
                            result += ",";
                        }
                        if let Some(write_tags) = &self.json_write_tag {
                            if let Some(s) = write_tags.get(&p[index].0) {
                                result += &get_csv_str(s.as_str());
                            } else {
                                result += ",";
                            }
                        } else {
                            result += ",";
                        }
                    }
                }
                result.push('\n');
                index += 1;
            }
        }
        result
    }

    pub fn get_point_ids(&self) -> Vec<u64> {
        let mut result = Vec::with_capacity(self.point_ids.len());
        for (id, _) in &self.point_ids {
            result.push(*id);
        }
        result
    }
}

pub fn split_json(value: &Value) -> BTreeMap<String, Value> {
    let mut result = BTreeMap::new();
    let mut stack = Vec::with_capacity(16);
    stack.push((value, String::new()));
    while let Some((v, s)) = stack.pop() {
        match v {
            Value::Object(obj) => {
                for (k, new_v) in obj {
                    let new_k = if !s.is_empty() {
                        format!("{}/{}", s, k)
                    } else {
                        k.clone()
                    };
                    stack.push((new_v, new_k));
                }
            }
            _ => {
                result.insert(s, v.clone());
            }
        }
    }
    result
}

pub fn merge_json(tags: &[String], values: &[Value]) -> Value {
    let mut result = Value::Object(Map::new());
    for i in 0..tags.len() {
        let mut stack = vec![&mut result];
        let names: Vec<&str> = tags[i].split('/').collect();
        for j in 0..names.len() {
            if j == names.len() - 1 {
                let obj = stack.pop().unwrap();
                obj[&names[j]] = values[i].clone();
            } else {
                let father = stack.pop().unwrap();
                if father[&names[j]] != Value::Null {
                    stack.push(father.get_mut(names[j]).unwrap());
                } else {
                    let value = Value::Object(Map::new());
                    father[&names[j]] = value;
                    stack.push(father.get_mut(names[j]).unwrap());
                }
            }
        }
    }
    result
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use serde_json::{json, Value};
    use crate::from_file;
    use crate::mqtt::{merge_json, MqttTransport, split_json};
    use crate::utils::check_mqtt_transport;

    #[test]
    fn test_parse_mqtt_csv() {
        let tp = MqttTransport::from_csv("tests/mqtt-test1.csv").unwrap();
        assert_eq!(tp.name, "Mqtt测试通道");
        assert_eq!(tp.mqtt_broker, ("127.0.0.1".to_string(), 1883u16));
        assert_eq!(tp.read_topic, "gwIn/read");
        assert_eq!(tp.write_topic, "gwOut/write");
        assert_eq!(tp.point_id, 9001);
        assert_eq!(tp.point_ids.len(), 10);
        let s = tp.export_csv(&HashMap::with_capacity(0));
        let tp2 = MqttTransport::from_csv_bytes(s.as_bytes()).unwrap();
        assert_eq!(tp, tp2);

        let tp = MqttTransport::from_csv("tests/mqtt-test2.csv").unwrap();
        assert_eq!(tp.user_name, Some(String::from("admin")));
        assert_eq!(tp.user_password, Some(String::from("admin")));
        assert!(tp.is_json);
        assert!(tp.is_transfer);
        let s = tp.export_csv(&HashMap::with_capacity(0));
        let tp2 = MqttTransport::from_csv_bytes(s.as_bytes()).unwrap();
        assert_eq!(tp, tp2);

        let tp = MqttTransport::from_csv("tests/mqtt-test3.csv").unwrap();
        assert_eq!(tp.point_ids.len(), 4);
        let s = tp.export_csv(&HashMap::with_capacity(0));
        let tp2 = MqttTransport::from_csv_bytes(s.as_bytes()).unwrap();
        assert_eq!(tp, tp2);

        let tp = MqttTransport::from_csv("tests/mqtt-transport-db1.csv").unwrap();
        assert_eq!(tp.point_ids.len(), 6);
        let s = tp.export_csv(&HashMap::with_capacity(0));
        let tp2 = MqttTransport::from_csv_bytes(s.as_bytes()).unwrap();
        assert_eq!(tp, tp2);
    }

    #[test]
    fn test_export_mqtt_model() {
        let tp1 = MqttTransport::from_csv("tests/mqtt-test1.csv").unwrap();
        let tp1_export = tp1.export_csv(&HashMap::with_capacity(0));
        let tp1_parse_from_export = MqttTransport::from_csv_bytes(tp1_export.as_bytes());
        assert_eq!(tp1_parse_from_export, Ok(tp1));

        let tp2 = MqttTransport::from_csv("tests/mqtt-test2.csv").unwrap();
        let tp2_export = tp2.export_csv(&HashMap::with_capacity(0));
        let tp2_parse_from_export = MqttTransport::from_csv_bytes(tp2_export.as_bytes());
        assert_eq!(tp2_parse_from_export, Ok(tp2));
    }

    #[test]
    fn test_parse_with_custom_json() {
        let v1 = json!({
            "code": 200,
            "success": true,
            "payload": {
                "features": [
                    "serde",
                    "json"
                ],
                "homepage": null
            }
        });
        let map = split_json(&v1);
        let keys: Vec<String> = map.keys().cloned().collect();
        let values: Vec<Value> = map.values().cloned().collect();
        let v2 = merge_json(&keys, &values);
        assert_eq!(v1, v2);
        let tp = MqttTransport::from_csv("tests/mqtt-test4.csv").unwrap();
        let s = tp.export_csv(&HashMap::with_capacity(0));
        // println!("{}", s);
        let tp2 = MqttTransport::from_csv_bytes(s.as_bytes()).unwrap();
        assert_eq!(tp, tp2);
        // println!("{:?}", tp);
        assert_eq!(tp.point_ids.len(), 10);
        assert_eq!(tp.json_filters.unwrap().len(), 1);
        assert_eq!(tp.json_tags.unwrap().len(), 3);
    }

    #[test]
    fn test_custom_json2() {
        let tp1 = MqttTransport::from_csv("tests/mqtt-test5.csv").unwrap();
        assert!(tp1.json_tags.is_some());
        assert_eq!(1, tp1.json_tags.as_ref().unwrap().len());
        let tags = tp1.json_tags.as_ref().unwrap().get("[0]");
        assert!(tags.is_some());
        assert_eq!(10, tags.unwrap().len());
        let tp1_export = tp1.export_csv(&HashMap::with_capacity(0));
        let tp1_parse_from_export = MqttTransport::from_csv_bytes(tp1_export.as_bytes());
        assert_eq!(tp1_parse_from_export, Ok(tp1));

        let r = MqttTransport::from_csv("tests/mqtt-test6.csv");
        assert_eq!(r, Err((2, 6)));
    }
    #[test]
    fn test_custom_json3() {
        let tp1 = MqttTransport::from_file("tests/mqtt-transport-nb1.xlsx");
        assert!(tp1.is_ok());
        let tp1 = tp1.unwrap();
        assert!(tp1.json_tags.is_some());
        let (points, _) = from_file("tests/points-nb1.xlsx").unwrap();
        let r = check_mqtt_transport(&points, &tp1);
        assert_eq!(r, Ok(()));
        let tp1_export = tp1.export_csv(&HashMap::with_capacity(0));
        // println!("{}", tp1_export);
        let tp1_parse_from_export = MqttTransport::from_csv_bytes(tp1_export.as_bytes());
        assert_eq!(tp1_parse_from_export, Ok(tp1));
    }
}
