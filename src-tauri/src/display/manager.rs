use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

use ddc_hi::Display;
use paris::{error, info, warn};
use tokio::{sync::{watch, OnceCell, RwLock}, task::yield_now};

use crate::rpc::{BoardMessageChannels, DisplaySetting};

use super::{display_handler::DisplayHandler, display_state::DisplayState};

pub struct DisplayManager {
    displays: Arc<RwLock<Vec<Arc<RwLock<DisplayHandler>>>>>,
    setting_request_handler: Option<tokio::task::JoinHandle<()>>,
    displays_changed_sender: Arc<watch::Sender<Vec<DisplayState>>>,
}

impl DisplayManager {
    pub async fn global() -> &'static Self {
        static DISPLAY_MANAGER: OnceCell<DisplayManager> = OnceCell::const_new();

        DISPLAY_MANAGER.get_or_init(|| Self::create()).await
    }

    pub async fn create() -> Self {
        let (displays_changed_sender, _) = watch::channel(Vec::new());
        let displays_changed_sender = Arc::new(displays_changed_sender);

        let mut instance = Self {
            displays: Arc::new(RwLock::new(Vec::new())),
            setting_request_handler: None,
            displays_changed_sender,
        };
        instance.fetch_displays().await;
        instance.subscribe_setting_request();
        instance
    }

    async fn fetch_displays(&self) {
        let mut displays = self.displays.write().await;
        displays.clear();

        let controllers = Display::enumerate();

        for display in controllers {
            let controller = Arc::new(RwLock::new(display));
            let state = Arc::new(RwLock::new(DisplayState::default()));
            let handler = DisplayHandler {
                state: state.clone(),
                controller: controller.clone(),
            };

            handler.fetch_state().await;

            displays.push(Arc::new(RwLock::new(handler)));
        }
    }

    pub async fn get_displays(&self) -> Vec<DisplayState> {
        let displays = self.displays.read().await;
        let mut states = Vec::new();
        for display in displays.iter() {
            let state = display.read().await.state.read().await.clone();
            states.push(state);
        }
        states
    }

    fn subscribe_setting_request(&mut self) {
        let displays = self.displays.clone();
        let displays_changed_sender = self.displays_changed_sender.clone();
        let handler = tokio::spawn(async move {
            let channels = BoardMessageChannels::global().await;
            let mut request_rx = channels.display_setting_request_sender.subscribe();

            while let Ok(message) = request_rx.recv().await {
                let displays = displays.write().await;

                let display = displays.get(message.display_index);
                if display.is_none() {
                    warn!("display#{} not found", message.display_index);
                    continue;
                }


                let display = display.unwrap().write().await;
                let result = match message.setting {
                    DisplaySetting::Brightness(value) => display.set_brightness(value as u16).await,
                    DisplaySetting::Contrast(value) => display.set_contrast(value as u16).await,
                    DisplaySetting::Mode(value) => display.set_mode(value as u16).await,
                };

                if let Err(err) = result {
                    error!("failed to set display setting: {}", err);
                    continue;
                }

                drop(display);

                let mut states = Vec::new();
                for display in displays.iter() {
                    let state = display.read().await.state.read().await.clone();
                    states.push(state);
                }

                if let Err(err) = displays_changed_sender.send(states) {
                    error!("failed to send displays changed event: {}", err);
                }
                yield_now().await;
            }
        });

        self.setting_request_handler = Some(handler);
    }

    pub fn subscribe_displays_changed(&self) -> watch::Receiver<Vec<DisplayState>> {
        self.displays_changed_sender.subscribe()
    }
}

impl Drop for DisplayManager {
    fn drop(&mut self) {
        if let Some(handler) = self.setting_request_handler.take() {
            info!("abort display setting request handler");
            handler.abort();
        }
    }
}
