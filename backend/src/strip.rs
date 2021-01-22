use std::time::Duration;
use std::time::Instant;

use anyhow::Result;
use palette::LinSrgb;
use rs_ws281x::{ChannelBuilder, Controller, ControllerBuilder, StripType};

use lights::effects::Effect;

pub struct LedStrip {
    cont: Controller,

    pixels: Vec<LinSrgb<u8>>,
    effect: Box<dyn Effect>,
}

impl LedStrip {
    pub fn new(length: usize, brightness: u8) -> Result<Self> {
        // TODO: Get this customizable more
        let cont = ControllerBuilder::new()
            .freq(800_000)
            .dma(10)
            .channel(
                0,
                ChannelBuilder::new()
                    .pin(18)
                    .count(length as i32)
                    .strip_type(StripType::Ws2811Rbg)
                    .brightness(brightness)
                    .build(),
            )
            .build()?;
        let pixels = vec![LinSrgb::new(0, 0, 0); length];

        Ok(Self {
            cont,
            pixels,

            effect: Box::new(lights::effects::Empty),
        })
    }

    pub fn len(&self) -> usize {
        self.pixels.len()
    }

    pub fn clear(&mut self) -> Result<()> {
        self.clear_color(LinSrgb::new(0, 0, 0))
    }

    pub fn clear_color(&mut self, color: LinSrgb<u8>) -> Result<()> {
        for pixel in self.pixels.iter_mut() {
            *pixel = color.clone();
        }
        Ok(())
    }

    pub fn update(&mut self, now: Instant) -> std::result::Result<Duration, lights::error::Error> {
        self.effect.render(&mut self.pixels, now)
    }

    pub fn render(&mut self) -> Result<()> {
        let leds = self.cont.leds_mut(0);
        for (i, pixel) in self.pixels.iter().enumerate() {
            let (r, g, b) = pixel.into_components();
            leds[i] = [r, g, b, 0];
        }
        self.cont.render()?;

        Ok(())
    }

    pub fn set_effect<E: Effect>(&mut self, effect: E) -> Result<()> {
        self.effect = Box::new(effect);
        Ok(())
    }
}
