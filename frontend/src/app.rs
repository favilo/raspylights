use lights::{details::Details, effects::EffectType};
use serde_json::json;
use yew::{
    format::{Json, Nothing},
    prelude::*,
    services::{
        fetch::{FetchTask, Request, Response},
        storage::{Area, StorageService},
        FetchService,
    },
};

use crate::{
    components,
    utils::{
        view_ball, view_balls, view_composite, view_empty, view_glow, view_rainbow, view_runescript,
    },
};

const EFFECT_KEY: &str = "org.favil.raspylights.effect";
const LAST_EFFECT_KEY: &str = "org.favil.raspylights.effect.last";

pub struct App {
    link: ComponentLink<Self>,
    storage: StorageService,
    model: Model,
}

#[derive(Debug)]
struct Model {
    details: Details,
    task: Option<FetchTask>,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            details: Default::default(),
            task: None,
        }
    }
}

pub enum Msg {
    Type(EffectType),
    EffectName(&'static str),
    Length(usize),
    Brightness(u8),
    FetchDetails(usize, u8),
    PostStatus(Details),
}

impl Component for App {
    type Message = Msg;

    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let storage = StorageService::new(Area::Local).unwrap();
        let model = Self::load_model(&storage, &link).unwrap_or_default();

        App {
            link,
            storage,
            model,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        log::debug!("App update called");
        let store = match msg {
            Msg::Type(t) => {
                self.store_last_effect(&t);
                self.model.details.effect = t;
                true
            }
            Msg::EffectName(name) => {
                // Used for when we click the title
                self.model.details.effect = self.load_last_effect(name);
                true
            }
            Msg::FetchDetails(l, b) => {
                self.model.details.length = l;
                self.model.details.brightness = b;
                self.model.task = None;
                false
            }
            Msg::PostStatus(details) => {
                self.model.task = None;
                self.model.details = details;
                false
            }
            Msg::Length(l) => {
                self.model.details.length = l;
                true
            }
            Msg::Brightness(b) => {
                self.model.details.brightness = b;
                true
            }
        };

        if store {
            log::debug!("Storing: {:?}", self.model.details);
            self.store_current_effect();
        }

        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let dropdown = self.view_selector();
        let preview = self.view_preview();
        let effect = self.view_own_effect();
        html! {
            <>
                // The layer of details
                // <div class="box" name="strip_details">
                <ybc::Box classes={ classes!("strip_details") }>
                    <label for="strip_length">{ "Number of LEDs" }</label>
                    <input type="number"
                        name="strip_length"
                        id="strip_length"
                        value={ self.model.details.length.to_string() }
                        onchange={
                            self.link.callback(|c: ChangeData|{
                                match c {
                                    ChangeData::Value(v) => {
                                        Msg::Length(v.parse().unwrap_or(100))
                                    }
                                    _ => {
                                        log::error!("Wrong ChangeData type");
                                        unreachable!("Should not have been possible for this input type")
                                    }
                                }
                            })
                        }
                    />
                    <label for="strip_brightness">{ "Strip Brightness (0-255)" }</label>
                    <input type="number"
                        name="strip_brightness"
                        id="strip_brightness"
                        value={ self.model.details.brightness.to_string() }
                        onchange={
                            self.link.callback(|c: ChangeData|{
                                match c {
                                    ChangeData::Value(v) => {
                                        Msg::Brightness(v.parse().unwrap_or(255))
                                    }
                                    _ => {
                                        log::error!("Wrong ChangeData type");
                                        unreachable!("Should not have been possible for this input type")
                                    }
                                }
                            })
                        }
                    />
                </ybc::Box>
                <ybc::Box classes={ classes!("main_preview") }>
                    { preview }
                </ybc::Box>
                // The effects
                <ybc::Box classes={ classes!("effect-select") }>
                    { dropdown }
                    <div class="effect">{ effect }</div>
                </ybc::Box>
            </>
        }
    }
}

impl App {
    fn load_model(
        storage: &StorageService,
        link: &ComponentLink<Self>,
    ) -> Result<Model, anyhow::Error> {
        let effect_str = storage.restore::<Result<String, anyhow::Error>>(EFFECT_KEY);
        let effect = if let Ok(effect_str) = effect_str {
            let effect: Result<EffectType, _> = serde_json::from_str(&effect_str);
            log::info!("Loading: {:?}", effect);
            effect.unwrap_or_else(|_| Default::default())
        } else {
            Default::default()
        };
        let req = Request::get("details")
            .body(Nothing)
            .expect("Need to get a length response");
        let callback = link.callback(move |response: Response<Json<anyhow::Result<Details>>>| {
            log::info!("{:#?}", response);
            let Json(data) = response.into_body();
            let (l, b) = data.map(|d| (d.length, d.brightness)).unwrap_or((100, 150));
            Msg::FetchDetails(l, b)
        });
        let task = FetchService::fetch(req, callback).expect("Request shouldn't fail");

        Ok(Model {
            details: Details {
                effect,
                ..Details::default()
            },
            task: Some(task),
        })
    }

    fn store_current_effect(&mut self) {
        let model: &Model = &self.model;
        let s: String = serde_json::to_string(&model.details.effect).unwrap();
        self.storage.store(EFFECT_KEY, Ok(s));
        let details = model.details.clone();
        let json = json!(&details);
        let req = Request::post("details")
            .body(Json(&json))
            .expect("Json of effect_type");
        let callback =
            self.link
                .callback(move |response: Response<Json<anyhow::Result<Details>>>| {
                    let Json(jsvalue) = response.into_body();
                    let details = jsvalue.unwrap_or(details.clone());
                    Msg::PostStatus(details)
                });
        let task = FetchService::fetch(req, callback).expect("Need this to go through");
        self.model.task = Some(task);
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
        let onclick = Some(self.link.callback(|ty| Msg::EffectName(ty)));
        html! {
            <components::Selector
                id = { "main" }
                ty = { self.model.details.effect.name() }
                onclick = { onclick }
            />
        }
    }

    fn view_preview(&self) -> Html {
        html! {
            <components::Preview
                length = { self.model.details.length }
                effect = { self.model.details.effect.clone() }
             />
        }
    }

    fn view_own_effect(&self) -> Html {
        self.view_effect(&self.model.details.effect)
    }

    fn view_effect(&self, t: &EffectType) -> Html {
        match t {
            EffectType::Empty(_) => view_empty(),
            EffectType::Ball(b) => view_ball(&b, &self.link, |ball| Msg::Type(ball.into())),
            EffectType::Balls(bs) => view_balls(&bs, &self.link, |balls| Msg::Type(balls.into())),
            EffectType::Glow(g) => view_glow(&g, &self.link, |g| Msg::Type(g.into())),
            EffectType::Composite(c) => view_composite(&c, &self.link, |c| Msg::Type(c.into())),
            EffectType::Rainbow(r) => view_rainbow(&r, &self.link, |r| Msg::Type(r.into())),
            EffectType::RuneScript(s) => {
                view_runescript(&s, &self.link, |s| Msg::Type(EffectType::RuneScript(s)))
            }
        }
    }
}
