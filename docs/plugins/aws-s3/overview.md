# AWS S3

A WebAssembly (WASM) plugin for secure command and control (C2) communication using AWS S3. This plugin enables bidirectional communication between C2 servers and agents through S3 bucket operations, providing covert channels for offensive security operations.

## Overview

This plugin implements a secure, cloud-based message passing system where:

- **C2 Servers** send commands to agents using the `send` action  
- **Agents** retrieve and execute commands using the `receive` action  
- **Communication** is organized by agent ID for multi-target operations
- **Cleanup** is automatic - messages are deleted after retrieval

## Features

- **Covert Communication**: Uses legitimate AWS S3 traffic to blend with normal enterprise operations
- **Multi-Agent Support**: Organize communications by unique agent identifiers
- **Automatic Cleanup**: Messages are consumed (deleted) when received to minimize forensic artifacts
- **Cross-Platform**: Runs in any language supporting Extism (Python, Go, Rust, JavaScript, etc.)
- **Secure Authentication**: Uses AWS Signature V4 for cryptographically signed requests
- **Multi-Encoding Support**: Handles various text encodings (UTF-8, UTF-16 LE/BE)
