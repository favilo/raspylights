use std::str::FromStr;

use lights::effects::EffectType;
use yew::prelude::*;
use yewtil::NeqAssign;

use crate::utils::{view_ball, view_balls, view_composite, view_empty, view_glow, view_rainbow};

#[derive(Clone, Debug)]
pub(crate) struct Composite {
    link: ComponentLink<Self>,

    props: Props,
}

impl Composite {
    fn to_effect(&self) -> lights::effects::Composite {
        self.props.composite.clone()
    }
}

pub(crate) enum Msg {
    SetFirst(EffectType),
    SetSecond(EffectType),
}

#[derive(Clone, PartialEq, Properties, Debug)]
pub(crate) struct Props {
    #[prop_or_default]
    pub composite: lights::effects::Composite,

    #[prop_or_default]
    pub onupdate: Option<Callback<lights::effects::Composite>>,
}

impl Component for Composite {
    type Message = Msg;

    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, props }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetFirst(effect) => self.props.composite.set_first(effect),
            Msg::SetSecond(effect) => self.props.composite.set_second(effect),
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
        html! {
            <>
                <div class="foreground box">
                    <h2>{ "Foreground" }</h2>
                    <div class="effect_select">
                        <super::Selector
                            id = { "second" }
                            ty = { self.props.composite.second().name() }
                            onclick = { Some(self.link.callback(|ty| {
                                Msg::SetSecond(EffectType::from_str(ty).expect("Don't pass wrong type"))
                            })) }
                        />
                    </div>
                    { self.view_effect(self.props.composite.second(), false) }
                </div>
                <div class="background box">
                    <h2>{ "Background" }</h2>
                    <div class="effect_select">
                        <super::Selector
                            id = { "first" }
                            ty = { self.props.composite.first().name() }
                            onclick = { Some(self.link.callback(|ty| {
                                Msg::SetFirst(EffectType::from_str(ty).expect("Don't pass wrong type"))
                            })) }
                        />
                    </div>
                    { self.view_effect(self.props.composite.first(), true) }
                </div>
            </>
        }
    }
}

impl Composite {
    fn view_effect(&self, t: &EffectType, first: bool) -> Html {
        match (t, first) {
            (EffectType::Empty(_), _) => view_empty(),
            (EffectType::Ball(b), first) => {
                if first {
                    view_ball(&b, &self.link, |ball| Msg::SetFirst(EffectType::Ball(ball)))
                } else {
                    view_ball(&b, &self.link, |ball| {
                        Msg::SetSecond(EffectType::Ball(ball))
                    })
                }
            }
            (EffectType::Balls(bs), first) => {
                if first {
                    view_balls(&bs, &self.link, |balls| {
                        Msg::SetFirst(EffectType::Balls(balls))
                    })
                } else {
                    view_balls(&bs, &self.link, |balls| {
                        Msg::SetSecond(EffectType::Balls(balls))
                    })
                }
            }
            (EffectType::Glow(g), first) => {
                if first {
                    view_glow(&g, &self.link, |glow| Msg::SetFirst(EffectType::Glow(glow)))
                } else {
                    view_glow(&g, &self.link, |glow| {
                        Msg::SetSecond(EffectType::Glow(glow))
                    })
                }
            }
            (EffectType::Composite(c), first) => {
                if first {
                    view_composite(&c, &self.link, |composite| {
                        Msg::SetFirst(EffectType::Composite(composite))
                    })
                } else {
                    view_composite(&c, &self.link, |composite| {
                        Msg::SetSecond(EffectType::Composite(composite))
                    })
                }
            }
            (EffectType::Rainbow(r), first) => {
                if first {
                    view_rainbow(&r, &self.link, |rainbow| {
                        Msg::SetFirst(EffectType::Rainbow(rainbow))
                    })
                } else {
                    view_rainbow(&r, &self.link, |rainbow| {
                        Msg::SetSecond(EffectType::Rainbow(rainbow))
                    })
                }
            }
        }
    }
}
