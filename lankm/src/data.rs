#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum KeyEventKind {
    Press,
    Release,
}

#[derive(Copy, Clone)]
pub struct KeyEvent {
    pub hid: u16,
    pub kind: KeyEventKind,
}

impl KeyEvent {
    const SIZE: usize = std::mem::size_of::<Self>();

    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let hid = self.hid.to_le_bytes();
        let kind = self.kind as u8;

        [hid[0], hid[1], kind, 0]
    }
}
