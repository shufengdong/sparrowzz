use std::collections::HashMap;

use yew::prelude::*;

use yew_bulma::calendar::*;
use yew_bulma::*;
use yew_bulma::layout::tiles::Tiles;

use crate::build_tiles;
pub enum Msg {
    DateRangePicked(u64, u64),
}

pub struct StartPage {
    tiles: Tiles,
    chart1: NodeRef,
    chart2: NodeRef,
    chart3: NodeRef,
    chart4: NodeRef,
}

impl Component for StartPage {
    type Message = Msg;
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        let tiles  = build_tiles(include_bytes!("../layout.xlsx").to_vec()).unwrap();
        Self {
            tiles,
            chart1: Default::default(),
            chart2: Default::default(),
            chart3: Default::default(),
            chart4: Default::default(),
        }
    }

    fn update(&mut self, _: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::DateRangePicked(start, end) => {
                alert(&format!("{} - {}", start, end));
            }
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let mut date_picker_text = HashMap::new();
        date_picker_text.insert("ok".to_string(), "确定".to_string());
        date_picker_text.insert("cancel".to_string(), "取消".to_string());
        date_picker_text.insert("choose_date".to_string(), "选择日期".to_string());
        date_picker_text.insert("now".to_string(), "当前时间".to_string());
        date_picker_text.insert("today".to_string(), "今日".to_string());
        date_picker_text.insert("clear".to_string(), "清除".to_string());
        date_picker_text.insert("validate".to_string(), "验证".to_string());
        html! {
            <>
            <Level>
                <LevelLeft>
                    <LevelItem>
                        <Title classes={classes!("has-text-primary")}>
                            <Icon classes ={classes!("icon-title")} size={Size::Small} awesome_icon={"fa fa-tachometer"} />
                            <span>{"Dashboard"}</span>
                        </Title>
                    </LevelItem>
                </LevelLeft>
                <LevelRight>
                    <LevelItem>
                        <DatePicker on_date_picked={link.callback(|(t1,t2)|Msg::DateRangePicked(t1,t2))}
                            text_map={date_picker_text.clone()} />
                    </LevelItem>
                    <LevelItem>
                        <DatePicker on_date_picked={link.callback(|(t1,t2)|Msg::DateRangePicked(t1, t2))}
                            text_map={date_picker_text} is_range={true} picker_type={PickerType::Datetime} is_button={true}/>
                    </LevelItem>
                </LevelRight>
            </Level>
            <Columns multiline={true}>
                <Column>
                    <Notification classes={Classes::from("is-primary")}>
                        <div class={"heading"}>{"Top Seller Total"}</div>
                        <Title>{"56,950"}</Title>
                        <Level>
                            <LevelItem>
                                <div>
                                    <div class={"heading"}>{"Sales $"}</div>
                                    <Title size={HeaderSize::Is5}>{"250K"}</Title>
                                </div>
                            </LevelItem>
                            <LevelItem>
                                <div>
                                    <div class={"heading"}>{"Overall $"}</div>
                                    <Title size={HeaderSize::Is5}>{"750K"}</Title>
                                </div>
                            </LevelItem>
                            <LevelItem>
                                <div>
                                    <div class={"heading"}>{"Sales $"}</div>
                                    <Title size={HeaderSize::Is5}>{"25%"}</Title>
                                </div>
                            </LevelItem>
                        </Level>
                    </Notification>
                </Column>
                <Column>
                    <Notification classes={Classes::from("is-warning")}>
                        <div class={"heading"}>{"Revenue / Expenses"}</div>
                        <Title>{"55% / 45%"}</Title>
                        <Level>
                            <LevelItem>
                                <div>
                                    <div class={"heading"}>{"Rev Prod $"}</div>
                                    <Title size={HeaderSize::Is5}>{"30%"}</Title>
                                </div>
                            </LevelItem>
                            <LevelItem>
                                <div>
                                    <div class={"heading"}>{"Rev Serv $"}</div>
                                    <Title size={HeaderSize::Is5}>{"25%"}</Title>
                                </div>
                            </LevelItem>
                            <LevelItem>
                                <div>
                                    <div class={"heading"}>{"Exp %"}</div>
                                    <Title size={HeaderSize::Is5}>{"45%"}</Title>
                                </div>
                            </LevelItem>
                        </Level>
                    </Notification>
                </Column>
                <Column>
                    <Notification classes={Classes::from("is-info")}>
                        <div class={"heading"}>{"Revenue / Expenses"}</div>
                        <Title>{"55% / 45%"}</Title>
                        <Level>
                            <LevelItem>
                                <div>
                                    <div class={"heading"}>{"Pos"}</div>
                                    <Title size={HeaderSize::Is5}>{"1560"}</Title>
                                </div>
                            </LevelItem>
                            <LevelItem>
                                <div>
                                    <div class={"heading"}>{"Neg"}</div>
                                    <Title size={HeaderSize::Is5}>{"368"}</Title>
                                </div>
                            </LevelItem>
                            <LevelItem>
                                <div>
                                    <div class={"heading"}>{"Pos/Neg %"}</div>
                                    <Title size={HeaderSize::Is5}>{"77%"}</Title>
                                </div>
                            </LevelItem>
                        </Level>
                    </Notification>
                </Column>
                <Column>
                    <Notification classes={Classes::from("is-danger")}>
                        <div class={"heading"}>{"Orders / Returns"}</div>
                        <Title>{"75% / 25%"}</Title>
                        <Level>
                            <LevelItem>
                                <div>
                                    <div class={"heading"}>{"Orders $"}</div>
                                    <Title size={HeaderSize::Is5}>{"425K"}</Title>
                                </div>
                            </LevelItem>
                            <LevelItem>
                                <div>
                                    <div class={"heading"}>{"Returns $"}</div>
                                    <Title size={HeaderSize::Is5}>{"106K"}</Title>
                                </div>
                            </LevelItem>
                            <LevelItem>
                                <div>
                                    <div class={"heading"}>{"Success %"}</div>
                                    <Title size={HeaderSize::Is5}>{"+ 28,5%"}</Title>
                                </div>
                            </LevelItem>
                        </Level>
                    </Notification>
                </Column>
            </Columns>
            <Columns multiline={true}>
                <Column classes={Classes::from("is-6")}>
                    <Message classes={Classes::from("is-dark")}>
                        <MessageHeader><p>{"Chart1"}</p></MessageHeader>
                        <MessageBody><div ref={self.chart1.clone()} style={"width: 100%"}/></MessageBody>
                    </Message>
                </Column>
                <Column classes={Classes::from("is-6")}>
                    <Message classes={Classes::from("is-dark")}>
                        <MessageHeader><p>{"Chart2"}</p></MessageHeader>
                        <MessageBody><div ref={self.chart2.clone()} style={"width: 100%"}/></MessageBody>
                    </Message>
                </Column>
            </Columns>
            </>
        }
    }
}
