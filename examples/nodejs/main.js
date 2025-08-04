const fs = require('fs');
const extism = require('@extism/extism');

// Define the C4Output interface
class C4Output {
    constructor() {
        this.success = false;
        this.status = '';
        this.messages = [];
    }
}

async function main() {
    plugin = await load_plugin();

    while (true) {
        try {
            // This agent has an ID of 12345
            // Receive messages from AWS S3 bucket for agent 12345
            const recMsg = JSON.stringify({
                action: "receive",
                params: {
                    agent_id: "12345",
                    access_key: "AKIAXXXXXXXXXXX",
                    secret_key: "SECRET",
                    region: "us-east-1",
                    bucket: "c4-testing"
                }
            });

            const recOutput = await plugin.call("c4", recMsg);
            const c4OutputRec = JSON.parse(recOutput.text());
            if (c4OutputRec.success && c4OutputRec.messages && c4OutputRec.messages.length > 0) {
                // Process the received messages
                for (const message of c4OutputRec.messages) {
                    // Here you can add your logic to process each message
                    console.log("Processing message:", message);
                }
            }
        } 
        catch (pluginError) {
            console.error("Plugin call error:", pluginError);
        }

        // Let's pretend we received a "whoami" message
        // Send a response back to the S3 bucket with the "server" as the recipient
        try {
            const message = "scottctaylor12"; // realistically, the message is probably a format specific to your C2
            const sendMsg = JSON.stringify({
                action: "send",
                params: {
                    agent_id: "server",
                    message: message,
                    access_key: "AKIAXXXXXXXXXXX",
                    secret_key: "SECRET",
                    region: "us-east-1",
                    bucket: "c4-testing"
                }
            });

            const sendOutput = await plugin.call("c4", sendMsg);
            const c4OutputSend = JSON.parse(sendOutput.text());
            if (c4OutputSend.success) {
                console.log("Message sent successfully:", c4OutputSend.status);
            } else {
                console.error("Failed to send message:", c4OutputSend.status);
            }
        }
        catch (sendError) {
            console.error("Error sending message:", sendError);
        }

        // Sleep for 5 seconds
        await new Promise(resolve => setTimeout(resolve, 5000));
    }
}

async function load_plugin() {
    const pluginPath = 'c4.wasm';
    if (!fs.existsSync(pluginPath)) {
        throw new Error(`Plugin file not found: ${pluginPath}`);
    }
    return extism.createPlugin(pluginPath, {
        runInWorker: true,
        allowedHosts: ["*"],
        useWasi: true,
    });
}

main().catch(err => {
    console.error("Plugin error:", err);
});
