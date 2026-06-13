# quicord-rs

日本語版：[README.ja.md](/README.ja.md)

A Rust library for building Discord bots.

The goal of quicord-rs is to provide a simple and ergonomic API for creating Discord bots.

## Features

### Interactions

* [x] Slash Commands
    * [x] Command options
* [x] Message Context Commands
* [x] User Context Commands
* [ ] Buttons
* [ ] Select Menus
* [ ] Modals

### Macros

* [x] `#[quicord_rs::slash_command]`
* [x] `#[quicord_rs::message_context]`
* [x] `#[quicord_rs::user_context]`
* [ ] `#[quicord_rs::button]`
* [ ] `#[quicord_rs::select_menu]`
* [ ] `#[quicord_rs::modal]`

## Example

```rust
use quicord_rs::{macros::slash_command, BotBuilder, InteractionContext};

#[slash_command(name = "ping", description = "Replies with Pong!")]
async fn ping(ctx: InteractionContext) -> anyhow::Result<()> {
    ctx.reply("Pong!").await?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let bot = BotBuilder::new("YOUR_BOT_TOKEN")
        .add_command(ping)
        .build()
        .await?;

    bot.start().await?;
    Ok(())
}
```

## License

This project is licensed under the MPL-2.0 License. See the [LICENSE](/LICENSE) file for details.

The example code in the `examples/` directory is licensed under CC0-1.0. See the [LICENSE-CC0](/LICENSE-CC0) file for
details.
