use std::collections::HashMap;
use wasm_bindgen::JsCast;
use web_sys::InputEvent;
use yew::prelude::*;
use yew_bulma::*;
use yew_bulma::calendar::get_timestamp;
use eig_domain::SetPointValue;
use eig_expr::{Expr, Token};
use crate::{get_headers, get_user_id, ParaType, Parameters, PointControl3};

pub enum Msg {
    SetBool(usize, bool),
    SetString(usize),
    SetOption(usize, String),
    None,
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub paras: Parameters,
}

pub struct ParaCard {
    bools: HashMap<usize, bool>,
}

impl Component for ParaCard {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let mut bools = HashMap::new();
        for index in 0..ctx.props().paras.points.len() {
            let input_type = &ctx.props().paras.para_types[index];
            if let ParaType::Checkbox = input_type {
                bools.insert(index, false);
            } else if let ParaType::Switch = input_type {
                bools.insert(index, false);
            } else if let ParaType::Radio = input_type {
                bools.insert(index, false);
            }
        }
        Self { bools }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetBool(i, b) => {
                let point_id = &ctx.props().paras.points[i];
                let input_type = &ctx.props().paras.para_types[i];
                let v = if ParaType::Checkbox.eq(input_type)
                    || ParaType::Radio.eq(input_type)
                    || ParaType::Switch.eq(input_type) {
                    if b {
                        Some(Expr::from_vec(vec![Token::Number(1.0)]))
                    } else {
                        Some(Expr::from_vec(vec![Token::Number(0.0)]))
                    }
                } else {
                    None
                };
                if let Some(expr) = v {
                    let user_id = get_user_id();
                    let v = SetPointValue {
                        point_id: *point_id,
                        sender_id: user_id as u64,
                        command: expr,
                        timestamp: get_timestamp(),
                    };
                    self.set_point(ctx, PointControl3 { commands: vec![v] });
                }
                if let Some(v) = self.bools.get_mut(&i) {
                    *v = b;
                    return true;
                }
            }
            Msg::SetString(i) => {
                let point_id = &ctx.props().paras.points[i];
                let name = format!("tf_{}", point_id);
                let value = get_input_value_by_name(&name);
                if let Ok(expr) = value.parse() {
                    let user_id = get_user_id();
                    let v = SetPointValue {
                        point_id: *point_id,
                        sender_id: user_id as u64,
                        command: expr,
                        timestamp: get_timestamp(),
                    };
                    self.set_point(ctx, PointControl3 { commands: vec![v] });
                } else {
                    alert("Wrong input");
                }
            }
            Msg::SetOption(i, value) => {
                if value == "None" {
                    return false;
                }
                let point_id = &ctx.props().paras.points[i];
                if let Ok(expr) = value.parse() {
                    let user_id = get_user_id();
                    let v = SetPointValue {
                        point_id: *point_id,
                        sender_id: user_id as u64,
                        command: expr,
                        timestamp: get_timestamp(),
                    };
                    self.set_point(ctx, PointControl3 { commands: vec![v] });
                } else {
                    alert("Wrong input");
                }
            }
            Msg::None => {}
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let paras = &ctx.props().paras;
        let input_html = (0..paras.points.len()).map(|i| {
            self.create_input(ctx, i)
        }).collect::<Html>();
        html! {
            <Card>
                <CardHeader>
                    <p class="card-header-title">
                        {paras.name.clone()}
                    </p>
                </CardHeader>
                <CardContent>
                    {input_html}
                </CardContent>
            </Card>
        }
    }
}

impl ParaCard {
    fn create_input(&self, ctx: &Context<Self>, i: usize) -> Html {
        let paras = &ctx.props().paras;
        let point_id = &paras.points[i];
        let input_type = &paras.para_types[i];
        let link = ctx.link();
        let label = if let Some(label) = paras.labels.get(i) {
            label.clone()
        } else {
            "".to_string()
        };
        match input_type {
            ParaType::Checkbox => {
                let checked = self.bools.get(&i).cloned().unwrap_or(false);
                html! {
                    <Field horizontal={true} >
                        <Checkbox checked={checked}
                            update={link.callback(move |b| Msg::SetBool(i, b))}>
                            {label}
                        </Checkbox>
                    </Field>
                }
            }
            ParaType::Radio => {
                let checked = self.bools.get(&i).cloned().unwrap_or(false);
                html! {
                    <Field horizontal={true} >
                        <Radio update={link.callback(move |_| Msg::SetBool(i, !checked))}
                            checked_value={"selected"}
                            value={if checked {"selected"} else {"none"}}>
                            <span>{label}</span>
                        </Radio>
                    </Field>
                }
            }
            ParaType::Switch => {
                let checked = self.bools.get(&i).cloned().unwrap_or(false);
                html! {
                    <Field horizontal={true} label={label}>
                        <input class={classes!("mui-switch", "mui-switch-animbg")} type="checkbox"
                            checked={checked}
                            onclick={link.callback(move |_| Msg::SetBool(i, !checked))} />
                    </Field>
                }
            }
            ParaType::Slider(lower, upper, step, is_vertical) => {
                let oninput = link.callback(move |e: InputEvent| {
                    let target = e.target().unwrap();
                    let input = target.dyn_into::<web_sys::HtmlInputElement>().unwrap();
                    Msg::SetOption(i, input.value())
                });
                html! {
                    <Field horizontal={true} label={label}>
                        <input class={"slider is-fullwidth"}  type={"range"}
                            orient={if *is_vertical {"vertical"} else {"horizontal"}}
                            oninput={oninput} step={step.to_string()} min={lower.to_string()}
                            max={upper.to_string()} value={lower.to_string()}
                        />
                    </Field>
                }
            }
            ParaType::Select(options) => {
                html! {
                    <Field horizontal={true} label={label}>
                        <Select update={link.callback(move |s| Msg::SetOption(i, s))} >
                            {for options.iter().map(|f| {
                                html! {<option value={f.to_string()}>{f.to_string()}</option>}
                            })}
                            <option value={"None"}>{"no_selection"}</option>
                        </Select>
                    </Field>
                }
            }
            ParaType::TextField => {
                let name = format!("tf_{}", point_id);
                html! {
                    <Field horizontal={true} label={label}>
                        <Control classes={classes!("is-expanded")}>
                            <Input placeholder={"eg: 10"} width={"12"} name={name}
                                onenterdown={link.callback(move |_| Msg::SetString(i))} />
                        </Control>
                        <Control>
                            <Button classes={classes!("is-outlined")}
                                onclick={link.callback(move |_| Msg::SetString(i))}>
                                <Icon awesome_icon={"fa fa-check"} />
                            </Button>
                        </Control>
                    </Field>
                }
            }
        }
    }
}

impl ParaCard {
    fn set_point(&self, ctx: &Context<Self>, cmd: PointControl3) {
        let url = "/api/v1/controls_cbor/points_by_expr";
        ctx.link().send_future(async move {
            let content = serde_cbor::to_vec(&cmd).unwrap();
            let body = js_sys::Uint8Array::from(content.as_ref()).dyn_into().unwrap();
            match async_ws_post_no_resp(url, &get_headers(), Some(body)).await {
                Ok(b) => {
                    if b {
                        alert("Success");
                    } else {
                        alert("Fail");
                    }
                }
                Err(err) => {
                    if err.to_string().eq(HEADER_TOKEN_INVALID) {
                        alert(&format!("Invalid header token for url: {url}"));
                    } else if err.to_string().eq(HEADER_PERMISSION_DENIED) {
                        alert(&format!("Permission denied for url: {}", url));
                    } else {}
                }
            }
            Msg::None
        });
    }
}