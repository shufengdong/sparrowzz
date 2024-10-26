use eig_domain::{MeasureValue, SetPointValue};
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use web_sys::InputEvent;
use yew::prelude::*;
use yew_bulma::calendar::get_timestamp;
use yew_bulma::*;

use crate::{get_headers, get_user_id, ParaType, Parameters, PointControl3, QueryWithId};

pub enum Msg {
    Refresh,
    ParaLoaded(Vec<MeasureValue>),
    SetBool(usize, bool),
    SetString(usize),
    SetOption(usize, String),
    SetParaSuccess(Vec<u64>),
    None,
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub paras: Parameters,
}

pub struct ParaCard {
    bools: HashMap<usize, bool>,
    floats: HashMap<usize, f64>,
    pos: HashMap<u64, usize>,
}

impl Component for ParaCard {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let mut bools = HashMap::new();
        let mut floats = HashMap::new();
        let mut pos = HashMap::new();
        for index in 0..ctx.props().paras.points.len() {
            let input_type = &ctx.props().paras.para_types[index];
            if ParaType::Checkbox.eq(input_type)
                || ParaType::Switch.eq(input_type) {
                bools.insert(index, false);
            } else {
                floats.insert(index, 0.0);
            }
            pos.insert(ctx.props().paras.points[index], index);
        }
        Self::query_para(ctx, &ctx.props().paras.points);
        Self { bools, floats, pos }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Refresh => {
                Self::query_para(ctx, &ctx.props().paras.points);
            }
            Msg::ParaLoaded(values) => {
                for v in values {
                    if let Some(index) = self.pos.get(&v.point_id) {
                        if let Some(b) = self.bools.get_mut(index) {
                            *b = v.get_value() > 0.0;
                        } else if let Some(f) = self.floats.get_mut(index) {
                            *f = v.get_value();
                        }
                    }
                }
                return true;
            }
            Msg::SetBool(i, b) => {
                let point_id = &ctx.props().paras.points[i];
                let value = if b {
                    "1.0"
                } else {
                    "0.0"
                };
                self.do_set_point(ctx, value, point_id);
            }
            Msg::SetString(i) => {
                let point_id = &ctx.props().paras.points[i];
                let name = format!("tf_{}", point_id);
                let value = get_input_value_by_name(&name);
                self.do_set_point(ctx, &value, point_id);
            }
            Msg::SetOption(i, value) => {
                if value == "None" {
                    return false;
                }
                let point_id = &ctx.props().paras.points[i];
                self.do_set_point(ctx, &value, point_id);
            }
            Msg::SetParaSuccess(points) => {
                Self::query_para(ctx, &points);
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
                    <Field horizontal={true} label={label}>
                        <Checkbox checked={checked}
                            update={link.callback(move |b| Msg::SetBool(i, b))}>
                        </Checkbox>
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
            ParaType::Slider(lower, upper, step) => {
                let current_v = self.floats.get(&i).cloned().unwrap_or(*lower).to_string();
                let oninput = link.callback(move |e: InputEvent| {
                    let target = e.target().unwrap();
                    let input = target.dyn_into::<web_sys::HtmlInputElement>().unwrap();
                    Msg::SetOption(i, input.value())
                });
                html! {
                    <Field horizontal={true} label={label}>
                        <input class={"slider is-fullwidth"}  type={"range"} orient={"horizontal"}
                            oninput={oninput} step={step.to_string()} min={lower.to_string()}
                            max={upper.to_string()} value={current_v}
                        />
                    </Field>
                }
            }
            ParaType::Select(options) => {
                let current_f = self.floats.get(&i).cloned().unwrap_or(0.0);
                let content = (0..options.len()).map(|i| {
                    let (name, f) = &options[i];
                    let to_show = if name.is_empty() {
                        f.to_string()
                    } else {
                        name.clone()
                    };
                    html! {
                        <option value={f.to_string()} selected={current_f == *f}>
                            {to_show}
                        </option>
                    }
                }).collect::<Html>();
                html! {
                    <Field horizontal={true} label={label}>
                        <Select update={link.callback(move |s| Msg::SetOption(i, s))} >
                            {content}
                            <option value={"None"}>{"no_selection"}</option>
                        </Select>
                    </Field>
                }
            }
            ParaType::Radio(options) => {
                let current_f = self.floats.get(&i).cloned().unwrap_or(0.0);
                let content = (0..options.len()).map(|j| {
                    let (name, f) = &options[j];
                    let to_show = if name.is_empty() {
                        f.to_string()
                    } else {
                        name.clone()
                    };
                    let checked_value = if current_f == *f {
                        f.to_string()
                    } else {
                        current_f.to_string()
                    };
                    let value = f.to_string();
                    html! {
                        <Radio update={link.callback(move |_| Msg::SetOption(i, value.clone()))}
                            checked_value={checked_value} value={f.to_string()}>
                            <span>{to_show}</span>
                        </Radio>
                    }
                }).collect::<Html>();
                html! {
                    <Field horizontal={true} label={label}>
                        <div class="radios">{content}</div>
                    </Field>
                }
            }
            ParaType::TextField => {
                let name = format!("tf_{}", point_id);
                let f = self.floats.get(&i).cloned().unwrap_or(0.0).to_string();
                html! {
                    <Field classes={classes!("has-addons")} horizontal={true} label={label}>
                        <Control classes={classes!("is-expanded")}>
                            <Input placeholder={"eg: 10"} width={"12"} name={name} value={f}
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

    fn do_set_point(&mut self, ctx: &Context<Self>, value: &str, point_id: &u64) {
        if let Ok(expr) = value.parse() {
            let user_id = get_user_id();
            let v = SetPointValue {
                point_id: *point_id,
                sender_id: user_id as u64,
                command: expr,
                timestamp: get_timestamp(),
            };
            Self::set_point(ctx, PointControl3 { commands: vec![v] });
        } else {
            alert("Wrong input");
        }
    }

    fn query_para(ctx: &Context<Self>, points: &[u64]) {
        let ids: Vec<String> = points.iter().map(|s| s.to_string()).collect();
        let ids = ids.join(",").to_string();
        let query = QueryWithId {
            id: None,
            ids: Some(ids),
        };
        let url = format!("/api/v1/pscpu/points/values_cbor/0{}", query.query_str());
        ctx.link().send_future(async move {
            match async_ws_get(&url, &get_headers()).await {
                Ok(bytes) => {
                    if let Ok(values) = serde_cbor::from_slice::<Vec<MeasureValue>>(&bytes) {
                        return Msg::ParaLoaded(values);
                    } else {
                        alert("Fail");
                    }
                }
                Err(err) => {
                    if err.to_string().eq(HEADER_TOKEN_INVALID) {
                        alert(&format!("Invalid header token for url: {url}"));
                    } else if err.to_string().eq(HEADER_PERMISSION_DENIED) {
                        alert(&format!("Permission denied for url: {}", url));
                    } else {
                        alert(&format!("Failed to load parameter values, err: {:?}", err));
                    }
                }
            }
            Msg::None
        });
    }
    fn set_point(ctx: &Context<Self>, cmd: PointControl3) {
        let url = "/api/v1/controls_cbor/points_by_expr";
        let points: Vec<u64> = cmd.commands.iter().map(|c| c.point_id).collect();
        ctx.link().send_future(async move {
            let content = serde_cbor::to_vec(&cmd).unwrap();
            let body = js_sys::Uint8Array::from(content.as_ref()).dyn_into().unwrap();
            match async_ws_post_no_resp(url, &get_headers(), Some(body)).await {
                Ok(b) => {
                    if !b {
                        alert("Fail to set parameter");
                    } else {
                        return Msg::SetParaSuccess(points);
                    }
                }
                Err(err) => {
                    if err.to_string().eq(HEADER_TOKEN_INVALID) {
                        alert(&format!("Invalid header token for url: {url}"));
                    } else if err.to_string().eq(HEADER_PERMISSION_DENIED) {
                        alert(&format!("Permission denied for url: {}", url));
                    } else {
                        alert(&format!("Failed to set parameter value, err: {:?}", err));
                    }
                }
            }
            Msg::None
        });
    }
}