use crate::command::{CommandHandler, scope::CommandScope};

pub struct UserContextCommandMetadata {
    pub name: &'static str,
    pub scope: CommandScope,
    pub run: CommandHandler,
}

#[linkme::distributed_slice]
pub static USER_CONTEXT_COMMANDS: [UserContextCommandMetadata];

pub struct MessageContextCommandMetadata {
    pub name: &'static str,
    pub scope: CommandScope,
    pub run: CommandHandler,
}

#[linkme::distributed_slice]
pub static MESSAGE_CONTEXT_COMMANDS: [MessageContextCommandMetadata];
