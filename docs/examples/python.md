# Python

For team servers and/or agents written in Python, a simplistic example is included that can be used as a rough template.

## Usage

1. Install Python dependencies
```
python3 -m pip install extism
```

2. The example below reads the C4 plugin (.wasm file) from the folder with the name `c4.wasm`

Thorough documentation on the Python Extism SDK package can be found at <https://github.com/extism/python-sdk>

The full example can be found at <https://github.com/scottctaylor12/C4/examples/python/>

## Examples

```python
import json
import time
from dataclasses import dataclass
from typing import List, Optional, Tuple

import extism

@dataclass
class C4Output:
    success: bool
    status: str
    messages: Optional[List[str]] = None

def load_plugin():
    with open("c4.wasm", "rb") as f:
        wasm_bytes = f.read()
    manifest = {
        "wasm": [{"data": wasm_bytes}],
        "allowed_hosts": ["*"],
    }
    plugin = extism.Plugin(
        manifest,
        wasi=True,
    )
    return plugin
 
def main():

	# Example usage of the AWS S3 C4 plugin.
	# Implementation details will vary based on your C2 and/or agent

    plugin = load_plugin()
    
    while True:
        # This agent has an ID of 12345
        # Receive messages from AWS S3 bucket for agent 12345
        rec_msg = json.dumps({
            "action": "receive",
            "params": {
                "agent_id": "12345",
                "access_key": "AKIAXXXXXXXXXXXXX",
                "secret_key": "SECRET",
                "region": "us-east-1",
                "bucket": "c4-testing"
            }
        })

        try:
            out = plugin.call("c4", rec_msg.encode('utf-8'))
            c4_output_dict = json.loads(out.decode('utf-8'))
            c4_output = C4Output(**c4_output_dict)

            if c4_output.success and c4_output.messages and len(c4_output.messages) > 0:
                for message in c4_output.messages:
                    # Process the message as needed
                    print(f"Received message: {message}")
        
        except Exception as e:
            print(f"Error while receiving message: {e}")

        # Let's pretend we received a "whoami" message
        # Send a response back to the S3 bucket with the "server" as the recipient
        message = "scottctaylor12"  # realistically, the message is probably a format specific to your C2

        send_msg = json.dumps({
            "action": "send",
            "params": {
                "agent_id": "server",
                "message": message,
                "access_key": "AKIAXXXXXXXXXXXXX",
                "secret_key": "SECRET",
                "region": "us-east-1",
                "bucket": "c4-testing"
            }
        })

        try:
            out = plugin.call("c4", send_msg.encode('utf-8'))
            c4_output_dict = json.loads(out.decode('utf-8'))
            c4_output = C4Output(**c4_output_dict)
            
            if c4_output.success:
                print("Message sent successfully")
            else:
                print(f"Failed to send message: {c4_output.status}")
        
        except Exception as err:
            print(f"Error during send: {err}")

        time.sleep(5)

if __name__ == "__main__":
    main()
```