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
const LAST_EFFECT_KEY: &str = "org.favil.raspylights.effect.last";

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
        let model = Self::load_model(&storage);

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
                // Used for when we click the title
                self.model.effect_type = self.load_last_effect(t.name())
            }
            Msg::SetEffect(effect) => {
                // Used to update the effect
                let effect_type = effect.to_cloned_type();
                self.store_last_effect(&effect_type);
                self.model.effect_type = effect_type;
            }
        }
        log::info!("Storing: {:?}", self.model.effect_type);
        self.store_model();
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let dropdown = self.view_selector();
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
    fn load_model(storage: &StorageService) -> Model {
        let effect_str = storage.restore::<Result<String, anyhow::Error>>(EFFECT_KEY);
        if let Ok(effect_str) = effect_str {
            let effect: Result<Model, _> = serde_json::from_str(&effect_str);
            log::info!("Loading: {:?}", effect);
            effect.unwrap_or_else(|_| Default::default())
        } else {
            Default::default()
        }
    }

    fn store_model(&mut self) {
        let model: &Model = &self.model;
        let s: String = serde_json::to_string(model).unwrap();
        self.storage.store(EFFECT_KEY, Ok(s));
    }

    fn load_last_effect(&mut self, ty: &str) -> EffectType {
        let last_effect = format!("{}.{}", LAST_EFFECT_KEY, ty);
        let effect_str = self
            .storage
            .restore::<Result<String, anyhow::Error>>(&last_effect);
        if let Ok(effect_str) = effect_str {
            let effect: Result<EffectType, _> = serde_json::from_str(&effect_str);
            log::info!("Loading: {:?}", effect);
            effect.unwrap_or_else(|_| EffectType::default_from_name(ty))
        } else {
            EffectType::default_from_name(ty)
        }
    }

    fn store_last_effect(&mut self, et: &EffectType) {
        let s: String = serde_json::to_string(et).unwrap();
        self.storage
            .store(&format!("{}.{}", LAST_EFFECT_KEY, et.name()), Ok(s));
    }

    fn view_selector(&self) -> Html {
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
