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
#[command(group(
    ArgGroup::new("selector")
        .args(["account", "user_id"])
        .required(true)
        .multiple(false)
))]
pub struct AccountDeleteArgs {
    /// Saved account index from `wechat-cli account list`.
    #[arg(long)]
    pub account: Option<usize>,
    /// Saved account user ID.
    #[arg(long)]
    pub user_id: Option<String>,
}

#[derive(Debug, Args)]
pub struct GetContextTokenArgs {
    /// Saved account user ID. If omitted, the first saved account is used.
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
#[command(after_help = "Saved account selection:
  1. --account <index>
  2. --user-id <user_id>
  3. default saved account index 0 if neither is provided

Explicit credentials mode:
  --bot-token <bot_token> --user-id <user_id> [--route-tag <route_tag>]

Rules:
  --account and --user-id cannot be used together in saved account mode
  --account cannot be used with explicit bot credential flags
  --context-token is always required and is never read from local cache")]
pub struct SendArgs {
    #[arg(
        long,
        help = "Saved account index from `wechat-cli account list`. If omitted together with `--user-id`, account index 0 is used"
    )]
    pub account: Option<usize>,
    #[arg(
        long,
        help = "Saved account user ID, or the target user ID when using explicit credentials"
    )]
    pub user_id: Option<String>,
    #[arg(long, help = "Explicit bot token. Requires `--user-id`")]
    pub bot_token: Option<String>,
    #[arg(
        long,
        help = "Optional explicit route tag header used with explicit credentials"
    )]
    pub route_tag: Option<String>,
    #[arg(
        long,
        help = "Context token printed by `get-context-token`. Always required for sending"
    )]
    pub context_token: Option<String>,
    #[arg(long, help = "Plain text message body")]
    pub text: Option<String>,
    #[arg(
        long,
        help = "File path to send. Image files are sent as image messages automatically"
    )]
    pub file: Option<PathBuf>,
    #[arg(long, requires = "file", help = "Optional caption for `--file`")]
    pub caption: Option<String>,
}
