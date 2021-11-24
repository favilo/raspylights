use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use lights::{
    details::Details,
    effects::{Effect, EffectType, Empty},
};
use palette::LinSrgb;
#[cfg(target_arch = "arm")]
use rs_ws281x::{ChannelBuilder, Controller, ControllerBuilder, StripType};

pub struct LedStrip {
    #[cfg(target_arch = "arm")]
    cont: Option<Controller>,

    pixels: Vec<LinSrgb<u8>>,
    details: Details,
    effect: Box<dyn Effect>,
}

// Just to make sure I don't try to send this anywhere
impl !Send for LedStrip {}

impl LedStrip {
    pub fn new(details: Details) -> Result<Self> {
        // TODO: Get this customizable more
        #[cfg(target_arch = "arm")]
        let cont = Some(Self::construct_controller(
            details.length,
            details.brightness,
        )?);
        let pixels = vec![LinSrgb::new(0, 0, 0); details.length];
        let effect = details.effect.clone().into_inner();

        Ok(Self {
            #[cfg(target_arch = "arm")]
            cont,
            pixels,

            details,
            effect,
        })
    }

    #[cfg(target_arch = "arm")]
    pub fn construct_controller(length: usize, brightness: u8) -> Result<Controller> {
        Ok(ControllerBuilder::new()
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
            .build()?)
    }

    #[allow(unused)]
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

    pub fn update(
        &mut self,
        now: DateTime<Utc>,
    ) -> std::result::Result<Duration, lights::error::Error> {
        self.effect.render(&mut self.pixels, now)
    }

    #[cfg(target_arch = "arm")]
    pub fn render(&mut self) -> Result<()> {
        debug_assert!(self.cont.is_some());
        let leds = self.cont.as_mut().expect("can't be None").leds_mut(0);
        for (i, pixel) in self.pixels.iter().enumerate() {
            let (r, g, b) = pixel.into_components();
            leds[i] = [r, g, b, 0];
        }
        self.cont.as_mut().expect("can't be None").render()?;

        Ok(())
    }

    #[cfg(not(target_arch = "arm"))]
    pub fn render(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn set_effect(&mut self, effect: EffectType) -> Result<()> {
        self.details.effect = effect.clone();
        self.effect = effect.into_inner();
        Ok(())
    }

    pub fn set_length(&mut self, length: usize) -> Result<()> {
        if self.details.length == length {
            return Ok(());
        }
        self.details.length = length;
        self.pixels = vec![LinSrgb::new(0, 0, 0); length];
        #[cfg(target_arch = "arm")]
        {
            let _ = self.cont.take();
            self.cont = Some(Self::construct_controller(length, self.details.brightness)?);
        }
        Ok(())
    }

    pub fn set_brightness(&mut self, brightness: u8) -> Result<()> {
        if self.details.brightness == brightness {
            return Ok(());
        }
        self.details.brightness = brightness;
        #[cfg(target_arch = "arm")]
        {
            // Explicityly drop the controller, to clean up pointers
            let old_cont = self.cont.take();
            drop(old_cont);
            self.cont = Some(Self::construct_controller(self.details.length, brightness)?);
        }
        Ok(())
    }
}
