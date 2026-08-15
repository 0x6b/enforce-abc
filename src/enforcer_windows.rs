use std::{
    ffi::c_void,
    io,
    io::Error,
    mem::{size_of, zeroed},
    ptr,
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

use anyhow::{Context, Result, anyhow, bail};
use log::{debug, error, info};
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::{
        Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent},
        Input::KeyboardAndMouse::{GetKeyboardLayout, HKL, KLF_SUBSTITUTE_OK, LoadKeyboardLayoutW},
        WindowsAndMessaging,
        WindowsAndMessaging::{
            DispatchMessageW, EVENT_SYSTEM_FOREGROUND, GUITHREADINFO, GetGUIThreadInfo,
            GetMessageW, GetWindowThreadProcessId, MSG, PostMessageW, TranslateMessage,
            WINEVENT_OUTOFCONTEXT, WM_INPUTLANGCHANGEREQUEST,
        },
    },
};

const US_ENGLISH_LANGID: usize = 0x0409;
const US_ENGLISH_KLID: &[u16] = &[
    b'0' as u16,
    b'0' as u16,
    b'0' as u16,
    b'0' as u16,
    b'0' as u16,
    b'4' as u16,
    b'0' as u16,
    b'9' as u16,
    0,
];

static ENGLISH_LAYOUT: AtomicPtr<c_void> = AtomicPtr::new(null_mut());

pub fn run() -> Result<()> {
    let layout = unsafe { LoadKeyboardLayoutW(US_ENGLISH_KLID.as_ptr(), KLF_SUBSTITUTE_OK) };
    if layout.is_null() {
        return Err(Error::last_os_error())
            .context("failed to load the English (United States) keyboard layout");
    }
    if layout as usize & 0xffff != US_ENGLISH_LANGID {
        bail!("English (United States) is not installed as a keyboard layout");
    }
    ENGLISH_LAYOUT.store(layout, Ordering::Relaxed);

    enforce_for_foreground_window();

    let hook = unsafe {
        SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            null_mut(),
            Some(on_foreground_changed),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        )
    };
    if hook.is_null() {
        return Err(Error::last_os_error()).context("failed to observe foreground windows");
    }
    let _hook = WinEventHook(hook);

    info!("Listening for app activation events. Forcing English (United States).");
    message_loop()
}

fn message_loop() -> Result<()> {
    let mut message: MSG = unsafe { zeroed() };
    loop {
        match unsafe { GetMessageW(&mut message, null_mut(), 0, 0) } {
            -1 => return Err(Error::last_os_error()).context("Windows message loop failed"),
            0 => return Ok(()),
            _ => unsafe {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            },
        };
    }
}

unsafe extern "system" fn on_foreground_changed(
    _hook: HWINEVENTHOOK,
    _event: u32,
    window: HWND,
    _object_id: i32,
    _child_id: i32,
    _event_thread: u32,
    _event_time: u32,
) {
    if let Err(why) = enforce(window) {
        error!("Failed to switch foreground app to English (United States): {why}");
    }
}

fn enforce_for_foreground_window() {
    let window = unsafe { WindowsAndMessaging::GetForegroundWindow() };
    if window.is_null() {
        debug!("No foreground window at startup");
    } else if let Err(why) = enforce(window) {
        error!("Failed to switch foreground app to English (United States): {why}");
    }
}

fn enforce(window: HWND) -> Result<()> {
    if window.is_null() {
        bail!("foreground event did not contain a window");
    }

    let thread = unsafe { GetWindowThreadProcessId(window, null_mut()) };
    if thread == 0 {
        return Err(Error::last_os_error()).context("failed to identify foreground thread");
    }

    let current = unsafe { GetKeyboardLayout(thread) };
    let english: HKL = ENGLISH_LAYOUT.load(Ordering::Relaxed);
    if current == english {
        debug!("Foreground thread {thread} already uses English (United States), no-op");
        return Ok(());
    }

    let target = focused_window(thread).unwrap_or(window);
    let posted =
        unsafe { PostMessageW(target, WM_INPUTLANGCHANGEREQUEST, 0 as WPARAM, english as LPARAM) };
    if posted == 0 {
        return Err(anyhow!(io::Error::last_os_error()))
            .context("failed to request an input language change");
    }

    debug!("Requested English (United States) for foreground thread {thread}");
    Ok(())
}

fn focused_window(thread: u32) -> Option<HWND> {
    let mut info: GUITHREADINFO = unsafe { zeroed() };
    info.cbSize = size_of::<GUITHREADINFO>() as u32;
    let found = unsafe { GetGUIThreadInfo(thread, &mut info) };
    (found != 0)
        .then_some(info.hwndFocus)
        .filter(|window| !window.is_null())
}

struct WinEventHook(HWINEVENTHOOK);

impl Drop for WinEventHook {
    fn drop(&mut self) {
        unsafe { UnhookWinEvent(self.0) };
    }
}
