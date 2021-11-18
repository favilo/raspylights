use chrono::Duration;
use palette::LinSrgb;

use yew::prelude::*;
use yewtil::NeqAssign;

#[derive(Clone, Debug)]
pub(crate) struct Rainbow {
    link: ComponentLink<Self>,

    props: Props,
}

impl Rainbow {
    fn to_effect(&self) -> lights::effects::Rainbow {
        self.props.rainbow.clone()
    }
}

pub(crate) enum Msg {
    AddColor,
    ReplaceColor(usize, LinSrgb<u8>),
    RemoveColor(usize),
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

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, props }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::AddColor => self.props.rainbow.add_color(LinSrgb::new(255, 0, 0)),
            Msg::ReplaceColor(idx, color) => self.props.rainbow.set_color(idx, color),
            Msg::RemoveColor(idx) => self.props.rainbow.remove_color(idx),
            Msg::Delay(delay) => self.props.rainbow.delay = Duration::milliseconds(delay),
            Msg::Direction => self.props.rainbow.direction *= -1,
        };
        self.props
            .onupdate
            .as_ref()
            .map(|u| u.emit(self.to_effect()));
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        let link = self.link.clone();
        let colors = self.props.rainbow.colors();
        let colors = colors.iter().cloned().enumerate().map(|(idx, color)| {
            html! {
                <ybc::Field>
                    <input type="color"
                        value={ format!("#{:02x}{:02x}{:02x}", color.red, color.green, color.blue) }
                        onchange={ link.callback(move |c: ChangeData| {
                            match &c {
                                ChangeData::Value(v) => {
                                    let mut buf = [0_u8; 3];
                                    let s = v.trim_start_matches('#');
                                    let v = hex::decode_to_slice(s, &mut buf);
                                    if v.is_err() {
                                        return Msg::ReplaceColor(idx, color.clone());
                                    }

                                    Msg::ReplaceColor(idx, LinSrgb::new(buf[0], buf[1], buf[2]))

                                }
                                _ => {
                                    log::error!("Wong changedata type");
                                    unreachable!("wrong changedata type?")
                                },
                            }
                        }) }
                    />
                    <ybc::Control>
                        <input type="button"
                            onclick={
                                self.link.callback(move |_| {
                                    Msg::RemoveColor(idx)
                                })
                            }
                            value={ "-" }
                        />
                    </ybc::Control>
                </ybc::Field>
            }
        });

        let direction = if self.props.rainbow.direction == 1 {
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
                                self.link.callback(move |_| {
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
                                self.link.callback(move |_| {
                                    Msg::Direction
                                })
                            }
                            value={ direction }
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
                                self.link.callback(move |ty| {
                                    let delay = match ty {
                                        ChangeData::Value(s) => {
                                            s.parse().unwrap_or(100)
                                        }
                                        _ => 100,
                                    };
                                    Msg::Delay(delay)
                                })
                            }
                            oninput={
                                self.link.callback(move |ty:InputData| {
                                    let delay = ty.value.parse().unwrap_or(100);
                                    Msg::Delay(delay)
                                })
                            }
                            value= { self.props.rainbow.delay.num_milliseconds().to_string() }
                        />
                        <label>
                            <input type="number" name="delay" id="delay_real" class="input"
                                onchange={
                                    self.link.callback(move |ty| {
                                        let delay = match ty {
                                            ChangeData::Value(s) => {
                                                s.parse().unwrap_or(100)
                                            }
                                            _ => 100,
                                        };
                                        Msg::Delay(delay)
                                    })
                                }
                                oninput={
                                    self.link.callback(move |ty:InputData| {
                                        let delay = ty.value.parse().unwrap_or(100);
                                        Msg::Delay(delay)
                                    })
                                }
                                value= { self.props.rainbow.delay.num_milliseconds().to_string() }
                            /><a class="button is-static">{ "ms" }</a></label>
                    </ybc::Control>
                </ybc::Field>
            </>
        }
    }
}
