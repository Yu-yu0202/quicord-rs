/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */
use quicord_rs::{
    builder::button::ButtonBuilder, macros::{button, slash_command},
    BotBuilder,
    InteractionContext,
};

#[slash_command(name = "show_button", description = "Show a button", scope = global)]
async fn show_button(ctx: InteractionContext) -> anyhow::Result<()> {
    let button1 = ButtonBuilder::new(quicord_rs::builder::button::ButtonStyle::Primary)
        .label("Click me!")
        .custom_id("button_click")
        .into_row_component();

    let button2 = ButtonBuilder::new(quicord_rs::builder::button::ButtonStyle::Secondary)
        .label("Disable this button")
        .custom_id("button_disable")
        .into_row_component();

    let res = quicord_rs::core::interaction::InteractionResponseBuilder::new()
        .components(vec![button1, button2]);

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

#[button(custom_id = "button_disable")]
async fn button_disable(ctx: InteractionContext) -> anyhow::Result<()> {
    let res = quicord_rs::core::interaction::InteractionResponseBuilder::new()
        .content("This button is disabled!");

    let button = ButtonBuilder::new(quicord_rs::builder::button::ButtonStyle::Secondary)
        .label("Disabled")
        .custom_id("button_disable")
        .disabled(true)
        .into_row_component();

    ctx.update(res.components(vec![button])).await?;

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
