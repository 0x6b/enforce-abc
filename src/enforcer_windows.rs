use std::{
    io::Error,
    mem::{size_of, zeroed},
    ptr::null_mut,
};

use anyhow::{Context, Result, bail};
use log::{error, info};
use windows_sys::Win32::{
    Foundation::HWND,
    UI::{
        Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent},
        Input::KeyboardAndMouse::{
            INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput, VK_NONCONVERT,
        },
        WindowsAndMessaging::{
            DispatchMessageW, EVENT_SYSTEM_FOREGROUND, GetMessageW, MSG, TranslateMessage,
            WINEVENT_OUTOFCONTEXT,
        },
    },
};

pub fn run() -> Result<()> {
    if let Err(why) = send_nonconvert() {
        error!("Failed to send Muhenkan key at startup: {why}");
    }

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

    info!("Listening for app activation events. Sending the Muhenkan key.");
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
    _window: HWND,
    _object_id: i32,
    _child_id: i32,
    _event_thread: u32,
    _event_time: u32,
) {
    if let Err(why) = send_nonconvert() {
        error!("Failed to send Muhenkan key: {why}");
    }
}

fn send_nonconvert() -> Result<()> {
    let inputs = [keyboard_input(0), keyboard_input(KEYEVENTF_KEYUP)];
    let sent = unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            size_of::<INPUT>() as i32,
        )
    };
    if sent != inputs.len() as u32 {
        bail!("SendInput sent {sent} of {} keyboard events", inputs.len());
    }
    Ok(())
}

fn keyboard_input(flags: u32) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VK_NONCONVERT,
                dwFlags: flags,
                ..Default::default()
            },
        },
    }
}

struct WinEventHook(HWINEVENTHOOK);

impl Drop for WinEventHook {
    fn drop(&mut self) {
        unsafe { UnhookWinEvent(self.0) };
    }
}
