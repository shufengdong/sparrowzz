use std::collections::HashMap;
use yew::prelude::*;

use yew_bulma::layout::tiles::Tiles;
use yew_bulma::*;

use crate::build_tiles;
pub enum Msg {
    start,
}

pub struct StartPage {
    tiles: Tiles,
}

impl Component for StartPage {
    type Message = Msg;
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        let tiles  = build_tiles(include_bytes!("../layout.xlsx").to_vec()).unwrap();
        Self { tiles }
    }

    fn update(&mut self, _: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::start => {
                alert("start");
            }
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut nodes = HashMap::with_capacity(22);
        for i in 0..22 {
            let v = html! {
                <p>{format!("{i}")}</p>
            };
            nodes.insert(i, v);
        }
        self.tiles.create_html(nodes)
    }
}
