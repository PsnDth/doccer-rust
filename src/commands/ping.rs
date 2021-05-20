use serenity::prelude::Context;
use serenity::model::prelude::Message;
use serenity::framework::standard::{
    CommandResult,
    macros::command,
};

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Pong!").await?;

    Ok(())
}