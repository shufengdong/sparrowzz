use std::collections::HashMap;
use yew::prelude::*;

use yew_bulma::layout::tiles::Tiles;
use yew_bulma::*;

use crate::{build_tiles, create_parameters, Parameters};
use crate::paracard::ParaCard;

pub enum Msg {
    start,
}

pub struct StartPage {
    tiles: Tiles,
    general_para: Parameters,
}

impl Component for StartPage {
    type Message = Msg;
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        let tiles = build_tiles(include_bytes!("../layout.xlsx").to_vec()).unwrap();
        let general_para = create_parameters(include_bytes!("../general.csv"));
        Self { tiles, general_para }
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
        nodes.insert(self.general_para.id, html! {
           <ParaCard paras={self.general_para.clone()} />
        });
        self.tiles.create_html(nodes)
    }
}
