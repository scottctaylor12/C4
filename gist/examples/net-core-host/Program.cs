using System;

using Extism.Sdk;

class Program
{
    // https://extism.github.io/dotnet-sdk/api/Extism.Sdk.Manifest.html#Extism_Sdk_Manifest_AllowedHosts
    static void Main() 
    {
        Plugin plugin = Initialize();
        //var output = plugin.Call("c4", "{\"action\": \"get_gists\", \"params\": {\"api_key\": \"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\", \"agent_id\": \"12345\"}}");
        var output = plugin.Call("c4", "");
        Console.WriteLine(output);
        output = plugin.Call("c4", "");
        Console.WriteLine(output);
        output = plugin.Call("c4", "");
        Console.WriteLine(output);
        output = plugin.Call("c4", "");
        Console.WriteLine(output);
    }
    public static Plugin Initialize()
    {
        // TODO: figure out how to embed wasm file at compile time rather than read from file
        var manifest = new Manifest(new PathWasmSource("./c4.wasm"))
        {
            AllowedHosts = new List<string> {"*"}
        };

        //using var plugin = new Plugin(manifest, new HostFunction[] { }, withWasi: true);
        var plugin = new Plugin(manifest, new HostFunction[] { }, withWasi: true);
        return plugin;
    }
}