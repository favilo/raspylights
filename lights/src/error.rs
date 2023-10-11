pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Bad effect type")]
    BadEffectType,

    #[error("Index out of range")]
    IndexOutOfRange,

    #[error("Heed Error")]
    HeedError,

    #[error(transparent)]
    RuneError(#[from] RuneError),

    #[error("Scripts not allowed in composite effects")]
    CompositeScriptError,
}

#[derive(Debug, thiserror::Error)]
pub enum RuneError {
    #[error("Compilation Error: {0}")]
    Compilation(String),

    #[error("No debug information")]
    NoDebugInfo,

    #[error("Rune Error")]
    Rune(#[from] rune::alloc::Error),

    #[error("Build Error: {0}")]
    Build(#[from] rune::BuildError),

    #[error("Context Error")]
    Context(#[from] rune::ContextError),

    #[error("Diagnostic Error: {0:?}")]
    Diagnostic(#[from] rune::diagnostics::FatalDiagnostic),

    #[error("Emit Error")]
    LoadSources(#[from] rune::diagnostics::EmitError),

    #[error("Runtime Error")]
    VmError(#[from] rune::runtime::VmError),
}
