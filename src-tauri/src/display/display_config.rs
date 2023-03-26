use std::time::SystemTime;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct DisplayConfig {
    pub id: usize,
    pub brightness: u16,
    pub max_brightness: u16,
    pub min_brightness: u16,
    pub contrast: u16,
    pub max_contrast: u16,
    pub min_contrast: u16,
    pub mode: u16,
    pub max_mode: u16,
    pub min_mode: u16,
    pub last_modified_at: SystemTime,
}

impl DisplayConfig {
    pub fn default(index: usize) -> Self {
        Self {
            id: index,
            brightness: 30,
            contrast: 50,
            mode: 0,
            last_modified_at: SystemTime::now(),
            max_brightness: 100,
            min_brightness: 0,
            max_contrast: 100,
            min_contrast: 0,
            max_mode: 15,
            min_mode: 0,
        }
    }
}
