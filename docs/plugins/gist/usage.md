# Gist: Usage

## Plugin Overview

The plugin exposes a single function `c4` that accepts JSON input with two possible actions:

* `send` - Upload a message by adding a file a GitHub Gist
* `receive` - Get messages by reading files from GitHub Gists

## Installation

**Download Plugin**

Download the compiled WASM plugin (`gist.wasm`) from the releases section or build from source:

```bash
cargo build --target wasm32-wasip1 --release
```

**Extism Runtime Requirements**

The plugin requires:

* Extism runtime with WASI support enabled
* Network access to gist.github.com domain

## Schema Reference

### Send Action

Used to send a message by adding a file to a GitHub Gist

**Input Schema**

```json
{
  "action": "send",
  "agent_id": "string",      // Target agent identifier
  "message": "string",       // Command or payload content
  "params": {
    "api_key": "string"      // GitHub Personal Access Token (PAT)
  }
}
```

**Parameters**

* `agent_id` Unique identifier for target receiver ("server" or the agent id)
* `message` Command text or payload content (UTF-8 encoded, 1-10MB recommended)
* `params.api_key` GitHub Personal Access Token to programmatically interact with GitHub Gist

**Example Success Response**

```json
{
  "success": true,
  "status": "Message added to existing Gist!",
  "messages": null
}
```

### Receive Action

**Input Schema**

```json
{
  "action": "receive",
  "agent_id": "string",   // This agent's identifier
  "params": {
    "api_key": "string"   // GitHub Personal Access Token (PAT)
  }
}
```

**Parameters**

* `agent_id` messages of unique id to read ("server" or the agent id)
* `params.api_key` GitHub Personal Access Token to programmatically interact with GitHub Gist

**Success Response (with commands)**

```json
{
  "success": true,
  "status": "New messages!",
  "messages": [
    "whoami",
    "ps"
  ]
}
```

**Success Response (no commands)**

```json
{
  "success": true,
  "status": "No new messages",
  "messages": null
}
```