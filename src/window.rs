use std::sync::Once;

use bindings::Windows::{
    Foundation::Numerics::Vector2,
    Graphics::SizeInt32,
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, PWSTR, RECT, WPARAM},
        System::{LibraryLoader::GetModuleHandleW, WinRT::ICompositorDesktopInterop},
        UI::WindowsAndMessaging::{
            AdjustWindowRectEx, CreateWindowExW, DefWindowProcW, GetClientRect, LoadCursorW,
            PostQuitMessage, RegisterClassW, ShowWindow, CREATESTRUCTW, CW_USEDEFAULT,
            GWLP_USERDATA, HMENU, IDC_ARROW, SW_SHOW, WINDOW_LONG_PTR_INDEX, WM_DESTROY,
            WM_LBUTTONDOWN, WM_MOUSEMOVE, WM_NCCREATE, WM_RBUTTONDOWN, WM_SIZE, WM_SIZING,
            WM_TIMER, WNDCLASSW, WS_EX_NOREDIRECTIONBITMAP, WS_OVERLAPPEDWINDOW,
        },
    },
    UI::Composition::{Compositor, Desktop::DesktopWindowTarget},
};

use panelgui::{MouseLeftPressed, MouseLeftPressedFocused, SendSlotEvent, SlotSize};
use windows::{Handle, Interface};

use crate::wide_strings::ToWide;

static REGISTER_WINDOW_CLASS: Once = Once::new();
static WINDOW_CLASS_NAME: &str = "game2049-rs.Window";

pub struct Window {
    handle: HWND,
    event_dst: Box<dyn SendSlotEvent>,
    // pool: LocalPool,
    mouse_pos: Vector2,
}

impl Window {
    pub fn new(
        title: &str,
        width: u32,
        height: u32,
        // pool: LocalPool,
        event_dst: impl SendSlotEvent + 'static,
    ) -> windows::Result<Box<Self>> {
        let class_name = WINDOW_CLASS_NAME.to_wide();
        let instance = unsafe { GetModuleHandleW(PWSTR(std::ptr::null_mut())).ok()? };
        REGISTER_WINDOW_CLASS.call_once(|| {
            let class = WNDCLASSW {
                hCursor: unsafe { LoadCursorW(HINSTANCE(0), IDC_ARROW).ok().unwrap() },
                hInstance: instance,
                lpszClassName: class_name.as_pwstr(),
                lpfnWndProc: Some(Self::wnd_proc),
                ..Default::default()
            };
            assert_ne!(unsafe { RegisterClassW(&class) }, 0);
        });

        let width = width as i32;
        let height = height as i32;
        let window_ex_style = WS_EX_NOREDIRECTIONBITMAP;
        let window_style = WS_OVERLAPPEDWINDOW;

        let (adjusted_width, adjusted_height) = {
            let mut rect = RECT {
                left: 0,
                top: 0,
                right: width as i32,
                bottom: height as i32,
            };
            unsafe {
                AdjustWindowRectEx(&mut rect, window_style, false, window_ex_style).ok()?;
            }
            (rect.right - rect.left, rect.bottom - rect.top)
        };
        let event_dst = Box::new(event_dst);
        let mouse_pos = Vector2::default();
        let mut result = Box::new(Self {
            handle: HWND(0),
            event_dst,
            // pool,
            mouse_pos,
        });

        let title = title.to_wide();
        let window = unsafe {
            CreateWindowExW(
                window_ex_style,
                class_name.as_pwstr(),
                title.as_pwstr(),
                window_style,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                adjusted_width,
                adjusted_height,
                HWND(0),
                HMENU(0),
                instance,
                result.as_mut() as *mut _ as _,
            )
            .ok()?
        };
        unsafe { ShowWindow(&window, SW_SHOW) };

        Ok(result)
    }

    pub fn size(&self) -> windows::Result<SizeInt32> {
        get_window_size(self.handle)
    }

    pub fn handle(&self) -> HWND {
        self.handle
    }

    pub fn create_window_target(
        &self,
        compositor: &Compositor,
        is_topmost: bool,
    ) -> windows::Result<DesktopWindowTarget> {
        let compositor_desktop: ICompositorDesktopInterop = compositor.cast()?;
        unsafe { compositor_desktop.CreateDesktopWindowTarget(self.handle(), is_topmost) }
    }

    fn message_handler(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match message {
            WM_DESTROY => {
                unsafe { PostQuitMessage(0) };
                return LRESULT(0);
            }
            WM_MOUSEMOVE => {
                let (x, y) = get_mouse_position(lparam);
                let point = Vector2 {
                    X: x as f32,
                    Y: y as f32,
                };
                self.mouse_pos = point;
                // self.game.on_pointer_moved(&point).unwrap();
            }
            WM_SIZE | WM_SIZING => {
                let new_size = self.size().unwrap();
                let new_size = Vector2 {
                    X: new_size.Width as f32,
                    Y: new_size.Height as f32,
                };
                self.event_dst.send_size(SlotSize(new_size)).unwrap();
            }
            WM_LBUTTONDOWN => {
                self.event_dst
                    .send_mouse_left_pressed(MouseLeftPressed(self.mouse_pos))
                    .unwrap();
                self.event_dst
                    .send_mouse_left_pressed_focused(MouseLeftPressedFocused(self.mouse_pos))
                    .unwrap();
            }
            WM_RBUTTONDOWN => {
                // self.game.on_pointer_pressed(true, false).unwrap();
            }
            WM_TIMER => {
                // dbg!("timer");
            }
            _ => {}
        }
        // self.pool.run_until_stalled();
        unsafe { DefWindowProcW(self.handle, message, wparam, lparam) }
    }

    unsafe extern "system" fn wnd_proc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if message == WM_NCCREATE {
            let cs = lparam.0 as *const CREATESTRUCTW;
            let this = (*cs).lpCreateParams as *mut Self;
            (*this).handle = window;

            SetWindowLong(window, GWLP_USERDATA, this as _);
        } else {
            let this = GetWindowLong(window, GWLP_USERDATA) as *mut Self;

            if let Some(this) = this.as_mut() {
                return this.message_handler(message, wparam, lparam);
            }
        }
        DefWindowProcW(window, message, wparam, lparam)
    }
}

fn get_window_size(window_handle: HWND) -> windows::Result<SizeInt32> {
    unsafe {
        let mut rect = RECT::default();
        let _ = GetClientRect(window_handle, &mut rect).ok()?;
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        Ok(SizeInt32 {
            Width: width,
            Height: height,
        })
    }
}

fn get_mouse_position(lparam: LPARAM) -> (isize, isize) {
    let x = lparam.0 & 0xffff;
    let y = (lparam.0 >> 16) & 0xffff;
    (x, y)
}

#[allow(non_snake_case)]
#[cfg(target_pointer_width = "32")]
unsafe fn SetWindowLong(window: HWND, index: WINDOW_LONG_PTR_INDEX, value: isize) -> isize {
    use bindings::Windows::Win32::UI::WindowsAndMessaging::SetWindowLongW;

    SetWindowLongW(window, index, value as _) as _
}

#[allow(non_snake_case)]
#[cfg(target_pointer_width = "64")]
unsafe fn SetWindowLong(window: HWND, index: WINDOW_LONG_PTR_INDEX, value: isize) -> isize {
    use bindings::Windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW;

    SetWindowLongPtrW(window, index, value)
}

#[allow(non_snake_case)]
#[cfg(target_pointer_width = "32")]
unsafe fn GetWindowLong(window: HWND, index: WINDOW_LONG_PTR_INDEX) -> isize {
    use bindings::Windows::Win32::UI::WindowsAndMessaging::SetWindowLongW;

    GetWindowLongW(window, index) as _
}

#[allow(non_snake_case)]
#[cfg(target_pointer_width = "64")]
unsafe fn GetWindowLong(window: HWND, index: WINDOW_LONG_PTR_INDEX) -> isize {
    use bindings::Windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW;

    GetWindowLongPtrW(window, index)
}
