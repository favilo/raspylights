use anyhow::anyhow;
use chrono::{DateTime, Duration, NaiveDateTime, TimeZone, Utc};
use gloo::timers::callback::Timeout;
use lights::effects::{Effect, EffectType};
use palette::LinSrgb;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlCanvasElement;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub(crate) struct Props {
    #[prop_or(100)]
    pub length: usize,

    #[prop_or_default]
    pub effect: EffectType,
}

pub(crate) enum Msg {
    Tick(DateTime<Utc>),
}

pub(crate) struct Preview {
    pixels: Vec<LinSrgb<u8>>,
    timer: Option<Timeout>,
    canvas: NodeRef,
    effect: Box<dyn Effect>,
}

impl Component for Preview {
    type Message = Msg;

    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        log::info!("creating preview");
        let pixels = vec![Default::default(); ctx.props().length];
        let effect = ctx.props().effect.clone().into_inner();
        let mut this = Self {
            pixels,
            effect,
            timer: None,
            canvas: Default::default(),
        };
        let dur = this
            .effect
            .render(&mut this.pixels, now())
            .unwrap_or(Duration::milliseconds(50));
        this.set_timer(ctx, dur);
        this
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Tick(t) => {
                self.pixels
                    .iter_mut()
                    .for_each(|c| *c = LinSrgb::new(0, 0, 0));
                self.timer = None;
                let dur = self
                    .effect
                    .render(&mut self.pixels, t)
                    .unwrap_or(Duration::milliseconds(50));
                if self.render_pixels().is_ok() {
                    self.set_timer(ctx, dur);
                } else {
                    log::error!("Error rendering pixels");
                }
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        log::info!("Rendering canvas");
        html! {
            <canvas
                width=1000
                height=100
                ref={ self.canvas.clone() }
                style={ "width: 100%; height: 50px;" }
             />
        }
    }
}

impl Preview {
    fn set_timer(&mut self, ctx: &Context<Self>, dur: Duration) {
        let link = ctx.link().clone();
        self.timer = Some(Timeout::new(dur.num_milliseconds() as u32, move || {
            link.send_message(Msg::Tick(now()));
        }));
    }

    fn render_pixels(&self) -> Result<(), anyhow::Error> {
        let canvas = self
            .canvas
            .cast::<HtmlCanvasElement>()
            .ok_or(anyhow!("Wrong canvas element"))?;
        let context = canvas
            .get_context("2d")
            .map_err(|_| anyhow!("can't get 2d context"))?
            .ok_or(anyhow!("Object none"))?
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .map_err(|_| anyhow!("bad cast"))?;

        let buffer = 15.0;
        let width = canvas.width();

        let max_boxes = 250.0;
        let box_width = (width as f64 - buffer * 2.0) / max_boxes;
        let height = canvas.height();
        let center = height as f64 / 2.0;

        context.set_fill_style(&JsValue::from_str("white"));
        context.fill_rect(
            buffer,
            center - box_width,
            max_boxes * box_width,
            box_width * 2.0,
        );
        self.pixels.iter().enumerate().for_each(|(idx, pixel)| {
            let (r, g, b) = pixel.into_components();
            context.set_fill_style(&JsValue::from_str(&format!("rgb({}, {}, {})", r, g, b)));
            context.fill_rect(
                buffer + idx as f64 * box_width,
                center - box_width,
                box_width,
                box_width * 4.0,
            );
        });

        Ok(())
    }
}

fn now() -> DateTime<Utc> {
    let i = instant::now();
    let secs = i / 1000.0;
    Utc.from_utc_datetime(&NaiveDateTime::from_timestamp(
        secs.trunc() as i64,
        (1_000_000_000.0_f64 * secs.fract()).trunc() as u32,
    ))
}
