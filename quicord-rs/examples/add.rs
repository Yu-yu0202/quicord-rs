// SPDX-License-Identifier: CC0-1.0
use quicord_rs::macros::slash_command;
use quicord_rs::{BotBuilder, InteractionContext, builder::response::ResponseBuilder};

#[slash_command(name = "add", description = "Adds two numbers together!", scope = global, options = [
    Integer("num1"),
    Integer("num2")
])]
async fn add(ctx: InteractionContext) -> anyhow::Result<()> {
    let num1 = ctx.options().unwrap().integer("num1").unwrap_or(0i64);
    let num2 = ctx.options().unwrap().integer("num2").unwrap_or(0i64);

    let sum = num1 + num2;
    let res = ResponseBuilder::new()
        .content(format!("The sum of {} and {} is {}!", num1, num2, sum))
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
