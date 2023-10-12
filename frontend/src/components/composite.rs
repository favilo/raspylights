use std::str::FromStr;

use lights::effects::EffectType;
use yew::prelude::*;

use crate::utils::{view_ball, view_balls, view_composite, view_empty, view_glow, view_rainbow};

#[derive(Clone, Debug)]
pub(crate) struct Composite {
    effect: lights::effects::Composite,
}

impl Composite {
    fn to_effect(&self) -> lights::effects::Composite {
        self.effect.clone()
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

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            effect: ctx.props().composite.clone(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let result = match msg {
            Msg::SetFirst(effect) => self.effect.set_first(effect),
            Msg::SetSecond(effect) => self.effect.set_second(effect),
        };
        if result.is_err() {
            log::error!("Tried to set the wrong type");
            return true;
        }

        ctx.props()
            .onupdate
            .as_ref()
            .map(|u| u.emit(self.to_effect()));
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                <div class="foreground box">
                    <h2>{ "Foreground" }</h2>
                    <div class="effect_select">
                        <super::Selector
                            id = { "second" }
                            ty = { ctx.props().composite.second().name() }
                            onclick = { Some(ctx.link().callback(|ty| {
                                Msg::SetSecond(EffectType::from_str(ty).expect("Don't pass wrong type"))
                            })) }
                            internal = true
                        />
                    </div>
                    { self.view_effect(ctx, ctx.props().composite.second(), false) }
                </div>
                <div class="background box">
                    <h2>{ "Background" }</h2>
                    <div class="effect_select">
                        <super::Selector
                            id = { "first" }
                            ty = { ctx.props().composite.first().name() }
                            onclick = { Some(ctx.link().callback(|ty| {
                                Msg::SetFirst(EffectType::from_str(ty).expect("Don't pass wrong type"))
                            })) }
                            internal = true
                        />
                    </div>
                    { self.view_effect(ctx, ctx.props().composite.first(), true) }
                </div>
            </>
        }
    }
}

impl Composite {
    fn view_effect(&self, ctx: &Context<Self>, t: &EffectType, first: bool) -> Html {
        match (t, first) {
            (EffectType::Empty(_), _) => view_empty(),
            (EffectType::Ball(b), first) => {
                if first {
                    view_ball(&b, &ctx.link(), |ball| {
                        Msg::SetFirst(EffectType::Ball(ball))
                    })
                } else {
                    view_ball(&b, &ctx.link(), |ball| {
                        Msg::SetSecond(EffectType::Ball(ball))
                    })
                }
            }
            (EffectType::Balls(bs), first) => {
                if first {
                    view_balls(&bs, &ctx.link(), |balls| {
                        Msg::SetFirst(EffectType::Balls(balls))
                    })
                } else {
                    view_balls(&bs, &ctx.link(), |balls| {
                        Msg::SetSecond(EffectType::Balls(balls))
                    })
                }
            }
            (EffectType::Glow(g), first) => {
                if first {
                    view_glow(&g, &ctx.link(), |glow| {
                        Msg::SetFirst(EffectType::Glow(glow))
                    })
                } else {
                    view_glow(&g, &ctx.link(), |glow| {
                        Msg::SetSecond(EffectType::Glow(glow))
                    })
                }
            }
            (EffectType::Composite(c), first) => {
                if first {
                    view_composite(&c, &ctx.link(), |composite| {
                        Msg::SetFirst(EffectType::Composite(composite))
                    })
                } else {
                    view_composite(&c, &ctx.link(), |composite| {
                        Msg::SetSecond(EffectType::Composite(composite))
                    })
                }
            }
            (EffectType::Rainbow(r), first) => {
                if first {
                    view_rainbow(&r, &ctx.link(), |rainbow| {
                        Msg::SetFirst(EffectType::Rainbow(rainbow))
                    })
                } else {
                    view_rainbow(&r, &ctx.link(), |rainbow| {
                        Msg::SetSecond(EffectType::Rainbow(rainbow))
                    })
                }
            }
            _ => {
                html! {
                    <>
                        <p> { "Unsupported type" } </p>
                    </>
                }
            }
        }
    }
}
