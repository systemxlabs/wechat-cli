use std::path::PathBuf;

use clap::{ArgGroup, Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "wechat-cli")]
#[command(about = "Command-line client for WeChat iLink bot APIs")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Login(LoginArgs),
    Account(AccountArgs),
    GetContextToken(GetContextTokenArgs),
    Send(SendArgs),
}

#[derive(Debug, Args)]
pub struct LoginArgs {
    #[arg(long)]
    pub base_url: Option<String>,
}

#[derive(Debug, Args)]
pub struct AccountArgs {
    #[command(subcommand)]
    pub command: AccountCommand,
}

#[derive(Debug, Subcommand)]
pub enum AccountCommand {
    List,
}

#[derive(Debug, Args)]
pub struct GetContextTokenArgs {
    #[arg(long)]
    pub user_id: Option<String>,
}

#[derive(Debug, Args)]
#[command(group(
    ArgGroup::new("message")
        .args(["text", "file"])
        .required(true)
        .multiple(false)
))]
pub struct SendArgs {
    #[arg(long)]
    pub user_id: Option<String>,
    #[arg(long)]
    pub context_token: Option<String>,
    #[arg(long)]
    pub text: Option<String>,
    #[arg(long)]
    pub file: Option<PathBuf>,
    #[arg(long, requires = "file")]
    pub caption: Option<String>,
}
