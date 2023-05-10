use std::{mem, sync::Arc};

use coreaudio::{
    audio_unit::macos_helpers::get_default_device_id,
    sys::{
        kAudioHardwareServiceDeviceProperty_VirtualMasterVolume, kAudioObjectPropertyScopeOutput,
        AudioObjectGetPropertyData, AudioObjectHasProperty, AudioObjectPropertyAddress,
        AudioObjectSetPropertyData,
    },
};
use paris::error;
use tokio::sync::{OnceCell, RwLock};

use crate::rpc::BoardMessageChannels;

pub struct VolumeManager {
    current_volume: Arc<RwLock<f32>>,
    handler: Option<tokio::task::JoinHandle<()>>,
    read_handler: Option<tokio::task::JoinHandle<()>>,
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
            read_handler: None,
        };

        instance.subscribe_volume_setting_request();
        instance.auto_read_volume();

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

    fn auto_read_volume(&mut self) {
        let current_volume = self.current_volume.clone();

        let handler = tokio::spawn(async move {
            let channel = BoardMessageChannels::global().await;
            let volume_changed_tx = channel.volume_changed_sender.clone();
            loop {
                match Self::read_volume() {
                    Ok(value) => {
                        let mut volume = current_volume.write().await;
                        if *volume != value {
                            if let Err(err) = volume_changed_tx.send(value) {
                                error!("failed to send volume changed event: {}", err);
                            }
                        }

                        *volume = value;
                    }
                    Err(err) => {
                        error!("failed to read volume: {}", err);
                    }
                }

                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            }
        });

        self.read_handler = Some(handler);
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

    fn read_volume() -> anyhow::Result<f32> {
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

        let mut size = mem::size_of::<f32>() as u32;

        let mut volume = 0.0f32;

        let result = unsafe {
            AudioObjectGetPropertyData(
                device_id,
                &address,
                0,
                std::ptr::null(),
                &mut size,
                &mut volume as *mut f32 as *mut std::ffi::c_void,
            )
        };

        if result != 0 {
            anyhow::bail!("Can not get audio property. result: {}", result);
        }

        if size != mem::size_of::<f32>() as u32 {
            anyhow::bail!("Can not get audio property. data size is not matched.");
        }

        log::debug!("current system volume of primary device: {}", volume);

        Ok(volume)
    }

    pub async fn get_volume(&self) -> f32 {
        self.current_volume.read().await.clone()
    }
}

impl Drop for VolumeManager {
    fn drop(&mut self) {
        log::info!("drop volume manager");
        if let Some(handler) = self.handler.take() {
            tokio::task::block_in_place(move || {
                handler.abort();
            });
        }

        if let Some(handler) = self.read_handler.take() {
            tokio::task::block_in_place(move || {
                handler.abort();
            });
        }
    }
}
