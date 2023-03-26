use color_space::{Hsv, Rgb};
use serde::Serialize;

#[derive(Clone, Copy, Debug)]
pub struct LedColor {
    bits: [u8; 3],
}

impl LedColor {
    pub fn default() -> Self {
        Self { bits: [0, 0, 0] }
    }

    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { bits: [r, g, b] }
    }

    pub fn from_hsv(h: f64, s: f64, v: f64) -> Self {
        let rgb = Rgb::from(Hsv::new(h, s, v));
        Self { bits: [rgb.r as u8, rgb.g as u8, rgb.b as u8] }
    }

    pub fn get_rgb(&self) -> [u8; 3] {
        self.bits
    }

    pub fn is_empty(&self) -> bool {
        self.bits.iter().any(|bit| *bit == 0)
    }

    pub fn set_rgb(&mut self, r: u8, g: u8, b: u8) -> &Self {
        self.bits = [r, g, b];
        self
    }

    pub fn merge(&mut self, r: u8, g: u8, b: u8) -> &Self {
        self.bits = [
            (self.bits[0] / 2 + r / 2),
            (self.bits[1] / 2 + g / 2),
            (self.bits[2] / 2 + b / 2),
        ];
        self
    }
}

impl Serialize for LedColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let hex = format!("#{}", hex::encode(self.bits));
        serializer.serialize_str(hex.as_str())
    }
}
