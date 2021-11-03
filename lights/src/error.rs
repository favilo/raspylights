pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Bad effect type")]
    BadEffectType,

    #[error("Index out of range")]
    IndexOutOfRange,
}
