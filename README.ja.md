# quicord-rs

`quicord-rs` は、Discord Bot を作成するための Rust クレートです。
シンプルかつ直感的に Discord Bot を開発できることを目指しています。

## 機能

### インタラクション

* [x] スラッシュコマンド
    * [x] オプションの取得
* [x] メッセージコンテキストコマンド
* [x] ユーザーコンテキストコマンド
* [ ] ボタン
* [ ] セレクトメニュー
* [ ] モーダル

### マクロ

* [x] `#[quicord_rs::slash_command]`
* [x] `#[quicord_rs::message_context]`
* [x] `#[quicord_rs::user_context]`
* [ ] `#[quicord_rs::button]`
* [ ] `#[quicord_rs::select_menu]`
* [ ] `#[quicord_rs::modal]`

## 使用例

```rust
use quicord_rs::{macros::{slash_command}, BotBuilder, InteractionContext};

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

## ライセンス

このプロジェクトは MPL-2.0 の下で公開されています。詳細は [LICENSE](/LICENSE) を参照してください。

ただし、`examples/` ディレクトリ内のコードは CC0-1.0 の下で公開されています。詳細は [LICENSE-CC0](/LICENSE-CC0) を参照してください。
