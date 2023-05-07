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
        log::info!("start display setting request handler");
        let handler = tokio::spawn(async move {
            let channels = BoardMessageChannels::global().await;

            let mut request_rx = channels.display_setting_request_sender.subscribe();

            log::info!("display setting request handler started");

            while let Ok(message) = request_rx.recv().await {
                let displays = displays.write().await;

                let display = displays.get(message.display_index);
                if display.is_none() {
                    warn!("display#{} not found", message.display_index);
                    continue;
                }

                log::info!("display setting request received. {:?}", message);

                let display = display.unwrap().write().await;
                match message.setting {
                    DisplaySetting::Brightness(value) => display.set_brightness(value as u16).await,
                    DisplaySetting::Contrast(value) => display.set_contrast(value as u16).await,
                    DisplaySetting::Mode(value) => display.set_mode(value as u16).await,
                }
                drop(display);

                log::info!("display setting request handled. {:?}", message);

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

    // fn read_display_config_by_ddc(index: usize) -> anyhow::Result<DisplayState> {
    //     let mut displays = Display::enumerate();
    //     match displays.get_mut(index) {
    //         Some(display) => {
    //             let mut config = DisplayState::default(index);
    //             match display.handle.get_vcp_feature(0x10) {
    //                 Ok(value) => {
    //                     config.max_brightness = value.maximum();
    //                     config.min_brightness = 0;
    //                     config.brightness = value.value();
    //                 }
    //                 Err(_) => {}
    //             };
    //             match display.handle.get_vcp_feature(0x12) {
    //                 Ok(value) => {
    //                     config.max_contrast = value.maximum();
    //                     config.min_contrast = 0;
    //                     config.contrast = value.value();
    //                 }
    //                 Err(_) => {}
    //             };
    //             match display.handle.get_vcp_feature(0xdc) {
    //                 Ok(value) => {
    //                     config.max_mode = value.maximum();
    //                     config.min_mode = 0;
    //                     config.mode = value.value();
    //                 }
    //                 Err(_) => {}
    //             };

    //             Ok(config)
    //         }
    //         None => anyhow::bail!("display#{} is missed.", index),
    //     }
    // }

    // async fn get_display(&self, index: usize) -> anyhow::Result<OwnedMutexGuard<DisplayState>> {
    //     let mut displays = self.displays.lock().await;
    //     match displays.get_mut(&index) {
    //         Some(config) => {
    //             let mut config = config.to_owned().lock_owned().await;
    //             if config.last_modified_at > SystemTime::now().sub(Duration::from_secs(10)) {
    //                 info!("cached");
    //                 return Ok(config);
    //             }
    //             return match Self::read_display_config_by_ddc(index) {
    //                 Ok(config) => {
    //                     let id = config.id;
    //                     let value = Arc::new(Mutex::new(config));
    //                     let valueGuard = value.clone().lock_owned().await;
    //                     displays.insert(id, value);
    //                     info!("read form ddc");
    //                     Ok(valueGuard)
    //                 }
    //                 Err(err) => {
    //                     warn!(
    //                         "can not read config from display by ddc, use CACHED value. {:?}",
    //                         err
    //                     );
    //                     config.last_modified_at = SystemTime::now();
    //                     Ok(config)
    //                 }
    //             };
    //         }
    //         None => {
    //             let config = Self::read_display_config_by_ddc(index).map_err(|err| {
    //                 anyhow::anyhow!(
    //                     "can not read config from display by ddc,use DEFAULT value. {:?}",
    //                     err
    //                 )
    //             })?;
    //             let id = config.id;
    //             let value = Arc::new(Mutex::new(config));
    //             let valueGuard = value.clone().lock_owned().await;
    //             displays.insert(id, value);
    //             Ok(valueGuard)
    //         }
    //     }
    // }

    // pub async fn set_display_brightness(
    //     &self,
    //     display_brightness: DisplayBrightness,
    // ) -> anyhow::Result<()> {
    //     match Display::enumerate().get_mut(display_brightness.display_index) {
    //         Some(display) => {
    //             match self.get_display(display_brightness.display_index).await {
    //                 Ok(mut config) => {
    //                     let curr = config.brightness;
    //                     info!("curr_brightness: {:?}", curr);
    //                     let mut target = match display_brightness.brightness {
    //                         Brightness::Relative(v) => curr.wrapping_add_signed(v),
    //                         Brightness::Absolute(v) => v,
    //                     };
    //                     if target.gt(&config.max_brightness) {
    //                         target = config.max_brightness;
    //                     } else if target.lt(&config.min_brightness) {
    //                         target = config.min_brightness;
    //                     }
    //                     config.brightness = target;
    //                     display
    //                         .handle
    //                         .set_vcp_feature(0x10, target as u16)
    //                         .map_err(|err| anyhow::anyhow!("can not set brightness. {:?}", err))?;

    //                     let rpc = rpc::Manager::global().await;

    //                     rpc.publish_desktop_cmd(
    //                         format!("display{}/brightness", display_brightness.display_index)
    //                             .as_str(),
    //                         target.to_be_bytes().to_vec(),
    //                     )
    //                     .await;
    //                 }
    //                 Err(err) => {
    //                     info!(
    //                         "can not get display#{} brightness. {:?}",
    //                         display_brightness.display_index, err
    //                     );
    //                     if let Brightness::Absolute(v) = display_brightness.brightness {
    //                         display.handle.set_vcp_feature(0x10, v).map_err(|err| {
    //                             anyhow::anyhow!("can not set brightness. {:?}", err)
    //                         })?;
    //                     };
    //                 }
    //             };
    //         }
    //         None => {
    //             warn!("display#{} is not found.", display_brightness.display_index);
    //         }
    //     }
    //     Ok(())
    // }
}

impl Drop for DisplayManager {
    fn drop(&mut self) {
        if let Some(handler) = self.setting_request_handler.take() {
            handler.abort();
        }
    }
}
