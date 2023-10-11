use chrono::Duration;
use palette::LinSrgb;

use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Clone, Debug)]
pub(crate) struct Glow {
    effect: lights::effects::Glow,
}

impl Glow {
    fn to_effect(&self) -> lights::effects::Glow {
        self.effect.clone()
    }
}

pub(crate) enum Msg {
    AddColor,
    ReplaceColor(usize, LinSrgb<u8>),
    RemoveColor(usize),
    Delay(i64),
    Steps(i64),
}

#[derive(Clone, PartialEq, Properties, Debug)]
pub(crate) struct Props {
    #[prop_or_default]
    pub onupdate: Option<Callback<lights::effects::Glow>>,
    #[prop_or_default]
    pub glow: lights::effects::Glow,
}

impl Component for Glow {
    type Message = Msg;

    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            effect: ctx.props().glow.clone(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::AddColor => self.effect.add_color(LinSrgb::new(255, 0, 0)),
            Msg::ReplaceColor(idx, color) => self.effect.set_color(idx, color),
            Msg::RemoveColor(idx) => self.effect.remove_color(idx),
            Msg::Delay(delay) => {
                self.effect.delay = Duration::milliseconds(delay);
            }
            Msg::Steps(steps) => self.effect.steps = steps as usize,
        };
        ctx.props()
            .onupdate
            .as_ref()
            .map(|u| u.emit(self.to_effect()));
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let colors = ctx.props().glow.colors();
        let colors = colors.iter().cloned().enumerate().map(|(idx, color)| {
            html! {
                <ybc::Field>
                    <input type="color"
                        value={ format!("#{:02x}{:02x}{:02x}", color.red, color.green, color.blue) }
                        onchange={ link.callback(move |c: Event| {
                            let target: HtmlInputElement = c.target().expect("should have target").dyn_into().unwrap_throw();
                            let value = target.value();
                            let mut buf = [0_u8; 3];
                            let s = value.trim_start_matches('#');
                            let v = hex::decode_to_slice(s, &mut buf);
                            if v.is_err() {
                                return Msg::ReplaceColor(idx, color.clone());
                            }

                            Msg::ReplaceColor(idx, LinSrgb::new(buf[0], buf[1], buf[2]))

                        }) }
                    />
                    <ybc::Control>
                        <input type="button"
                            onclick={
                                ctx.link().callback(move |_| {
                                    Msg::RemoveColor(idx)
                                })
                            }
                            value={ "-" }
                        />
                    </ybc::Control>
                </ybc::Field>
            }
        });

        html! {
            <>
                <div classes={ classes!("glow-colors") }>
                    { for colors }
                    <ybc::Control>
                        <input type="button"
                            onclick={
                                ctx.link().callback(move |_| {
                                    Msg::AddColor
                                })
                            }
                            value={ "+" }
                        />
                    </ybc::Control>
                </div>
                <ybc::Field addons={ true }>
                    <label class="label">{ "Steps between colors: " }</label>
                    <ybc::Control classes={ classes!("has-addons") }>
                        <input type="range"
                            class="input"
                            min="1"
                            max="100"
                            step="1"
                            id="steps"
                            name="steps"
                            onchange={
                                ctx.link().callback(move |ty: Event| {
                                    let target: HtmlInputElement = ty.target().expect("Should have target").dyn_into().unwrap_throw();
                                    let value = target.value();
                                    let steps = value.parse().unwrap_or(10);
                                    Msg::Steps(steps)
                                })
                            }
                            oninput={
                                ctx.link().callback(move |ty:InputEvent| {
                                    let event: Event = ty.dyn_into().unwrap_throw();
                                    let target: HtmlInputElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
                                    let steps = target.value().parse().unwrap_or(10);
                                    Msg::Steps(steps)
                                })
                            }
                            value= { ctx.props().glow.steps.to_string() }
                        />
                        <label>
                            <input type="number" name="steps" id="steps_real" class="input"
                                onchange={
                                    ctx.link().callback(move |ty: Event| {
                                        let target: HtmlInputElement = ty.target().expect("Should have target").dyn_into().unwrap_throw();
                                        let value = target.value();
                                        let steps = value.parse().unwrap_or(10);
                                        Msg::Steps(steps)
                                    })
                                }
                                oninput={
                                    ctx.link().callback(move |ty: InputEvent| {
                                        let event: Event = ty.dyn_into().unwrap_throw();
                                        let target: HtmlInputElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
                                        let steps = target.value().parse().unwrap_or(10);
                                        Msg::Steps(steps)
                                    })
                                }
                                value= { ctx.props().glow.steps.to_string() }
                            /><a class="button is-static">{ "ms" }</a></label>
                    </ybc::Control>
                </ybc::Field>
                <ybc::Field addons={ true }>
                    <label class="label">{ "Delay between movements: " }</label>
                    <ybc::Control classes={ classes!("has-addons") }>
                        <input type="range"
                            class="input"
                            min="10"
                            max="1000"
                            step="10"
                            id="delay"
                            name="delay"
                            onchange={
                                ctx.link().callback(move |ty: Event| {
                                    let target: HtmlInputElement = ty.target().unwrap_throw().dyn_into().unwrap_throw();
                                    let value = target.value();
                                    let delay = value.parse().unwrap_or(100);
                                    Msg::Delay(delay)
                                })
                            }
                            oninput={
                                ctx.link().callback(move |ty:InputEvent| {
                                    let event: Event = ty.dyn_into().unwrap_throw();
                                    let target: HtmlInputElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
                                    let value = target.value();
                                    let delay = value.parse().unwrap_or(100);
                                    Msg::Delay(delay)
                                })
                            }
                            value= { ctx.props().glow.delay.num_milliseconds().to_string() }
                        />
                        <label>
                            <input type="number" name="delay" id="delay_real" class="input"
                                onchange={
                                    ctx.link().callback(move |ty: Event| {
                                        let target: HtmlInputElement = ty.target().unwrap_throw().dyn_into().unwrap_throw();
                                        let value = target.value();
                                        let delay = value.parse().unwrap_or(100);
                                        Msg::Delay(delay)
                                    })
                                }
                                oninput={
                                    ctx.link().callback(move |ty:InputEvent| {
                                        let event: Event = ty.dyn_into().unwrap_throw();
                                        let target: HtmlInputElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
                                        let value = target.value();
                                        let delay = value.parse().unwrap_or(100);
                                            Msg::Delay(delay)
                                    })
                                }
                                value= { ctx.props().glow.delay.num_milliseconds().to_string() }
                            /><a class="button is-static">{ "ms" }</a></label>
                    </ybc::Control>
                </ybc::Field>
            </>
        }
    }
}
