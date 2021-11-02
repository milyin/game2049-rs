#![windows_subsystem = "windows"]

// use std::time::Duration;

mod interop;
mod wide_strings;
mod window;
use bindings::Windows::UI::Color;
use bindings::Windows::{
    Foundation::Numerics::Vector2,
    Win32::{
        Foundation::HWND,
        System::WinRT::{RoInitialize, RO_INIT_SINGLETHREADED},
        UI::WindowsAndMessaging::{DispatchMessageW, GetMessageW, SetTimer, TranslateMessage, MSG},
    },
    UI::Colors,
};
use futures::executor::LocalPool;
use interop::create_dispatcher_queue_controller_for_current_thread;
use panelgui::{BackgroundKeeper, CellLimit, FrameKeeper, RibbonKeeper, RibbonOrientation};
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
    frame.frame_visual()?.SetSize(window_size)?;

    let slot = frame.open_slot()?;
    // let background_keeper = BackgroundKeeper::new(&frame, slot.clone(), Colors::White()?, true)?;
    let ribbon_keeper = RibbonKeeper::new(frame.clone(), slot, RibbonOrientation::Horizontal)?;
    let ribbon = ribbon_keeper.tag();
    let left = ribbon.add_cell(CellLimit::default())?;
    let center = ribbon.add_cell(CellLimit::new(2.0, Vector2 { X: 1.0, Y: 1.0 }, 300., None))?;
    let right = ribbon.add_cell(CellLimit::default())?;
    let _left_bkg_keeper = BackgroundKeeper::new(frame.clone(), left, Colors::Red()?, true)?;
    let _center_bkg_keeper = BackgroundKeeper::new(frame.clone(), center, Colors::Green()?, true)?;
    let _right_bkg_keeper = BackgroundKeeper::new(frame.clone(), right, Colors::Blue()?, true)?;

    // frame.spawn_local({
    //     let frame_tag = frame.clone();
    //     async move {
    //         let slot = frame_tag.open_modal_slot()?;
    //         task::sleep(Duration::from_secs(5)).await;
    //         dbg!("show");
    //         let background_keeper =
    //             BackgroundKeeper::new(&frame_tag, slot.clone(), Colors::Red()?, true)?;
    //         dbg!("red");
    //         let background = background_keeper.tag();
    //         task::sleep(Duration::from_secs(5)).await;
    //         background.set_color(Colors::Blue()?)?;
    //         dbg!("blue");
    //         // frame_tag.close_slot(slot)?;
    //         slot.join().await
    //         // Ok(())
    //     }
    // })?;

    let window = Window::new("2049-rs", window_width, window_height, pool, frame.clone())?;
    let target = window.create_window_target(&frame.compositor()?, false)?;
    target.SetRoot(frame.frame_visual()?)?;

    let mut message = MSG::default();
    unsafe {
        const IDT_TIMER1: usize = 1;
        SetTimer(window.handle(), IDT_TIMER1, 10, None);
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
