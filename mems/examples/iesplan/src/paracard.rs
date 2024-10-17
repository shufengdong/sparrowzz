use wasm_bindgen::JsCast;
use web_sys::InputEvent;
use yew::prelude::*;
use yew_bulma::*;

pub enum ParaType {
    // show expresion, true expression, false expression
    Checkbox,
    Radio,
    Switch,
    Select(Vec<f64>),
    Slider(f64, f64, f64, bool),
    TextField,
}

pub enum Msg {
    SetBool(usize, bool),
    SetString(usize),
    SetOption(usize, String),
}

pub struct ParaCard {
    labels: Vec<String>,
    points: Vec<u64>,
    para_types: Vec<ParaType>,
}

impl Component for ParaCard {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        todo!()
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let input_html = (0..self.points.len()).map(|i| {
            self.create_input(ctx, i)
        }).collect::<Html>();
        html! {
            <Card>
                <CardContent>
                    {input_html}
                </CardContent>
            </Card>
        }
    }
}

impl ParaCard {
    fn create_input(&self, ctx: &Context<Self>, i: usize) -> Html {
        let point_id = &self.points[i];
        let input_type = &self.para_types[i];
        let link = ctx.link();
        let label = if let Some(label) = self.labels.get(i) {
            label.clone()
        } else {
            "".to_string()
        };
        match input_type {
            ParaType::Checkbox => {
                let checked = true;
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
                let checked = true;
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
                let checked = true;
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