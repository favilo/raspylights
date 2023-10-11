use chrono::Duration;

use palette::LinSrgb;

use yew::prelude::*;
use yewtil::NeqAssign;

#[derive(Clone, Debug)]
pub(crate) struct Ball {
    effect: lights::effects::Ball,
}

impl Ball {
    fn to_effect(&self) -> lights::effects::Ball {
        self.effect.clone()
    }
}

pub(crate) enum Msg {
    Color(LinSrgb<u8>),
    Pos(usize),
    // Count(usize),
    Direction(i8),
    Bounce(bool),
    Delay(i64),
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

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            effect: ctx.props().ball.clone(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Color(color) => {
                self.effect.color = color;
            }
            Msg::Pos(pos) => {
                self.effect.position = pos;
            }
            // Msg::Count(count) => {
            //     self.effect.count = count;
            // }
            Msg::Direction(direction) => {
                self.effect.direction = direction;
            }
            Msg::Bounce(bounce) => {
                self.effect.bounce = bounce;
            }
            Msg::Delay(delay) => {
                self.effect.delay = Duration::milliseconds(delay);
            }
        }
        ctx.props()
            .onupdate
            .as_ref()
            .map(|u| u.emit(self.to_effect()));
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let is_bounce = ctx.props().ball.is_bounce();
        let ball_direction = ctx.props().ball.direction;
        let behaviour = if is_bounce { "Bouncing" } else { "Wrapping" };
        let direction = if ball_direction == 1 {
            "Forward"
        } else {
            "Backward"
        };
        let color = ctx.props().ball.color();
        html! {
            <>
                <ybc::Field>
                    <label for="ball_color" class="label">{ "Color: " }</label>
                    <input type="color"
                        id={ "ball_color" }
                        name={ "ball_color" }
                        value={ format!("#{:02x}{:02x}{:02x}", color.red, color.green, color.blue) }
                        // onchange={ ctx.link().callback(move |c: ChangeData| {
                        //     match &c {
                        //         ChangeData::Value(v) => {
                        //             let mut buf = [0_u8; 3];
                        //             let s = v.trim_start_matches('#');
                        //             let v = hex::decode_to_slice(s, &mut buf);
                        //             if v.is_err() {
                        //                 return Msg::Color(color);
                        //             }

                        //             Msg::Color(LinSrgb::new(buf[0], buf[1], buf[2]))

                        //         }
                        //         _ => {
                        //             log::error!("Wong changedata type");
                        //             unreachable!("wrong changedata type?")
                        //         },
                        //     }
                        // }) }
                    />
                </ybc::Field>
                <ybc::Field>
                    <label for="starting_point" class="label">
                        { "Starting Point:" }
                    </label>
                    <input type="number"
                        value="0"
                        id="starting_point"
                        name="starting_point"
                        // onchange={
                        //     ctx.link().callback(move |c| {
                        //         match c {
                        //             ChangeData::Value(v) => {
                        //                 let pos = v.parse().unwrap_or(100);
                        //                 Msg::Pos(pos)
                        //             }
                        //             _ => {
                        //                 log::error!("Wong changedata type");
                        //                 unreachable!("wrong changedata type?")
                        //             }
                        //         }

                        //     })
                        // }
                    />
                </ybc::Field>
                <ybc::Field addons={ true }>
                    <label for="behavior" class="label">{ "Behavior: " }</label>
                    <ybc::Control>
                        <input type="button"
                            id="behavior"
                            name="behavior"
                            onclick={
                                ctx.link().callback(move |_| {
                                    Msg::Bounce(!is_bounce)
                                })
                            }
                            value={ behaviour }
                        />
                    </ybc::Control>
                </ybc::Field>
                <ybc::Field addons={ true }>
                    <label for="direction" class="label">{ "Direction: " }</label>
                    <ybc::Control>
                        <input type="button"
                            id="direction"
                            name="direction"
                            onclick={
                                ctx.link().callback(move |_| {
                                    Msg::Direction(ball_direction * -1)
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
                            // onchange={
                            //     ctx.link().callback(move |ty| {
                            //         let delay = match ty {
                            //             ChangeData::Value(s) => {
                            //                 s.parse().unwrap_or(100)
                            //             }
                            //             _ => 100,
                            //         };
                            //         Msg::Delay(delay)
                            //     })
                            // }
                            // oninput={
                            //     ctx.link().callback(move |ty:InputData| {
                            //         let delay = ty.value.parse().unwrap_or(100);
                            //         Msg::Delay(delay)
                            //     })
                            // }
                            value= { ctx.props().ball.delay.num_milliseconds().to_string() }
                        />
                        <label>
                            <input type="number" name="delay" id="delay_real" class="input"
                                // onchange={
                                //     ctx.link().callback(move |ty| {
                                //         let delay = match ty {
                                //             ChangeData::Value(s) => {
                                //                 s.parse().unwrap_or(100)
                                //             }
                                //             _ => 100,
                                //         };
                                //         Msg::Delay(delay)
                                //     })
                                // }
                                // oninput={
                                //     ctx.link().callback(move |ty:InputData| {
                                //         let delay = ty.value.parse().unwrap_or(100);
                                //         Msg::Delay(delay)
                                //     })
                                // }
                                value= { ctx.props().ball.delay.num_milliseconds().to_string() }
                            /><a class="button is-static">{ "ms" }</a></label>
                    </ybc::Control>
                </ybc::Field>
            </>
        }
    }
}
