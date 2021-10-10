use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use crate::strip::LedStrip;
use anyhow::Result;
use lights::effects;
use palette::LinSrgb;
use signal_hook::consts;
use tide::{prelude::*, Request};

mod strip;

#[allow(dead_code)]
fn make_colors() -> Result<()> {
    let term = Arc::new(AtomicBool::new(false));
    for sig in &[
        consts::SIGTERM,
        consts::SIGINT,
        consts::SIGHUP,
        consts::SIGQUIT,
        consts::SIGPIPE,
    ] {
        signal_hook::flag::register(*sig, Arc::clone(&term))?;
    }

    let mut strip = LedStrip::new(200, 255)?;

    let effect = effects::Balls::new(&[
        effects::Ball::bounce(LinSrgb::new(255, 0, 0), 50, strip.len()),
        effects::Ball::wrap(LinSrgb::new(0, 0, 255), 100, strip.len()),
        effects::Ball::bounce_backward(LinSrgb::new(0, 255, 0), 50, strip.len()),
        effects::Ball::wrap_backward(LinSrgb::new(255, 255, 0), 100, strip.len()),
    ]);
    let background = effects::Glow::new(
        vec![
            LinSrgb::new(0, 0, 0),
            LinSrgb::new(255, 0, 0),
            LinSrgb::new(0, 0, 0),
            LinSrgb::new(0, 255, 0),
            LinSrgb::new(0, 0, 0),
            LinSrgb::new(0, 0, 255),
        ],
        50,
        Duration::from_millis(50),
    );
    let effect = effects::Composite::new(background, effect)?;
    strip.set_effect(effect)?;

    loop {
        strip.clear()?;
        let now = Instant::now();

        if term.load(Ordering::Relaxed) {
            break;
        }
        let d = strip.update(now)?;
        strip.render()?;
        thread::sleep(d - now.elapsed());
    }

    println!("Done");
    strip.render()?;

    Ok(())
}

async fn web_main() -> Result<()> {
    tide::log::start();

    let mut app = tide::Server::with_state(());
    app.at("/").serve_file("./frontend/index.html")?;
    app.at("/pkg").serve_dir("./frontend/pkg/")?;
    app.at("/mdc").serve_dir("./frontend/static/mdc/")?;
    app.at("/style.css")
        .serve_file("./frontend/static/style.css")?;
    app.listen("0.0.0.0:8000").await?;
    Ok(())
}

fn main() -> Result<()> {
    // make_colors()?;
    async_std::task::block_on(async { web_main().await })?;
    Ok(())
}
