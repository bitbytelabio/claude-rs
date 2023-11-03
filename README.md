# Claude AI - Rust ( Unofficial )

This project provides an unofficial API for Claude AI, allowing users to access and interact with Claude AI written in Rust. Inspired by [Claude-API](https://github.com/KoushikNavuluri/Claude-API/tree/main)

#### Current Version == 0.1.0

## Table of contents

- [Claude AI-API ( Unofficial )](#claude-ai-api--unofficial-) - [Current Version == 1.0.17 ( Added Timeouts,Faster Requests,File handling Fixed.. )](#current-version--1017--added-timeoutsfaster-requestsfile-handling-fixed-)
  - [Table of contents](#table-of-contents)
  - [Use Cases](#use-cases)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
  - [Usage](#usage)
  - [List All Conversations](#list-all-conversations)
  - [Send Message](#send-message)
  - [Send Message with attachment](#send-message-with-attachment)
  - [Delete Conversation](#delete-conversation)
  - [Chat Conversation History](#chat-conversation-history)
  - [Create New Chat](#create-new-chat)
  - [Reset All Conversations](#reset-all-conversations)
  - [Rename Chat](#rename-chat)
  - [Disclaimer](#disclaimer)

## Use Cases

    1. Console ChatBot ( Check in usecases folder for sample console chatbot )

    2. Discord Chatbot

    3. Many more can be done....

## Installation

Create a Cargo.toml file as example bellow:

```toml
[package]
name = "example"
version = "0.1.0"
edition = "2021"

[dependencies]
claude-rs = { git = "https://github.com/bitbytelabio/claude-rs.git" }
```

## Usage

## Disclaimer

This project provides an unofficial API for Claude AI and is not affiliated with or endorsed by Claude AI or Anthropic. Use it at your own risk.

Please refer to the official Claude AI documentation[https://claude.ai/docs] for more information on how to use Claude AI.
