mod background;
mod window;

use bindings::Windows::Foundation::Numerics::Vector2;
pub use window::{Window, WindowKeeper, WindowTag};

pub enum Error {
    AsyncObject(async_object::Error),
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

pub trait Panel {
    fn set_position(position: Vector2) -> crate::Result<()>;
    fn set_size(size: Vector2) -> crate::Result<()>;
}
