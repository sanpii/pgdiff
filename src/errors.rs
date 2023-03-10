pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Database(#[from] elephantry::Error),
    #[error("{0}")]
    Fmt(#[from] std::fmt::Error),
}
