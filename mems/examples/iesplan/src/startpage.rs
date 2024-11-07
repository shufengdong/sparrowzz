use gloo_timers::callback::Timeout;
use std::collections::HashMap;
use yew::prelude::*;

use yew_bulma::layout::tiles::Tiles;
use yew_bulma::*;

use crate::paracard::ParaCard;
use crate::{build_tiles, create_parameters, Parameters};

pub enum Msg {
    Start,
    Stop,
}

pub struct StartPage {
    tiles: Tiles,
    cards: Vec<Parameters>,
    timer: Option<Timeout>,
    is_running: bool
}

impl Component for StartPage {
    type Message = Msg;
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        let tiles = build_tiles(include_bytes!("../layoutV3.xlsx").to_vec()).unwrap();
        let card0 = create_parameters(include_bytes!("../card/card0.csv"));
        let card2 = create_parameters(include_bytes!("../card/card2.csv"));
        let card3 = create_parameters(include_bytes!("../card/card3.csv"));
        let card4 = create_parameters(include_bytes!("../card/card4.csv"));
        let card5 = create_parameters(include_bytes!("../card/card5.csv"));
        let card6 = create_parameters(include_bytes!("../card/card6.csv"));
        let card7 = create_parameters(include_bytes!("../card/card7.csv"));
        let card8 = create_parameters(include_bytes!("../card/card8.csv"));
        let card9 = create_parameters(include_bytes!("../card/card9.csv"));
        let card10 = create_parameters(include_bytes!("../card/card10.csv"));
        let card11 = create_parameters(include_bytes!("../card/card11.csv"));
        let cards = vec![card0, card2, card3, card4, card5, card6, card7, card8, card9, card10, card11];
        Self { tiles, cards, timer: None, is_running: false }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let link = ctx.link();
        match msg {
            Msg::Start => {
                self.is_running = true;
                let stop_running = link.callback(|()| Msg::Stop);
                let timeout = Timeout::new(5_000, move || {
                    stop_running.emit(());
                });
                self.timer = Some(timeout);
            }
            Msg::Stop => {
                self.is_running = false;
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let mut nodes = HashMap::with_capacity(12);
        nodes.insert(1, html! {
           <Button loading={self.is_running} classes={classes!("is-primary", "is-fullwidth", "is-large")} 
                onclick={link.callback(|_|Msg::Start)}>{"开始计算"}</Button>  
        });
        for card in &self.cards {
            nodes.insert(card.id, html! {
                <ParaCard paras={card.clone()} />
            });
        }
        self.tiles.create_html(nodes)
    }

    fn destroy(&mut self, _: &Context<Self>) {
        if let Some(timer) = self.timer.take() {
            timer.cancel();
        }
    }
}
