use std::ops::Index;

use color_space::{Hsv, Rgb};
use serde::Serialize;

#[derive(Clone, Copy, Debug)]
pub struct LedColor([u8; 3]);

impl LedColor {
    pub fn default() -> Self {
        Self ([0, 0, 0] )
    }

    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self ([r, g, b])
    }

    pub fn from_hsv(h: f64, s: f64, v: f64) -> Self {
        let rgb = Rgb::from(Hsv::new(h, s, v));
        Self ([rgb.r as u8, rgb.g as u8, rgb.b as u8])
    }

    pub fn get_rgb(&self) -> [u8; 3] {
        self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.iter().any(|bit| *bit == 0)
    }

    pub fn set_rgb(&mut self, r: u8, g: u8, b: u8) -> &Self {
        self.0 = [r, g, b];
        self
    }

    pub fn merge(&mut self, r: u8, g: u8, b: u8) -> &Self {
        self.0 = [
            (self.0[0] / 2 + r / 2),
            (self.0[1] / 2 + g / 2),
            (self.0[2] / 2 + b / 2),
        ];
        self
    }

    pub fn as_bytes (&self) -> [u8; 3] {
        self.0
    }
}

impl Serialize for LedColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let hex = format!("#{}", hex::encode(self.0));
        serializer.serialize_str(hex.as_str())
    }
}
