# Confluence: Usage

## Plugin Overview

The plugin exposes a single function `c4` that accepts JSON input with two possible actions:

* `send` - Send a message by creating a new documentation page in Confluence containing your message
* `receive` - Get messages by reading documentation in Confluence. Pages will be deleted after receiving the message.

## Installation

**Download Plugin**

Download the compiled WASM plugin (`confluence.wasm`) from the releases section or build from source:

```bash
cargo build --target wasm32-wasip1 --release
```

**Extism Runtime Requirements**

The plugin requires:

* Extism runtime with WASI support enabled
* Network access to *.atlassian.net domain

## Schema Reference

### Send Action

Used to send a message by creating a new documentation page in Confluence.
The new Confluence page will be created in the specified `space` in a folder named after the `agent_id` specified.

**Input Schema**

```json
{
    "action": "send",
    "params": {
        "agent_id": "12345",                            // target agent identifier
        "api_token": "SECRET",                          // Atlassian API Key
        "base_url": "https://<DOMAIN>.atlassian.net",   // URL of Confluence site
        "space": "TEST",                                // Confluence documentation space
        "email": "example@example.com",                 // Atlassian username (email)
        "message": "test message"                       // message to send    
    }
}
```

**Parameters**

* `api_token` Atlassian API Token to programmatically interact with Confluence
* `agent_id` Unique identifier for target receiver ("server" or the agent id)
* `base_url` URL of the Confluence site to send/receive messages to/from
* `space` Confluence documentation space to create and delete documentation from
* `email` Atlassian username in the form of an email address (owner of the API Token)
* `message` Command text or payload content (UTF-8 encoded, 1-10MB recommended)

**Example Success Response**

```json
{
    "success": true, 
    "status": "Successfully received 1 message(s)", 
    "messages":["test message"]
}

```

### Receive Action

Used to receive new messages by searching for documentation in Confluence in the `agent_id`'s folder. 
After reading the message from the Confluence, the documentation page is deleted.

**Input Schema**

```json
{
    "action": "receive",
    "params": {
        "agent_id": "12345",                            // target agent identifier
        "api_token": "SECRET",                          // Atlassian API Key
        "base_url": "https://<DOMAIN>.atlassian.net",   // URL of Confluence site
        "space": "TEST",                                // Confluence documentation space
        "email": "example@example.com",                 // Atlassian username (email)
    }
}
```

**Parameters**

* `api_token` Atlassian API Token to programmatically interact with Confluence
* `agent_id` Unique identifier for target receiver ("server" or the agent id)
* `base_url` URL of the Confluence site to send/receive messages to/from
* `space` Confluence documentation space to create and delete documentation from
* `email` Atlassian username in the form of an email address (owner of the API Token)

**Success Response (with message)**

```json
{
  "success": true,
  "status": "Successfully received 1 message(s)",
  "messages": [
    "whoami",
    "ps"
  ]
}
```

**Success Response (no message)**

```json
{
  "success": true,
  "status": "No messages found",
  "messages": null
}
```