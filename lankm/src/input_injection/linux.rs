use crate::data::{KeyEvent, KeyEventKind};

use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};
use evdev::AttributeSet;

// taken from drives/hid/usbhid/usbkbd.c in the linux 6.10.7 source
pub(crate) const HID_TO_LINUX_TABLE: [u8; 252] = [
    0, 0, 0, 0, 30, 48, 46, 32, 18, 33, 34, 35, 23, 36, 37, 38, 50, 49, 24, 25, 16, 19, 31, 20, 22,
    47, 17, 45, 21, 44, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 28, 1, 14, 15, 57, 12, 13, 26, 27, 43, 43,
    39, 40, 41, 51, 52, 53, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 87, 88, 99, 70, 119, 110,
    102, 104, 111, 107, 109, 106, 105, 108, 103, 69, 98, 55, 74, 78, 96, 79, 80, 81, 75, 76, 77,
    71, 72, 73, 82, 83, 86, 127, 116, 117, 183, 184, 185, 186, 187, 188, 189, 190, 191, 192, 193,
    194, 134, 138, 130, 132, 128, 129, 131, 137, 133, 135, 136, 113, 115, 114, 0, 0, 0, 121, 0, 89,
    93, 124, 92, 94, 95, 0, 0, 0, 122, 123, 90, 91, 85, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 29, 42,
    56, 125, 97, 54, 100, 126, 164, 166, 165, 163, 161, 115, 114, 113, 150, 158, 159, 128, 136,
    177, 178, 176, 142, 152, 173, 140,
];

pub struct InputInjector {
    virtual_device: VirtualDevice,
}

impl InputInjector {
    pub fn new() -> Self {
        let keys = &mut AttributeSet::<evdev::Key>::new();
        for i in 0..256 {
            keys.insert(evdev::Key::new(i));
        }

        let virtual_device = VirtualDeviceBuilder::new()
            .unwrap()
            .name("lankm-virtual-dev")
            .with_keys(keys)
            .unwrap()
            .build()
            .unwrap();

        Self { virtual_device }
    }

    pub fn emit(&mut self, event: KeyEvent) {
        let value = match event.kind {
            KeyEventKind::Release => 0,
            KeyEventKind::Press => 1,
        };
        let scancode = HID_TO_LINUX_TABLE[event.hid as usize];

        let events = &[evdev::InputEvent::new(
            evdev::EventType::KEY,
            scancode as u16,
            value,
        )];
        self.virtual_device.emit(events).unwrap();
    }
}
