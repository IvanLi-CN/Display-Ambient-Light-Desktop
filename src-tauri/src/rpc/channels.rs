use std::sync::Arc;

use tokio::sync::{broadcast, OnceCell};

use super::DisplaySettingRequest;

pub struct BoardMessageChannels {
    pub display_setting_request_sender: Arc<broadcast::Sender<DisplaySettingRequest>>,
    pub volume_setting_request_sender: Arc<broadcast::Sender<f32>>,
    pub volume_changed_sender: Arc<broadcast::Sender<f32>>,
}

impl BoardMessageChannels {
    pub async fn global() -> &'static Self {
        static BOARD_MESSAGE_CHANNELS: OnceCell<BoardMessageChannels> = OnceCell::const_new();

        BOARD_MESSAGE_CHANNELS.get_or_init(|| async {Self::new()}).await
    }

    pub fn new() -> Self {
        let (display_setting_request_sender, _) = broadcast::channel(16);
        let display_setting_request_sender = Arc::new(display_setting_request_sender);

        let (volume_setting_request_sender, _) = broadcast::channel(16);
        let volume_setting_request_sender = Arc::new(volume_setting_request_sender);

        let (volume_changed_sender, _) = broadcast::channel(2);
        let volume_changed_sender = Arc::new(volume_changed_sender);

        Self {
            display_setting_request_sender,
            volume_setting_request_sender,
            volume_changed_sender,
        }
    }
}