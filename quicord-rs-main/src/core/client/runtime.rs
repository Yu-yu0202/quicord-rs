/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use crate::Bot;
use crate::core::client::TaskShutdownPolicy;
use crate::core::client::tls;
use std::sync::Arc;
use tokio::sync::{Semaphore, watch};
use tokio::task::{JoinError, JoinHandle, JoinSet};
use tracing::{debug, error, info, warn};
use twilight_gateway::{EventTypeFlags, StreamExt, error::ReceiveMessageErrorType};
use twilight_model::gateway::CloseFrame;
use twilight_model::gateway::event::Event;

#[cfg(unix)]
use tokio::signal::unix::{Signal, SignalKind, signal};

#[derive(Clone, Copy, Debug)]
enum ShutdownCause {
    Interrupt,
    Terminate,
    Requested,
}

impl ShutdownCause {
    const fn label(self) -> &'static str {
        match self {
            Self::Interrupt => "SIGINT (Ctrl+C)",
            Self::Terminate => "SIGTERM",
            Self::Requested => "external request",
        }
    }
}

#[cfg(unix)]
struct ShutdownSignal {
    sigint: Signal,
    sigterm: Signal,
}

#[cfg(not(unix))]
struct ShutdownSignal;

impl ShutdownSignal {
    fn new() -> anyhow::Result<Self> {
        #[cfg(unix)]
        {
            Ok(Self {
                sigint: signal(SignalKind::interrupt())?,
                sigterm: signal(SignalKind::terminate())?,
            })
        }

        #[cfg(not(unix))]
        {
            Ok(Self)
        }
    }

    async fn wait(&mut self) -> ShutdownCause {
        #[cfg(unix)]
        {
            tokio::select! {
                _ = self.sigint.recv() => ShutdownCause::Interrupt,
                _ = self.sigterm.recv() => ShutdownCause::Terminate,
            }
        }

        #[cfg(not(unix))]
        {
            let _ = tokio::signal::ctrl_c().await;
            ShutdownCause::Interrupt
        }
    }
}

/// Handle for a bot event loop running on a background task.
pub struct BotTask {
    shutdown: watch::Sender<bool>,
    join: JoinHandle<anyhow::Result<()>>,
}

impl BotTask {
    /// Requests a graceful shutdown of the gateway event loop.
    pub fn shutdown(&self) {
        let _ = self.shutdown.send(true);
    }

    /// Waits until the background event loop terminates.
    pub async fn join(self) -> anyhow::Result<()> {
        self.join.await.map_err(anyhow::Error::from)?
    }
}

impl Bot {
    /// Performs the initialization shared by foreground and background runs.
    async fn prepare_runtime(&mut self) -> anyhow::Result<()> {
        tls::init_rustls();
        self.register_commands().await
    }

    /// Registers commands and runs the event loop on the current task.
    pub async fn start(mut self) -> anyhow::Result<()> {
        self.prepare_runtime().await?;

        info!("Starting bot as Application ID {}...", self.application_id);
        let (_shutdown, receiver) = watch::channel(false);
        self.event_loop(receiver).await
    }

    /// Registers commands and spawns the event loop on a background task.
    pub async fn spawn(mut self) -> anyhow::Result<BotTask> {
        self.prepare_runtime().await?;

        info!(
            "Spawning bot task as Application ID {}...",
            self.application_id
        );

        let (shutdown, receiver) = watch::channel(false);
        let shutdown_guard = shutdown.clone();
        let join = tokio::spawn(async move {
            let _shutdown_guard = shutdown_guard;
            let result = self.event_loop(receiver).await;
            if let Err(error) = &result {
                error!(?error, "Shard event loop terminated with error");
            }
            result
        });

        Ok(BotTask { shutdown, join })
    }

    /// Runs the gateway event loop until termination or shutdown.
    async fn event_loop(&mut self, mut shutdown: watch::Receiver<bool>) -> anyhow::Result<()> {
        info!("Starting event loop...");

        let semaphore = Arc::new(Semaphore::new(self.max_concurrent_handlers.get()));
        let mut tasks = JoinSet::new();
        let mut shutdown_signal = ShutdownSignal::new()?;

        loop {
            tokio::select! {
                completed = tasks.join_next(), if !tasks.is_empty() => {
                    if let Some(Err(error)) = completed {
                        log_task_error(error);
                    }
                }

                event = self.next_event(), if semaphore.available_permits() > 0 => {
                    match event {
                        Ok(Some(event)) => {
                            debug!("Received event: {:?}", event.kind());

                            if let Some(handler) = self.route_event(&event)
                                && let Some(task) = self.prepare_routed_event(handler, event)
                            {
                                let permit = semaphore
                                    .clone()
                                    .try_acquire_owned()
                                    .expect("a permit was available before task creation");
                                tasks.spawn(async move {
                                    let _permit = permit;
                                    task.run().await;
                                });
                            }
                        }
                        Ok(None) => {
                            break;
                        }
                        Err(error) => return Err(error),
                    }
                }

                cause = shutdown_signal.wait() => {
                    self.close_for_shutdown(cause);
                    break;
                }

                changed = shutdown.changed() => {
                    if changed.is_ok() && *shutdown.borrow() {
                        self.close_for_shutdown(ShutdownCause::Requested);
                        break;
                    }
                }
            }
        }

        finish_tasks(&mut tasks, self.task_shutdown_policy).await;

        info!("Stopped bot.");
        Ok(())
    }

    fn close_for_shutdown(&mut self, cause: ShutdownCause) {
        info!(reason = cause.label(), "Stopping bot...");
        self.shard.close(CloseFrame::NORMAL);
    }

    /// Retrieves the next gateway event, applying the configured reconnect policy.
    async fn next_event(&mut self) -> anyhow::Result<Option<Event>> {
        let mut reconnect_attempts = 0;

        loop {
            match self.shard.next_event(EventTypeFlags::all()).await {
                Some(Ok(event)) => return Ok(Some(event)),
                Some(Err(error)) => match error.kind() {
                    ReceiveMessageErrorType::Reconnect => {
                        reconnect_attempts += 1;
                        let Some(delay) = self
                            .gateway_retry_policy
                            .delay_after_attempt(reconnect_attempts)
                        else {
                            return Err(anyhow::anyhow!(
                                "gateway reconnection failed {} consecutive times",
                                reconnect_attempts
                            ));
                        };

                        warn!(
                            attempt = reconnect_attempts,
                            max_attempts = self.gateway_retry_policy.max_attempts().get(),
                            ?delay,
                            "Gateway reconnection failed; retrying after backoff"
                        );
                        tokio::time::sleep(delay).await;
                    }
                    kind => warn!(?kind, "Discarding malformed gateway event"),
                },
                None => {
                    error!("Unexpected end of event stream");
                    return Ok(None);
                }
            }
        }
    }
}

async fn finish_tasks(tasks: &mut JoinSet<()>, policy: TaskShutdownPolicy) {
    if policy == TaskShutdownPolicy::Abort {
        tasks.abort_all();
    }

    while let Some(result) = tasks.join_next().await {
        if let Err(error) = result {
            log_task_error(error);
        }
    }
}

fn log_task_error(error: JoinError) {
    if error.is_panic() {
        error!(?error, "Routed event task panicked");
    } else if !error.is_cancelled() {
        error!(?error, "Routed event task failed");
    }
}

#[cfg(test)]
mod tests {
    use super::super::GatewayRetryPolicy;
    use std::{num::NonZeroU32, time::Duration};

    #[test]
    fn gateway_retry_backoff_is_capped_and_finite() {
        let policy = GatewayRetryPolicy::new(
            Duration::from_millis(10),
            Duration::from_millis(25),
            NonZeroU32::new(3).unwrap(),
        );

        assert_eq!(
            policy.delay_after_attempt(1),
            Some(Duration::from_millis(10))
        );
        assert_eq!(
            policy.delay_after_attempt(2),
            Some(Duration::from_millis(20))
        );
        assert_eq!(
            policy.delay_after_attempt(3),
            Some(Duration::from_millis(25))
        );
        assert_eq!(policy.delay_after_attempt(4), None);
    }
}
