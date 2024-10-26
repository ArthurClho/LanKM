use std::sync::mpsc;
use std::thread;

use evdev::{EventType, InputEventKind};

use crate::data::{KeyEvent, KeyEventKind, Modifiers};

const fn invert_linux_table(table: &[u8; 252]) -> [u8; 252] {
    let mut inverted = [0; 252];

    // not allowed in const fn: for (scancode, index) in table.enumerate()
    let mut index = 0;
    while index < table.len() {
        let scancode = table[index];

        inverted[scancode as usize] = index as u8;

        index += 1;
    }

    inverted
}

const LINUX_TO_HID_TABLE: [u8; 252] =
    invert_linux_table(&crate::input_injection::HID_TO_LINUX_TABLE);

fn device_thread(mut kbd: evdev::Device, sender: mpsc::Sender<KeyEvent>) {
    let mut mods = Modifiers::empty();

    loop {
        let events = kbd.fetch_events().unwrap();
        for event in events {
            if let InputEventKind::Key(key) = event.kind() {
                // Ignore key repeats
                if event.value() == 2 {
                    continue;
                }

                let hid = LINUX_TO_HID_TABLE[key.0 as usize] as u16;
                let kind = match event.value() {
                    0 => KeyEventKind::Release,
                    1 => KeyEventKind::Press,
                    value => {
                        log::error!("Unknown event value: {}", value);
                        continue;
                    }
                };

                match hid {
                    0xE0 | 0xE4 => mods.set(Modifiers::CTRL, kind == KeyEventKind::Press),
                    0xE1 | 0xE5 => mods.set(Modifiers::SHIFT, kind == KeyEventKind::Press),
                    0xE2 | 0xE6 => mods.set(Modifiers::ALT, kind == KeyEventKind::Press),
                    _ => {}
                }

                sender.send(KeyEvent { hid, kind, mods }).unwrap();
            }
        }
    }
}

pub fn init(sender: mpsc::Sender<KeyEvent>) {
    log::debug!("Enumerating devices");
    let mut keyboards = Vec::new();
    for (path, device) in evdev::enumerate() {
        let dev_name = device.name().unwrap_or("<no name>");
        log::debug!("Found device at {}: {}", path.display(), dev_name);

        if device.supported_events().contains(EventType::KEY) {
            log::debug!("Using {} as keyboard", dev_name);
            keyboards.push(device);
        }
    }
    log::debug!("Done enumerating devices");

    for kbd in keyboards.into_iter() {
        log::debug!(
            "Starting thread for device: {}",
            kbd.name().unwrap_or("<no name>")
        );
        let kbd_sender = sender.clone();
        thread::spawn(move || device_thread(kbd, kbd_sender));
    }
}
