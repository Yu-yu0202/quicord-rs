/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
use crate::core::client::tls;
use crate::util::logger::init_logger;
use crate::Bot;
use tokio::signal::unix::{signal, SignalKind};
use tracing::{debug, error, info};
use twilight_gateway::{EventTypeFlags, StreamExt};
use twilight_model::gateway::event::Event;
use twilight_model::gateway::CloseFrame;

impl Bot {
    /// Registers commands and runs the event loop on the current task.
    pub async fn start(mut self) -> anyhow::Result<()> {
        tls::init_rustls();
        init_logger();

        self.register_commands().await?;

        info!("Starting bot as Application ID {}...", self.application_id);
        self.event_loop().await
    }

    /// Registers commands and spawns the event loop on a background task.
    pub async fn spawn(mut self) -> anyhow::Result<()> {
        tls::init_rustls();
        init_logger();

        self.register_commands().await?;

        info!(
            "Spawning bot task as Application ID {}...",
            self.application_id
        );

        tokio::spawn(async move {
            if let Err(e) = self.event_loop().await {
                error!("Shard event loop terminated with error: {:?}", e);
            }
        });

        Ok(())
    }

    /// Runs the gateway event loop until termination or shutdown.
    async fn event_loop(&mut self) -> anyhow::Result<()> {
        info!("Starting event loop...");

        #[cfg(unix)]
        let mut sigint = signal(SignalKind::interrupt())?;
        #[cfg(unix)]
        let mut sigterm = signal(SignalKind::terminate())?;

        loop {
            tokio::select! {
                event = self.next_event() => {
                    match event {
                        Some(event) => {
                            debug!("Received event: {:?}", event.kind());

                            if let Some(handler) = self.route_event(&event) {
                                Self::handle_routed_event(
                                    self.client.clone(),
                                    self.storage.clone(),
                                    handler,
                                    event,
                                )
                                .await;
                            }
                        }
                        None => {
                            break;
                        }
                    }
                }

                _ = async {
                    #[cfg(unix)] { sigint.recv().await }
                    #[cfg(not(unix))] { tokio::signal::ctrl_c().await.ok() }
                } => {
                    info!("SIGINT (Ctrl+C) detected. Stopping bot...");
                    self.shard.close(CloseFrame::NORMAL);
                    break;
                }

                _ = async {
                    #[cfg(unix)] { sigterm.recv().await }
                    #[cfg(not(unix))] { std::future::pending::<()>().await }
                } => {
                    info!("SIGTERM detected. Stopping bot...");
                    self.shard.close(CloseFrame::NORMAL);
                    break;
                }
            }
        }

        info!("Stopped bot.");
        Ok(())
    }

    /// Retrieves the next gateway event, retrying transient receive errors.
    async fn next_event(&mut self) -> Option<Event> {
        loop {
            match self.shard.next_event(EventTypeFlags::all()).await {
                Some(Ok(event)) => return Some(event),
                Some(Err(e)) => error!("Error receiving event: {:?}", e),
                None => {
                    error!("Unexpected end of event stream");
                    return None;
                }
            }
        }
    }
}
