---
name: wechat-cli
description: Use the `wechat-cli` command-line tool to request login QR codes, poll QR-code status for non-interactive agent flows, manage accounts (add, list, delete), wait for an inbound message to obtain a `context_token`, and send text, image, or file messages. Trigger this skill when a task requires operating the local `wechat-cli` binary instead of editing its source code.
---

# wechat-cli

Use this skill when the task is to operate the local `wechat-cli` program.

## Preconditions

- Run commands in the repository root.
- Install the binary first if needed with `cargo install --path .`.
- Prefer invoking the installed command directly: `wechat-cli`
- Network access to `https://ilinkai.weixin.qq.com` is required.
- A valid send operation requires:
  - either one saved bot account or explicit credentials
  - the target `user_id`
  - a fresh `context_token`

## Identity Model

- `user_id` ends with `@im.wechat`.
- `bot_id` ends with `@im.bot`.
- The bot is bound to exactly one `user_id`.
- Re-login for the same WeChat user creates a new `bot_id`.
- The bot can only send messages to its currently bound `user_id`.

Message direction:

- user -> bot: `from_user_id = user_id`, `to_user_id = bot_id`
- bot -> user: `to_user_id = user_id`

## Login

Use non-interactive login workflow for agents that cannot display QR codes interactively:

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
5. When `status` becomes `confirmed`, read `bot_token`, `bot_id`, and `user_id` from the JSON output.
6. Use those credentials directly. These commands do not save anything locally.

## Account

### List Accounts

```bash
wechat-cli account list
```

Use this to find saved account indexes and inspect `user_id` and `bot_id`.

### Add Account

```bash
wechat-cli account add \
  --user-id <user_id> \
  --bot-id <bot_id> \
  --token <bot_token> \
  [--route-tag <route_tag>]
```

Requirements:

- `user_id` must end with `@im.wechat`
- `bot_id` must end with `@im.bot`
- `token` cannot be empty

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

Before sending, you need a `context_token`. Obtain it by:

```bash
wechat-cli get-context-token [--user-id <user_id>]
```

Behavior:

- The command waits for the next inbound message for that saved account.
- The token is printed to stdout.
- The token is not stored locally.

If `--user-id` is omitted, saved account index `0` is used.

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
  --token <bot_token> \
  --user-id <user_id> \
  --context-token <token> \
  [--route-tag <route_tag>] \
  --text "hello"
```

## Account Selection Rules For `send`

Saved-account mode selection order:

1. `--account <index>` - explicit account index
2. `--user-id <user_id>` - explicit user ID
3. default saved account index `0`

## Important Rules

- `--context-token` is always required for `send`.
- `--text` and `--file` are mutually exclusive.
- `--caption` only works with `--file`.
- Image files are sent as image messages automatically.
- Non-image files are sent as file messages.
- Do not assume a previously seen `bot_id` is still valid after re-login.
- If the server reports session expiration, obtain fresh credentials again.

## Local Storage

- Storage root: `~/.config/wechat-cli/`
- Accounts file: `~/.config/wechat-cli/accounts.json`

Stored locally:

- saved accounts

Not stored locally:

- `qrcode` output
- `qrcode-status` output
- `context_token`
- `get_updates_buf`

## Help

Use built-in help before guessing flags:

```bash
wechat-cli --help
wechat-cli login --help
wechat-cli qrcode --help
wechat-cli qrcode-status --help
wechat-cli account --help
wechat-cli get-context-token --help
wechat-cli send --help
```
