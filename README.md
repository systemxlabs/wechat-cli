# wechat-cli

`wechat-cli` is a command-line client for the WeChat iLink Bot API.

It supports:

- QR code login
- Listing local accounts
- Waiting for the next incoming message to fetch `context_token`
- Sending text, images, and files

## Requirements

- Rust
- Access to `https://ilinkai.weixin.qq.com`
- A WeChat client that can scan the login QR code

## Build

```bash
cargo build
```

Binary:

```text
target/debug/wechat-cli
```

## Quick Start

Login:

```bash
wechat-cli login
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
wechat-cli send --user-id <user_id> --text "hello"
```

Send a file:

```bash
wechat-cli send --user-id <user_id> --file ./demo.pdf
```

Send an image:

```bash
wechat-cli send --user-id <user_id> --file ./demo.png
```

Send an image or file with a caption:

```bash
wechat-cli send --user-id <user_id> --file ./demo.png --caption "this is an image"
```

## Commands

### `login`

```bash
wechat-cli login [--base-url <base_url>]
```

Default `base_url`:

```text
https://ilinkai.weixin.qq.com
```

### `account list`

```bash
wechat-cli account list
```

### `get-context-token`

```bash
wechat-cli get-context-token [--user-id <user_id>]
```

### `send`

```bash
wechat-cli send [OPTIONS] <--text <TEXT>|--file <FILE>>
```

Options:

- `--user-id <USER_ID>`
- `--context-token <CONTEXT_TOKEN>`
- `--text <TEXT>`
- `--file <FILE>`
- `--caption <CAPTION>`

Rules:

- `--text` and `--file` are mutually exclusive
- `--caption` can only be used with `--file`
- Image files are sent as image messages automatically
- Other files are sent as file messages

## Workflow

Recommended flow:

1. Run `login`
2. Run `get-context-token --user-id <user_id>`
3. Send one message from the bound WeChat user to the bot
4. Run `send`

If no cached `context_token` is available, pass it explicitly:

```bash
wechat-cli send --user-id <user_id> --context-token <token> --text "hello"
```

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
~/.cache/wechat-cli/
```

Main files:

```text
~/.cache/wechat-cli/accounts.json
~/.cache/wechat-cli/get_updates_buf/<user_id>.txt
~/.cache/wechat-cli/contexts/<user_id>.json
```

Notes:

- `accounts.json` stores all accounts
- `contexts/<user_id>.json` stores the most recent session `context_token`
- Old storage layouts are not read for compatibility

## Limitations

- Sending requires a valid session `context_token`
- If the server returns `Session expired`, you must log in again
- The bot cannot proactively message arbitrary users
- Voice and video sending are not supported yet

## Help

```bash
wechat-cli --help
wechat-cli login --help
wechat-cli account --help
wechat-cli get-context-token --help
wechat-cli send --help
```
