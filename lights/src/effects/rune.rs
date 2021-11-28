mod types;

use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    sync::Arc,
};

use chrono::TimeZone;
use once_cell::sync::Lazy;
use rune::{termcolor::StandardStream, Diagnostics, EmitDiagnostics, Options, Sources};
use runestick::{
    debug::DebugArgs, Any, Component, Context, RuntimeContext, Source, Unit, Value, Vm,
};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result, RuneError};

use super::{Effect, EffectType, Instant};

static REQUIRED_FNS: Lazy<HashMap<Component, DebugArgs>> = Lazy::new(|| {
    [
        ("init", DebugArgs::Named(vec![])),
        (
            "render",
            DebugArgs::Named(vec![
                "state".to_string(),
                "pixels".to_string(),
                "t".to_string(),
            ]),
        ),
        (
            "is_ready",
            DebugArgs::Named(vec!["state".to_string(), "t".to_string()]),
        ),
    ]
    .into_iter()
    .map(|(n, a)| (n.to_owned(), a))
    .map(|(n, a)| (n.into_boxed_str(), a))
    .map(|b| (Component::Str(b.0), b.1))
    .collect()
});

// TODO: We want a single Context/RuntimeContext that is cloned between all scripts
// TODO: We need a budget to run so we don't overflow the time waiting for the next tick

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RuneScript {
    #[serde(skip)]
    runtime: Arc<RuntimeContext>,

    #[serde(skip)]
    unit: Arc<Unit>,

    #[serde(skip)]
    private_data: Value,

    // TODO: Make this a new type that we control, for loading from the database
    pub(crate) sourcecode: SourceCode,
}

impl RuneScript {
    pub fn from_source(sourcecode: SourceCode) -> Result<Self, RuneError> {
        // TODO: Figure out which functions and stuff we want to provide to Rune
        let mut context = Context::with_default_modules()?;
        context.install(&types::module()?)?;

        let options = Options::default();
        let mut sources = Sources::new();
        sources.insert(Source::new("main", &sourcecode.to_string()));

        let mut diagnostics = Diagnostics::new();
        let result = rune::load_sources(&context, &options, &mut sources, &mut diagnostics);

        if !diagnostics.is_empty() {
            let mut writer = StandardStream::stderr(rune::termcolor::ColorChoice::Always);
            diagnostics.emit_diagnostics(&mut writer, &sources)?;
            // Ideally we won't get here, and I can just not make this more awesome.
            return Err(RuneError::Compilation("Error compiling".into()).into());
        }

        let unit = result?;
        Self::validate_source(&unit)?;

        let unit = Arc::new(unit);
        let context = Arc::new(context);
        let runtime = Arc::new(context.runtime());

        let vm = Vm::new(Arc::clone(&runtime), Arc::clone(&unit));
        let private_data = vm.call(&["init"], ())?;
        log::info!("Loaded private_data: {:?}", private_data);
        log::info!("private_data info: {:#?}", private_data.type_info()?);
        if let Value::Object(ref o) = private_data {
            log::info!("private_data info: {:#?}", o);
        }
        let private_data = private_data;

        Ok(Self {
            unit,
            runtime,
            private_data,

            sourcecode,
        })
    }

    fn validate_source(unit: &Unit) -> Result<(), RuneError> {
        let debug_info = unit.debug_info().ok_or(RuneError::NoDebugInfo)?;
        let function_signatures = debug_info
            .functions
            .values()
            .map(|t| {
                log::info!("t: {:?}", t.path.as_vec());
                t
            })
            .collect::<Vec<_>>();
        let found_req_fns = function_signatures
            .iter()
            .map(|&di| di.path.as_vec().last().unwrap().to_owned())
            .filter(|component: &Component| REQUIRED_FNS.contains_key(&component))
            .collect::<HashSet<Component>>();
        let reqd = REQUIRED_FNS
            .keys()
            .into_iter()
            .cloned()
            .collect::<HashSet<_>>();
        let missing_fns = reqd.difference(&found_req_fns).collect::<Vec<_>>();
        if !missing_fns.is_empty() {
            log::error!("Don't contain all the functions");
            return Err(RuneError::Compilation(format!(
                "Missing functions: {:?}",
                missing_fns
            )));
        }
        let broken_fns = debug_info
            .functions
            .values()
            .filter_map(|di| {
                REQUIRED_FNS
                    .get(di.path.as_vec().last().unwrap())
                    .map(|v| (v, di))
            })
            .filter(|(v, di)| format!("{:?}", v) != format!("{:?}", di.args))
            .collect::<Vec<_>>();
        log::info!("broken: {:#?}", broken_fns);
        Ok(())
    }
}

impl Default for RuneScript {
    fn default() -> Self {
        RuneScript::from_source(SourceCode::default()).unwrap()
    }
}

impl PartialEq for RuneScript {
    fn eq(&self, other: &Self) -> bool {
        // We really only care about sourcecode, the other values can change between runs.
        self.sourcecode == other.sourcecode
    }
}

impl Effect for RuneScript {
    fn render(
        &mut self,
        pixels: &mut [palette::LinSrgb<u8>],
        t: super::Instant,
    ) -> crate::error::Result<chrono::Duration> {
        let vm = Vm::new(Arc::clone(&self.runtime), Arc::clone(&self.unit));
        let state = &self.private_data;
        let mut scrixels: types::Scrixels = pixels.into();

        let dur = vm
            .call(&["render"], (state, &mut scrixels, t.timestamp_millis()))
            .map_err(RuneError::from)
            .map_err(Error::from)?
            .into_integer()
            .map_err(RuneError::from)
            .map_err(Error::from)?;
        for (i, pixel) in pixels.iter_mut().enumerate() {
            *pixel = scrixels.0[i].into();
        }
        Ok(chrono::Duration::milliseconds(dur))
    }

    fn is_ready(&self, t: Instant) -> Result<bool> {
        let vm = Vm::new(Arc::clone(&self.runtime), Arc::clone(&self.unit));
        let state = &self.private_data;
        let ready = vm
            .call(&["is_ready"], (state, t.timestamp_millis()))
            .map_err(RuneError::from)
            .map_err(Error::from)?
            .as_bool()
            .map_err(RuneError::from)
            .map_err(Error::from)?;
        Ok(ready)
    }

    fn to_cloned_type(&self) -> EffectType {
        EffectType::RuneScript(self.sourcecode.clone())
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum SourceCode {
    Source(String),
}

impl Display for SourceCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceCode::Source(code) => write!(f, "{}", code),
        }
    }
}

impl Default for SourceCode {
    fn default() -> Self {
        Self::Source(
            r#"
            struct Nothing;

            pub fn render(state, pixels, t) {
                println(`Running ${pixels.len()}`);
                //  Run again in 1 second
                1000
            }

            pub fn is_ready(state, t) {
                true
            }

            pub fn init() {
                Nothing
            }
            "#
            .to_owned(),
        )
    }
}
