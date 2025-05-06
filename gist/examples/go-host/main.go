package main

import (
	"context"
	"fmt"

	_ "embed"

	extism "github.com/extism/go-sdk"
)

//go:embed c4.wasm
var wasmBytes []byte

// type C4Output struct {
// 	Success bool     `json:"success"`
// 	Message string   `json:"message"`
// 	Tasks   []string `json:"tasks"`
// }

func main() {
	plugin := initialize()
	exit, out, err := plugin.Call("c4", nil)
	if err != nil {
		fmt.Println(err)
		fmt.Println("Exit Code: " + string(int(exit)))
	}
	fmt.Println(string(out))

	exit, out, err = plugin.Call("c4", nil)
	if err != nil {
		fmt.Println(err)
		fmt.Println("Exit Code: " + string(int(exit)))
	}
	fmt.Println(string(out))

	exit, out, err = plugin.Call("c4", nil)
	if err != nil {
		fmt.Println(err)
		fmt.Println("Exit Code: " + string(int(exit)))
	}
	fmt.Println(string(out))

}

func initialize() *extism.Plugin {
	//extism call c4.wasm c4 --input "https://ifconfig.so" --allow-host "*" --wasi
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
	config := extism.PluginConfig{
		EnableWasi: true,
	}
	plugin, err := extism.NewPlugin(ctx, manifest, config, []extism.HostFunction{})
	if err != nil {
		fmt.Println("Failed to initialize plugin")
	}
	return plugin
	/*
		//data := []byte(`{"action": "get_gists", "params": {"api_key": "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", "agent_id": "12345"}}`)
		data := []byte("")
		exit, out, err := plugin.Call("c4", data)
		if err != nil {
			fmt.Println(err)
			fmt.Println("Exit Code: " + string(int(exit)))
		}
		//var c4_output C4Output
		//err = json.Unmarshal(out, &c4_output)
		fmt.Println(out)
	*/
}
