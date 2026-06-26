/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
use quicord_rs::{
    macros::{button, slash_command}, BotBuilder,
    InteractionContext,
};

#[slash_command(name = "show_button", description = "Show a button", scope = global)]
async fn show_button(ctx: InteractionContext) -> anyhow::Result<()> {
    let button = quicord_rs::core::interaction::ButtonBuilder::new(
        quicord_rs::core::interaction::ButtonStyle::Primary,
    )
    .label("Click me!")
    .custom_id("button_click")
    .build();

    let res = quicord_rs::core::interaction::InteractionResponseBuilder::new().components(vec![
        quicord_rs::core::interaction::ActionRowBuilder::new()
            .component(button)
            .build()
            .into(),
    ]);

    ctx.reply(res.build()).await?;

    Ok(())
}

#[button(custom_id = "button_click")]
async fn button_click(ctx: InteractionContext) -> anyhow::Result<()> {
    let res = quicord_rs::core::interaction::InteractionResponseBuilder::new()
        .content("You clicked the button!");

    ctx.reply(res.build()).await?;

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
