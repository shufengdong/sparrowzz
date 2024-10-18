use wasm_bindgen::prelude::*;
use web_sys::Element;
use crate::startpage::StartPage;
use yew_bulma::layout::tiles::Tiles;
use eig_domain::{csv_string, csv_u64, csv_usize};
use eig_domain::excel::get_first_sheet_merged_cells;
pub mod startpage;
mod paracard;

#[wasm_bindgen]
pub fn create_view(e: Element) {
    yew::Renderer::<StartPage>::with_root(e).render();
}

#[derive(Clone, Debug, PartialEq)]
pub enum ParaType {
    // show expresion, true expression, false expression
    Checkbox,
    Radio,
    Switch,
    Select(Vec<f64>),
    Slider(f64, f64, f64, bool),
    TextField,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Parameters {
    id: usize,
    name: String,
    height: usize,
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
    let height = csv_usize(&record, 2).unwrap();
    let mut labels = Vec::new();
    let mut points = Vec::new();
    let mut para_types = Vec::new();
    for record in records {
        let row = record.unwrap();
        points.push(csv_u64(&row, 0).unwrap());
        labels.push(csv_string(&row, 1).unwrap());
        para_types.push(ParaType::Checkbox);
    }
    Parameters { id, name, height, labels, points, para_types }
}

pub fn build_tiles(xlsx_bytes: Vec<u8>) -> Option<Tiles> {
    let (m, n, merge_map) = get_first_sheet_merged_cells(xlsx_bytes)?;
    let mut class_vec = Vec::new();
    let mut is_dealt = vec![false; (m * n) as usize];
    for i in 0..m {
        for j in 0..n {
            let index = (i * n + j) as usize;
            if is_dealt[index] {
                continue;
            }
            let mut class_str = "cell".to_string();
            let coordinate = (i, j);
            if let Some((end_row, end_col)) = merge_map.get(&coordinate) {
                let row_span = *end_row - i + 1;
                let col_span = *end_col - j + 1;
                if row_span > 1 {
                    class_str.push_str(&format!(" is-row-span-{row_span}"))
                }
                if col_span > 1 {
                    class_str.push_str(&format!(" is-col-span-{col_span}"))
                }
                class_vec.push(class_str);
                for row in i..=*end_row {
                    for col in j..=*end_col {
                        let pos = (row * n + col) as usize;
                        is_dealt[pos] = true;
                    }
                }
            } else {
                class_vec.push(class_str);
            }
        }
    }
    let tiles = Tiles { id: "".to_string(), class_str: class_vec, with_box: true };
    Some(tiles)
}