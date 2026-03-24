---
name: wechat-cli
description: Use the `wechat-cli` command-line tool to log in a WeChat bot, manage accounts, and send text, image, or file messages to users.
---

# wechat-cli

## Install
```
cargo install wechat-cli
```

## Login

Use non-interactive login workflow:

1. Request a QR code:
   ```bash
   wechat-cli qrcode
   ```
2. Read `qrcode_id` and `qrcode_url` from the JSON output.
3. Show the QR code to the human outside the CLI.
4. Poll status:
   ```bash
   wechat-cli qrcode-status --qrcode-id <qrcode_id>
   ```
5. When `status` becomes `confirmed`, read `bot_token` and `user_id` from the JSON output.
6. Use `wechat-cli account add` to save those credentials.

## Account

### List Accounts

```bash
wechat-cli account list
```

Use this to find saved account indexes and inspect `user_id`.

### Add Account

```bash
wechat-cli account add \
  --user-id <user_id> \
  --bot-token <bot_token> \
  [--route-tag <route_tag>]
```

### Delete Account

By index:

```bash
wechat-cli account delete --account <index>
```

By user ID:

```bash
wechat-cli account delete --user-id <user_id>
```

## Send

### Prerequisites

Before sending, you need a `context_token`.

### Send Text

```bash
wechat-cli send \
  [--account <index> | --user-id <user_id>] \
  --context-token <token> \
  --text "hello"
```

### Send Image

```bash
wechat-cli send \
  [--account <index> | --user-id <user_id>] \
  --context-token <token> \
  --file <image_path>
```

### Send File

```bash
wechat-cli send \
  [--account <index> | --user-id <user_id>] \
  --context-token <token> \
  --file <file_path>
```

### Send With Caption

```bash
wechat-cli send \
  [--account <index> | --user-id <user_id>] \
  --context-token <token> \
  --file <file_path> \
  --caption "caption text"
```

### Explicit Credential Mode

Use this only when the task explicitly provides raw credentials instead of relying on saved accounts.

```bash
wechat-cli send \
  --bot-token <bot_token> \
  --user-id <user_id> \
  --context-token <token> \
  [--route-tag <route_tag>] \
  --text "hello"
```

## Important Rules

- The bot is bound to exactly one `user_id`.
- Re-login for the same WeChat user rotates the bound bot credentials.
- The bot can only send messages to its currently bound `user_id`.
- `--context-token` is always required for `send`.
- `--text` and `--file` are mutually exclusive.
- `--caption` only works with `--file`.

## Local Storage

- Storage root: `~/.config/wechat-cli/`
- Accounts file: `~/.config/wechat-cli/accounts.json`

## Help

Use built-in help before guessing flags:

```bash
wechat-cli --help
wechat-cli qrcode --help
wechat-cli qrcode-status --help
wechat-cli account --help
wechat-cli send --help
```
