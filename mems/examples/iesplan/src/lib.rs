use crate::startpage::StartPage;
use eig_domain::excel::get_first_sheet_merged_cells;
use eig_domain::{csv_str, csv_string, csv_u64, csv_usize, SetPointValue};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::{Element, Headers};
use yew_bulma::layout::tiles::Tiles;
pub mod startpage;
mod paracard;

#[wasm_bindgen]
pub fn create_view(e: Element) {
    yew::Renderer::<StartPage>::with_root(e).render();
}

#[wasm_bindgen(raw_module = "/mems-view-bin.js")]
extern "C" {
    pub fn get_headers() -> Headers;
    pub fn get_user_id() -> u16;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PointControl3 {
    pub commands: Vec<SetPointValue>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QueryWithId {
    pub id: Option<u64>,
    pub ids: Option<String>,
}

impl QueryWithId {
    pub fn query_str(&self) -> String {
        let mut query = String::new();
        if let Some(id) = self.id {
            query.push_str(&format!("?id={}", id));
        } else if let Some(ids) = &self.ids {
            query.push_str(&format!("?ids={ids}"));
        }
        query
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ParaType {
    // show expression, true expression, false expression
    Checkbox,
    Switch,
    Radio(Vec<(String, f64)>),
    Select(Vec<(String, f64)>),
    // min, max, step
    Slider(f64, f64, f64),
    TextField,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Parameters {
    id: usize,
    name: String,
    labels: Vec<String>,
    points: Vec<u64>,
    para_types: Vec<ParaType>,
}

pub fn create_parameters(content: &[u8]) -> Parameters {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(content);
    let mut records = rdr.records();
    let record = records.next().unwrap().unwrap();
    let id = csv_usize(&record, 0).unwrap();
    let name = csv_string(&record, 1).unwrap();
    let mut labels = Vec::new();
    let mut points = Vec::new();
    let mut para_types = Vec::new();
    for record in records {
        let row = record.unwrap();
        points.push(csv_u64(&row, 0).unwrap());
        labels.push(csv_string(&row, 1).unwrap());
        let para_type_s = csv_str(&row, 2).unwrap().to_uppercase();
        let para_type = match para_type_s.as_str() {
            "CHECKBOX" => ParaType::Checkbox,
            "SWITCH" => ParaType::Switch,
            "TEXTFIELD" => ParaType::TextField,
            "SLIDER" => {
                let v = csv_str(&row, 3).unwrap();
                let s_vec: Vec<&str> = v.split(";").collect();
                let min = s_vec[0].parse().unwrap();
                let max = s_vec[1].parse().unwrap();
                let step = s_vec[2].parse().unwrap();
                ParaType::Slider(min, max, step)
            },
            "SELECT" => {
                let v = csv_str(&row, 3).unwrap();
                let options = parse_options(v);
                ParaType::Select(options)
            }
            "RADIO" => {
                let v = csv_str(&row, 3).unwrap();
                let options = parse_options(v);
                ParaType::Radio(options)
            }
            _ => ParaType::TextField
        };
        para_types.push(para_type);
    }
    Parameters { id, name, labels, points, para_types }
}

fn parse_options(v: &str) -> Vec<(String, f64)> {
    let options = v.split(";")
        .map(|s| {
            let options: Vec<&str> = s.split(":").collect();
            if options.len() == 2 {
                (options[0].to_string(), options[1].parse::<f64>().unwrap())
            } else {
                ("".to_string(), s.parse::<f64>().unwrap())
            }
        }).collect();
    options
}

pub fn build_tiles(xlsx_bytes: Vec<u8>) -> Option<Tiles> {
    let (m, n, merge_map, values) = get_first_sheet_merged_cells(xlsx_bytes)?;
    let mut class_str = Vec::new();
    let mut style_str = Vec::new();
    let mut is_dealt = vec![false; (m * n) as usize];
    for i in 0..m {
        for j in 0..n {
            let index = (i * n + j) as usize;
            if is_dealt[index] {
                continue;
            }
            let mut class_s = "kanban-div cell".to_string();
            let coordinate = (i, j);
            if let Some((end_row, end_col)) = merge_map.get(&coordinate) {
                let row_span = *end_row - i + 1;
                let col_span = *end_col - j + 1;
                if row_span > 1 {
                    class_s.push_str(&format!(" is-row-span-{row_span}"))
                }
                if col_span > 1 {
                    class_s.push_str(&format!(" is-col-span-{col_span}"))
                }
                class_str.push(class_s);
                for row in i..=*end_row {
                    for col in j..=*end_col {
                        let pos = (row * n + col) as usize;
                        is_dealt[pos] = true;
                    }
                }
            } else {
                class_str.push(class_s);
            }
            let h = values.get(&index).cloned().unwrap_or("100".to_string());
            let s = format!("height:{h}px");
            style_str.push(s);
        }
    }
    let tiles = Tiles { id: "".to_string(), class_str, style_str, with_box: true };
    Some(tiles)
}