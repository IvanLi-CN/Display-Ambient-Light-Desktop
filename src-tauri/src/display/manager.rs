use std::{env::current_dir, sync::Arc, time::Duration};

use ddc_hi::Display;
use dirs::config_dir;
use paris::{error, info, warn};
use tokio::{
    sync::{broadcast, watch, OnceCell, RwLock},
    task::yield_now,
};

use crate::{
    display::DisplayStateWrapper,
    rpc::{BoardMessageChannels, DisplaySetting},
};

use super::{
    display_handler::{DisplayHandler, SafeDisplay},
    display_state::DisplayState,
};

const CONFIG_FILE_NAME: &str = "cc.ivanli.ambient_light/displays.toml";

pub struct DisplayManager {
    displays: Arc<RwLock<Vec<Arc<RwLock<DisplayHandler>>>>>,
    setting_request_handler: Option<tokio::task::JoinHandle<()>>,
    displays_changed_sender: Arc<watch::Sender<Vec<DisplayState>>>,
    auto_save_state_handler: Option<tokio::task::JoinHandle<()>>,
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
            auto_save_state_handler: None,
        };
        instance.fetch_displays().await;
        instance.restore_states().await;
        instance.fetch_state_of_displays().await;
        instance.subscribe_setting_request();
        instance.auto_save_state_of_displays();
        instance
    }

    fn auto_save_state_of_displays(&mut self) {
        let displays = self.displays.clone();

        let handler = tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(10)).await;
                Self::save_states(displays.clone()).await;
                Self::send_displays_changed(displays.clone()).await;
            }
        });

        self.auto_save_state_handler = Some(handler);
    }

    async fn send_displays_changed(displays: Arc<RwLock<Vec<Arc<RwLock<DisplayHandler>>>>>) {
        let mut states = Vec::new();

        for display in displays.read().await.iter() {
            let state = display.read().await.state.read().await.clone();
            states.push(state);
        }

        let channel = BoardMessageChannels::global().await;
        let tx = channel.displays_changed_sender.clone();
        if let Err(err) = tx.send(states) {
            error!("Failed to send displays changed: {}", err);
        }
    }

    async fn fetch_displays(&self) {
        let mut displays = self.displays.write().await;
        displays.clear();

        let controllers = Display::enumerate();

        for display in controllers {
            let safe_display = SafeDisplay::new(display);
            let controller = Arc::new(RwLock::new(safe_display));
            let state = Arc::new(RwLock::new(DisplayState::default()));
            let handler = DisplayHandler {
                state: state.clone(),
                controller: controller.clone(),
            };

            displays.push(Arc::new(RwLock::new(handler)));
        }
    }

    async fn fetch_state_of_displays(&self) {
        let displays = self.displays.read().await;

        for display in displays.iter() {
            let display = display.read().await;
            display.fetch_state().await;
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

            loop {
                if let Err(err) = request_rx.recv().await {
                    match err {
                        broadcast::error::RecvError::Closed => {
                            info!("display setting request channel closed");
                            break;
                        }
                        broadcast::error::RecvError::Lagged(_) => {
                            warn!("display setting request channel lagged");
                            continue;
                        }
                    }
                }

                let message = request_rx.recv().await.unwrap();

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

    async fn restore_states(&self) {
        let path = config_dir()
            .unwrap_or(current_dir().unwrap())
            .join(CONFIG_FILE_NAME);

        if !path.exists() {
            log::info!("config file not found: {}. skip read.", path.display());
            return;
        }

        let text = std::fs::read_to_string(path);
        if let Err(err) = text {
            log::error!("failed to read config file: {}", err);
            return;
        }

        let text = text.unwrap();
        let wrapper = toml::from_str::<DisplayStateWrapper>(&text);

        if let Err(err) = wrapper {
            log::error!("failed to parse display states file: {}", err);
            return;
        }

        let states = wrapper.unwrap().states;

        let displays = self.displays.read().await;
        for (index, display) in displays.iter().enumerate() {
            let display = display.read().await;
            let mut state = display.state.write().await;
            let saved = states.get(index);
            if let Some(saved) = saved {
                state.brightness = saved.brightness;
                state.contrast = saved.contrast;
                state.mode = saved.mode;
                log::info!("restore display config. display#{}: {:?}", index, state);
            }
        }

        log::info!(
            "restore display config. store displays: {}, online displays: {}",
            states.len(),
            displays.len()
        );
    }

    async fn save_states(displays: Arc<RwLock<Vec<Arc<RwLock<DisplayHandler>>>>>) {
        let path = config_dir()
            .unwrap_or(current_dir().unwrap())
            .join(CONFIG_FILE_NAME);

        let displays = displays.read().await;
        let mut states = Vec::new();
        for display in displays.iter() {
            let state = display.read().await.state.read().await.clone();
            states.push(state);
        }

        let wrapper = DisplayStateWrapper::new(states);

        let text = toml::to_string(&wrapper);
        if let Err(err) = text {
            log::error!("failed to serialize display states: {}", err);
            log::error!("display states: {:?}", &wrapper);
            return;
        }

        let text = text.unwrap();
        if path.exists() {
            if let Err(err) = std::fs::remove_file(&path) {
                log::error!("failed to remove old config file: {}", err);
                return;
            }
        }

        if let Err(err) = std::fs::write(&path, text) {
            log::error!("failed to write config file: {}", err);
            return;
        }

        log::debug!(
            "save display config. store displays: {}, online displays: {}",
            wrapper.states.len(),
            displays.len()
        );
    }

    pub fn subscribe_displays_changed(&self) -> watch::Receiver<Vec<DisplayState>> {
        self.displays_changed_sender.subscribe()
    }
}

impl Drop for DisplayManager {
    fn drop(&mut self) {
        log::info!("dropping display manager=============");
        if let Some(handler) = self.setting_request_handler.take() {
            handler.abort();
        }

        if let Some(handler) = self.auto_save_state_handler.take() {
            handler.abort();
        }
    }
}
