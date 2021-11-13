use std::time::Duration;

use palette::LinSrgb;

use yew::prelude::*;
use yewtil::NeqAssign;

#[derive(Clone, Debug)]
pub(crate) struct Ball {
    link: ComponentLink<Self>,

    props: Props,
}

impl Ball {
    fn to_effect(&self) -> lights::effects::Ball {
        self.props.ball.clone()
    }
}

pub(crate) enum Msg {
    Color(LinSrgb<u8>),
    Pos(usize),
    // Count(usize),
    Direction(i8),
    Bounce(bool),
    Delay(u64),
}

#[derive(Clone, PartialEq, Properties, Debug)]
pub(crate) struct Props {
    #[prop_or_default]
    pub onupdate: Option<Callback<lights::effects::Ball>>,
    #[prop_or_default]
    pub ball: lights::effects::Ball,
}

impl Component for Ball {
    type Message = Msg;

    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, props }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Color(color) => {
                self.props.ball.color = color;
            }
            Msg::Pos(pos) => {
                self.props.ball.position = pos;
            }
            // Msg::Count(count) => {
            //     self.props.ball.count = count;
            // }
            Msg::Direction(direction) => {
                self.props.ball.direction = direction;
            }
            Msg::Bounce(bounce) => {
                self.props.ball.bounce = bounce;
            }
            Msg::Delay(delay) => {
                self.props.ball.delay = Duration::from_millis(delay);
            }
        }
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
        let is_bounce = self.props.ball.is_bounce();
        let ball_direction = self.props.ball.direction;
        let behaviour = if is_bounce { "Bouncing" } else { "Wrapping" };
        let direction = if ball_direction == 1 {
            "Forward"
        } else {
            "Backward"
        };
        let color = self.props.ball.color();
        html! {
            <>
                <h1>{ "Ball" }</h1>
                <p>
                    <label for="ball_color">{ "Color: " }</label>
                    <input type="color"
                        id={ "ball_color" }
                        name={ "ball_color" }
                        value={ format!("#{:02x}{:02x}{:02x}", color.red, color.green, color.blue) }
                        onchange={ self.link.callback(move |c: ChangeData| {
                            match &c {
                                ChangeData::Value(v) => {
                                    let mut buf = [0_u8; 3];
                                    let s = v.trim_start_matches('#');
                                    let v = hex::decode_to_slice(s, &mut buf);
                                    if v.is_err() {
                                        return Msg::Color(color);
                                    }

                                    Msg::Color(LinSrgb::new(buf[0], buf[1], buf[2]))

                                }
                                _ => {
                                    log::error!("Wong changedata type");
                                    unreachable!("wrong changedata type?")
                                },
                            }
                        }) }
                    />
                </p>
                <p>
                    <label for="starting_point">{ "Starting Point:" }</label>
                <input type="number"
                    value="0"
                    id="starting_point"
                    name="starting_point"
                    onchange={
                        self.link.callback(move |c| {
                            match c {
                                ChangeData::Value(v) => {
                                    let pos = v.parse().unwrap_or(100);
                                    Msg::Pos(pos)
                                }
                                _ => {
                                    log::error!("Wong changedata type");
                                    unreachable!("wrong changedata type?")
                                }
                            }

                        })
                    }
                 />
                </p>
                <p>
                    <label for="behavior">{ "Behavior: " }</label>
                    <input type="button"
                        id="behavior"
                        name="behavior"
                        onclick={
                            self.link.callback(move |_| {
                                Msg::Bounce(!is_bounce)
                            })
                        }
                        value={ behaviour }
                    />
                </p>
                <p>
                    <label for="direction">{ "Direction: " }</label>
                    <input type="button"
                        id="direction"
                        name="direction"
                        onclick={
                            self.link.callback(move |_| {
                                Msg::Direction(ball_direction * -1)
                            })
                        }
                        value={ direction }
                    />
                </p>
                <p>
                    <label for="delay">{ "Delay between movements: " }</label>
                    <input type="range"
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
                        value= { self.props.ball.delay.as_millis().to_string() }
                    />
                    <input type="number" name="delay" id="delay_real"
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
                        value= { self.props.ball.delay.as_millis().to_string() }
                    />
                    <span>{ "ms" }</span>
                </p>
            </>
        }
    }
}
