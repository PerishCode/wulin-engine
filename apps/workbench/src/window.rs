use std::sync::atomic::{AtomicIsize, Ordering};

use anyhow::{Context, Result};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::System::Console::{CTRL_BREAK_EVENT, CTRL_C_EVENT, SetConsoleCtrlHandler};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::{BOOL, w};

pub const WIDTH: u32 = 1280;
pub const HEIGHT: u32 = 720;
static WINDOW_HANDLE: AtomicIsize = AtomicIsize::new(0);

pub unsafe fn create() -> Result<HWND> {
    let module = unsafe { GetModuleHandleW(None) }.context("GetModuleHandleW failed")?;
    let instance = HINSTANCE(module.0);
    let class_name = w!("WulinEngineWorkbenchWindow");
    let window_class = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_proc),
        hInstance: instance,
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW) }.context("LoadCursorW failed")?,
        lpszClassName: class_name,
        ..Default::default()
    };
    if unsafe { RegisterClassW(&window_class) } == 0 {
        return Err(windows::core::Error::from_thread()).context("RegisterClassW failed");
    }

    let style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX;
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: WIDTH as i32,
        bottom: HEIGHT as i32,
    };
    unsafe { AdjustWindowRect(&mut rect, style, false) }.context("AdjustWindowRect failed")?;
    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            w!("Wulin Engine Workbench"),
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

pub unsafe fn show(hwnd: HWND) {
    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOW);
    }
}

pub unsafe fn teardown() -> Result<()> {
    WINDOW_HANDLE.store(0, Ordering::Release);
    unsafe { SetConsoleCtrlHandler(Some(console_ctrl_handler), false) }
        .context("removing console control handler failed")
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
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
