use std::{
    mem,
    sync::{Arc, RwLock},
};

use coreaudio::{
    audio_unit::macos_helpers::get_default_device_id,
    sys::{
        kAudioHardwareServiceDeviceProperty_VirtualMasterVolume, kAudioObjectPropertyScopeOutput,
        AudioObjectHasProperty, AudioObjectPropertyAddress, AudioObjectSetPropertyData,
    },
};
use paris::error;
use tokio::sync::OnceCell;

use crate::rpc::BoardMessageChannels;

pub struct VolumeManager {
    current_volume: Arc<RwLock<f32>>,
    handler: Option<tokio::task::JoinHandle<()>>,
}

impl VolumeManager {
    pub async fn global() -> &'static Self {
        static VOLUME_MANAGER: OnceCell<VolumeManager> = OnceCell::const_new();

        VOLUME_MANAGER
            .get_or_init(|| async { Self::create() })
            .await
    }

    pub fn create() -> Self {
        let mut instance = Self {
            current_volume: Arc::new(RwLock::new(0.0)),
            handler: None,
        };

        instance.subscribe_volume_setting_request();

        instance
    }

    fn subscribe_volume_setting_request(&mut self) {
        let handler = tokio::spawn(async {
            let channels = BoardMessageChannels::global().await;
            let mut request_rx = channels.volume_setting_request_sender.subscribe();

            while let Ok(volume) = request_rx.recv().await {
                if let Err(err) = Self::set_volume(volume) {
                    error!("failed to set volume: {}", err);
                }
            }
        });

        self.handler = Some(handler);
    }

    fn set_volume(volume: f32) -> anyhow::Result<()> {
        log::debug!("set volume: {}", volume);

        let device_id = get_default_device_id(false);

        if device_id.is_none() {
            anyhow::bail!("default audio output device is not found.");
        }

        let device_id = device_id.unwrap();

        let address = AudioObjectPropertyAddress {
            mSelector: kAudioHardwareServiceDeviceProperty_VirtualMasterVolume,
            mScope: kAudioObjectPropertyScopeOutput,
            mElement: 0,
        };

        log::debug!("device id: {}", device_id);
        log::debug!("address: {:?}", address);

        if 0 == unsafe { AudioObjectHasProperty(device_id, &address) } {
            anyhow::bail!("Can not get audio property");
        }

        let size = mem::size_of::<f32>() as u32;

        let result = unsafe {
            AudioObjectSetPropertyData(
                device_id,
                &address,
                0,
                std::ptr::null(),
                size,
                &volume as *const f32 as *const std::ffi::c_void,
            )
        };

        if result != 0 {
            anyhow::bail!("Can not set audio property");
        }

        Ok(())
    }
}
