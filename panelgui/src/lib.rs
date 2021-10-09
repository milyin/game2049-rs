mod window;

pub use window::{Window, WindowKeeper, WindowTag};

pub enum Error {
    AsyncObjectDestroyed,
    Windows(windows::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<windows::Error> for Error {
    fn from(e: windows::Error) -> Self {
        Error::Windows(e)
    }
}
