use quicord_rs::macros::slash_command;
use quicord_rs::{BotBuilder, InteractionContext};

#[slash_command(name = "add", description = "Adds two numbers together!", scope = global, options = [
    Integer("num1"),
    Integer("num2")
])]
async fn add(ctx: InteractionContext) -> anyhow::Result<()> {
    let num1 = ctx.required_option::<i64>("num1")?;
    let num2 = ctx.required_option::<i64>("num2")?;

    let sum = num1 + num2;
    let res = quicord_rs::core::interaction::InteractionResponseBuilder::new()
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
