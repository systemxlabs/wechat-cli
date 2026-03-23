---
name: wechat-cli
description: Use the `wechat-cli` command-line tool to request login QR codes, poll QR-code status for non-interactive agent flows, inspect saved accounts, wait for an inbound message to obtain a `context_token`, and send text, image, or file messages. Trigger this skill when a task requires operating the local `wechat-cli` binary instead of editing its source code.
---

# wechat-cli

Use this skill when the task is to operate the local `wechat-cli` program.

## Preconditions

- Run commands in the repository root.
- Install the binary first if needed with `cargo install --path .`.
- Prefer invoking the installed command directly:
  `wechat-cli`
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

## Non-Interactive Login Workflow

1. Request a QR code:
   `wechat-cli qrcode`
2. Read `qrcode_id` and `qrcode_url` from the JSON output.
3. Show the QR code to the human outside the CLI.
4. Poll status:
   `wechat-cli qrcode-status --qrcode-id <qrcode_id>`
5. When `status` becomes `confirmed`, read `bot_token`, `bot_id`, and `user_id` from the JSON output.
6. Use those credentials directly. These commands do not save anything locally.

## Commands

### Request QR Code

```
wechat-cli qrcode
```

This prints JSON with:

- `qrcode_id`
- `qrcode_url`

### Query QR Code Status

```
wechat-cli qrcode-status --qrcode-id <qrcode_id>
```

This prints JSON with:

- `qrcode_id`
- `status`

When confirmed, it also includes:

- `bot_token`
- `bot_id`
- `user_id`

### List Accounts

```
wechat-cli account list
```

Use this to find saved account indexes and inspect `user_id` and `bot_id`.

### Get Context Token

```
wechat-cli get-context-token --user-id <user_id>
```

Behavior:

- The command waits for the next inbound message for that saved account.
- The token is printed to stdout.
- The token is not stored locally.

If `--user-id` is omitted, saved account index `0` is used.

### Send Text

```
wechat-cli send --user-id <user_id> --context-token <token> --text "hello"
```

### Send File

```
wechat-cli send --user-id <user_id> --context-token <token> --file <file_path>
```

### Send Image

```
wechat-cli send --user-id <user_id> --context-token <token> --file <image_path>
```

### Send File Or Image With Caption

```
wechat-cli send --user-id <user_id> --context-token <token> --file <file_path> --caption "caption"
```

## Account Selection Rules For `send`

Saved-account mode selection order:

1. `--account <index>`
2. `--user-id <user_id>`
3. default saved account index `0`

Examples:

```
wechat-cli send --account 0 --context-token <token> --text "hello"
```

```
wechat-cli send --user-id <user_id> --context-token <token> --text "hello"
```

## Explicit Credential Mode

Use this only when the task explicitly provides raw credentials instead of relying on saved accounts.

Required flags:

- `--token <token>`
- `--user-id <user_id>`
- `--context-token <token>`

Optional flags:

- `--route-tag <route_tag>`

Example:

```
wechat-cli send --token <bot_token> --user-id <user_id> --context-token <token> --text "hello"
```

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

```
wechat-cli --help
wechat-cli qrcode --help
wechat-cli qrcode-status --help
wechat-cli account --help
wechat-cli get-context-token --help
wechat-cli send --help
```
