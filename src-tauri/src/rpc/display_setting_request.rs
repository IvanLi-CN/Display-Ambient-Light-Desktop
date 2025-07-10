#[derive(Clone, Debug)]
pub enum DisplaySetting {
    Brightness(u8),
    Contrast(u8),
    Mode(u8),
}

#[derive(Clone, Debug)]
pub struct DisplaySettingRequest {
    pub display_index: usize,
    pub setting: DisplaySetting,
}
