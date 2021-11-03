use crate::components;

use lights::effects::{Ball, Balls, Composite, Effect, EffectType, Glow};
use yew::{html::IntoPropValue, prelude::*};

pub fn view_empty() -> Html {
    html! {
        <>
            <h4> { "Empty" }</h4>
        </>
    }
}

pub fn view_ball<COMP, F, IN, M>(ball: &Ball, link: &ComponentLink<COMP>, lambda: F) -> Html
where
    COMP: Component,
    F: Fn(IN) -> M + 'static,
    M: Into<COMP::Message>,
    Option<Callback<IN>>: IntoPropValue<Option<Callback<Ball>>>,
{
    html! {
        <components::Ball
            ball = { ball.clone() }
            onupdate = { Some(link.callback(lambda)) }
         />
    }
}

pub fn view_balls<COMP, F, IN, M>(balls: &Balls, link: &ComponentLink<COMP>, lambda: F) -> Html
where
    COMP: Component,
    F: Fn(IN) -> M + 'static,
    M: Into<COMP::Message>,
    Option<Callback<IN>>: IntoPropValue<Option<Callback<Balls>>>,
{
    html! {
        <components::Balls
            balls = { balls.clone() }
            onupdate = { Some(link.callback(lambda)) }
         />
    }
}

pub fn view_glow<COMP, F, IN, M>(glow: &Glow, link: &ComponentLink<COMP>, lambda: F) -> Html
where
    COMP: Component,
    F: Fn(IN) -> M + 'static,
    M: Into<COMP::Message>,
    Option<Callback<IN>>: IntoPropValue<Option<Callback<Glow>>>,
{
    html! {
        // <>
        //     <h4> { "Glow" }</h4>
        // </>
        <components::Glow
            glow = { glow.clone() }
            onupdate = { Some(link.callback(lambda)) }
         />
    }
}

pub fn view_composite<COMP, F, IN, M>(
    composite: &Composite,
    link: &ComponentLink<COMP>,
    lambda: F,
) -> Html
where
    COMP: Component,
    F: Fn(IN) -> M + 'static,
    M: Into<COMP::Message>,
    Option<Callback<IN>>: IntoPropValue<Option<Callback<Composite>>>,
{
    html! {
        <components::Composite
            composite = { composite.clone() }
            onupdate = { Some(link.callback(lambda)) }
         />
    }
}
