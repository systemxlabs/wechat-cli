---
name: wechat-cli
description: Use the `wechat-cli` command-line tool to log in to WeChat iLink Bot, inspect saved accounts, wait for an inbound message to obtain a `context_token`, and send text, image, or file messages. Trigger this skill when a task requires operating the local `wechat-cli` binary instead of editing its source code.
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
  - one logged-in bot account
  - the bound `user_id`
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

## Main Workflow

1. Run login:
   `wechat-cli login`
2. List saved accounts if needed:
   `wechat-cli account list`
3. Obtain a `context_token` by waiting for the next inbound message:
   `wechat-cli get-context-token --user-id <user_id>`
4. Ask the human to send one message from the bound WeChat user to the bot.
5. Copy the printed `context_token`.
6. Send a reply using that token.

## Commands

### Login

```
wechat-cli login
```

Optional custom base URL:

```
wechat-cli login --base-url <base_url>
```

### List Accounts

```
wechat-cli account list
```

Use this to find saved account indexes and inspect `user_id`, `bot_id`, and `base_url`.

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

- `--base-url <base_url>`
- `--route-tag <route_tag>`

Default base URL:

- `https://ilinkai.weixin.qq.com`

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
- If the server reports session expiration, log in again.

## Local Storage

- Storage root: `~/.config/wechat-cli/`
- Accounts file: `~/.config/wechat-cli/accounts.json`

Stored locally:

- saved accounts

Not stored locally:

- `context_token`
- `get_updates_buf`

## Help

Use built-in help before guessing flags:

```
wechat-cli --help
wechat-cli login --help
wechat-cli account --help
wechat-cli get-context-token --help
wechat-cli send --help
```
