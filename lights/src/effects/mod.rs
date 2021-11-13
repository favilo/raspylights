use std::{
    fmt::Debug,
    str::FromStr,
    time::{Duration, Instant},
};

use crate::error::{Error, Result};
use enum_dispatch::enum_dispatch;
use mopa::mopafy;
use palette::{convert::FromColor, Gradient, Hsv, LinSrgb};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[enum_dispatch(Effect)]
pub enum EffectType {
    Empty(Empty),
    Ball(Ball),
    Balls(Balls),
    Glow(Glow),
    // Rainbow(Rainbow),
    Composite(Composite),
}

impl EffectType {
    pub fn iter_names() -> impl Iterator<Item = &'static str> {
        [
            Self::Empty(Empty).name(),
            Self::Ball(Default::default()).name(),
            Self::Balls(Default::default()).name(),
            Self::Glow(Default::default()).name(),
            Self::Composite(Default::default()).name(),
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
        }
    }

    pub fn into_inner(self) -> Box<dyn Effect> {
        match self {
            EffectType::Empty(Empty) => Box::new(Empty),
            EffectType::Composite(c) => Box::new(c),
            EffectType::Ball(b) => Box::new(b),
            EffectType::Balls(bs) => Box::new(bs),
            EffectType::Glow(g) => Box::new(g),
        }
    }

    pub fn inner_ref(&self) -> &dyn Effect {
        match self {
            EffectType::Empty(Empty) => &Empty,
            EffectType::Composite(c) => c,
            EffectType::Ball(b) => b,
            EffectType::Balls(bs) => bs,
            EffectType::Glow(g) => g,
        }
    }

    pub fn inner_mut_ref(&mut self) -> &mut dyn Effect {
        match self {
            EffectType::Empty(e) => e,
            EffectType::Composite(c) => c,
            EffectType::Ball(b) => b,
            EffectType::Balls(bs) => bs,
            EffectType::Glow(g) => g,
        }
    }

    pub fn default_from_name(name: &str) -> Self {
        match name {
            "Empty" => Self::Empty(Empty),
            "Ball" => Self::Ball(Default::default()),
            "Balls" => Self::Balls(Default::default()),
            "Glow" => Self::Glow(Default::default()),
            "Composite" => Self::Composite(Default::default()),
            _ => Default::default(),
        }
    }

    pub fn render(&mut self, controller: &mut [LinSrgb<u8>], t: Instant) -> Result<Duration> {
        match self {
            EffectType::Empty(Empty) => Empty.render(controller, t),
            EffectType::Ball(b) => b.render(controller, t),
            EffectType::Balls(bs) => bs.render(controller, t),
            EffectType::Glow(g) => g.render(controller, t),
            EffectType::Composite(c) => c.render(controller, t),
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
            // "Rainbow" => Ok(Self::Rainbow(Default::default())),
            _ => Err(Error::BadEffectType),
        }
    }
}

#[enum_dispatch]
pub trait Effect:
    Debug + mopa::Any + serde_traitobject::Serialize + serde_traitobject::Deserialize
{
    fn render(&mut self, controller: &mut [LinSrgb<u8>], t: Instant) -> Result<Duration>;
    fn is_ready(&self, t: Instant) -> bool;

    fn to_cloned_type(&self) -> EffectType;
}

mopafy!(Effect);

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Empty;

impl Effect for Empty {
    fn render(&mut self, _controller: &mut [LinSrgb<u8>], _t: Instant) -> Result<Duration> {
        Ok(Duration::from_millis(100))
    }

    fn is_ready(&self, _t: Instant) -> bool {
        true
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
        Ok(Self(Box::new(first), Box::new(second)))
    }

    pub fn first(&self) -> &EffectType {
        &self.0
    }

    pub fn set_first(&mut self, e: EffectType) {
        self.0 = Box::new(e);
    }

    pub fn second(&self) -> &EffectType {
        &self.1
    }

    pub fn set_second(&mut self, e: EffectType) {
        self.1 = Box::new(e);
    }
}

impl Effect for Composite {
    fn render(&mut self, controller: &mut [LinSrgb<u8>], t: Instant) -> Result<Duration> {
        let d = self.0.inner_mut_ref().render(controller, t)?;
        Ok(std::cmp::min(
            d,
            self.1.inner_mut_ref().render(controller, t)?,
        ))
    }

    fn is_ready(&self, t: Instant) -> bool {
        [self.0.inner_ref(), self.1.inner_ref()]
            .iter()
            .any(|e| e.is_ready(t))
    }

    fn to_cloned_type(&self) -> EffectType {
        EffectType::Composite(self.clone())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Ball {
    pub color: LinSrgb<u8>,
    pub position: usize,
    pub count: usize,
    pub direction: i8,
    pub bounce: bool,

    pub delay: Duration,
    #[serde(with = "serde_millis")]
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

    pub fn wrap(color: LinSrgb<u8>, millis: u64, count: usize) -> Self {
        Self::new(color, 0, 1, false, Duration::from_millis(millis), count)
    }

    pub fn wrap_backward(color: LinSrgb<u8>, millis: u64, count: usize) -> Self {
        Self::new(
            color,
            count - 1,
            -1,
            false,
            Duration::from_millis(millis),
            count,
        )
    }

    pub fn bounce(color: LinSrgb<u8>, millis: u64, count: usize) -> Self {
        Self::new(color, 0, 1, true, Duration::from_millis(millis), count)
    }

    pub fn bounce_backward(color: LinSrgb<u8>, millis: u64, count: usize) -> Self {
        Self::new(
            color,
            count - 1,
            -1,
            true,
            Duration::from_millis(millis),
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
            delay: Duration::from_millis(100),
            next_update: None,
        }
    }
}

impl Effect for Ball {
    fn render(&mut self, pixels: &mut [LinSrgb<u8>], t: Instant) -> Result<Duration> {
        if self.count != pixels.len() {
            self.count = pixels.len();
        }
        if self.is_ready(t) {
            self.update_state();
            let pixel = (self.position as isize + self.direction as isize) as usize % pixels.len();
            self.position = pixel;
            self.next_update = Some(t + self.delay);
        }
        pixels[self.position] = self.color;

        let time_left = self
            .next_update
            .map(|u| u.duration_since(t))
            .unwrap_or(Duration::from_secs(0));
        if self.delay > time_left {
            Ok(self.delay
                - self
                    .next_update
                    .map(|u| u.duration_since(t))
                    .unwrap_or(Duration::from_secs(0)))
        } else {
            Ok(self.delay)
        }
    }

    fn is_ready(&self, t: Instant) -> bool {
        self.next_update.is_none() || t >= self.next_update.unwrap()
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
}

impl Effect for Balls {
    fn render(&mut self, controller: &mut [LinSrgb<u8>], t: Instant) -> Result<Duration> {
        let mut min = Duration::from_secs(1);
        for ball in self.0.iter_mut() {
            let d = ball.render(controller, t)?;
            if d < min {
                min = d;
            }
        }

        Ok(min)
    }

    fn is_ready(&self, t: Instant) -> bool {
        self.0.iter().any(|b| b.is_ready(t))
    }

    fn to_cloned_type(&self) -> EffectType {
        EffectType::Balls(self.clone())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Glow {
    colors: Vec<LinSrgb<u8>>,
    color_idx: usize,
    cur_color: LinSrgb<u8>,
    step: usize,
    steps: usize,

    delay: Duration,
    #[serde(with = "serde_millis")]
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
}

impl Default for Glow {
    fn default() -> Self {
        Self::new(vec![LinSrgb::new(0, 0, 0)], 1, Duration::from_millis(100))
    }
}

impl Effect for Glow {
    fn render(&mut self, pixels: &mut [LinSrgb<u8>], t: Instant) -> Result<Duration> {
        let color: LinSrgb<u8> = if self.is_ready(t) {
            let gradient = Gradient::new(vec![
                Hsv::from_color(self.colors[self.color_idx].into_format()),
                Hsv::from_color(
                    self.colors[(self.color_idx + 1) % self.colors.len()].into_format(),
                ),
            ]);
            let color = gradient.get(self.step as f32 / self.steps as f32);
            self.step += 1;
            if self.step == self.steps {
                self.step = 0;
                self.color_idx = (self.color_idx + 1) % self.colors.len();
            }

            self.next_update = Some(t + self.delay);
            LinSrgb::from_color(color).into_format()
        } else {
            LinSrgb::new(0, 0, 0)
        };

        for pixel in pixels {
            *pixel = color;
        }

        let time_left = self.next_update.map(|u| u.duration_since(t));
        if time_left.is_none() {
            Ok(self.delay)
        } else if self.delay > time_left.unwrap() {
            Ok(self.delay - time_left.unwrap())
        } else {
            Ok(self.delay)
        }
    }

    fn is_ready(&self, t: Instant) -> bool {
        self.next_update.map(|u| t >= u).unwrap_or(true)
    }

    fn to_cloned_type(&self) -> EffectType {
        EffectType::Glow(self.clone())
    }
}
