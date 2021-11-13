// use palette::LinSrgb;

use yew::prelude::*;
use yewtil::NeqAssign;

#[derive(Clone, Debug)]
pub(crate) struct Glow {
    link: ComponentLink<Self>,

    props: Props,
}

pub(crate) enum Msg {
    // Color(LinSrgb<u8>),
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

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        let _ = &self.link;
        todo!()
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        html! {
            <>
                <h4> { "Glow" }</h4>
            </>
        }
    }
}
