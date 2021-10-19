#![windows_subsystem = "windows"]

mod interop;
mod wide_strings;
mod window;
use bindings::Windows::{
    Foundation::Numerics::Vector2,
    Win32::{
        Foundation::HWND,
        System::WinRT::{RoInitialize, RO_INIT_SINGLETHREADED},
        UI::WindowsAndMessaging::{DispatchMessageW, GetMessageW, TranslateMessage, MSG},
    },
    UI::Colors,
};
use futures::executor::LocalPool;
use interop::create_dispatcher_queue_controller_for_current_thread;
use panelgui::{BackgroundKeeper, FrameKeeper};
use window::Window;

fn run() -> panelgui::Result<()> {
    unsafe { RoInitialize(RO_INIT_SINGLETHREADED)? };
    let _controler = create_dispatcher_queue_controller_for_current_thread()?;

    let window_width = 800;
    let window_height = 600;

    let window_size = Vector2 {
        X: window_width as f32,
        Y: window_height as f32,
    };

    let pool = LocalPool::new();

    let frame_keeper = FrameKeeper::new(pool.spawner())?;
    let frame = frame_keeper.tag();

    frame.set_size(window_size)?;
    let slot = frame.open_modal_slot()?;
    let _background = BackgroundKeeper::new(&frame, slot, Colors::White()?, false)?;

    let window = Window::new("2049-rs", window_width, window_height, pool, frame)?;
    let target = window.create_window_target(frame_keeper.compositor(), false)?;
    target.SetRoot(frame_keeper.root_visual())?;

    let mut message = MSG::default();
    unsafe {
        while GetMessageW(&mut message, HWND(0), 0, 0).into() {
            TranslateMessage(&mut message);
            DispatchMessageW(&mut message);
        }
    }

    Ok(())
}

fn main() {
    let result = run();

    // We do this for nicer HRESULT printing when errors occur.
    if let Err(error) = result {
        match error {
            // TODO - trace stack with error-chain
            panelgui::Error::Windows(error) => error.code().unwrap(),
            panelgui::Error::AsyncObject(error) => eprintln!("{}", error),
            error => eprintln!("{}", error),
        }
    }
}
