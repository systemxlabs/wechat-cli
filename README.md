# wechat-cli

![License](https://img.shields.io/badge/license-MIT-blue.svg)
[![Crates.io](https://img.shields.io/crates/v/wechat-cli.svg)](https://crates.io/crates/wechat-cli)

A CLI tool to interact with a Wechat iLink bot.

## Features

- Interactive QR code login for human
- Non-interactive QR code retrieval and status polling for agents 
- Manage multi accounts
- Get `context_token`
- Send text, images, and files to WeChat users

## Installation

```bash
cargo install wechat-cli
```

## Usage

```
Usage: wechat-cli <COMMAND>

Commands:
  login              Log in with a QR code and save the account locally
  qrcode             Request a login QR code and print it as JSON without saving anything locally
  qrcode-status      Query a login QR code status and print it as JSON without saving anything locally
  account            Inspect saved accounts
  get-context-token  Wait for the next inbound message and print its context token
  send               Send a text, image, or file message
  help               Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print helpt-cli [COMMAND]
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

# Poll QR code status
wechat-cli qrcode-status --qrcode-id <qrcode_id>
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
  --bot-token <bot_token> \
  [--route-tag <route_tag>]
```

Delete an account by index:

```bash
wechat-cli account delete --account <index>
```

### Get Context Token

Wait for the next incoming message and print the `context_token`.

Using a saved account:

```bash
wechat-cli get-context-token --account <index>
```

Using explicit credentials:

```bash
wechat-cli get-context-token \
  --bot-token <bot_token> \
  --user-id <user_id> \
  [--route-tag <route_tag>]
```

### Send

Send a text message using a saved account:

```bash
wechat-cli send \
  --account <index> \
  --context-token <token> \
  --text "hello"
```

Send an image using a saved account:

```bash
wechat-cli send \
  --account <index> \
  --context-token <token> \
  --file ./image.png
```

Send a file with caption using a saved account:

```bash
wechat-cli send \
  --account <index> \
  --context-token <token> \
  --file ./image.png \
  --caption "this is an image"
```

Use explicit credentials (without saved account):

```bash
wechat-cli send \
  --bot-token <bot_token> \
  --user-id <user_id> \
  --context-token <token> \
  [--route-tag <route_tag>] \
  --text "hello"
```

## Storage

Local files are stored under:

```
~/.config/wechat-cli/
```

Main file:

```
~/.config/wechat-cli/accounts.json
```
