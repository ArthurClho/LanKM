#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum KeyEventKind {
    Press = 0,
    Release = 1,
}

impl From<u8> for KeyEventKind {
    fn from(n: u8) -> Self {
        match n {
            0 => KeyEventKind::Press,
            1 => KeyEventKind::Release,
            _ => panic!("Invalid KeyEventKind value"),
        }
    }
}

#[derive(Copy, Clone)]
pub struct KeyEvent {
    pub hid: u16,
    pub kind: KeyEventKind,
}

impl KeyEvent {
    const SIZE: usize = std::mem::size_of::<Self>();

    pub fn to_bytes(self) -> [u8; Self::SIZE] {
        let hid = self.hid.to_le_bytes();
        let kind = self.kind as u8;

        [hid[0], hid[1], kind, 0]
    }

    pub fn from_bytes(bytes: [u8; Self::SIZE]) -> Self {
        let hid = u16::from_le_bytes([bytes[0], bytes[1]]);
        let kind: KeyEventKind = bytes[2].into();
        Self { hid, kind }
    }
}
