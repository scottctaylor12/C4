# Go

For team servers and/or agents written in Go, a simplistic example is included that can be used as a rough template.

## Usage

1. Install Go dependencies
```
go get github.com/extism/go-sdk
go get github.com/tetratelabs/wazero
```

2. The C4 plugin (.wasm file) should be placed in the same folder as the code as it will be compiled directly into the Go binary.

3. Compile the code using the standard Go binary
```
go build
```

Thorough documentation on using the Extism Go SDK can be found at <https://github.com/extism/go-sdk>

Below is an example of using a C4 plugin in Go. See the full example at: <https://github.com/scottctaylor12/C4/examples/go/>

## Example

```go
package main

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	_ "embed"

	extism "github.com/extism/go-sdk"
	"github.com/tetratelabs/wazero"
)

//go:embed c4.wasm
var wasmBytes []byte

type C4Output struct {
	Success  bool     `json:"success"`
	Status   string   `json:"status"`
	Messages []string `json:"messages"`
}

func main() {

	/*
		Example usage of the AWS S3 C4 plugin.
		Implementation details will vary based on your C2 and/or agent
	*/

	// For extra logging while debugging
	//extism.SetLogLevel(extism.LogLevelDebug)

	// load C4 plugin
	plugin := initialize()

	for {

		// Receive messages from AWS S3 bucket
		rec_msg := "{\"action\":\"receive\",\"params\":{\"agent_id\":\"12345\",\"access_key\":\"AKIAAAAAAAAAAAAA\",\"secret_key\":\"SECRET\",\"region\":\"us-east-1\",\"bucket\":\"c4-testing\"}}"
		exit, out, err := plugin.Call("c4", []byte(rec_msg))
		if err != nil {
			fmt.Println(err)
			fmt.Println("Exit Code: " + string(int(exit)))
		}
		var c4_output C4Output
		_ = json.Unmarshal(out, &c4_output)
		fmt.Println(c4_output.Messages)
		if (len(c4_output.Messages) > 0) && c4_output.Success {
			// Process the received messages
			fmt.Println(c4_output.Messages)
		}

		// let's pretend we received a "whoami" message
		// Send a response back to the S3 bucket with the "server" as the recipient
		var message string = "scottctaylor12" // realistically, the message is probably a format specific to your C2
		var send_msg string = fmt.Sprintf("{\"action\":\"send\",\"params\":{\"agent_id\":\"12345\",\"message\":\"%s\",\"access_key\":\"AKIAAAAAAAAAAAAA\",\"secret_key\":\"SECRET\",\"region\":\"us-east-1\",\"bucket\":\"c4-testing\"}}", message)
		exit, out, err = plugin.Call("c4", []byte(send_msg))
		if err != nil {
			fmt.Println(err)
			fmt.Println("Exit Code: " + string(int(exit)))
		}
		_ = json.Unmarshal(out, &c4_output)
		if c4_output.Success {
			fmt.Println("Message sent successfully")
		} else {
			fmt.Println("Failed to send message: " + c4_output.Status)
		}

		time.Sleep(10 * time.Second) // typical sleep time
	}
}

func initialize() *extism.Plugin {
	manifest := extism.Manifest{
		Wasm: []extism.Wasm{
			extism.WasmData{
				Data: wasmBytes,
			},
		},
		AllowedHosts: []string{
			"*",
		},
	}
	ctx := context.Background()
	moduleConfig := wazero.NewModuleConfig().
		WithSysWalltime().
		WithSysNanotime().
		WithSysNanosleep()
	config := extism.PluginConfig{
		ModuleConfig: moduleConfig,
		EnableWasi:   true,
	}
	plugin, err := extism.NewPlugin(ctx, manifest, config, []extism.HostFunction{})
	if err != nil {
		fmt.Println("Failed to initialize plugin")
	}
	return plugin
}
```