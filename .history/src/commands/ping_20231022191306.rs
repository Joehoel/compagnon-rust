use crate::{Context, Error};

/// Pong!
#[poise::command(slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let message = ctx.reply("Pong!").await?;
    
    let timestamp = message.message().await?.timestamp.timestamp_millis();
    let other_timestamp = ctx.created_at().timestamp_millis();

    let ping = timestamp - other_timestamp;
    let reply = format!("Pong! `{}ms`", ping);

    message.edit(ctx, |m| m.content(reply)).await?;

    Ok(())
}
