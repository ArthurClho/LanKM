use std::sync::mpsc;
use std::thread;

use evdev::{EventType, InputEventKind};

use crate::event::{KeyEvent, KeyEventKind, Modifiers};

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
}

fn device_thread(mut args: DeviceThreadArgs) {
    let mut mods = Modifiers::empty();
    let dev_name = args.kbd.name().unwrap_or("<no name>").to_owned();

    loop {
        let events = args.kbd.fetch_events().unwrap();
        for event in events {
            if let InputEventKind::Key(key) = event.kind() {
                // Ignore key repeats
                if event.value() == 2 {
                    continue;
                }

                let hid = match LINUX_TO_HID_TABLE.get(key.0 as usize) {
                    Some(hid) => *hid as u16,
                    None => {
                        log::warn!("Unknown HID {} from device {}", key.0, dev_name);
                        continue;
                    }
                };
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
            }
        }
    }
}

pub fn init(sender: mpsc::Sender<KeyEvent>) {
    log::debug!("Enumerating devices");
    let mut keyboards = Vec::new();
    for (path, device) in evdev::enumerate() {
        let dev_name = device.name().unwrap_or("<no name>");
        log::debug!(
            "Found device at {}: {} supporting events: {:?}",
            path.display(),
            dev_name,
            device.supported_events()
        );

        if device.supported_events().contains(EventType::KEY)
            && device.supported_events().contains(EventType::REPEAT)
        {
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
        let mut sending = false;

        loop {
            let event = inj_receiver.recv().unwrap();

            if event.hid == 0x2B
                && event.kind == KeyEventKind::Press
                && event.mods.contains(Modifiers::CTRL | Modifiers::ALT)
            {
                sending = !sending;
                log::info!("Turned {} sending", if sending { "On" } else { "Off" });

                if sending {
                    // Release ctrl and alt
                    let mut release = |hid| {
                        injector.emit(KeyEvent {
                            hid,
                            kind: KeyEventKind::Release,
                            mods: Modifiers::empty(),
                        })
                    };

                    release(0xE0);
                    release(0xE4);
                    release(0xE2);
                    release(0xE6);
                } else {
                    // Release on client
                    let release = |hid| {
                        sender
                            .send(KeyEvent {
                                hid,
                                kind: KeyEventKind::Release,
                                mods: Modifiers::empty(),
                            })
                            .unwrap();
                    };

                    release(0xE0);
                    release(0xE4);
                    release(0xE2);
                    release(0xE6);
                }
            } else if sending {
                sender.send(event).unwrap();
            } else {
                injector.emit(event);
            }
        }
    });

    for mut kbd in keyboards.into_iter() {
        log::debug!(
            "Starting thread for device: {}",
            kbd.name().unwrap_or("<no name>")
        );

        kbd.grab().unwrap();

        let args = DeviceThreadArgs {
            kbd,
            sender: inj_sender.clone(),
        };
        thread::spawn(move || device_thread(args));
    }
}
