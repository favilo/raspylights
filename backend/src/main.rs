#![feature(negative_impls)]

mod storage;
mod strip;

use std::{
    fs::File,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

use anyhow::Result;
use async_std::{
    channel::{self, Receiver, Sender},
    prelude::FutureExt,
    sync::{Mutex, RwLock},
    task,
};
use chrono::Utc;
#[cfg(target_arch = "arm")]
use daemonize::Daemonize;
use lights::{details::Details, effects::RuneScript, error::Error};
use serde_json::Value;
use signal_hook::consts;
use tide::{http::mime, prelude::*, Request, Response};

use crate::{storage::Storage, strip::LedStrip};

async fn render_main(
    receiver: Receiver<Details>,
    details: Arc<RwLock<Details>>,
    power: Arc<AtomicBool>,
    term: Arc<AtomicBool>,
    storage: Storage,
) -> Result<()> {
    let storage = Arc::new(Mutex::new(storage));
    let mut strip = LedStrip::new(details.read().await.clone())?;
    strip.set_effect(details.read().await.effect.clone())?;

    let script = RuneScript::default();
    // log::info!("Script: {:#?}", script);

    loop {
        strip.clear()?;
        let start = Utc::now();

        if term.load(Ordering::Relaxed) {
            break;
        }

        if !power.load(Ordering::Relaxed) {
            // Power is off, lets render black every 100 ms
            task::sleep(std::time::Duration::from_millis(100)).await;
            strip.render()?;
            continue;
        }

        if let Ok(deets) = receiver.try_recv() {
            log::info!("We got some deets: {:#?}", deets);
            strip.set_effect(deets.effect.clone())?;
            strip.set_length(deets.length)?;
            strip.set_brightness(deets.brightness)?;
            *details.write().await = deets.clone();

            // TODO: Make this function async, so we can await in a different thread
            storage
                .lock()
                .await
                .store("main", deets)
                .await
                .map_err(|_| Error::HeedError)?;
        }
        let d = strip.update(start)?;
        strip.render()?;

        task::sleep(std::time::Duration::from_millis(
            (d - start.signed_duration_since(Utc::now()))
                .num_milliseconds()
                .try_into()
                .unwrap(),
        ))
        .await;
    }

    strip.render()?;

    Ok(())
}

#[derive(Clone)]
struct State {
    details: Arc<RwLock<Details>>,
    sender: Sender<Details>,
    power: Arc<AtomicBool>,
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

async fn get_power(req: Request<State>) -> tide::Result {
    let resp = Response::builder(200)
        .body(json!({"active": req.state().power.load(Ordering::Relaxed)}))
        .content_type(mime::JSON)
        .build();
    Ok(resp.into())
}

async fn post_power(mut req: Request<State>) -> tide::Result {
    let json: Value = req.body_json().await?;
    let power: bool = json.get("active").and_then(Value::as_bool).unwrap_or(false);
    req.state().power.store(power, Ordering::Relaxed);
    let resp = Response::builder(200)
        .body(json!({ "active": power }))
        .content_type(mime::JSON)
        .build();
    Ok(resp.into())
}

async fn web_main(
    sender: Sender<Details>,
    details: Arc<RwLock<Details>>,
    term: Arc<AtomicBool>,
    power: Arc<AtomicBool>,
) -> Result<()> {
    let die = async {
        loop {
            task::sleep(std::time::Duration::from_millis(50)).await;
            if term.load(Ordering::Relaxed) {
                break;
            }
        }
        Ok(())
    };

    let mut app = tide::Server::with_state(State {
        details,
        sender,
        power,
    });
    app.at("/").serve_file("./frontend/index.html")?;
    app.at("/details").get(get_details);
    app.at("/details").post(post_details);
    app.at("/power").get(get_power);
    app.at("/power").post(post_power);
    app.at("/pkg").serve_dir("./frontend/pkg/")?;
    app.at("/mdc").serve_dir("./frontend/static/mdc/")?;
    app.at("/style.css")
        .serve_file("./frontend/static/style.css")?;
    app.listen("0.0.0.0:8000").race(die).await?;
    Ok(())
}

fn main() -> Result<()> {
    tide::log::start();

    #[cfg(target_arch = "arm")]
    {
        let stdout = File::create("/home/pi/raspylights.out").unwrap();
        let stderr = File::create("/home/pi/raspylights.err").unwrap();

        let daemonize = Daemonize::new()
            .pid_file("/home/pi/raspylights.pid")
            .working_directory("/home/pi")
            .stdout(stdout)
            .stderr(stderr);

        // If we are running on arm, we want to be a daemon.
        daemonize.start()?;
    }

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
        .expect("load failed")
        .unwrap_or_default();
    log::info!("Details loaded: {:#?}", details);
    let details = Arc::new(RwLock::new(details));
    let power = Arc::new(AtomicBool::new(true));
    let power2 = Arc::clone(&power);
    let details2 = Arc::clone(&details);
    let term2 = Arc::clone(&term);
    let render = thread::spawn(move || {
        task::block_on(async {
            render_main(receiver, details, power2, term2, storage)
                .await
                .unwrap();
        });
    });
    let task = thread::spawn(move || {
        task::block_on(async { web_main(sender, details2, term, power).await })
            .expect("block should work")
    });
    render.join().expect("Rendering stopped");
    task.join().expect("task completed");
    Ok(())
}
