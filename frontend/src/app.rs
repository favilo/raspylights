use gloo::net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use lights::{details::Details, effects::EffectType};
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::HtmlInputElement;
use yew::{platform::spawn_local, prelude::*};

use crate::{
    components,
    utils::{
        view_ball, view_balls, view_composite, view_empty, view_glow, view_rainbow, view_runescript,
    },
};

const EFFECT_KEY: &str = "org.favil.raspylights.effect";
const LAST_EFFECT_KEY: &str = "org.favil.raspylights.effect.last";

pub struct App {
    model: Model,
}

#[derive(Debug)]
struct Model {
    details: Details,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            details: Default::default(),
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

    fn create(ctx: &Context<Self>) -> Self {
        let model = Self::load_model(ctx).unwrap_or_default();

        App { model }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
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
                false
            }
            Msg::PostStatus(details) => {
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
            self.store_current_effect(ctx);
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let dropdown = self.view_selector(ctx);
        let preview = self.view_preview();
        log::info!("Preview made: {:#?}", preview);
        let effect = self.view_own_effect(ctx);
        html! {
            <>
                // The layer of details
                <ybc::Box classes={ classes!("strip_details") }>
                    <label for="strip_length">{ "Number of LEDs" }</label>
                    <input type="number"
                        name="strip_length"
                        id="strip_length"
                        value={ self.model.details.length.to_string() }
                        onchange={
                            ctx.link().callback(|c: Event|{
                                let target: HtmlInputElement = c.target().unwrap_throw().dyn_into().unwrap_throw();
                                let value = target.value();
                                Msg::Length(value.parse().unwrap_or(100))
                            })
                        }
                    />
                    <label for="strip_brightness">{ "Strip Brightness (0-255)" }</label>
                    <input type="number"
                        name="strip_brightness"
                        id="strip_brightness"
                        min="10"
                        max="1000"
                        value={ self.model.details.brightness.to_string() }
                        onchange={
                            ctx.link().callback(|e: Event|{
                                let target: HtmlInputElement = e.target().unwrap_throw().dyn_into().unwrap_throw();
                                let value = target.value();
                                Msg::Brightness(value.parse().unwrap_or(127))
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
    fn load_model(ctx: &Context<Self>) -> Result<Model, anyhow::Error> {
        let effect = LocalStorage::get::<EffectType>(EFFECT_KEY).unwrap_or(EffectType::default());
        let callback = ctx.link().callback(move |response: String| {
            let data = serde_json::from_str::<Details>(&response);
            log::info!("Details: {:#?}", data);
            let (l, b) = data.map(|d| (d.length, d.brightness)).unwrap_or((100, 150));
            Msg::FetchDetails(l, b)
        });
        spawn_local(async move {
            let req = Request::get("/details")
                .send()
                .await
                .expect("Need to get a length response")
                .text()
                .await
                .expect("Needed to get body");
            callback.emit(req);
        });

        Ok(Model {
            details: Details {
                effect,
                ..Details::default()
            },
        })
    }

    fn store_current_effect(&mut self, ctx: &Context<Self>) {
        let model: &Model = &self.model;
        LocalStorage::set(EFFECT_KEY, &model.details.effect).unwrap();
        let details = model.details.clone();
        let callback = ctx.link().callback(move |response: String| {
            let details = serde_json::from_str::<Details>(&response).unwrap_or_default();
            Msg::PostStatus(details)
        });
        spawn_local(async move {
            let req = Request::post("details")
                .json(&details)
                .expect("json serialized properly")
                .send()
                .await
                .expect("Json of effect_type")
                .text()
                .await
                .expect("Need to get body");
            callback.emit(req);
        })
    }

    fn load_last_effect(&mut self, ty: &str) -> EffectType {
        let last_effect = format!("{}.{}", LAST_EFFECT_KEY, ty);
        let effect = LocalStorage::get::<EffectType>(&last_effect).unwrap_or_else(|_| match ty {
            "Empty" => EffectType::Empty(lights::effects::Empty),
            "Ball" => EffectType::Ball(lights::effects::Ball::default()),
            "Balls" => EffectType::Balls(lights::effects::Balls::default()),
            "Glow" => EffectType::Glow(lights::effects::Glow::default()),
            "Rainbow" => EffectType::Rainbow(lights::effects::Rainbow::default()),
            "Composite" => EffectType::Composite(lights::effects::Composite::default()),
            "Rune Script" => EffectType::RuneScript(lights::effects::SourceCode::default()),
            _ => panic!(),
        });
        effect
    }

    fn store_last_effect(&mut self, et: &EffectType) {
        LocalStorage::set(&format!("{}.{}", LAST_EFFECT_KEY, et.name()), et).unwrap();
    }

    fn view_selector(&self, ctx: &Context<Self>) -> Html {
        let onclick = Some(ctx.link().callback(|ty| Msg::EffectName(ty)));
        html! {
            <components::Selector
                id = { "main" }
                ty = { self.model.details.effect.name() }
                onclick = { onclick }
            />
        }
    }

    fn view_preview(&self) -> Html {
        log::info!("Rendering preview of {:#?}", self.model.details.effect);
        html! {
            <components::Preview
                length = { self.model.details.length }
                effect = { self.model.details.effect.clone() }
             />
        }
    }

    fn view_own_effect(&self, ctx: &Context<Self>) -> Html {
        self.view_effect(ctx, &self.model.details.effect)
    }

    fn view_effect(&self, ctx: &Context<Self>, t: &EffectType) -> Html {
        match t {
            EffectType::Empty(_) => view_empty(),
            EffectType::Ball(b) => view_ball(&b, &ctx.link(), |ball| Msg::Type(ball.into())),
            EffectType::Balls(bs) => view_balls(&bs, &ctx.link(), |balls| Msg::Type(balls.into())),
            EffectType::Glow(g) => view_glow(&g, &ctx.link(), |g| Msg::Type(g.into())),
            EffectType::Composite(c) => view_composite(&c, &ctx.link(), |c| Msg::Type(c.into())),
            EffectType::Rainbow(r) => view_rainbow(&r, &ctx.link(), |r| Msg::Type(r.into())),
            EffectType::RuneScript(s) => {
                view_runescript(&s, &ctx.link(), |s| Msg::Type(EffectType::RuneScript(s)))
            }
        }
    }
}
