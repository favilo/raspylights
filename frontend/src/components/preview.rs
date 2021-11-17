use anyhow::anyhow;
use chrono::{DateTime, Duration, NaiveDateTime, TimeZone, Utc};
use lights::effects::EffectType;
use palette::LinSrgb;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlCanvasElement;
use yew::{
    prelude::*,
    services::{timeout::TimeoutTask, TimeoutService},
};

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
    link: ComponentLink<Self>,
    props: Props,

    pixels: Vec<LinSrgb<u8>>,
    timer: Option<Box<TimeoutTask>>,
    canvas: NodeRef,
}

impl Component for Preview {
    type Message = Msg;

    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let pixels = vec![Default::default(); props.length];
        let mut this = Self {
            link,
            props,
            pixels,
            timer: None,
            canvas: Default::default(),
        };
        let dur = this
            .props
            .effect
            .render(&mut this.pixels, now())
            .unwrap_or(Duration::milliseconds(50));
        this.set_timer(dur);
        this
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Tick(t) => {
                self.pixels
                    .iter_mut()
                    .for_each(|c| *c = LinSrgb::new(0, 0, 0));
                self.timer = None;
                let dur = self
                    .props
                    .effect
                    .render(&mut self.pixels, t)
                    .unwrap_or(Duration::milliseconds(50));
                if self.render_pixels().is_ok() {
                    self.set_timer(dur);
                } else {
                    log::error!("Error rendering pixels");
                }
                true
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        let mut changed = false;
        if props.length != self.props.length {
            self.pixels.resize(props.length, Default::default());
            changed = true;
        }
        if props.effect != self.props.effect {
            self.props.effect = props.effect;
            let dur = self.props.effect.render(&mut self.pixels, now()).unwrap();
            self.set_timer(dur);
            changed = true;
        }
        changed
    }

    fn view(&self) -> Html {
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
    fn set_timer(&mut self, dur: Duration) {
        let handle: TimeoutTask = TimeoutService::spawn(
            std::time::Duration::from_millis(dur.num_milliseconds().try_into().unwrap()),
            self.link.callback(|_| Msg::Tick(now())),
        );
        self.timer = Some(Box::new(handle));
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

        // let l = self.pixels.len();
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
