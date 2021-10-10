use lights::effects::{Ball, Balls, Composite, Effect, EffectType, Empty, Glow};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use yew::{
    prelude::*,
    services::storage::{Area, StorageService},
};
use yew_mdc_widgets::{MdcWidget, Tab, TabBar};

use crate::components;

const EFFECT_TYPE_KEY: &str = "org.favil.raspylights.effect_type";
const EFFECT_KEY: &str = "org.favil.raspylights.effect";

pub struct App {
    link: ComponentLink<Self>,
    storage: StorageService,
    model: Model,
}

#[derive(Serialize, Deserialize, Debug)]
struct EffectPair(
    EffectType,
    #[serde(with = "serde_traitobject")] Box<dyn Effect>,
);

#[derive(Serialize, Deserialize, Debug)]
struct Model {
    effect_type: EffectType,
    #[serde(with = "serde_traitobject")]
    effect: Box<dyn Effect>,

    effects: Vec<EffectPair>,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            effect_type: EffectType::Empty,
            effect: Box::new(Empty),

            effects: Vec::new(),
        }
    }
}

pub enum Msg {
    Redraw,
    SetType(EffectType),
    SetEffect(Box<dyn Effect>),
    AddBall,
}

impl Component for App {
    type Message = Msg;

    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let storage = StorageService::new(Area::Local).unwrap();
        let effect_str = storage.restore::<Result<String, anyhow::Error>>(EFFECT_KEY);
        let model = if let Ok(effect_str) = effect_str {
            let effect: Result<Model, _> = serde_json::from_str(&effect_str);
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
            Msg::Redraw => {
                log::info!("Redrawing");
            }
            Msg::SetType(t) => {
                self.model.effect_type = t;
                self.model.effect = t.to_default();
            }
            Msg::SetEffect(effect) => {
                self.model.effect_type = effect.to_type();
                self.model.effect = effect;
            }
            Msg::AddBall => self
                .model
                .effect
                .downcast_mut::<Balls>()
                .expect("Should only be called from Balls")
                .add_ball(),
        }
        log::info!("Storing: {:?}", self.model.effect);
        let model = &self.model;
        let s = serde_json::to_string(model).unwrap();
        self.storage.store(EFFECT_KEY, Ok(s));
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let dropdown = self.view_dropdown("main", self.model.effect_type);
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
    fn view_dropdown(&self, id: &str, ty: EffectType) -> Html {
        let effects = EffectType::iter().collect::<Vec<_>>();
        let options: Vec<_> = effects
            .iter()
            .cloned()
            .enumerate()
            .map(|(idx, i)| -> Tab {
                let id = format!("{:?}", i);
                let mut t = Tab::new().label(id).tab_index(idx as isize);
                if i == ty {
                    t = t.active();
                }
                t.on_click(self.link.callback(move |_| Msg::SetType(i)))
            })
            .collect::<_>();

        let id_str = format!("effect_select_{}", id);

        html! {
            <>
                // <p> { "Select the effect you want to use:" } </p>
                {
                    TabBar::new().id(&id_str).tabs(options)
                }
            </>
        }
    }

    fn view_own_effect(&self) -> Html {
        self.view_effect(self.model.effect_type, &self.model.effect)
    }

    fn view_effect(&self, t: EffectType, effect: &Box<dyn Effect>) -> Html {
        match t {
            EffectType::Empty => self.view_empty(),
            EffectType::Composite => {
                self.view_composite(&effect.downcast_ref::<Composite>().unwrap())
            }
            EffectType::Ball => self.view_ball(effect.downcast_ref::<Ball>().unwrap(), 0),
            EffectType::Balls => self.view_balls(effect.downcast_ref::<Balls>().unwrap()),
            EffectType::Glow => self.view_glow(effect.downcast_ref::<Glow>().unwrap()),
            EffectType::Rainbow => todo!(),
        }
    }

    fn view_empty(&self) -> Html {
        html! {
                <>
                    <h4>{ "Empty" }</h4>
                </>
        }
    }

    fn view_composite(&self, composite: &Composite) -> Html {
        html! {
            <>
                <div class="background box">
                    <h1>{ "Background" }</h1>
                    <div class="effect_select box">{
                        self.view_dropdown("background", composite.first().to_type())
                    }</div>
                    { self.view_effect(composite.first().to_type(), composite.first()) }
                </div>
                <div class="foreground box">
                    <h1>{ "Foreground" }</h1>
                    <div class="effect_select box">{
                        self.view_dropdown("foreground", composite.second().to_type())
                    }</div>
                    { self.view_effect(composite.second().to_type(), composite.second()) }
                </div>
            </>
        }
    }

    fn view_ball(&self, ball: &Ball, idx: usize) -> Html {
        html! {
            <components::Ball
                color = ball.color
                idx = idx
                onupdate = { Some(self.link.callback(|msg| msg)) }
             />
        }
    }

    fn view_balls(&self, balls: &Balls) -> Html {
        let inner: Vec<_> = balls
            .balls()
            .iter()
            .enumerate()
            .map(|(idx, b)| {
                html! {
                    <>
                        <p>{ format!("#{}", idx) }</p>
                        { self.view_ball(b, idx) }
                        // Need the delete button
                    </>
                }
            })
            .collect();

        html! {
            <components::Balls
                onupdate = { Some(self.link.callback(|msg| msg)) } >
            </components::Balls>
        }
    }

    fn view_glow(&self, glow: &Glow) -> Html {
        log::warn!("Glow: {:?}", glow);

        html! {
            <>
                <h1>{ "Glow" }</h1>
            </>
        }
    }
}
