extern crate self as quicord_rs_main;

pub mod command;
pub mod core;

pub use core::{Bot, BotBuilder, InteractionContext};

pub mod util;

pub use linkme;

pub mod log {
    pub use tracing::{debug, debug_span, error, error_span, info, info_span, trace, warn};
}
