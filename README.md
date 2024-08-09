**Staff** is a toolkit for dead-simple usage out of the box promts and llms in your project and command line.

DRY & magic flow is a core principle.

Current state is WIP.

## Idea/Vision

Any good prompt combined with a llm is a pure magic. And you don't need to repeat yourself and find a good prompt, you can find well-prepared by community grimoires (combination of prompt, agents, functions & llms or only a prompt) then you learn this grimoire and can you use it on any text. Staff can help you with mastering your grimoires. Be brave in your path.

Structure of the Grimoire 0.1 version:

```md
---
title:
description:
tags: []
author:
---

[//]: # (SYSTEM)\n
Here any System prompt...

[//]: # (USER)\n
Here any User prompt...
```

You can use a grimoire with such packed of knowledge structure or it can be just a simple text without any metatags or SYSTEM/USER prompts. In that case everything is a SYSTEM prompt.

In the future version I want to add support for models & functions.

## Installation

You can install it from [pip](https://pypi.org/), [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)

```bash
pip install staff-cli
# cargo install staff-cli --locked # Not available yet. Yep, rust project can be installed only using pip yet :)
```

Or you can build staff-cli from source. You need to install rustup, clone the repository and then

```bash
cargo install --path ./crates/cli --locked
```

## CONFIGURATION

The Staff reads your configuration from .config. Example of configuration is config.example.toml.

Right now Staff supports Ollama models and GigaChat models. You must provide credentials for at least one model.

Read the file config.example.toml, write your configuration and copy it to $HOME/.config/staff-cli/config.toml

// Lin: Some(/home/alice/.config/staff-cli/config.toml)

// Win: Some(C:\Users\Alice\AppData\Roaming\staff-cli\config.toml)

// Mac: Some(/Users/Alice/Library/Application Support/staff-cli/config.toml)

## USAGE

How to get ideas from the youtube video? You need to have installed [youtube_transcript_api](https://github.com/jdepoix/youtube-transcript-api)

```bash
youtube_transcript_api TY0o3Zs8u78 --languages en --format text | tr '\n' ' ' | xargs cargo run -- cast -n extract_wisdom
```

## FOR CONTRIBUTORS

About GigaChat model:

On UBUNTU/DEBIAN we need to install libssl-dev

```bash
sudo apt-get install libssl-dev
```

Install certificates from `https://www.gosuslugi.ru/crt`
