mod account;
mod api;
mod bot;
mod cli;
mod context;
mod errors;
mod media;
mod send;
mod storage;
mod watch;

pub use errors::{Error, Result};

use crate::bot::{LoginOptions, login};
use anyhow::Result as AnyResult;
use clap::Parser;
use cli::{AccountCommand, Cli, Command};

#[tokio::main]
async fn main() -> AnyResult<()> {
    init_tracing();
    let cli = Cli::parse();

    match cli.command {
        Command::Login(args) => {
            let user_id = login(LoginOptions {
                base_url: args.base_url,
            })
            .await?;
            println!("logged in as user `{user_id}`");
        }
        Command::Account(args) => match args.command {
            AccountCommand::List => print_accounts()?,
        },
        Command::GetContextToken(args) => {
            watch::get_context_token(args.user_id.as_deref()).await?;
        }
        Command::Send(args) => {
            send::send(
                args.user_id.as_deref(),
                args.context_token.as_deref(),
                args.text.as_deref(),
                args.file.as_deref(),
                args.caption.as_deref(),
            )
            .await?;
        }
    }

    Ok(())
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "warn,wechat_cli=info".into()),
        )
        .with_target(false)
        .try_init();
}

fn print_accounts() -> AnyResult<()> {
    let accounts = account::list_accounts()?;
    if accounts.is_empty() {
        println!("no saved users");
        return Ok(());
    }

    for entry in accounts {
        let route_tag = entry
            .config
            .as_ref()
            .and_then(|config| config.route_tag.as_deref())
            .unwrap_or("-");
        println!("user_id: {}", entry.user_id);
        println!("bot_id: {}", entry.data.bot_id);
        println!("base_url: {}", entry.data.base_url);
        println!("saved_at: {}", entry.data.saved_at);
        println!("route_tag: {route_tag}");
        println!();
    }

    Ok(())
}
