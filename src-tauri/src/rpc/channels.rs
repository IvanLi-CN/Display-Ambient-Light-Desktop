use std::sync::Arc;

use tokio::sync::{broadcast, OnceCell};

use super::DisplaySettingRequest;

pub struct BoardMessageChannels {
    pub display_setting_request_sender: Arc<broadcast::Sender<DisplaySettingRequest>>,
}

impl BoardMessageChannels {
    pub async fn global() -> &'static Self {
        static BOARD_MESSAGE_CHANNELS: OnceCell<BoardMessageChannels> = OnceCell::const_new();

        BOARD_MESSAGE_CHANNELS.get_or_init(|| async {Self::new()}).await
    }

    pub fn new() -> Self {
        let (display_setting_request_sender, _) = broadcast::channel(16);
        let display_setting_request_sender = Arc::new(display_setting_request_sender);

        Self {
            display_setting_request_sender,
        }
    }
}