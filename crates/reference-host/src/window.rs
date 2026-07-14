use std::cell::RefCell;
use std::mem;
use std::sync::atomic::{AtomicIsize, Ordering};

use anyhow::{Context, Result};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::System::Console::{CTRL_BREAK_EVENT, CTRL_C_EVENT, SetConsoleCtrlHandler};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::{BOOL, HSTRING, PCWSTR};

use crate::input::{NativeMessage, PostedMessage};

static WINDOW_HANDLE: AtomicIsize = AtomicIsize::new(0);

#[derive(Clone, Copy)]
pub struct Config {
    pub class_name: &'static str,
    pub title: &'static str,
    pub width: u32,
    pub height: u32,
}

thread_local! {
    static INPUT_MESSAGES: RefCell<Vec<NativeMessage>> = const { RefCell::new(Vec::new()) };
}

pub fn create(config: Config) -> Result<HWND> {
    let module = unsafe { GetModuleHandleW(None) }.context("GetModuleHandleW failed")?;
    let instance = HINSTANCE(module.0);
    let class_name = HSTRING::from(config.class_name);
    let title = HSTRING::from(config.title);
    let window_class = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_proc),
        hInstance: instance,
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW) }.context("LoadCursorW failed")?,
        lpszClassName: PCWSTR(class_name.as_ptr()),
        ..Default::default()
    };
    if unsafe { RegisterClassW(&window_class) } == 0 {
        return Err(windows::core::Error::from_thread()).context("RegisterClassW failed");
    }

    let style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX;
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: config.width as i32,
        bottom: config.height as i32,
    };
    unsafe { AdjustWindowRect(&mut rect, style, false) }.context("AdjustWindowRect failed")?;
    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(title.as_ptr()),
            style,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            rect.right - rect.left,
            rect.bottom - rect.top,
            None,
            None,
            Some(instance),
            None,
        )
    }
    .context("CreateWindowExW failed")?;
    WINDOW_HANDLE.store(hwnd.0 as isize, Ordering::Release);
    unsafe { SetConsoleCtrlHandler(Some(console_ctrl_handler), true) }
        .context("SetConsoleCtrlHandler failed")?;
    Ok(hwnd)
}

/// Makes the caller-owned reference window visible.
///
/// # Safety
///
/// `hwnd` must identify the live window returned by [`create`] on the calling thread.
pub unsafe fn show(hwnd: HWND) {
    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOW);
    }
}

pub fn pump_messages() -> bool {
    let mut message = MSG::default();
    while unsafe { PeekMessageW(&mut message, None, 0, 0, PM_REMOVE) }.as_bool() {
        if message.message == WM_QUIT {
            return false;
        }
        unsafe {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
    true
}

pub fn request_close(hwnd: HWND) -> Result<()> {
    unsafe { PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0)) }
        .context("posting window close failed")
}

pub fn teardown() -> Result<()> {
    WINDOW_HANDLE.store(0, Ordering::Release);
    unsafe { SetConsoleCtrlHandler(Some(console_ctrl_handler), false) }
        .context("removing console control handler failed")
}

pub fn drain_input() -> Vec<NativeMessage> {
    INPUT_MESSAGES.with(|messages| mem::take(&mut *messages.borrow_mut()))
}

pub fn post_input(hwnd: HWND, messages: &[PostedMessage]) -> Result<()> {
    for message in messages {
        let (message, wparam, lparam) = match *message {
            PostedMessage::Key { key, down, system } => {
                let message = match (system, down) {
                    (false, true) => WM_KEYDOWN,
                    (false, false) => WM_KEYUP,
                    (true, true) => WM_SYSKEYDOWN,
                    (true, false) => WM_SYSKEYUP,
                };
                let mut bits = 1_isize;
                if system {
                    bits |= 1_isize << 29;
                }
                if !down {
                    bits |= (1_isize << 30) | (1_isize << 31);
                }
                (message, WPARAM(usize::from(key)), LPARAM(bits))
            }
            PostedMessage::FocusLost => (WM_KILLFOCUS, WPARAM(0), LPARAM(0)),
        };
        unsafe { PostMessageW(Some(hwnd), message, wparam, lparam) }
            .with_context(|| format!("posting native input message 0x{message:04X} failed"))?;
    }
    Ok(())
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_KEYDOWN | WM_KEYUP => {
            capture_key(wparam, message == WM_KEYDOWN);
            LRESULT(0)
        }
        WM_SYSKEYDOWN | WM_SYSKEYUP => {
            capture_key(wparam, message == WM_SYSKEYDOWN);
            unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
        }
        WM_KILLFOCUS => {
            INPUT_MESSAGES.with(|messages| messages.borrow_mut().push(NativeMessage::FocusLost));
            unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
        }
        WM_CLOSE => {
            let _ = unsafe { DestroyWindow(hwnd) };
            LRESULT(0)
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

fn capture_key(wparam: WPARAM, down: bool) {
    INPUT_MESSAGES.with(|messages| {
        messages.borrow_mut().push(NativeMessage::Key {
            key: wparam.0,
            down,
        });
    });
}

unsafe extern "system" fn console_ctrl_handler(control: u32) -> BOOL {
    if !matches!(control, CTRL_C_EVENT | CTRL_BREAK_EVENT) {
        return false.into();
    }
    let raw = WINDOW_HANDLE.load(Ordering::Acquire);
    if raw == 0 {
        return false.into();
    }
    let hwnd = HWND(raw as *mut _);
    unsafe { PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0)) }
        .is_ok()
        .into()
}
