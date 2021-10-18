// mod background;
// mod cell;
mod frame;
// mod ribbon;
mod slot;

use async_object::EventStream;
use bindings::Windows::{Foundation::Numerics::Vector2, UI::Composition::Visual};
// pub use cell::{Cell, CellKeeper, CellTag};
pub use frame::{Frame, FrameKeeper, FrameTag};
use futures::task::SpawnError;
// pub use ribbon::{Ribbon, RibbonKeeper, RibbonTag};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Bad element index")]
    BadIndex,
    #[error(transparent)]
    Spawn(SpawnError),
    #[error(transparent)]
    AsyncObject(async_object::Error),
    #[error(transparent)]
    Windows(windows::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<windows::Error> for Error {
    fn from(e: windows::Error) -> Self {
        Error::Windows(e)
    }
}

impl From<async_object::Error> for Error {
    fn from(e: async_object::Error) -> Self {
        Error::AsyncObject(e)
    }
}
impl From<SpawnError> for Error {
    fn from(e: SpawnError) -> Self {
        Error::Spawn(e)
    }
}
