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

struct DeviceThreadArgs {
    pub kbd: evdev::Device,
    pub sender: mpsc::Sender<KeyEvent>,
    pub inj_sender: mpsc::Sender<KeyEvent>,
}

fn device_thread(mut args: DeviceThreadArgs) {
    let mut mods = Modifiers::empty();

    loop {
        let events = args.kbd.fetch_events().unwrap();
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

                args.sender.send(KeyEvent { hid, kind, mods }).unwrap();
                args.inj_sender.send(KeyEvent { hid, kind, mods }).unwrap();
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

    // Because we can't stop keyboard events from being propagated like
    // in the windows implementation we do a little dance here: Grab all
    // devices we can, and when we want them to propagate funnel all events
    // into an injector
    let (inj_sender, inj_receiver) = mpsc::channel::<KeyEvent>();
    thread::spawn(move || {
        let mut injector = crate::input_injection::InputInjector::new();

        loop {
            let event = inj_receiver.recv().unwrap();
            injector.emit(event);
        }
    });

    for mut kbd in keyboards.into_iter() {
        log::debug!(
            "Starting thread for device: {}",
            kbd.name().unwrap_or("<no name>")
        );

        kbd.grab().unwrap();

        let args = DeviceThreadArgs {
            kbd: kbd,
            sender: sender.clone(),
            inj_sender: inj_sender.clone(),
        };
        thread::spawn(move || device_thread(args));
    }
}
