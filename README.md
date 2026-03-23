# wechat-cli

![License](https://img.shields.io/badge/license-MIT-blue.svg)
[![Crates.io](https://img.shields.io/crates/v/wechat-cli.svg)](https://crates.io/crates/wechat-cli)
[![Docs](https://docs.rs/wechat-cli/badge.svg)](https://docs.rs/wechat-cli/latest/)

`wechat-cli` is a command-line client for the WeChat iLink Bot API.

It supports both interactive terminal flows and machine-readable JSON output for automation and agent workflows.

It supports:

- QR code login
- Non-interactive QR code retrieval and status polling for agents
- Listing local accounts
- Waiting for the next incoming message to print `context_token`
- Sending text, images, and files

## Requirements

- Rust
- Access to `https://ilinkai.weixin.qq.com`
- A WeChat client that can scan the login QR code

## Install

```bash
cargo install wechat-cli
```

Install from the current repository checkout:

```bash
cargo install --path .
```

## Quick Start

Login:

```bash
wechat-cli login
```

Request a login QR code as JSON without saving anything locally:

```bash
wechat-cli qrcode
```

Query a QR code status as JSON without saving anything locally:

```bash
wechat-cli qrcode-status --qrcode-id <qrcode_id>
```

List local accounts:

```bash
wechat-cli account list
```

Wait for the next incoming user message and print `context_token`:

```bash
wechat-cli get-context-token --user-id <user_id>
```

Send a text message:

```bash
wechat-cli send --user-id <user_id> --context-token <token> --text "hello"
```

Send a file:

```bash
wechat-cli send --user-id <user_id> --context-token <token> --file ./demo.pdf
```

Send an image:

```bash
wechat-cli send --user-id <user_id> --context-token <token> --file ./demo.png
```

Send an image or file with a caption:

```bash
wechat-cli send --user-id <user_id> --context-token <token> --file ./demo.png --caption "this is an image"
```

## Commands

### `login`

```bash
wechat-cli login
```

### `account list`

```bash
wechat-cli account list
```

Each entry includes an `account` index that can be passed back into `wechat-cli send --account <index>`.

### `qrcode`

```bash
wechat-cli qrcode
```

Prints JSON like:

```json
{
  "qrcode_id": "...",
  "qrcode_url": "..."
}
```

The JSON is written to stdout and is intended for non-interactive consumers.

### `qrcode-status`

```bash
wechat-cli qrcode-status --qrcode-id <qrcode_id>
```

Prints JSON like:

```json
{
  "qrcode_id": "...",
  "status": "wait"
}
```

When the QR code is confirmed, the JSON also includes:

- `bot_token`
- `bot_id`
- `user_id`

The JSON is written to stdout and is intended for non-interactive consumers.

### `get-context-token`

```bash
wechat-cli get-context-token [--user-id <user_id>]
```

### `send`

```bash
wechat-cli send [OPTIONS] <--text <TEXT>|--file <FILE>>
```

Options:

- `--account <INDEX>`
- `--user-id <USER_ID>`
- `--token <TOKEN>`
- `--route-tag <ROUTE_TAG>`
- `--context-token <CONTEXT_TOKEN>` required
- `--text <TEXT>`
- `--file <FILE>`
- `--caption <CAPTION>`

Rules:

- `--account <index>` selects a saved account
- if `--account` and `--user-id` are both omitted, saved account index `0` is used
- `--token` switches `send` into explicit credential mode
- `--text` and `--file` are mutually exclusive
- `--caption` can only be used with `--file`
- Image files are sent as image messages automatically
- Other files are sent as file messages

## Workflow

Recommended flow:

1. Run `login`
2. Run `get-context-token --user-id <user_id>`
3. Send one message from the bound WeChat user to the bot
4. Copy the printed token
5. Run `send --context-token <token>`

Example:

```bash
wechat-cli send --account 0 --context-token <token> --text "hello"
```

Non-interactive agent flow:

1. Run `qrcode`
2. Show `qrcode_url` to the human or render it elsewhere
3. Poll `qrcode-status --qrcode-id <qrcode_id>`
4. When `status` becomes `confirmed`, read `bot_token`, `bot_id`, and `user_id` from stdout
5. Use those credentials directly, or run interactive `login` if you want local persistence

## IDs

- `user_id` ends with `@im.wechat`
- `bot_id` ends with `@im.bot`
- Re-login for the same `user_id` changes the bound `bot_id`
- The bot can only send messages to its currently bound `user_id`

Message direction:

- user -> bot: `from_user_id = user_id`, `to_user_id = bot_id`
- bot -> user: `to_user_id = user_id`

## Storage

Local files are stored under:

```text
~/.config/wechat-cli/
```

Main files:

```text
~/.config/wechat-cli/accounts.json
```

Notes:

- `accounts.json` stores all accounts
- `context_token` is not stored locally
- `get_updates_buf` is not stored locally
- `qrcode` and `qrcode-status` do not update local files
- Old storage layouts are not read for compatibility

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
