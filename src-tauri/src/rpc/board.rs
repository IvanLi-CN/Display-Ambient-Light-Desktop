use std::{sync::Arc, time::Duration};

use paris::{error, info, warn};
use tokio::{io, net::UdpSocket, sync::RwLock, task::yield_now, time::timeout};

use crate::{rpc::DisplaySettingRequest, volume::{VolumeManager, self}};

use super::{BoardConnectStatus, BoardInfo, BoardMessageChannels};

#[derive(Debug)]
pub struct Board {
    pub info: Arc<RwLock<BoardInfo>>,
    socket: Option<Arc<UdpSocket>>,
    listen_handler: Option<tokio::task::JoinHandle<()>>,
    volume_changed_subscriber_handler: Option<tokio::task::JoinHandle<()>>,
    state_of_displays_changed_subscriber_handler: Option<tokio::task::JoinHandle<()>>,
}

impl Board {
    pub fn new(info: BoardInfo) -> Self {
        Self {
            info: Arc::new(RwLock::new(info)),
            socket: None,
            listen_handler: None,
            volume_changed_subscriber_handler: None,
            state_of_displays_changed_subscriber_handler: None,
        }
    }

    pub async fn init_socket(&mut self) -> anyhow::Result<()> {
        let info = self.info.clone();
        let info = info.read().await;
        let socket = UdpSocket::bind("0.0.0.0:0").await?;

        socket.connect((info.address, info.port)).await?;
        let socket = Arc::new(socket);
        self.socket = Some(socket.clone());

        let handler = tokio::spawn(async move {
            let mut buf = [0u8; 128];

            let board_message_channels = crate::rpc::channels::BoardMessageChannels::global().await;

            let display_setting_request_sender = board_message_channels
                .display_setting_request_sender
                .clone();
            let volume_setting_request_sender =
                board_message_channels.volume_setting_request_sender.clone();

            loop {
                match socket.try_recv(&mut buf) {
                    Ok(len) => {
                        log::info!("recv: {:?}", &buf[..len]);
                        if buf[0] == 3 {
                            let result =
                                display_setting_request_sender.send(DisplaySettingRequest {
                                    display_index: buf[1] as usize,
                                    setting: crate::rpc::DisplaySetting::Brightness(buf[2]),
                                });

                            if let Err(err) = result {
                                error!("send display setting request to channel failed: {:?}", err);
                            }
                        } else if buf[0] == 4 {
                            let result = volume_setting_request_sender.send(buf[1] as f32 / 100.0);
                            if let Err(err) = result {
                                error!("send volume setting request to channel failed: {:?}", err);
                            }
                        }
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        yield_now().await;
                        continue;
                    }
                    Err(e) => {
                        error!("socket recv error: {:?}", e);
                        break;
                    }
                }
            }
        });
        self.listen_handler = Some(handler);

        self.subscribe_volume_changed().await;
        self.state_of_displays_changed().await;

        Ok(())
    }

    async fn subscribe_volume_changed(&mut self) {
        let channel = BoardMessageChannels::global().await;
        let mut volume_changed_rx = channel.volume_changed_sender.subscribe();
        let info = self.info.clone();
        let socket = self.socket.clone();

        let handler = tokio::spawn(async move {
            loop {
                let volume: Result<f32, tokio::sync::broadcast::error::RecvError> = volume_changed_rx.recv().await;
                if let Err(err) = volume {
                    match err {
                        tokio::sync::broadcast::error::RecvError::Closed => {
                            log::error!("volume changed channel closed");
                            break;
                        },
                        tokio::sync::broadcast::error::RecvError::Lagged(_) => {
                            log::info!("volume changed channel lagged");
                            continue;
                        },
                    }
                }

                let volume = volume.unwrap();

                let info = info.read().await;
                if socket.is_none() || info.connect_status != BoardConnectStatus::Connected {
                    log::info!("board is not connected, skip send volume changed");
                    continue;
                }

                let socket = socket.as_ref().unwrap();

                let mut buf = [0u8; 2];
                buf[0] = 4;
                buf[1] = (volume * 100.0) as u8;

                if let Err(err) = socket.send(&buf).await {
                    log::warn!("send volume changed failed: {:?}", err);
                }
            }
        });

        let volume_manager = VolumeManager::global().await;
        let volume = volume_manager.get_volume().await;

        if let Some(socket) = self.socket.as_ref() {
            let buf = [4, (volume * 100.0) as u8];
            if let Err(err) = socket.send(&buf).await {
                log::warn!("send volume failed: {:?}", err);
            }
        } else {
            log::warn!("socket is none, skip send volume");
        }

        self.volume_changed_subscriber_handler = Some(handler);
    }

    async fn state_of_displays_changed(&mut self) {
        let channel: &BoardMessageChannels = BoardMessageChannels::global().await;
        let mut state_of_displays_changed_rx = channel.displays_changed_sender.subscribe();
        let info = self.info.clone();
        let socket = self.socket.clone();

        let handler = tokio::spawn(async move {
           loop {
                let states: Result<Vec<crate::display::DisplayState>, tokio::sync::broadcast::error::RecvError> = state_of_displays_changed_rx.recv().await;
                if let Err(err) = states {
                    match err {
                        tokio::sync::broadcast::error::RecvError::Closed => {
                            log::error!("state of displays changed channel closed");
                            break;
                        },
                        tokio::sync::broadcast::error::RecvError::Lagged(_) => {
                            log::info!("state of displays changed channel lagged");
                            continue;
                        },
                    }
                }

                let info = info.read().await;
                if socket.is_none() || info.connect_status != BoardConnectStatus::Connected {
                    log::info!("board is not connected, skip send state of displays changed");
                    continue;
                }

                let socket = socket.as_ref().unwrap();

                let mut buf = [0u8; 3];
                let states = states.unwrap();
                for (index, state) in states.iter().enumerate() {
                    buf[0] = 3;
                    buf[1] = index as u8;
                    buf[2] = state.brightness as u8;

                    log::info!("send state of displays changed: {:?}", &buf[..]);

                    if let Err(err) = socket.send(&buf).await {
                        log::warn!("send state of displays changed failed: {:?}", err);
                    }
                }

           }
        });

        self.state_of_displays_changed_subscriber_handler = Some(handler);
    }

    pub async fn send_colors(&self, buf: &[u8]) {
        let info = self.info.read().await;
        if self.socket.is_none() || info.connect_status != BoardConnectStatus::Connected {
            return;
        }

        let socket = self.socket.as_ref().unwrap();

        socket.send(buf).await.unwrap();
    }

    pub async fn check(&self) -> anyhow::Result<()> {
        let info = self.info.read().await;
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect((info.address, info.port)).await?;
        drop(info);

        let instant = std::time::Instant::now();

        socket.send(&[1]).await?;
        let mut buf = [0u8; 1];
        let recv_future = socket.recv(&mut buf);

        let check_result = timeout(Duration::from_secs(1), recv_future).await;
        let mut info = self.info.write().await;
        match check_result {
            Ok(_) => {
                let ttl = instant.elapsed();
                if buf == [1] {
                    info.connect_status = BoardConnectStatus::Connected;
                } else {
                    if let BoardConnectStatus::Connecting(retry) = info.connect_status {
                        if retry < 10 {
                            info.connect_status = BoardConnectStatus::Connecting(retry + 1);
                            info!("reconnect: {}", retry + 1);
                        } else {
                            info.connect_status = BoardConnectStatus::Disconnected;
                            warn!("board Disconnected: bad pong.");
                        }
                    } else if info.connect_status != BoardConnectStatus::Disconnected {
                        info.connect_status = BoardConnectStatus::Connecting(1);
                    }
                }
                info.ttl = Some(ttl.as_millis());
            }
            Err(_) => {
                if let BoardConnectStatus::Connecting(retry) = info.connect_status {
                    if retry < 10 {
                        info.connect_status = BoardConnectStatus::Connecting(retry + 1);
                        info!("reconnect: {}", retry + 1);
                    } else {
                        info.connect_status = BoardConnectStatus::Disconnected;
                        warn!("board Disconnected: timeout");
                    }
                } else if info.connect_status != BoardConnectStatus::Disconnected {
                    info.connect_status = BoardConnectStatus::Connecting(1);
                }

                info.ttl = None;
            }
        }

        info.checked_at = Some(std::time::SystemTime::now());

        Ok(())
    }
}

impl Drop for Board {
    fn drop(&mut self) {
        info!("board drop");

        if let Some(handler) = self.listen_handler.take() {
            handler.abort();
        }

        if let Some(handler) = self.volume_changed_subscriber_handler.take() {
            handler.abort();
        }

        if let Some(handler) = self.state_of_displays_changed_subscriber_handler.take() {
            handler.abort();
        }

    }
}
