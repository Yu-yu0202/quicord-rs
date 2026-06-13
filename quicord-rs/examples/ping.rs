// SPDX-License-Identifier: CC0-1.0
use quicord_rs::macros::slash_command;
use quicord_rs::{BotBuilder, InteractionContext};

#[slash_command(name = "ping", description = "Replies with Pong!", scope = global)]
async fn ping(ctx: InteractionContext) -> anyhow::Result<()> {
    let embed = quicord_rs::core::interaction::EmbedBuilder::new()
        .title("Pong!")
        .description("This is a response from the Rust library.")
        .color(0x00FF00)
        .validate()?
        .build();

    let res = quicord_rs::core::interaction::InteractionResponseBuilder::new()
        .embeds([embed])
        .build();

    ctx.reply(res).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let bot = BotBuilder::new(std::env::var("DISCORD_TOKEN")?)
        .build()
        .await?;

    bot.start().await?;

    Ok(())
}
