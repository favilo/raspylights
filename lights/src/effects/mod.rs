use std::{
    fmt::Debug,
    str::FromStr,
    time::{Duration, Instant},
};

use crate::error::{Error, Result};
use mopa::mopafy;
use palette::{convert::FromColor, Gradient, Hsv, LinSrgb};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, strum::EnumIter, Serialize, Deserialize, PartialEq)]
pub enum EffectType {
    Empty,
    Ball,
    Balls,
    Glow,
    Rainbow,
    Composite,
}

impl EffectType {
    pub fn to_default(&self) -> Box<dyn Effect> {
        match self {
            EffectType::Empty => Box::new(Empty),
            EffectType::Composite => Box::new(Composite::default()),
            EffectType::Ball => Box::new(Ball::default()),
            EffectType::Balls => Box::new(Balls::default()),
            EffectType::Glow => Box::new(Glow::default()),
            EffectType::Rainbow => todo!(),
        }
    }
}

impl FromStr for EffectType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Empty" => Ok(Self::Empty),
            "Composite" => Ok(Self::Composite),
            "Balls" => Ok(Self::Balls),
            "Glow" => Ok(Self::Glow),
            "Rainbow" => todo!(),
            _ => Err(Error::BadEffectType),
        }
    }
}

pub trait Effect:
    Debug + mopa::Any + serde_traitobject::Serialize + serde_traitobject::Deserialize
{
    fn render(&mut self, controller: &mut [LinSrgb<u8>], t: Instant) -> Result<Duration>;
    fn is_ready(&self, t: Instant) -> bool;
    fn to_type(&self) -> EffectType;
}

mopafy!(Effect);

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Empty;

impl Effect for Empty {
    fn render(&mut self, _controller: &mut [LinSrgb<u8>], _t: Instant) -> Result<Duration> {
        Ok(Duration::from_millis(100))
    }

    fn is_ready(&self, _t: Instant) -> bool {
        true
    }

    fn to_type(&self) -> EffectType {
        EffectType::Empty
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Composite(
    #[serde(with = "serde_traitobject")] Box<dyn Effect>,
    #[serde(with = "serde_traitobject")] Box<dyn Effect>,
);

impl Default for Composite {
    fn default() -> Self {
        Self::new(Empty, Empty).expect("empty")
    }
}

impl Composite {
    pub fn new<E, F>(first: E, second: F) -> Result<Self>
    where
        E: Effect,
        F: Effect,
    {
        Ok(Self(Box::new(first), Box::new(second)))
    }

    pub fn first(&self) -> &Box<dyn Effect> {
        &self.0
    }

    pub fn second(&self) -> &Box<dyn Effect> {
        &self.1
    }
}

impl Effect for Composite {
    fn render(&mut self, controller: &mut [LinSrgb<u8>], t: Instant) -> Result<Duration> {
        let d = self.0.render(controller, t)?;
        Ok(std::cmp::min(d, self.1.render(controller, t)?))
    }

    fn is_ready(&self, t: Instant) -> bool {
        [&self.0, &self.1].iter().any(|e| e.is_ready(t))
    }

    fn to_type(&self) -> EffectType {
        EffectType::Composite
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
            color: LinSrgb::new(0, 0, 0),
            position: 0,
            count: 0,
            direction: 1,
            bounce: false,
            delay: Duration::from_millis(100),
            next_update: None,
        }
    }
}

impl Effect for Ball {
    fn render(&mut self, pixels: &mut [LinSrgb<u8>], t: Instant) -> Result<Duration> {
        if self.is_ready(t) {
            self.update_state();
            let pixel = (self.position as isize + self.direction as isize) as usize % self.count;
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

    fn to_type(&self) -> EffectType {
        EffectType::Ball
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
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

    fn to_type(&self) -> EffectType {
        EffectType::Balls
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

    fn to_type(&self) -> EffectType {
        EffectType::Glow
    }
}
