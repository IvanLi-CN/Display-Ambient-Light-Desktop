use std::{sync::Arc, time::SystemTime};

use ddc_hi::{Ddc, Display};
use tokio::sync::RwLock;

use super::DisplayState;

// Safe wrapper for Display that implements Send + Sync
pub struct SafeDisplay {
    display: Display,
}

unsafe impl Send for SafeDisplay {}
unsafe impl Sync for SafeDisplay {}

impl SafeDisplay {
    pub fn new(display: Display) -> Self {
        Self { display }
    }

    pub fn get_mut(&mut self) -> &mut Display {
        &mut self.display
    }
}

pub struct DisplayHandler {
    pub state: Arc<RwLock<DisplayState>>,
    pub controller: Arc<RwLock<SafeDisplay>>,
}

impl DisplayHandler {
    pub async fn fetch_state(&self) {
        let mut controller = self.controller.write().await;

        let mut temp_state = *self.state.read().await;

        if let Ok(value) = controller.get_mut().handle.get_vcp_feature(0x10) {
            temp_state.max_brightness = value.maximum();
            temp_state.min_brightness = 0;
            temp_state.brightness = value.value();
        };
        if let Ok(value) = controller.get_mut().handle.get_vcp_feature(0x12) {
            temp_state.max_contrast = value.maximum();
            temp_state.min_contrast = 0;
            temp_state.contrast = value.value();
        };
        if let Ok(value) = controller.get_mut().handle.get_vcp_feature(0xdc) {
            temp_state.max_mode = value.maximum();
            temp_state.min_mode = 0;
            temp_state.mode = value.value();
        };

        temp_state.last_fetched_at = SystemTime::now();

        let mut state = self.state.write().await;
        *state = temp_state;
    }

    pub async fn set_brightness(&self, brightness: u16) -> anyhow::Result<()> {
        let mut controller = self.controller.write().await;
        let mut state = self.state.write().await;

        controller
            .get_mut()
            .handle
            .set_vcp_feature(0x10, brightness)
            .map_err(|err| anyhow::anyhow!("can not set brightness. {:?}", err))?;

        state.brightness = brightness;

        state.last_modified_at = SystemTime::now();

        Ok(())
    }

    pub async fn set_contrast(&self, contrast: u16) -> anyhow::Result<()> {
        let mut controller = self.controller.write().await;
        let mut state = self.state.write().await;

        controller
            .get_mut()
            .handle
            .set_vcp_feature(0x12, contrast)
            .map_err(|err| anyhow::anyhow!("can not set contrast. {:?}", err))?;

        state.contrast = contrast;
        state.last_modified_at = SystemTime::now();

        Ok(())
    }

    pub async fn set_mode(&self, mode: u16) -> anyhow::Result<()> {
        let mut controller = self.controller.write().await;
        let mut state = self.state.write().await;

        controller
            .get_mut()
            .handle
            .set_vcp_feature(0xdc, mode)
            .map_err(|err| anyhow::anyhow!("can not set mode. {:?}", err))?;

        state.mode = mode;
        state.last_modified_at = SystemTime::now();

        Ok(())
    }
}
