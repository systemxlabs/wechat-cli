use std::path::PathBuf;

use clap::{ArgGroup, Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "wechat-cli")]
#[command(about = "Command-line client for WeChat iLink bot APIs")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Log in with a QR code and save the account locally.
    Login(LoginArgs),
    /// Request a login QR code and print it as JSON without saving anything locally.
    Qrcode(QrcodeArgs),
    /// Query a login QR code status and print it as JSON without saving anything locally.
    QrcodeStatus(QrcodeStatusArgs),
    /// Inspect saved accounts.
    Account(AccountArgs),
    /// Wait for the next inbound message and print its context token.
    GetContextToken(GetContextTokenArgs),
    /// Send a text, image, or file message.
    Send(SendArgs),
}

#[derive(Debug, Args)]
pub struct LoginArgs {}

#[derive(Debug, Args)]
pub struct QrcodeArgs {}

#[derive(Debug, Args)]
pub struct QrcodeStatusArgs {
    /// QR code ID returned by `wechat-cli qrcode`.
    #[arg(long)]
    pub qrcode_id: String,
}

#[derive(Debug, Args)]
pub struct AccountArgs {
    #[command(subcommand)]
    pub command: AccountCommand,
}

#[derive(Debug, Subcommand)]
pub enum AccountCommand {
    /// List saved accounts in index order.
    List,
    /// Add or overwrite a saved account.
    Add(AccountAddArgs),
    /// Delete a saved account.
    Delete(AccountDeleteArgs),
}

#[derive(Debug, Args)]
pub struct AccountAddArgs {
    /// WeChat user ID ending with `@im.wechat`.
    #[arg(long)]
    pub user_id: String,
    /// Bot token used to authenticate API requests.
    #[arg(long)]
    pub bot_token: String,
    /// Optional route tag saved for later requests.
    #[arg(long)]
    pub route_tag: Option<String>,
}

#[derive(Debug, Args)]
pub struct AccountDeleteArgs {
    /// Saved account index from `wechat-cli account list`.
    #[arg(long)]
    pub account: usize,
}

#[derive(Debug, Args)]
#[command(after_help = "Authentication Modes:
  1. Saved Account Mode:
     --account <index>
  
  2. Explicit Credentials Mode:
     --bot-token <token> --user-id <user_id> [--route-tag <tag>]

Usage Rules:
  - You must use exactly one of the modes above.
  - --account cannot be combined with --bot-token or --user-id.
  - In Explicit mode, both --bot-token and --user-id are required.")]
pub struct GetContextTokenArgs {
    #[arg(
        long,
        help = "Saved account index from `wechat-cli account list`. Required for Saved Account Mode."
    )]
    pub account: Option<usize>,
    #[arg(long, help = "Target user ID. Required in Explicit Credentials Mode.")]
    pub user_id: Option<String>,
    #[arg(
        long,
        help = "Explicit bot token. Required in Explicit Credentials Mode."
    )]
    pub bot_token: Option<String>,
    #[arg(
        long,
        help = "Optional route tag used only in Explicit Credentials Mode."
    )]
    pub route_tag: Option<String>,
}

#[derive(Debug, Args)]
#[command(group(
    ArgGroup::new("message")
        .args(["text", "file"])
        .required(true)
        .multiple(false)
))]
#[command(after_help = "Authentication Modes:
  1. Saved Account Mode:
     --account <index>
  
  2. Explicit Credentials Mode:
     --bot-token <token> --user-id <user_id> [--route-tag <tag>]

Usage Rules:
  - You must use exactly one of the modes above.
  - --account cannot be combined with --bot-token or --user-id.
  - In Explicit mode, both --bot-token and --user-id are required.
  - --context-token is always required for sending and must be provided explicitly.")]
pub struct SendArgs {
    #[arg(
        long,
        help = "Saved account index from `wechat-cli account list`. Required for Saved Account Mode."
    )]
    pub account: Option<usize>,
    #[arg(long, help = "Target user ID. Required in Explicit Credentials Mode.")]
    pub user_id: Option<String>,
    #[arg(
        long,
        help = "Explicit bot token. Required in Explicit Credentials Mode."
    )]
    pub bot_token: Option<String>,
    #[arg(
        long,
        help = "Optional route tag used only in Explicit Credentials Mode."
    )]
    pub route_tag: Option<String>,
    #[arg(
        long,
        help = "Context token from `get-context-token`. Always required."
    )]
    pub context_token: Option<String>,
    #[arg(long, help = "Plain text message body")]
    pub text: Option<String>,
    #[arg(
        long,
        help = "File path to send. Images are detected and sent as image messages."
    )]
    pub file: Option<PathBuf>,
    #[arg(long, requires = "file", help = "Optional caption for `--file`")]
    pub caption: Option<String>,
}
