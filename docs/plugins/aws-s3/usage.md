# Usage

## Plugin Overview

The plugin exposes a single function `c4` that accepts JSON input with two possible actions:

* `send` - Upload a text file containing a *message* to S3
* `receive` - Check S3 bucket for messages by id

## Installation

**Download Plugin**

Download the compiled WASM plugin (`aws_s3.wasm`) from the releases section or build from source:

```bash
cargo build --target wasm32-wasip1 --release
```

**Extism Runtime Requirements**

The plugin requires:

* Extism runtime with WASI support enabled
* Network access to *.amazonaws.com domains
* System time accuracy (within 15 minutes of AWS servers)


## API Reference

### Send Action

Used to send a message to the S3 bucket. Messages 

**Input Schema**

```json
{
  "action": "send",
  "params": {
    "agent_id": "string",      // Target agent identifier
    "message": "string",       // Command or payload content  
    "access_key": "string",    // AWS Access Key ID
    "secret_key": "string",    // AWS Secret Access Key
    "region": "string",        // AWS region (e.g. "us-east-1")
    "bucket": "string",        // S3 bucket name
  }
}
```

**Parameters**

* `agent_id` Unique identifier for target receiver ("server" or the agent id)
* `message` Command text or payload content (UTF-8 encoded, 1-10MB recommended)
* `access_key` AWS Access Key ID from IAM user
* `secret_key` AWS Secret Access Key from IAM user
* `region` AWS region where bucket is located
* `bucket` S3 bucket name for C2 communications

**Example Success Response**

```json
{
  "success": true,
  "status": "Successfully uploaded message to s3://c4-bucket/agent-007/1751313612345678900.txt",
  "messages": null
}
```

**Example Failure Response**

```json
{
  "success": false,
  "status": "Failed to upload file. S3 error: Access Denied",
  "messages": null
}
```

### Receive Action

**Input Scheme**

```json
{
  "action": "receive", 
  "params": {
    "agent_id": "string",      // This agent's identifier
    "access_key": "string",    // AWS Access Key ID
    "secret_key": "string",    // AWS Secret Access Key
    "region": "string",        // AWS region
    "bucket": "string",        // S3 bucket name
  }
}
```

**Parameters**

* `agent_id` messages of unique id to read ("server" or the agent id)
* `access_key` AWS Access Key ID from IAM user
* `secret_key` AWS Secret Access Key from IAM user
* `region` AWS region where bucket is located
* `bucket` S3 bucket name for C2 communications

**Success Response (with commands)**

```json
{
  "success": true,
  "status": "Successfully read 2 file(s)",
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
  "status": "No messages found",
  "messages": null
}
```