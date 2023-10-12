use chrono::Duration;
use palette::LinSrgb;

use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Clone, Debug)]
pub(crate) struct Rainbow {
    effect: lights::effects::Rainbow,
}

impl Rainbow {
    fn to_effect(&self) -> lights::effects::Rainbow {
        self.effect.clone()
    }
}

pub(crate) enum Msg {
    AddColor,
    ReplaceColor(usize, LinSrgb<u8>),
    RemoveColor(usize),
    Spacing(usize),
    Delay(i64),
    Direction,
}

#[derive(Clone, PartialEq, Properties, Debug)]
pub(crate) struct Props {
    #[prop_or_default]
    pub onupdate: Option<Callback<lights::effects::Rainbow>>,
    #[prop_or_default]
    pub rainbow: lights::effects::Rainbow,
}

impl Component for Rainbow {
    type Message = Msg;

    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            effect: ctx.props().rainbow.clone(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::AddColor => self.effect.add_color(LinSrgb::new(255, 0, 0)),
            Msg::ReplaceColor(idx, color) => self.effect.set_color(idx, color),
            Msg::RemoveColor(idx) => self.effect.remove_color(idx),
            Msg::Delay(delay) => self.effect.delay = Duration::milliseconds(delay),
            Msg::Direction => self.effect.direction *= -1,
            Msg::Spacing(s) => self.effect.set_spacing(s),
        };
        ctx.props()
            .onupdate
            .as_ref()
            .map(|u| u.emit(self.to_effect()));
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link().clone();
        let colors = ctx.props().rainbow.colors();
        let colors = colors.iter().cloned().enumerate().map(|(idx, color)| {
            html! {
                <ybc::Field>
                    <input type="color"
                        value={ format!("#{:02x}{:02x}{:02x}", color.red, color.green, color.blue) }
                        onchange={ link.callback(move |c: Event| {
                            let target: HtmlInputElement = c.dyn_into().unwrap_throw();
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

        let direction = if ctx.props().rainbow.direction == 1 {
            "Forward"
        } else {
            "Backward"
        };

        html! {
            <>
                <div classes={ classes!("rainbow-colors") }>
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
                    <label for="direction" class="label">{ "Direction: " }</label>
                    <ybc::Control>
                        <input type="button"
                            id="direction"
                            name="direction"
                            onclick={
                                ctx.link().callback(move |_| {
                                    Msg::Direction
                                })
                            }
                            value={ direction }
                        />
                    </ybc::Control>
                </ybc::Field>
                <ybc::Field addons={ true }>
                    <label class="label">{ "Spacing between colors" }</label>
                    <ybc::Control classes={ classes!("has-addons") }>
                        <input type="number" name="spacing" id="spacing_real" class="input"
                            onchange={
                                ctx.link().callback(move |e: Event| {
                                    let target: HtmlInputElement = e.target().unwrap_throw().dyn_into().unwrap_throw();
                                    let value = target.value();
                                    let spacing = value.parse().unwrap_or(3);
                                    Msg::Spacing(spacing)
                                })
                            }
                            oninput={
                                ctx.link().callback(move |e:InputEvent| {
                                    let event: Event = e.dyn_into().unwrap_throw();
                                    let target: HtmlInputElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
                                    let value = target.value();
                                    let spacing = value.parse().unwrap_or(100);
                                    Msg::Spacing(spacing)
                                })
                            }
                            value= { ctx.props().rainbow.spacing().to_string() }
                            />
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
                                ctx.link().callback(move |e: Event| {
                                    let target: HtmlInputElement = e.target().unwrap_throw().dyn_into().unwrap_throw();
                                    let value = target.value();
                                    let delay = value.parse().unwrap_or(100) ;
                                    Msg::Delay(delay)
                                })
                            }
                            oninput={
                                ctx.link().callback(move |e: InputEvent| {
                                    let event: Event = e.dyn_into().unwrap_throw();
                                    let target: HtmlInputElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
                                    let delay = target.value().parse().unwrap_or(100);
                                    Msg::Delay(delay)
                                })
                            }
                            value= { ctx.props().rainbow.delay.num_milliseconds().to_string() }
                        />
                        <label>
                            <input type="number" name="delay" id="delay_real" class="input"
                                onchange={
                                    ctx.link().callback(move |e: Event| {
                                        let event: Event = e.dyn_into().unwrap_throw();
                                        let target: HtmlInputElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
                                        let value = target.value();
                                        let delay = value.parse().unwrap_or(100) ;
                                        Msg::Delay(delay)
                                    })
                                }
                                oninput={
                                    ctx.link().callback(move |e:InputEvent| {
                                        let event: Event = e.dyn_into().unwrap_throw();
                                        let target: HtmlInputElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
                                        let delay = target.value().parse().unwrap_or(100);
                                        Msg::Delay(delay)
                                    })
                                }
                                value= { ctx.props().rainbow.delay.num_milliseconds().to_string() }
                            /><a class="button is-static">{ "ms" }</a></label>
                    </ybc::Control>
                </ybc::Field>
            </>
        }
    }
}
