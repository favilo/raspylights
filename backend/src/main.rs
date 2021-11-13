mod storage;
mod strip;

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use anyhow::Result;
use async_std::{
    channel::{self, Receiver, Sender},
    prelude::FutureExt,
    sync::{Mutex, RwLock},
    task,
};
use lights::{details::Details, effects, error::Error};
use palette::LinSrgb;
use signal_hook::consts;
use tide::{http::mime, log, prelude::*, Request, Response};

use crate::{storage::Storage, strip::LedStrip};

async fn render_main(
    receiver: Receiver<Details>,
    details: Arc<RwLock<Details>>,
    term: Arc<AtomicBool>,
    storage: Storage,
) -> Result<()> {
    let storage = Arc::new(Mutex::new(storage));
    let mut strip = LedStrip::new(details.read().await.clone())?;
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
    let effect = effects::Composite::new(background.into(), effect.into())?;
    strip.set_effect(effect.into())?;

    loop {
        strip.clear()?;
        let now = Instant::now();

        if term.load(Ordering::Relaxed) {
            break;
        }
        if let Ok(deets) = receiver.try_recv() {
            log::info!("We got some deets: {:#?}", deets);
            strip.set_effect(deets.effect.clone())?;
            strip.set_length(deets.length)?;
            *details.write().await = deets.clone();

            // TODO: Make this function async, so we can await in a different thread
            storage
                .lock()
                .await
                .store("main", deets)
                .await
                .map_err(|_| Error::HeedError)?;
        }
        let d = strip.update(now)?;
        strip.render()?;

        task::sleep(d - now.elapsed()).await;
    }

    strip.render()?;

    Ok(())
}

#[derive(Clone)]
struct State {
    details: Arc<RwLock<Details>>,
    sender: Sender<Details>,
}

async fn get_details(req: Request<State>) -> tide::Result {
    let resp = Response::builder(200)
        .body(json!(*req.state().details.read().await))
        .content_type(mime::JSON)
        .build();
    Ok(resp.into())
}

async fn post_details(mut req: Request<State>) -> tide::Result {
    let details: Details = req.body_json().await?;
    let state = req.state();
    state.sender.send(details.clone()).await?;
    let resp = Response::builder(200)
        .body(json!(details))
        .content_type(mime::JSON)
        .build();
    Ok(resp.into())
}

async fn web_main(
    sender: Sender<Details>,
    details: Arc<RwLock<Details>>,
    term: Arc<AtomicBool>,
) -> Result<()> {
    let die = async {
        loop {
            task::sleep(Duration::from_millis(50)).await;
            if term.load(Ordering::Relaxed) {
                break;
            }
        }
        Ok(())
    };

    let mut app = tide::Server::with_state(State { details, sender });
    app.at("/").serve_file("./frontend/index.html")?;
    app.at("/details").get(get_details);
    app.at("/details").post(post_details);
    app.at("/pkg").serve_dir("./frontend/pkg/")?;
    app.at("/mdc").serve_dir("./frontend/static/mdc/")?;
    app.at("/style.css")
        .serve_file("./frontend/static/style.css")?;
    app.listen("0.0.0.0:8000").race(die).await?;
    Ok(())
}

fn main() -> Result<()> {
    tide::log::start();

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

    let mut storage = Storage::open("./db/effects.db").map_err(|_| Error::HeedError)?;
    let (sender, receiver) = channel::bounded(1);
    let details = storage
        .load("main")
        .map_err(|_| Error::HeedError)?
        .unwrap_or_default();
    log::info!("Details loaded: {:#?}", details);
    let details = Arc::new(RwLock::new(details));
    let details2 = Arc::clone(&details);
    let term2 = Arc::clone(&term);
    let render = thread::spawn(|| {
        task::block_on(async {
            render_main(receiver, details, term2, storage)
                .await
                .unwrap();
        });
    });
    let task = thread::spawn(move || {
        task::block_on(async { web_main(sender, details2, term).await }).expect("block should work")
    });
    render.join().expect("Rendering stopped");
    task.join().expect("task completed");
    Ok(())
}
