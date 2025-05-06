pub mod defs;

use std::collections::{BTreeMap, HashMap};
use wasi::{CLOCKID_REALTIME, clock_time_get};

use defs::*;
use extism_pdk::{*};
use json;

// main
#[plugin_fn]
pub fn c4(raw_input: String) -> FnResult<Json<Output>> {
    let input: Input = json::from_str(&raw_input).unwrap();
    let result: Result<Output, Error> = match input.action {
        Action::Receive => receive(input.params),
        Action::Send => send(input.params),
        Action::Custom(action) => Ok(Output {success: false, message: format!("Invalid action: {}", action), tasks: None}),
    };

    match result {
        Ok(result) => Ok(Json(result)),
        Err(e) => Ok(Json(Output {
            success: false,
            message: format!("Error: {}", e),
            tasks: None,
        })),
    }
}

fn receive(params: json::Value) -> Result<Output, Error> {
    let mut tasks: Vec<String> = Vec::new();

    // extract params from input
    let p: ReceiveParams = ReceiveParams {
        api_key: params["api_key"].as_str().ok_or(extism_pdk::Error::msg("Missing or invalid api_key".to_string()))?.to_string(),
        id: params["id"].as_str().ok_or(extism_pdk::Error::msg("Missing or invalid id".to_string()))?.to_string(),
    };
    
    // get all gists
    let gists: Vec<Gist> = get_gists(p.api_key.clone())?;

    info!("Fetched {} gists:", gists.len());
    for gist in gists.iter() {
        // check if gist exists for this agent
        if gist.description == p.id {
            let files: &json::Value = &gist.files;
            // check if files exist in agent's gist
            if let Some(map) = files.as_object() {
                let mut delete: String = "".to_string();
                for (key, value) in map {
                    // get content of file in gist
                    let req: HttpRequest = HttpRequest::new(format!("{}", value["raw_url"].as_str().unwrap().to_string()))
                        .with_header("Accept", "application/vnd.github+json")
                        .with_header("Authorization", format!("Bearer {}", p.api_key))
                        .with_header("X-GitHub-Api-Version", "2022-11-28");
                    let resp: HttpResponse = http::request::<()>(&req, None)
                        .unwrap();
                    let body_string: String = String::from_utf8(resp.body()).unwrap();
                    tasks.push(body_string);
                    delete = delete + format!("\"{}\":null,", key).as_str();
                }
                // Using 1 API call, delete the file(s) from the gist
                info!("{{\"files\":{{{}}}}}", delete);
                if delete.len() > 0 {
                    delete = delete[0..delete.len()-1].to_string();
                    info!("{{\"files\":{{{}}}}}", delete);
                    // delete file from gist now that we have the message
                    let req: HttpRequest = HttpRequest {
                        url: gist.url.clone(),
                        method: Some("PATCH".to_string()),
                        headers: BTreeMap::from([
                            ("Accept".to_string(), "application/vnd.github+json".to_string()),
                            ("Authorization".to_string(), format!("Bearer {}", p.api_key)),
                            ("X-GitHub-Api-Version".to_string(), "2022-11-28".to_string()),
                        ]),
                    };
                    let resp: HttpResponse = http::request::<String>(
                        &req, 
                        Some(format!("{{\"files\":{{{}}}}}", delete)),
                        )
                        .unwrap();
                    let body_string: String = String::from_utf8(resp.body()).unwrap();
                    info!("Response: {}", body_string);
                }
            }
            else {
                return Ok(Output { success: true, message: "No new messages".to_string(), tasks: None})   
            }
        }
    }
    let msg_size = tasks.len();
    info!("{}", msg_size);
    if tasks.len() == 0 {
        return Ok(Output { success: true, message: "No new messages".to_string(), tasks: None})
    } else {
        return Ok(Output { success: true, message: "New messages!".to_string(), tasks: Some(tasks)})
    }
}

fn send(_params: json::Value) -> Result<Output, Error> {
    // extract params from input
    let p: SendParams = SendParams {
        api_key: _params["api_key"].as_str().ok_or(extism_pdk::Error::msg("Missing or invalid api_key".to_string()))?.to_string(),
        id: _params["id"].as_str().ok_or(extism_pdk::Error::msg("Missing or invalid id".to_string()))?.to_string(),
        message: _params["message"].as_str().ok_or(extism_pdk::Error::msg("Missing or invalid message".to_string()))?.to_string(),
    };

    let nodes_raw: String = var::get("nodes")?.unwrap_or_else(|| "{}".to_string());
    let mut nodes: HashMap<String, String> = json::from_str(&nodes_raw)
        .unwrap_or_else(|_| HashMap::new());

    // if the agent is not in the map, update map with current gists
    if !nodes.contains_key(&p.id) {
        let gists: Vec<Gist> = get_gists(p.api_key.clone())?;
        for gist in gists.iter() {
            nodes.insert(gist.description.clone(), gist.id.clone());
        }
        if !nodes.contains_key(&p.id) {
            // gist does not exist for this agent
            // create a new gist for this agent
            let req: HttpRequest = HttpRequest {
                url: "https://api.github.com/gists".to_string(),
                method: Some("POST".to_string()),
                headers: BTreeMap::from([
                    ("Accept".to_string(), "application/vnd.github+json".to_string()),
                    ("Authorization".to_string(), format!("Bearer {}", p.api_key)),
                    ("X-GitHub-Api-Version".to_string(), "2022-11-28".to_string()),
                ]),
            };
            let now_time: u64;
            unsafe {
                now_time = clock_time_get(CLOCKID_REALTIME, 0).unwrap();
            }
            let body: String = format!("{{\"description\": \"{}\", \"public\": false, \"files\": {{\"{}\": {{\"content\": \"{}\"}}}}}}",
                p.id, 
                now_time, 
                p.message
            );
            let resp: HttpResponse = http::request::<String>(&req, Some(body))
                .unwrap();
            //add node from response to map
            let body_string: String = String::from_utf8(resp.body()).unwrap();
            let gist: Gist = json::from_str(&body_string)
                .unwrap();
            nodes.insert(gist.description.clone(), gist.id.clone());
            // update the nodes variable
            var::set("nodes", json::to_string(&nodes).unwrap())?;
            return Ok(Output { success: true, message: "Gist created!".to_string(), tasks: None})
        }
    }
    // if the agent is in the map, patch the gist with the message
    let gist_id: String = nodes.get(&p.id).unwrap().to_string();
    let req: HttpRequest = HttpRequest {
        url: format!("https://api.github.com/gists/{}", gist_id),
        method: Some("PATCH".to_string()),
        headers: BTreeMap::from([
            ("Accept".to_string(), "application/vnd.github+json".to_string()),
            ("Authorization".to_string(), format!("Bearer {}", p.api_key)),
            ("X-GitHub-Api-Version".to_string(), "2022-11-28".to_string()),
        ]),
    };
    let now_time: u64;
            unsafe {
                now_time = clock_time_get(CLOCKID_REALTIME, 0).unwrap();
            }
    let body: String = format!(
        "{{\"files\": {{\"{}\": {{\"content\": \"{}\"}}}}}}",
        now_time,
        p.message
    );
    let _resp: HttpResponse = http::request::<String>(&req, Some(body))
        .unwrap();

    return Ok(Output { success: true, message: "Message added to existing Gist!".to_string(), tasks: None})
}

/*
fn file_chunk {
    // TODO: file chunking
}
*/

fn get_gists(api_key: String) -> Result<Vec<Gist>, Error> {
    let req: HttpRequest = HttpRequest::new("https://api.github.com/gists")
        .with_header("Accept", "application/vnd.github+json")
        .with_header("Authorization", format!("Bearer {}", api_key))
        .with_header("X-GitHub-Api-Version", "2022-11-28");
    let resp: HttpResponse = http::request::<()>(&req, None)
        .unwrap();
    let body_string: String = String::from_utf8(resp.body()).unwrap();
    let gists: Vec<Gist> = json::from_str(&body_string)
        .unwrap();
    Ok(gists)
}