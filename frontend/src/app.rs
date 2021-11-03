use std::str::FromStr;

use lights::effects::{Ball, Balls, Composite, Effect, EffectType, Empty, Glow};
use serde::{Deserialize, Serialize};
use yew::{
    prelude::*,
    services::storage::{Area, StorageService},
};

use crate::{
    components,
    utils::{view_ball, view_balls, view_composite, view_empty, view_glow},
};

const EFFECT_KEY: &str = "org.favil.raspylights.effect";

pub struct App {
    link: ComponentLink<Self>,
    storage: StorageService,
    model: Model,
}

#[derive(Serialize, Deserialize, Debug)]
struct Model {
    effect_type: EffectType,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            effect_type: EffectType::Empty,
        }
    }
}

pub enum Msg {
    SetType(EffectType),
    SetEffect(Box<dyn Effect>),
}

impl Component for App {
    type Message = Msg;

    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let storage = StorageService::new(Area::Local).unwrap();
        let effect_str = storage.restore::<Result<String, anyhow::Error>>(EFFECT_KEY);
        let model = if let Ok(effect_str) = effect_str {
            let effect: Result<Model, _> = serde_json::from_str(&effect_str);
            log::info!("Loading: {:?}", effect);
            effect.unwrap_or_else(|_| Default::default())
        } else {
            Default::default()
        };

        App {
            link,
            storage,
            model,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        log::info!("App update called");
        match msg {
            Msg::SetType(t) => {
                self.model.effect_type = t;
            }
            Msg::SetEffect(effect) => {
                self.model.effect_type = effect.to_cloned_type();
            }
        }
        log::info!("Storing: {:?}", self.model.effect_type);
        let model: &Model = &self.model;
        let s: String = serde_json::to_string(model).unwrap();
        self.storage.store(EFFECT_KEY, Ok(s));
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let dropdown = self.view_dropdown();
        let effect = self.view_own_effect();
        html! {
            <>
                <div class="effect_select box">
                    { dropdown }
                    <div class="effect">{ effect }</div>
                </div>
            </>
        }
    }
}

impl App {
    fn view_dropdown(&self) -> Html {
        html! {
            <components::Selector
                id = { "main" }
                ty = { self.model.effect_type.name() }
                onclick = { Some(self.link.callback(|ty| {
                    Msg::SetType(EffectType::from_str(ty).expect("Don't pass wrong type"))
                })) }
            />
        }
    }

    fn view_own_effect(&self) -> Html {
        self.view_effect(&self.model.effect_type)
    }

    fn view_effect(&self, t: &EffectType) -> Html {
        match t {
            EffectType::Empty => view_empty(),
            EffectType::Ball(b) => view_ball(&b, &self.link, |ball| Msg::SetEffect(Box::new(ball))),
            EffectType::Balls(bs) => {
                view_balls(&bs, &self.link, |balls| Msg::SetEffect(Box::new(balls)))
            }
            EffectType::Glow(g) => view_glow(&g, &self.link, |g| Msg::SetEffect(Box::new(g))),
            EffectType::Composite(c) => {
                view_composite(&c, &self.link, |c| Msg::SetEffect(Box::new(c)))
            } // EffectType::Rainbow => todo!(),
        }
    }
}
