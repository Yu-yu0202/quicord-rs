/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use quicord_rs::{
    BotBuilder, InteractionContext,
    builder::{modal::ModalBuilder, text_input::TextInputBuilder},
    macros::{modal, slash_command},
};

#[slash_command(name = "show_modal", description = "Show a modal", scope = global)]
async fn show_modal(ctx: InteractionContext) -> anyhow::Result<()> {
    let modal = ModalBuilder::new("modal_submit", "My Modal")
        .text_input(
            TextInputBuilder::short("input1")
                .label("Input 1")
                .placeholder("Enter something...")
                .required(true)
                .build(),
        )
        .text_input(
            TextInputBuilder::paragraph("input2")
                .label("Input 2")
                .placeholder("Enter more text...")
                .required(false)
                .build(),
        )
        .build();

    ctx.show_modal(modal).await?;

    Ok(())
}

#[modal(custom_id = "modal_submit")]
async fn modal_submit(ctx: InteractionContext) -> anyhow::Result<()> {
    let modal_data = ctx.modal().unwrap();

    let input1_value = modal_data.text("input1").unwrap_or("Failed to get value");
    let input2_value = modal_data.text("input2").unwrap_or("Failed to get value");

    let response_content = format!(
        "You submitted the modal!\nInput 1: {}\nInput 2: {}",
        input1_value, input2_value
    );

    ctx.reply(response_content).await?;

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
