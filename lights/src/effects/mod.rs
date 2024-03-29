mod rune;
use std::{fmt::Debug, iter, str::FromStr};

use chrono::{serde::ts_milliseconds_option, DateTime, Duration, Utc};
use dyn_clone::DynClone;
use enterpolation::{linear::ConstEquidistantLinear, Curve};
use enum_dispatch::enum_dispatch;
use mopa::mopafy;
use palette::{convert::FromColor, Hsv, LinSrgb};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationMilliSeconds};

pub use self::rune::*;
use crate::error::{Error, Result};

type Instant = DateTime<Utc>;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum EffectType {
    Empty(Empty),
    Ball(Ball),
    Balls(Balls),
    Glow(Glow),
    Rainbow(Rainbow),
    Composite(Composite),
    RuneScript(SourceCode),
}

impl EffectType {
    pub fn iter_names() -> impl Iterator<Item = &'static str> {
        [
            "Empty",
            "Ball",
            "Balls",
            "Glow",
            "Rainbow",
            "Composite",
            "Rune Script",
        ]
        .into_iter()
    }

    pub fn name(&self) -> &'static str {
        match self {
            EffectType::Empty(Empty) => "Empty",
            EffectType::Composite(_) => "Composite",
            EffectType::Ball(_) => "Ball",
            EffectType::Balls(_) => "Balls",
            EffectType::Glow(_) => "Glow",
            EffectType::Rainbow(_) => "Rainbow",
            EffectType::RuneScript(_) => "Rune Script",
        }
    }

    pub fn into_inner(self) -> Box<dyn Effect> {
        match self {
            EffectType::Empty(Empty) => Box::new(Empty),
            EffectType::Composite(c) => Box::new(c),
            EffectType::Ball(b) => Box::new(b),
            EffectType::Balls(bs) => Box::new(bs),
            EffectType::Glow(g) => Box::new(g),
            EffectType::Rainbow(r) => Box::new(r),
            EffectType::RuneScript(s) => {
                let source = RuneScript::from_source(s);
                if source.is_err() {
                    log::error!("Error compiling: {:?}", source);
                    panic!("Error");
                }
                Box::new(source.unwrap())
            }
        }
    }

    pub fn inner_ref(&self) -> &dyn Effect {
        match self {
            EffectType::Empty(Empty) => &Empty,
            EffectType::Composite(c) => c,
            EffectType::Ball(b) => b,
            EffectType::Balls(bs) => bs,
            EffectType::Glow(g) => g,
            EffectType::Rainbow(r) => r,
            EffectType::RuneScript(s) => todo!(),
        }
    }

    pub fn inner_mut_ref(&mut self) -> &mut dyn Effect {
        match self {
            EffectType::Empty(e) => e,
            EffectType::Composite(c) => c,
            EffectType::Ball(b) => b,
            EffectType::Balls(bs) => bs,
            EffectType::Glow(g) => g,
            EffectType::Rainbow(r) => r,
            EffectType::RuneScript(s) => todo!(),
        }
    }

    pub fn default_from_name(name: &str) -> Self {
        match name {
            "Empty" => Self::Empty(Empty),
            "Ball" => Self::Ball(Default::default()),
            "Balls" => Self::Balls(Default::default()),
            "Glow" => Self::Glow(Default::default()),
            "Rainbow" => Self::Rainbow(Default::default()),
            "Composite" => Self::Composite(Default::default()),
            "Rune Script" => Self::RuneScript(Default::default()),
            _ => Default::default(),
        }
    }
}

impl Default for EffectType {
    fn default() -> Self {
        Self::Empty(Empty)
    }
}

impl FromStr for EffectType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Empty" => Ok(Self::Empty(Empty)),
            "Composite" => Ok(Self::Composite(Default::default())),
            "Ball" => Ok(Self::Ball(Default::default())),
            "Balls" => Ok(Self::Balls(Default::default())),
            "Glow" => Ok(Self::Glow(Default::default())),
            "Rainbow" => Ok(Self::Rainbow(Default::default())),
            "Rune Script" => Ok(Self::RuneScript(Default::default())),
            _ => Err(Error::BadEffectType),
        }
    }
}

impl From<Empty> for EffectType {
    fn from(orig: Empty) -> Self {
        Self::Empty(orig)
    }
}

impl From<Ball> for EffectType {
    fn from(orig: Ball) -> Self {
        Self::Ball(orig)
    }
}

impl From<Balls> for EffectType {
    fn from(orig: Balls) -> Self {
        Self::Balls(orig)
    }
}

impl From<Glow> for EffectType {
    fn from(orig: Glow) -> Self {
        Self::Glow(orig)
    }
}

impl From<Composite> for EffectType {
    fn from(orig: Composite) -> Self {
        Self::Composite(orig)
    }
}

impl From<Rainbow> for EffectType {
    fn from(orig: Rainbow) -> Self {
        Self::Rainbow(orig)
    }
}

impl From<RuneScript> for EffectType {
    fn from(orig: RuneScript) -> Self {
        Self::RuneScript(orig.sourcecode)
    }
}

#[enum_dispatch]
pub trait Effect:
    Debug + mopa::Any + serde_traitobject::Serialize + serde_traitobject::Deserialize + DynClone
{
    fn render(&mut self, pixels: &mut [LinSrgb<u8>], t: Instant) -> Result<Duration>;
    fn is_ready(&self, t: Instant) -> Result<bool>;

    fn to_cloned_type(&self) -> EffectType;
}

mopafy!(Effect);

dyn_clone::clone_trait_object!(Effect);

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Empty;

impl Effect for Empty {
    fn render(&mut self, _pixels: &mut [LinSrgb<u8>], _t: Instant) -> Result<Duration> {
        Ok(Duration::milliseconds(100))
    }

    fn is_ready(&self, _t: Instant) -> Result<bool> {
        Ok(true)
    }

    fn to_cloned_type(&self) -> EffectType {
        EffectType::Empty(Empty)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Composite(Box<EffectType>, Box<EffectType>);

impl Default for Composite {
    fn default() -> Self {
        Self::new(EffectType::default(), EffectType::default()).expect("empty")
    }
}

impl Composite {
    pub fn new(first: EffectType, second: EffectType) -> Result<Self> {
        if matches!(first, EffectType::RuneScript(_)) || matches!(second, EffectType::RuneScript(_))
        {
            Err(Error::CompositeScriptError)
        } else {
            Ok(Self(Box::new(first), Box::new(second)))
        }
    }

    pub fn first(&self) -> &EffectType {
        self.0.as_ref()
    }

    pub fn set_first(&mut self, e: EffectType) -> Result<()> {
        if matches!(e, EffectType::RuneScript(_)) {
            return Err(Error::CompositeScriptError);
        }
        self.0 = Box::new(e);
        Ok(())
    }

    pub fn second(&self) -> &EffectType {
        self.1.as_ref()
    }

    pub fn set_second(&mut self, e: EffectType) -> Result<()> {
        if matches!(e, EffectType::RuneScript(_)) {
            return Err(Error::CompositeScriptError);
        }
        self.1 = Box::new(e);
        Ok(())
    }
}

impl Effect for Composite {
    fn render(&mut self, pixels: &mut [LinSrgb<u8>], t: Instant) -> Result<Duration> {
        let d = self.0.inner_mut_ref().render(pixels, t)?;
        Ok(std::cmp::min(d, self.1.inner_mut_ref().render(pixels, t)?))
    }

    fn is_ready(&self, t: Instant) -> Result<bool> {
        [self.0.inner_ref(), self.1.inner_ref()]
            .iter()
            .try_fold(false, |accum, e| Ok(accum || e.is_ready(t)?))
    }

    fn to_cloned_type(&self) -> EffectType {
        EffectType::Composite(self.clone())
    }
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Ball {
    pub color: LinSrgb<u8>,
    pub position: usize,
    pub count: usize,
    pub direction: i8,
    pub bounce: bool,

    #[serde_as(as = "DurationMilliSeconds<i64>")]
    pub delay: Duration,
    #[serde(with = "ts_milliseconds_option")]
    next_update: Option<Instant>,
}

impl Ball {
    pub fn new(
        color: LinSrgb<u8>,
        position: usize,
        direction: i8,
        bounce: bool,
        delay: Duration,
        count: usize,
    ) -> Self {
        Self {
            color,
            position,
            direction,
            bounce,
            count,

            delay,
            next_update: None,
        }
    }

    pub fn wrap(color: LinSrgb<u8>, millis: i64, count: usize) -> Self {
        Self::new(color, 0, 1, false, Duration::milliseconds(millis), count)
    }

    pub fn wrap_backward(color: LinSrgb<u8>, millis: i64, count: usize) -> Self {
        Self::new(
            color,
            count - 1,
            -1,
            false,
            Duration::milliseconds(millis),
            count,
        )
    }

    pub fn bounce(color: LinSrgb<u8>, millis: i64, count: usize) -> Self {
        Self::new(color, 0, 1, true, Duration::milliseconds(millis), count)
    }

    pub fn bounce_backward(color: LinSrgb<u8>, millis: i64, count: usize) -> Self {
        Self::new(
            color,
            count - 1,
            -1,
            true,
            Duration::milliseconds(millis),
            count,
        )
    }

    fn update_state(&mut self) {
        if self.bounce {
            if self.position == self.count - 1 {
                self.direction = -1;
            } else if self.position == 0 {
                self.direction = 1;
            }
        } else {
            if self.position == self.count - 1 && self.direction > 0 {
                self.position = 0;
            } else if self.position == 0 && self.direction < 0 {
                self.position = self.count - 1;
            }
        }
    }

    pub fn color(&self) -> LinSrgb<u8> {
        self.color
    }

    pub fn is_bounce(&self) -> bool {
        self.bounce
    }
}

impl Default for Ball {
    fn default() -> Self {
        Self {
            color: LinSrgb::new(255, 0, 0),
            position: 0,
            count: 1,
            direction: 1,
            bounce: false,
            delay: Duration::milliseconds(100),
            next_update: None,
        }
    }
}

impl Effect for Ball {
    fn render(&mut self, pixels: &mut [LinSrgb<u8>], t: Instant) -> Result<Duration> {
        if self.count != pixels.len() {
            self.count = pixels.len();
        }
        if self.is_ready(t)? {
            self.update_state();
            let pixel = (self.position as isize + self.direction as isize) as usize % pixels.len();
            self.position = pixel;
            self.next_update = Some(t + self.delay);
        }
        pixels[self.position] = self.color;

        let time_left = self
            .next_update
            .map(|u| u.signed_duration_since(t))
            .unwrap_or(Duration::seconds(0));
        if self.delay > time_left {
            Ok(self.delay - time_left)
        } else {
            Ok(self.delay)
        }
    }

    fn is_ready(&self, t: Instant) -> Result<bool> {
        Ok(self.next_update.is_none() || t >= self.next_update.unwrap())
    }

    fn to_cloned_type(&self) -> EffectType {
        EffectType::Ball(self.clone())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Balls(Vec<Ball>);

impl Balls {
    pub fn new(balls: &[Ball]) -> Self {
        Self(balls.to_vec())
    }

    pub fn balls(&self) -> &[Ball] {
        &self.0
    }

    pub fn add_ball(&mut self) {
        self.0.push(Ball::default());
    }

    pub fn set_ball(&mut self, idx: usize, ball: Ball) -> Result<()> {
        *self.0.get_mut(idx).ok_or(Error::IndexOutOfRange)? = ball;
        Ok(())
    }

    pub fn remove_ball(&mut self, idx: usize) {
        self.0.remove(idx);
    }
}

impl Effect for Balls {
    fn render(&mut self, pixels: &mut [LinSrgb<u8>], t: Instant) -> Result<Duration> {
        let mut min = Duration::seconds(1);
        for ball in self.0.iter_mut() {
            let d = ball.render(pixels, t)?;
            if d < min {
                min = d;
            }
        }

        Ok(min)
    }

    fn is_ready(&self, t: Instant) -> Result<bool> {
        self.0
            .iter()
            .try_fold(false, |accum, b| Ok(accum || b.is_ready(t)?))
    }

    fn to_cloned_type(&self) -> EffectType {
        EffectType::Balls(self.clone())
    }
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Glow {
    colors: Vec<LinSrgb<u8>>,
    color_idx: usize,
    cur_color: LinSrgb<u8>,
    step: usize,
    pub steps: usize,

    #[serde_as(as = "DurationMilliSeconds<i64>")]
    pub delay: Duration,
    #[serde(with = "ts_milliseconds_option")]
    next_update: Option<Instant>,
}

impl Glow {
    pub fn new(colors: Vec<LinSrgb<u8>>, steps: usize, delay: Duration) -> Self {
        Self {
            cur_color: colors[0],
            colors,
            color_idx: 0,
            step: 0,
            steps,

            delay,
            next_update: None,
        }
    }

    pub fn colors(&self) -> &[LinSrgb<u8>] {
        &self.colors[..]
    }

    pub fn set_color(&mut self, idx: usize, color: LinSrgb<u8>) {
        self.colors[idx] = color;
    }

    pub fn add_color(&mut self, color: LinSrgb<u8>) {
        self.colors.push(color);
    }

    pub fn remove_color(&mut self, idx: usize) {
        self.colors.remove(idx);
    }
}

impl Default for Glow {
    fn default() -> Self {
        Self::new(
            vec![LinSrgb::new(0, 0, 0), LinSrgb::new(255, 0, 0)],
            10,
            Duration::milliseconds(100),
        )
    }
}

impl Effect for Glow {
    fn render(&mut self, pixels: &mut [LinSrgb<u8>], t: Instant) -> Result<Duration> {
        let color: LinSrgb<u8> = if self.is_ready(t)? {
            let gradient = ConstEquidistantLinear::<f32, _, 2>::equidistant_unchecked([
                self.colors[self.color_idx].into_format::<f32>(),
                self.colors[(self.color_idx + 1) % self.colors.len()].into_format(),
            ]);
            let color = gradient.take(self.steps).skip(self.step).next().unwrap();
            self.step += 1;
            if self.step == self.steps {
                self.step = 0;
                self.color_idx = (self.color_idx + 1) % self.colors.len();
            }

            self.next_update = Some(t + self.delay);
            color.into_format()
        } else {
            LinSrgb::new(0, 0, 0)
        };

        for pixel in pixels {
            *pixel = color;
        }

        let time_left = self.next_update.map(|u| u.signed_duration_since(t));
        if time_left.is_none() || self.delay <= time_left.unwrap() {
            Ok(self.delay)
        } else {
            Ok(self.delay - time_left.unwrap())
        }
    }

    fn is_ready(&self, t: Instant) -> Result<bool> {
        Ok(self.next_update.map(|u| t >= u).unwrap_or(true))
    }

    fn to_cloned_type(&self) -> EffectType {
        EffectType::Glow(self.clone())
    }
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Rainbow {
    colors: Vec<LinSrgb<u8>>,
    color_width: usize,
    spacing: usize,
    pub direction: i8,

    step: usize,

    color_strip: Vec<LinSrgb<u8>>,

    #[serde_as(as = "DurationMilliSeconds<i64>")]
    pub delay: Duration,
    #[serde(with = "ts_milliseconds_option")]
    next_update: Option<Instant>,
}

impl Rainbow {
    pub fn new(
        colors: Vec<LinSrgb<u8>>,
        color_width: usize,
        spacing: usize,
        direction: i8,
        delay: Duration,
    ) -> Self {
        let mut this = Self {
            colors,
            color_width,
            spacing,
            direction,
            step: 0,
            color_strip: vec![],
            delay,
            next_update: None,
        };
        this.generate_color_strip();
        this
    }

    fn generate_color_strip(&mut self) {
        self.color_strip = self
            .colors
            .iter()
            .cloned()
            .map(|c| {
                iter::repeat(c)
                    .take(self.color_width)
                    .chain(iter::repeat(LinSrgb::new(0u8, 0, 0)).take(self.spacing))
            })
            .flatten()
            .collect::<Vec<_>>();
        log::info!("strip length: {}", self.color_strip.len());
    }

    pub fn colors(&self) -> &[LinSrgb<u8>] {
        &self.colors[..]
    }

    pub fn set_color(&mut self, idx: usize, color: LinSrgb<u8>) {
        self.colors[idx] = color;
        self.generate_color_strip();
    }

    pub fn add_color(&mut self, color: LinSrgb<u8>) {
        self.colors.push(color);
        self.generate_color_strip();
    }

    pub fn remove_color(&mut self, idx: usize) {
        self.colors.remove(idx);
        self.generate_color_strip();
    }

    pub fn set_spacing(&mut self, spacing: usize) {
        self.spacing = spacing;
        self.generate_color_strip();
    }

    pub fn spacing(&self) -> usize {
        self.spacing
    }

    pub fn set_color_width(&mut self, width: usize) {
        self.color_width = width;
        self.generate_color_strip();
    }

    pub fn color_width(&self) -> usize {
        self.color_width
    }
}

impl Effect for Rainbow {
    fn render(&mut self, pixels: &mut [LinSrgb<u8>], t: Instant) -> Result<Duration> {
        let len = pixels.len();
        let tmp = &self.color_strip[..]
            .iter()
            .cloned()
            .cycle()
            .take(len)
            .collect::<Vec<_>>();
        pixels[self.step..].copy_from_slice(&tmp[..len - self.step]);
        pixels[..self.step].copy_from_slice(&tmp[len - self.step..]);
        // pixels[self.step] = LinSrgb::new(255, 255, 255);

        if self.is_ready(t)? {
            self.step =
                (self.step as isize + (1 * self.direction as isize)) as usize % pixels.len();
            self.next_update = Some(t + self.delay);
        }

        let time_left = self.next_update.map(|u| u.signed_duration_since(t));
        if time_left.is_none() || self.delay <= time_left.unwrap() {
            Ok(self.delay)
        } else {
            Ok(self.delay - time_left.unwrap())
        }
    }

    fn is_ready(&self, t: Instant) -> Result<bool> {
        Ok(self.next_update.map(|u| t >= u).unwrap_or(true))
    }

    fn to_cloned_type(&self) -> EffectType {
        EffectType::Rainbow(self.clone())
    }
}

impl Default for Rainbow {
    fn default() -> Self {
        Self::new(
            vec![
                LinSrgb::new(0xff, 0x00, 0x00),
                LinSrgb::new(0xFF, 0x7F, 0x00),
                LinSrgb::new(0xFF, 0xFF, 0x00),
                LinSrgb::new(0x00, 0xFF, 0x00),
                LinSrgb::new(0x00, 0x00, 0xFF),
                LinSrgb::new(0x4B, 0x00, 0x82),
                LinSrgb::new(0x94, 0x00, 0xD3),
            ],
            1,
            3,
            1,
            Duration::milliseconds(100),
        )
    }
}
