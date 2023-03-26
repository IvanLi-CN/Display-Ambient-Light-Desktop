use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum Brightness {
    Relative(i16),
    Absolute(u16),
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct DisplayBrightness {
    pub brightness: Brightness,
    pub display_index: usize,
}