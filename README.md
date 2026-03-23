# wechat-cli

![License](https://img.shields.io/badge/license-MIT-blue.svg)
[![Crates.io](https://img.shields.io/crates/v/wechat-cli.svg)](https://crates.io/crates/wechat-cli)
[![Docs](https://docs.rs/wechat-cli/badge.svg)](https://docs.rs/wechat-cli/latest/wechat_cli/)

A command-line client for the WeChat iLink Bot API.

## Features

- QR code login for interactive sessions
- Non-interactive QR code retrieval and status polling for agent workflows
- Add, list, and delete local accounts
- Wait for the next incoming message to obtain a `context_token`
- Send text, images, and files to WeChat users

## Installation

```bash
cargo install wechat-cli
```

## Usage

```
wechat-cli [COMMAND]
```

### Login

Interactive QR code login:

```bash
wechat-cli login
```

Non-interactive QR code workflow for agents:

```bash
# Request a QR code
wechat-cli qrcode
```

Output:
```json
{
  "qrcode_id": "...",
  "qrcode_url": "..."
}
```

```bash
# Poll QR code status
wechat-cli qrcode-status --qrcode-id <qrcode_id>
```

Output:
```json
{
  "qrcode_id": "...",
  "status": "wait"
}
```

When confirmed:
```json
{
  "qrcode_id": "...",
  "status": "confirmed",
  "bot_token": "...",
  "bot_id": "...",
  "user_id": "..."
}
```

### Account

List saved accounts:

```bash
wechat-cli account list
```

Add a new account:

```bash
wechat-cli account add \
  --user-id <user_id> \
  --bot-id <bot_id> \
  --token <bot_token> \
  [--route-tag <route_tag>]
```

Delete an account by index:

```bash
wechat-cli account delete --account <index>
```

Delete an account by user ID:

```bash
wechat-cli account delete --user-id <user_id>
```

### Send

Send a text message:

```bash
wechat-cli send \
  [--account <index> | --user-id <user_id>] \
  --context-token <token> \
  --text "hello"
```

Send an image:

```bash
wechat-cli send \
  [--account <index> | --user-id <user_id>] \
  --context-token <token> \
  --file ./image.png
```

Send a file:

```bash
wechat-cli send \
  [--account <index> | --user-id <user_id>] \
  --context-token <token> \
  --file ./document.pdf
```

Send a file with caption:

```bash
wechat-cli send \
  [--account <index> | --user-id <user_id>] \
  --context-token <token> \
  --file ./image.png \
  --caption "this is an image"
```

Use explicit credentials:

```bash
wechat-cli send \
  --token <bot_token> \
  --user-id <user_id> \
  --context-token <token> \
  [--route-tag <route_tag>] \
  --text "hello"
```

### Get Context Token

Wait for the next incoming message and print the `context_token`:

```bash
wechat-cli get-context-token [--user-id <user_id>]
```

## Account Selection Rules

For `send` command, the account is selected in this order:

1. `--account <index>` - explicit account index
2. `--user-id <user_id>` - explicit user ID
3. default saved account index `0`

## IDs

- `user_id` ends with `@im.wechat`
- `bot_id` ends with `@im.bot`
- Re-login for the same `user_id` changes the bound `bot_id`
- The bot can only send messages to its currently bound `user_id`

## Storage

Local files are stored under:

```
~/.config/wechat-cli/
```

Main file:

```
~/.config/wechat-cli/accounts.json
```

Notes:

- `accounts.json` stores all accounts
- `context_token` is not stored locally
- `qrcode` and `qrcode-status` do not update local files

## Limitations

- Sending requires explicitly passing a valid `--context-token`
- If the server returns `Session expired`, you must log in again
- The bot cannot proactively message arbitrary users
- Voice and video sending are not supported yet

## Help

```bash
wechat-cli --help
wechat-cli login --help
wechat-cli qrcode --help
wechat-cli qrcode-status --help
wechat-cli account --help
wechat-cli get-context-token --help
wechat-cli send --help
```
