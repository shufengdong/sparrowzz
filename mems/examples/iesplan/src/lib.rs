use wasm_bindgen::prelude::*;
use web_sys::Element;

use crate::startpage::StartPage;
use yew_bulma::layout::tiles::Tiles;
use eig_domain::excel::get_first_sheet_merged_cells;
pub mod startpage;

#[wasm_bindgen]
pub fn create_view(e: Element) {
    yew::Renderer::<StartPage>::with_root(e).render();
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