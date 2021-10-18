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
    UI::Composition::Compositor,
};
use futures::executor::LocalPool;
use interop::create_dispatcher_queue_controller_for_current_thread;
use panelgui::FrameKeeper;
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

    let mut pool = LocalPool::new();

    let frame = FrameKeeper::new(pool.spawner())?;
    let frame_tag = frame.tag();

    frame.get_mut().set_size(window_size)?;

    let window = Window::new("2049-rs", window_width, window_height, frame_tag)?;
    let target = window.create_window_target(frame.compositor(), false)?;
    target.SetRoot(frame.root_visual())?;

    let mut message = MSG::default();
    unsafe {
        while GetMessageW(&mut message, HWND(0), 0, 0).into() {
            TranslateMessage(&mut message);
            DispatchMessageW(&mut message);
            pool.run_until_stalled();
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
