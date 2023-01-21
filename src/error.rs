use thiserror::Error;

pub(crate) type LibResult<T> = Result<T>;
pub(crate) type LibError = Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {}
