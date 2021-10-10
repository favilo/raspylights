use std::time::Duration;

use palette::LinSrgb;

use yew::prelude::*;

use crate::app;

#[derive(Clone, Debug)]
pub(crate) struct Ball {
    link: ComponentLink<Self>,

    ball: lights::effects::Ball,
    onupdate: Option<Callback<app::Msg>>,
    idx: usize,
}

impl Ball {
    fn to_effect(&self) -> lights::effects::Ball {
        self.ball.clone()
    }
}

pub(crate) enum Msg {
    Color(LinSrgb<u8>),
    Pos(usize),
    Count(usize),
    Direction(i8),
    Bounce(bool),
}

#[derive(Clone, PartialEq, Properties, Debug)]
pub(crate) struct Props {
    #[prop_or_default]
    pub onupdate: Option<Callback<app::Msg>>,
    #[prop_or(LinSrgb::new(255, 0, 0))]
    pub color: LinSrgb<u8>,
    #[prop_or_default]
    pub position: usize,
    #[prop_or(1)]
    pub count: usize,
    #[prop_or(1)]
    pub direction: i8,
    #[prop_or(false)]
    pub bounce: bool,
    #[prop_or(Duration::from_millis(200))]
    pub delay: Duration,
    #[prop_or(0)]
    pub idx: usize,
}

impl Component for Ball {
    type Message = Msg;

    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        log::info!("Props: {:?}", props);
        let ball = lights::effects::Ball::new(
            props.color,
            props.position,
            props.direction,
            props.bounce,
            props.delay,
            props.count,
        );
        Self {
            link,
            ball,
            idx: props.idx,
            onupdate: props.onupdate,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Color(color) => {
                self.ball.color = color;
            }
            Msg::Pos(pos) => {
                self.ball.position = pos;
            }
            Msg::Count(count) => {
                self.ball.count = count;
            }
            Msg::Direction(direction) => {
                self.ball.direction = direction;
            }
            Msg::Bounce(bounce) => {
                self.ball.bounce = bounce;
            }
        }
        // self.onupdate.as_ref().map(|u| u.emit(app::msg::redraw));
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.ball.color = props.color;
        self.ball.position = props.position;
        self.ball.count = props.count;
        self.ball.direction = props.direction;
        self.ball.bounce = props.bounce;
        self.ball.delay = props.delay;
        true
    }

    fn view(&self) -> Html {
        let behaviour = if self.ball.is_bounce() {
            "Bouncing"
        } else {
            "Wrapping"
        };
        let color = self.ball.color();
        log::info!("Color: {}, {}, {}", color.red, color.green, color.blue);
        html! {
            <>
                <h1>{ "Ball" }</h1>
                <p>{ "Color: " }
            <input type="color"
                value={ format!("#{:02x}{:02x}{:02x}", color.red, color.green, color.blue) }
                onchange={ self.link.callback(move |c: ChangeData| {
                    match &c {
                        ChangeData::Value(v) => {
                            let mut buf = [0_u8; 3];
                            let s = v.trim_start_matches('#');
                            let v = hex::decode_to_slice(s, &mut buf);
                            if v.is_err() {
                                return Msg::Color(color.clone());
                            }

                            Msg::Color(LinSrgb::new(buf[0], buf[1], buf[2]))

                        }
                        _ => unreachable!("wrong changedata type?"),
                    }
                }) }
            />
                </p>
                <p>{ "Behaviour: " }<span>{ behaviour }</span></p>
                </>
        }
    }

    fn rendered(&mut self, _first_render: bool) {}

    fn destroy(&mut self) {}
}
