#!/bin/bash

extism call target/wasm32-unknown-unknown/release/c4.wasm c4 --input '{"action": "send", "params": {"api_key": "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", "id": "server", "message": "how about now?"}}'  --allow-host "*" --wasi --log-level info
