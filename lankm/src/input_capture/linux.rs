use std::sync::mpsc;
use std::thread;

use evdev::{EventType, InputEventKind};

use crate::data::{KeyEvent, KeyEventKind};

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

fn device_thread(sender: mpsc::Sender<KeyEvent>) {
    loop {
        for event in kbd.fetch_events().unwrap() {
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

                kbd_sender.send(KeyEvent { hid, kind }).unwrap();
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
        thread::spawn(move || device_thread(kbd, sender.clone()));
    }
}
