mod cli;
mod commands;
mod errors;
mod storage;
mod wechat;

pub use errors::{Error, Result};

use anyhow::Result as AnyResult;
use clap::Parser;
use cli::{AccountCommand, Cli, Command};
use commands::login::{LoginOptions, login};

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
            AccountCommand::List => commands::account::print_accounts()?,
        },
        Command::GetContextToken(args) => {
            commands::get_context_token::run(args.user_id.as_deref()).await?;
        }
        Command::Send(args) => {
            commands::send::run(
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
