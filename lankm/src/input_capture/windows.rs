use std::thread;

use crate::event::{Event, KeyEvent, KeyEventKind, Modifiers};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_TYPE, KEYBDINPUT, KEYBD_EVENT_FLAGS, VK_LCONTROL, VK_LMENU,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage, HHOOK,
    KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

// From https://learn.microsoft.com/en-us/windows/win32/inputdev/about-keyboard-input#scan-codes
// SCANCODE_TABLE[windows scan code] = hid
static SCANCODE_TABLE: [u16; 256] = [
    0, 41, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 45, 46, 42, 43, 20, 26, 8, 21, 23, 28, 24, 12,
    18, 19, 47, 48, 40, 224, 4, 22, 7, 9, 10, 11, 13, 14, 15, 51, 52, 53, 225, 50, 29, 27, 6, 25,
    5, 17, 16, 54, 55, 56, 229, 85, 226, 44, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 83, 71,
    95, 96, 97, 86, 92, 93, 94, 87, 89, 90, 91, 98, 99, 0, 0, 100, 68, 69, 103, 0, 0, 140, 0, 0, 0,
    0, 0, 0, 0, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 0, 136, 145, 144, 135, 0, 0,
    148, 147, 146, 138, 0, 139, 0, 137, 133, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
];
static EXTENDED_TABLE: [u16; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 182, 0, 0, 0, 0, 0, 0, 0, 0, 181, 0, 0, 88,
    228, 0, 0, 226, 402, 205, 0, 183, 0, 0, 0, 0, 0, 0, 0, 0, 0, 234, 0, 233, 0, 547, 0, 0, 84, 0,
    70, 230, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 72, 0, 74, 82, 75, 0, 80, 0, 79, 0, 77, 81, 78,
    73, 76, 0, 0, 0, 0, 0, 0, 0, 227, 231, 101, 102, 130, 0, 0, 0, 131, 0, 545, 554, 551, 550, 549,
    548, 404, 394, 387, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

type CallbackFunction = Box<dyn FnMut(Event) -> bool + Send>;

static mut GLOBAL_CALLBACK: Option<CallbackFunction> = None;

static mut GLOBAL_MODS: Modifiers = Modifiers::empty();

unsafe extern "system" fn keyboard_hook(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let kbd_event: KBDLLHOOKSTRUCT = *(l_param.0 as *const _);

    if kbd_event.flags.0 & 0x00000010 != 0 {
        // An injected event, very likely to be our own SendInput call
        return CallNextHookEx(HHOOK(std::ptr::null_mut()), code, w_param, l_param);
    }

    let hid = if kbd_event.flags.0 & 1 == 1 {
        EXTENDED_TABLE[kbd_event.scanCode as usize]
    } else {
        SCANCODE_TABLE[kbd_event.scanCode as usize]
    };

    let kind = match w_param.0 as u32 {
        WM_KEYDOWN | WM_SYSKEYDOWN => KeyEventKind::Press,
        WM_KEYUP | WM_SYSKEYUP => KeyEventKind::Release,
        _ => panic!("Invalid wParam"),
    };

    // TODO: Copied from Linux implementation, extract into a method
    match hid {
        0xE0 | 0xE4 => GLOBAL_MODS.set(Modifiers::CTRL, kind == KeyEventKind::Press),
        0xE1 | 0xE5 => GLOBAL_MODS.set(Modifiers::SHIFT, kind == KeyEventKind::Press),
        0xE2 | 0xE6 => GLOBAL_MODS.set(Modifiers::ALT, kind == KeyEventKind::Press),
        _ => {}
    }

    let event = match hid {
        0x2B if kind == KeyEventKind::Press
            && GLOBAL_MODS.contains(Modifiers::CTRL | Modifiers::ALT) =>
        {
            Event::Hotkey
        }
        _ => Event::Key(KeyEvent {
            hid,
            kind,
            mods: Modifiers::empty(),
        }),
    };

    let cb = GLOBAL_CALLBACK.as_mut().unwrap();

    let handled = cb(event);

    match (event, handled) {
        (Event::Key(_), true) => LRESULT(1),
        (Event::Key(_), false) | (Event::Hotkey, false) => {
            CallNextHookEx(HHOOK(std::ptr::null_mut()), code, w_param, l_param)
        }
        (Event::Hotkey, true) => {
            // Release CTRL and ALT
            let release_ctrl_key = KEYBDINPUT {
                wVk: VK_LCONTROL,
                wScan: 0x1D,
                dwFlags: KEYBD_EVENT_FLAGS(0x0002),
                time: 0,
                dwExtraInfo: 0,
            };
            let release_ctrl_input = INPUT {
                r#type: INPUT_TYPE(1),
                Anonymous: INPUT_0 {
                    ki: release_ctrl_key,
                },
            };
            let release_alt_key = KEYBDINPUT {
                wVk: VK_LMENU,
                wScan: 0x38,
                dwFlags: KEYBD_EVENT_FLAGS(0x0002),
                time: 0,
                dwExtraInfo: 0,
            };
            let release_alt_input = INPUT {
                r#type: INPUT_TYPE(1),
                Anonymous: INPUT_0 {
                    ki: release_alt_key,
                },
            };
            SendInput(
                &[release_ctrl_input, release_alt_input],
                std::mem::size_of::<INPUT>() as i32,
            );

            LRESULT(1)
        }
    }
}

pub fn init<F: FnMut(Event) -> bool + 'static + Send>(callback: F) {
    thread::spawn(move || {
        unsafe {
            GLOBAL_CALLBACK = Some(Box::new(callback));

            SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(keyboard_hook),
                HINSTANCE(std::ptr::null_mut()),
                0,
            )
            .unwrap();
        }

        unsafe {
            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, HWND(std::ptr::null_mut()), 0, 0).into() {
                let _ = TranslateMessage(&msg);
                let _ = DispatchMessageW(&msg);
            }
        }
    });
}
