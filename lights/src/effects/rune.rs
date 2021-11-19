use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use once_cell::sync::Lazy;
use rune::{termcolor::StandardStream, Diagnostics, EmitDiagnostics, Options, Sources};
use runestick::{AnyObj, Context, Hash, RuntimeContext, Source, Unit, Value, Vm};

use serde::{Deserialize, Serialize};

use crate::error::RuneError;

static REQUIRED_FNS: Lazy<HashMap<&'static str, Hash>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert("calculate", Hash::type_hash(&["calculate"]));
    map.insert("init", Hash::type_hash(&["init"]));
    map
});

#[derive(Debug, Serialize, Deserialize)]
pub struct RuneScript {
    #[serde(skip)]
    runtime: Arc<RuntimeContext>,

    #[serde(skip)]
    unit: Arc<Unit>,

    #[serde(skip)]
    private_data: Value,

    // TODO: Make this a new type that we control, for loading from the database
    sourcecode: String,
}

impl RuneScript {
    pub fn from_source(sourcecode: String) -> Result<Self, RuneError> {
        // TODO: Figure out which functions and stuff we want to provide to Rune
        let context = Context::with_default_modules()?;
        let options = Options::default();
        let mut sources = Sources::new();
        sources.insert(Source::new("main", &sourcecode));

        let mut diagnostics = Diagnostics::new();
        let result = rune::load_sources(&context, &options, &mut sources, &mut diagnostics);

        if !diagnostics.is_empty() {
            let mut writer = StandardStream::stderr(rune::termcolor::ColorChoice::Always);
            diagnostics.emit_diagnostics(&mut writer, &sources)?;
            // Ideally we won't get here, and I can just not make this more awesome.
            return Err(RuneError::Compilation("Error compiling".into()));
        }

        let unit = result?;

        let function_hashes = unit.iter_functions().map(|t| t.0).collect::<HashSet<_>>();
        let missing_fns = REQUIRED_FNS
            .iter()
            // TODO: Verify the function signatures
            .filter(|(_, h)| !function_hashes.contains(h))
            .map(|t| t.0)
            .collect::<Vec<_>>();
        if !missing_fns.is_empty() {
            log::error!("Don't contain all the functions");
            return Err(RuneError::Compilation(format!(
                "Missing functions: {:?}",
                missing_fns
            )));
        }
        let unit = Arc::new(unit);
        let context = Arc::new(context);
        let runtime = Arc::new(context.runtime());

        let vm = Vm::new(Arc::clone(&runtime), Arc::clone(&unit));
        let private_data = vm.call(&["init"], ())?;
        log::debug!("Loaded private_data: {:?}", private_data);

        Ok(Self {
            unit,
            runtime,

            sourcecode,

            private_data,
        })
    }
}

impl Default for RuneScript {
    fn default() -> Self {
        RuneScript::from_source(
            r#"
            pub fn calculate(a, b) {
                println("Hello World");
                a + b
            }

            pub fn init() {
                println("initialize");
            }
            "#
            .to_owned(),
        )
        .unwrap()
    }
}
