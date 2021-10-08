mod interop;
mod wide_strings;
mod window;
use bindings::Windows::{
    Foundation::Numerics::Vector2,
    Win32::System::WinRT::{RoInitialize, RO_INIT_SINGLETHREADED},
    UI::Composition::Compositor,
};
use interop::create_dispatcher_queue_controller_for_current_thread;
use window::Window;

fn run() -> windows::Result<()> {
    unsafe { RoInitialize(RO_INIT_SINGLETHREADED)? };
    let _controler = create_dispatcher_queue_controller_for_current_thread()?;

    let window_width = 800;
    let window_height = 600;

    let window_size = Vector2 {
        X: window_width as f32,
        Y: window_height as f32,
    };

    let compositor = Compositor::new()?;
    let root = compositor.CreateContainerVisual()?;
    root.SetRelativeSizeAdjustment(Vector2::new(1.0, 1.0))?;

    let window = Window::new("2048-rs", window_width, window_height)?;
    let target = window.create_window_target(&compositor, false)?;
    target.SetRoot(&root)?;

    Ok(())
}

fn main() {
    let result = run();

    // We do this for nicer HRESULT printing when errors occur.
    if let Err(error) = result {
        error.code().unwrap();
    }
}
