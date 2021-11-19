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
}

#[derive(Debug, thiserror::Error)]
pub enum RuneError {
    #[error("Compilation Error: {0}")]
    Compilation(String),

    #[error("Rune Error")]
    Rune(#[from] rune::Error),

    #[error("Context Error")]
    Context(#[from] runestick::ContextError),

    #[error("Diagnostic Error")]
    Diagnostic(#[from] rune::DiagnosticsError),

    #[error("Load Sources Error")]
    LoadSources(#[from] rune::LoadSourcesError),

    #[error("Runtime Error")]
    VmError(#[from] runestick::VmError),
}
