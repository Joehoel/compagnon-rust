mod commands;

use anyhow::Context as _;
use poise::serenity_prelude::Message;
use poise::{serenity_prelude as serenity, Event};
use rustrict::CensorStr;
use shuttle_poise::ShuttlePoise;
use shuttle_secrets::SecretStore;
use sqlx::PgPool;

use crate::commands::ping::*;
use crate::commands::register::*;

#[derive(Clone)]
pub struct Data {
    pub pool: PgPool,
}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(sqlx::FromRow)]
struct User {
    pub username: String,
}

async fn add_user(pool: PgPool, username: String) -> Result<User, Error> {
    println!("Adding user {}", username);

    let user = sqlx::query_as::<_, User>("INSERT INTO users (username) VALUES ($1) RETURNING id")
        .bind(username)
        .fetch_one(&pool)
        .await
        .expect("Failed to add user");

    Ok(user)
}

#[shuttle_runtime::main]
async fn poise(
    #[shuttle_shared_db::Postgres] pool: PgPool,
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> ShuttlePoise<Data, Error> {
    sqlx::migrate!()
        .run(&pool)
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    // Get the discord token set in `Secrets.toml`
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            event_handler: |_ctx, event, _framework, _data| {
                Box::pin(event_handler(_ctx, event, _framework))
            },
            commands: vec![ping(), register()],
            ..Default::default()
        })
        .token(discord_token)
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
        )
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { pool })
            })
        })
        .build()
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(framework.into())
}

async fn event_handler(
    _ctx: &serenity::Context,
    event: &Event<'_>,
    _framework: poise::FrameworkContext<'_, Data, Error>,
) -> Result<(), Error> {
    match event {
        Event::Ready { data_about_bot } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }
        Event::Message { new_message } => on_message(new_message),
        _ => {}
    }
    Ok(())
}

fn on_message(ctx: message: &Message) {
    println!("Message: {}", message.content);
}
