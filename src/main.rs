mod cli;
mod commands;
mod storage;
mod wechat;

use anyhow::Result as AnyResult;
use clap::Parser;
use cli::{AccountCommand, Cli, Command};
use commands::login::login;

#[tokio::main]
async fn main() -> AnyResult<()> {
    init_tracing();
    let cli = Cli::parse();

    match cli.command {
        Command::Login(_args) => {
            let user_id = login().await?;
            println!("logged in as user `{user_id}`");
        }
        Command::Qrcode(_args) => {
            commands::qrcode::print_qrcode().await?;
        }
        Command::QrcodeStatus(args) => {
            commands::qrcode::print_qrcode_status(&args.qrcode_id).await?;
        }
        Command::Account(args) => match args.command {
            AccountCommand::List => commands::account::print_accounts()?,
        },
        Command::GetContextToken(args) => {
            commands::get_context_token::run(args.user_id.as_deref()).await?;
        }
        Command::Send(args) => {
            commands::send::run(
                args.account,
                args.user_id.as_deref(),
                args.token.as_deref(),
                args.route_tag.as_deref(),
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
