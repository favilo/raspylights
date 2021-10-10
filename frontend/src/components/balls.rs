use std::time::Duration;

use palette::LinSrgb;

use yew::prelude::*;

use crate::app;

#[derive(Clone, Debug)]
pub(crate) struct Balls {
    link: ComponentLink<Self>,

    balls: lights::effects::Balls,
    onupdate: Option<Callback<app::Msg>>,
}

impl Balls {
    fn to_effect(&self) -> lights::effects::Balls {
        self.balls.clone()
    }
}

pub(crate) enum Msg {
    Update(usize, super::Ball),
}

#[derive(Clone, PartialEq, Properties, Debug)]
pub(crate) struct Props {
    #[prop_or_default]
    pub onupdate: Option<Callback<app::Msg>>,

    #[prop_or_default]
    pub children: Children,
}

impl Component for Balls {
    type Message = Msg;

    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        log::info!("Props: {:?}", props);
        let balls = lights::effects::Balls::new(&[]);
        Self {
            link,
            balls,
            onupdate: props.onupdate,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            _ => todo!(),
        }
        // self.onupdate.as_ref().map(|u| u.emit(app::msg::redraw));
        // true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        html! {
            <></>
        }
    }

    fn rendered(&mut self, _first_render: bool) {}

    fn destroy(&mut self) {}
}
