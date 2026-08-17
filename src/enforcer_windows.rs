use std::{
    io::Error,
    mem::{size_of, zeroed},
    ptr::null_mut,
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::Duration,
};

use anyhow::{Context, Result, bail};
use log::{error, info};
use windows_sys::Win32::{
    Foundation::HWND,
    UI::{
        Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent},
        Input::KeyboardAndMouse::{
            GetAsyncKeyState, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
            SendInput, VK_MENU, VK_NONCONVERT,
        },
        WindowsAndMessaging::{
            DispatchMessageW, EVENT_SYSTEM_FOREGROUND, GetMessageW, MSG, TranslateMessage,
            WINEVENT_OUTOFCONTEXT,
        },
    },
};

const NONCONVERT_SCAN_CODE: u16 = 0x7b;
const ALT_RELEASE_POLL_INTERVAL: Duration = Duration::from_millis(10);

static ACTIVATION_GENERATION: AtomicU64 = AtomicU64::new(0);

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
    send_nonconvert_after_alt_release();
}

fn send_nonconvert_after_alt_release() {
    let generation = ACTIVATION_GENERATION
        .fetch_add(1, Ordering::Relaxed)
        .wrapping_add(1);
    if !alt_is_pressed() {
        if let Err(why) = send_nonconvert() {
            error!("Failed to send Muhenkan key: {why}");
        }
        return;
    }

    thread::spawn(move || {
        while alt_is_pressed() {
            if ACTIVATION_GENERATION.load(Ordering::Relaxed) != generation {
                return;
            }
            thread::sleep(ALT_RELEASE_POLL_INTERVAL);
        }
        if ACTIVATION_GENERATION.load(Ordering::Relaxed) == generation
            && let Err(why) = send_nonconvert()
        {
            error!("Failed to send Muhenkan key after Alt was released: {why}");
        }
    });
}

fn alt_is_pressed() -> bool {
    unsafe { GetAsyncKeyState(VK_MENU as i32) as u16 & 0x8000 != 0 }
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
                wScan: NONCONVERT_SCAN_CODE,
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
