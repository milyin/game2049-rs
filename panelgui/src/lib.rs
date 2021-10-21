mod background;
mod frame;
// mod ribbon;
mod slot;

pub use background::{Background, BackgroundKeeper, BackgroundTag};
pub use frame::{Frame, FrameKeeper, FrameTag};
// pub use ribbon::{Ribbon, RibbonKeeper, RibbonTag};
pub use slot::{Slot, SlotKeeper, SlotTag};

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
