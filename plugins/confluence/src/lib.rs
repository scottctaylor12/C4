use extism_pdk::*;
use serde::{Deserialize, Serialize};
use wasi::{clock_time_get, CLOCKID_REALTIME};

#[derive(Serialize, Deserialize)]
struct PluginInput {
    action: String,
    params: ConfluenceConfig,
}

#[derive(Serialize, Deserialize)]
struct ConfluenceConfig {
    agent_id: String,
    api_token: String,
    base_url: String,
    space: String,
    email: String,
    message: Option<String>, // For send action
}

#[derive(Serialize, Deserialize)]
struct PluginResponse {
    success: bool,
    status: String,
    messages: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
struct ConfluencePage {
    id: String,
    title: String,
    #[serde(rename = "type")]
    page_type: String,
    space: ConfluenceSpace,
    body: Option<ConfluenceBody>,
    version: Option<ConfluenceVersion>,
}

#[derive(Serialize, Deserialize)]
struct ConfluenceSpace {
    key: String,
}

#[derive(Serialize, Deserialize)]
struct ConfluenceBody {
    storage: ConfluenceStorage,
}

#[derive(Serialize, Deserialize)]
struct ConfluenceStorage {
    value: String,
    representation: String,
}

#[derive(Serialize, Deserialize)]
struct ConfluenceVersion {
    number: i32,
}

#[derive(Serialize, Deserialize)]
struct ConfluenceSearchResult {
    results: Vec<ConfluencePage>,
    size: i32,
}

#[derive(Serialize, Deserialize)]
struct CreatePageRequest {
    #[serde(rename = "type")]
    page_type: String,
    title: String,
    space: ConfluenceSpace,
    body: ConfluenceBody,
    ancestors: Option<Vec<ConfluenceAncestor>>,
}

#[derive(Serialize, Deserialize)]
struct ConfluenceAncestor {
    id: String,
}

#[plugin_fn]
pub fn c4(input: String) -> FnResult<String> {
    info!("Received input: '{}'", input);
    
    if input.is_empty() {
        return Err(WithReturnCode::new(Error::msg("Empty input received"), 1));
    }
    
    let plugin_input: PluginInput = serde_json::from_str(&input)
        .map_err(|e| WithReturnCode::new(Error::msg(format!("Invalid input format: {} | Input was: '{}'", e, input)), 1))?;

    match plugin_input.action.as_str() {
        "send" => handle_send_action(plugin_input.params),
        "receive" => handle_receive_action(plugin_input.params),
        _ => Err(WithReturnCode::new(Error::msg(format!("Unknown action: {}", plugin_input.action)), 1)),
    }
}

fn handle_send_action(config: ConfluenceConfig) -> FnResult<String> {
    let message = config.message.as_ref()
        .ok_or_else(|| WithReturnCode::new(Error::msg("Missing 'message' field for send action"), 1))?;

    info!("Sending message to Confluence space: {}, agent_id: {}", config.space, config.agent_id);

    // Step 1: Check if agent_id folder (parent page) exists
    let agent_folder_id = match find_agent_folder(&config)? {
        Some(id) => {
            info!("Found existing agent folder with ID: {}", id);
            id
        }
        None => {
            info!("Agent folder not found, creating new one");
            create_agent_folder(&config)?
        }
    };

    // Step 2: Create a new page under the agent folder with current epoch time as title
    let epoch_time = get_current_epoch_time();
    let page_title = epoch_time.to_string();
    
    info!("Creating new message page with title: {}", page_title);
    
    create_message_page(&config, &agent_folder_id, &page_title, message)?;

    let response = PluginResponse {
        success: true,
        status: format!("Successfully posted message to Confluence space '{}' under agent folder '{}'", config.space, config.agent_id),
        messages: None,
    };

    serde_json::to_string(&response)
        .map_err(|e| WithReturnCode::new(Error::msg(format!("Failed to serialize response: {}", e)), 1))
}

fn handle_receive_action(config: ConfluenceConfig) -> FnResult<String> {
    info!("Receiving messages from Confluence space: {}, agent_id: {}", config.space, config.agent_id);

    // Step 1: Find the agent folder
    let agent_folder_id = match find_agent_folder(&config)? {
        Some(id) => {
            info!("Found agent folder with ID: {}", id);
            id
        }
        None => {
            info!("No agent folder found for agent_id: {}", config.agent_id);
            let response = PluginResponse {
                success: true,
                status: "No messages found - agent folder does not exist".to_string(),
                messages: None,
            };
            return serde_json::to_string(&response)
                .map_err(|e| WithReturnCode::new(Error::msg(format!("Failed to serialize response: {}", e)), 1));
        }
    };

    // Step 2: Find all child pages (messages) under the agent folder
    let message_pages = find_message_pages(&config, &agent_folder_id)?;

    if message_pages.is_empty() {
        info!("No message pages found under agent folder");
        let response = PluginResponse {
            success: true,
            status: "No messages found".to_string(),
            messages: None,
        };
        return serde_json::to_string(&response)
            .map_err(|e| WithReturnCode::new(Error::msg(format!("Failed to serialize response: {}", e)), 1));
    }

    info!("Found {} message pages to process", message_pages.len());

    // Step 3: Read content from each message page and collect messages
    let mut messages = Vec::new();
    let mut failed_pages = Vec::new();

    for page in &message_pages {
        info!("Reading content from page: {} (ID: {})", page.title, page.id);
        
        match read_page_content(&config, &page.id) {
            Ok(content) => {
                info!("Successfully read content from page: {}", page.title);
                messages.push(content);
            }
            Err(e) => {
                info!("Failed to read content from page {}: {:?}", page.title, e);
                failed_pages.push(page.title.clone());
            }
        }
    }

    // Step 4: Delete the message pages after reading
    for page in &message_pages {
        if failed_pages.contains(&page.title) {
            info!("Skipping deletion of page {} due to read failure", page.title);
            continue;
        }

        info!("Deleting page: {} (ID: {})", page.title, page.id);
        match delete_page(&config, &page.id) {
            Ok(_) => {
                info!("Successfully deleted page: {}", page.title);
            }
            Err(e) => {
                info!("Warning: Failed to delete page {}: {:?}", page.title, e);
                // Don't fail the whole operation if delete fails, just log it
            }
        }
    }

    // Step 5: Return response
    let response = if messages.is_empty() {
        PluginResponse {
            success: false,
            status: format!("Failed to read any messages. Errors occurred with: {}", failed_pages.join(", ")),
            messages: None,
        }
    } else if failed_pages.is_empty() {
        PluginResponse {
            success: true,
            status: format!("Successfully received {} message(s)", messages.len()),
            messages: Some(messages),
        }
    } else {
        PluginResponse {
            success: true,
            status: format!("Received {} message(s), failed to read {} page(s): {}", 
                messages.len(), failed_pages.len(), failed_pages.join(", ")),
            messages: Some(messages),
        }
    };

    serde_json::to_string(&response)
        .map_err(|e| WithReturnCode::new(Error::msg(format!("Failed to serialize response: {}", e)), 1))
}

fn find_agent_folder(config: &ConfluenceConfig) -> FnResult<Option<String>> {
    // Ensure base_url includes /wiki if not already present
    let base_url = if config.base_url.ends_with("/wiki") {
        config.base_url.clone()
    } else {
        format!("{}/wiki", config.base_url)
    };
    
    let url = format!("{}/rest/api/content", base_url);
    // Use URL encoding for the title to handle special characters
    let encoded_title = url_encode(&config.agent_id);
    let query_params = format!("?spaceKey={}&title={}&type=page&limit=10", config.space, encoded_title);
    let full_url = format!("{}{}", url, query_params);
    
    info!("Searching for agent folder: {}", full_url);
    
    let auth_header = create_basic_auth_header(&config.email, &config.api_token);
    info!("Using auth header: Basic [REDACTED]");
    
    let request = HttpRequest::new(&full_url)
        .with_method("GET")
        .with_header("Authorization", &auth_header)
        .with_header("Accept", "application/json")
        .with_header("Content-Type", "application/json");

    let response = http::request::<()>(&request, None)
        .map_err(|e| WithReturnCode::new(Error::msg(format!("HTTP request failed: {:?}", e)), 1))?;

    info!("Search response status: {}", response.status());

    let response_body = response.body();
    let body = String::from_utf8(response_body)
        .map_err(|e| WithReturnCode::new(Error::msg(format!("Invalid UTF-8 response: {}", e)), 1))?;

    info!("Search response body: {}", body);

    // Handle status 0 (network/connection issues) or HTTP errors
    if response.status() == 0 {
        if body.contains("errorMessage") {
            return Err(WithReturnCode::new(Error::msg(format!("Network/Connection error. Check --allow-host setting and Confluence URL. Response: {}", body)), 1));
        } else if body.is_empty() {
            return Err(WithReturnCode::new(Error::msg("HTTP request failed - status 0 with empty body indicates network/connection failure. Check --allow-host and network connectivity."), 1));
        }
    }
    
    if response.status() == 403 {
        return Err(WithReturnCode::new(Error::msg(format!("Authentication failed (403). Check your API token, email, and Confluence permissions. Response: {}", body)), 1));
    }
    
    if response.status() >= 400 {
        return Err(WithReturnCode::new(Error::msg(format!("Failed to search for agent folder. Status: {} - Response: {}", response.status(), body)), 1));
    }

    // Try to parse as the expected format first
    match serde_json::from_str::<ConfluenceSearchResult>(&body) {
        Ok(search_result) => {
            info!("Successfully parsed search result with {} results", search_result.results.len());
            // Look for exact title match in results
            for result in search_result.results {
                info!("Checking page: '{}' with ID: {}", result.title, result.id);
                if result.title == config.agent_id {
                    info!("Found exact match for agent folder: {} with ID: {}", result.title, result.id);
                    return Ok(Some(result.id));
                }
            }
            info!("No exact title match found for agent_id: {}", config.agent_id);
            Ok(None)
        }
        Err(parse_err) => {
            info!("Failed to parse as ConfluenceSearchResult: {}", parse_err);
            // If parsing fails, try to parse as a generic JSON to understand the structure
            match serde_json::from_str::<serde_json::Value>(&body) {
                Ok(json_value) => {
                    info!("Unexpected JSON structure: {}", serde_json::to_string_pretty(&json_value).unwrap_or_else(|_| "Could not pretty print".to_string()));
                    
                    // Check if it has results array even if parsing failed
                    if let Some(results_array) = json_value.get("results").and_then(|v| v.as_array()) {
                        if results_array.is_empty() {
                            return Ok(None);
                        }
                        // Try to extract ID from results if they exist and title matches
                        for item in results_array {
                            if let (Some(id), Some(title)) = (
                                item.get("id").and_then(|v| v.as_str()),
                                item.get("title").and_then(|v| v.as_str())
                            ) {
                                info!("Found page in manual parsing: '{}' with ID: {}", title, id);
                                if title == config.agent_id {
                                    info!("Manual parsing found exact match: {} with ID: {}", title, id);
                                    return Ok(Some(id.to_string()));
                                }
                            }
                        }
                    }
                    
                    // Agent folder not found
                    Ok(None)
                }
                Err(json_err) => {
                    Err(WithReturnCode::new(Error::msg(format!("Failed to parse search response as JSON: {} | Response body: {}", json_err, body)), 1))
                }
            }
        }
    }
}

fn create_agent_folder(config: &ConfluenceConfig) -> FnResult<String> {
    // Ensure base_url includes /wiki if not already present
    let base_url = if config.base_url.ends_with("/wiki") {
        config.base_url.clone()
    } else {
        format!("{}/wiki", config.base_url)
    };
    
    let url = format!("{}/rest/api/content", base_url);
    
    let create_request = CreatePageRequest {
        page_type: "page".to_string(),
        title: config.agent_id.clone(),
        space: ConfluenceSpace {
            key: config.space.clone(),
        },
        body: ConfluenceBody {
            storage: ConfluenceStorage {
                value: format!("<p>Agent folder for: {}</p>", config.agent_id),
                representation: "storage".to_string(),
            },
        },
        ancestors: None, // Root level page in the space
    };

    let request_body = serde_json::to_string(&create_request)
        .map_err(|e| WithReturnCode::new(Error::msg(format!("Failed to serialize create request: {}", e)), 1))?;

    info!("Creating agent folder with body: {}", request_body);

    let auth_header = create_basic_auth_header(&config.email, &config.api_token);
    
    let request = HttpRequest::new(&url)
        .with_method("POST")
        .with_header("Authorization", &auth_header)
        .with_header("Accept", "application/json")
        .with_header("Content-Type", "application/json");

    let response = http::request(&request, Some(request_body.as_bytes()))
        .map_err(|e| WithReturnCode::new(Error::msg(format!("HTTP request failed: {:?}", e)), 1))?;

    info!("Create folder response status: {}", response.status());

    let response_body = response.body();
    let body = String::from_utf8(response_body)
        .map_err(|e| WithReturnCode::new(Error::msg(format!("Invalid UTF-8 response: {}", e)), 1))?;

    info!("Create folder response body: {}", body);

    // Handle status 0 (network/connection issues) or HTTP errors
    if response.status() == 0 {
        // Check if this is a "page already exists" error - if so, try to find the existing page
        if body.contains("A page with this title already exists") || body.contains("A page already exists with the same TITLE") {
            info!("Page already exists, trying to find existing agent folder");
            // Try to find the existing page again with a more thorough search
            return find_existing_agent_folder_by_search(config);
        } else if body.contains("errorMessage") {
            return Err(WithReturnCode::new(Error::msg(format!("Network/Connection error creating agent folder. Check --allow-host setting and Confluence URL. Response: {}", body)), 1));
        } else if body.is_empty() {
            return Err(WithReturnCode::new(Error::msg("HTTP request failed - status 0 with empty body indicates network/connection failure. Check --allow-host and network connectivity."), 1));
        }
    }

    if response.status() == 403 {
        return Err(WithReturnCode::new(Error::msg(format!("Authentication failed (403) creating agent folder. Check your API token, email, and Confluence permissions. Response: {}", body)), 1));
    }

    if response.status() == 400 {
        // Check if this is a "page already exists" error
        if body.contains("A page with this title already exists") || body.contains("A page already exists with the same TITLE") {
            info!("Page already exists (400 error), trying to find existing agent folder");
            return find_existing_agent_folder_by_search(config);
        }
    }

    if response.status() >= 400 {
        return Err(WithReturnCode::new(Error::msg(format!("Failed to create agent folder. Status: {} - Response: {}", response.status(), body)), 1));
    }

    let created_page: ConfluencePage = serde_json::from_str(&body)
        .map_err(|e| WithReturnCode::new(Error::msg(format!("Failed to parse create response: {}", e)), 1))?;

    Ok(created_page.id)
}

fn create_message_page(config: &ConfluenceConfig, parent_id: &str, title: &str, message: &str) -> FnResult<()> {
    // Ensure base_url includes /wiki if not already present
    let base_url = if config.base_url.ends_with("/wiki") {
        config.base_url.clone()
    } else {
        format!("{}/wiki", config.base_url)
    };
    
    let url = format!("{}/rest/api/content", base_url);
    
    let create_request = CreatePageRequest {
        page_type: "page".to_string(),
        title: title.to_string(),
        space: ConfluenceSpace {
            key: config.space.clone(),
        },
        body: ConfluenceBody {
            storage: ConfluenceStorage {
                value: format!("<p>{}</p>", html_escape(message)),
                representation: "storage".to_string(),
            },
        },
        ancestors: Some(vec![ConfluenceAncestor {
            id: parent_id.to_string(),
        }]),
    };

    let request_body = serde_json::to_string(&create_request)
        .map_err(|e| WithReturnCode::new(Error::msg(format!("Failed to serialize create request: {}", e)), 1))?;

    info!("Creating message page with body: {}", request_body);

    let auth_header = create_basic_auth_header(&config.email, &config.api_token);
    
    let request = HttpRequest::new(&url)
        .with_method("POST")
        .with_header("Authorization", &auth_header)
        .with_header("Accept", "application/json")
        .with_header("Content-Type", "application/json");

    let response = http::request(&request, Some(request_body.as_bytes()))
        .map_err(|e| WithReturnCode::new(Error::msg(format!("HTTP request failed: {:?}", e)), 1))?;

    info!("Create message response status: {}", response.status());

    let response_body = response.body();
    let body = String::from_utf8(response_body)
        .map_err(|e| WithReturnCode::new(Error::msg(format!("Invalid UTF-8 response: {}", e)), 1))?;

    info!("Create message response body: {}", body);

    if response.status() == 403 {
        return Err(WithReturnCode::new(Error::msg(format!("Authentication failed (403) creating message page. Check your API token, email, and Confluence permissions. Response: {}", body)), 1));
    }

    if response.status() >= 400 {
        return Err(WithReturnCode::new(Error::msg(format!("Failed to create message page. Status: {} - Response: {}", response.status(), body)), 1));
    }

    info!("Message page created successfully");
    Ok(())
}

fn create_basic_auth_header(email: &str, token: &str) -> String {
    let credentials = format!("{}:{}", email, token);
    let encoded = base64_encode(credentials.as_bytes());
    format!("Basic {}", encoded)
}

fn base64_encode(input: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    
    for chunk in input.chunks(3) {
        let mut buf = [0u8; 3];
        for (i, &byte) in chunk.iter().enumerate() {
            buf[i] = byte;
        }
        
        let b = ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32);
        
        result.push(CHARS[((b >> 18) & 63) as usize] as char);
        result.push(CHARS[((b >> 12) & 63) as usize] as char);
        result.push(if chunk.len() > 1 { CHARS[((b >> 6) & 63) as usize] as char } else { '=' });
        result.push(if chunk.len() > 2 { CHARS[(b & 63) as usize] as char } else { '=' });
    }
    
    result
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn get_current_epoch_time() -> u64 {
    unsafe {
        match clock_time_get(CLOCKID_REALTIME, 0) {
            Ok(nanoseconds) => nanoseconds / 1_000_000_000, // Convert to seconds
            Err(_) => {
                // Fallback to a reasonable timestamp
                1751313600 // Approximate current time in seconds
            }
        }
    }
}

fn find_existing_agent_folder_by_search(config: &ConfluenceConfig) -> FnResult<String> {
    // More comprehensive search when we know the page exists
    let base_url = if config.base_url.ends_with("/wiki") {
        config.base_url.clone()
    } else {
        format!("{}/wiki", config.base_url)
    };
    
    let url = format!("{}/rest/api/content", base_url);
    // Search without title filter first, then filter in results
    let query_params = format!("?spaceKey={}&type=page&limit=50", config.space);
    let full_url = format!("{}{}", url, query_params);
    
    info!("Searching all pages in space to find existing agent folder: {}", full_url);
    
    let auth_header = create_basic_auth_header(&config.email, &config.api_token);
    
    let request = HttpRequest::new(&full_url)
        .with_method("GET")
        .with_header("Authorization", &auth_header)
        .with_header("Accept", "application/json")
        .with_header("Content-Type", "application/json");

    let response = http::request::<()>(&request, None)
        .map_err(|e| WithReturnCode::new(Error::msg(format!("HTTP request failed: {:?}", e)), 1))?;

    info!("Comprehensive search response status: {}", response.status());

    let response_body = response.body();
    let body = String::from_utf8(response_body)
        .map_err(|e| WithReturnCode::new(Error::msg(format!("Invalid UTF-8 response: {}", e)), 1))?;

    info!("Comprehensive search response body preview: {}", &body[..std::cmp::min(500, body.len())]);

    if response.status() >= 400 {
        return Err(WithReturnCode::new(Error::msg(format!("Failed to search for existing agent folder. Status: {} - Response: {}", response.status(), body)), 1));
    }

    if let Ok(search_result) = serde_json::from_str::<ConfluenceSearchResult>(&body) {
        // Look for exact title match in results
        for result in search_result.results {
            info!("Found page: '{}' with ID: {}", result.title, result.id);
            if result.title == config.agent_id {
                info!("Found existing agent folder: {} with ID: {}", result.title, result.id);
                return Ok(result.id);
            }
        }
    }

    Err(WithReturnCode::new(Error::msg(format!("Could not find existing agent folder '{}' even though creation failed with 'already exists' error", config.agent_id)), 1))
}

fn url_encode(input: &str) -> String {
    // Simple URL encoding for query parameters
    input.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            c => format!("%{:02X}", c as u8),
        })
        .collect()
}

fn find_message_pages(config: &ConfluenceConfig, parent_id: &str) -> FnResult<Vec<ConfluencePage>> {
    // Get child pages of the agent folder
    let base_url = if config.base_url.ends_with("/wiki") {
        config.base_url.clone()
    } else {
        format!("{}/wiki", config.base_url)
    };
    
    let url = format!("{}/rest/api/content/{}/child/page", base_url, parent_id);
    let query_params = "?limit=100"; // Get up to 100 child pages
    let full_url = format!("{}{}", url, query_params);
    
    info!("Finding message pages under parent ID {}: {}", parent_id, full_url);
    
    let auth_header = create_basic_auth_header(&config.email, &config.api_token);
    
    let request = HttpRequest::new(&full_url)
        .with_method("GET")
        .with_header("Authorization", &auth_header)
        .with_header("Accept", "application/json")
        .with_header("Content-Type", "application/json");

    let response = http::request::<()>(&request, None)
        .map_err(|e| WithReturnCode::new(Error::msg(format!("HTTP request failed: {:?}", e)), 1))?;

    info!("Find message pages response status: {}", response.status());

    let response_body = response.body();
    let body = String::from_utf8(response_body)
        .map_err(|e| WithReturnCode::new(Error::msg(format!("Invalid UTF-8 response: {}", e)), 1))?;

    info!("Find message pages response body preview: {}", &body[..std::cmp::min(500, body.len())]);

    if response.status() == 403 {
        return Err(WithReturnCode::new(Error::msg(format!("Authentication failed (403) finding message pages. Check permissions. Response: {}", body)), 1));
    }

    if response.status() >= 400 {
        return Err(WithReturnCode::new(Error::msg(format!("Failed to find message pages. Status: {} - Response: {}", response.status(), body)), 1));
    }

    // Try to parse the response
    match serde_json::from_str::<ConfluenceSearchResult>(&body) {
        Ok(search_result) => {
            info!("Found {} child pages", search_result.results.len());
            Ok(search_result.results)
        }
        Err(parse_err) => {
            info!("Failed to parse as ConfluenceSearchResult: {}", parse_err);
            // Try manual parsing as fallback
            match serde_json::from_str::<serde_json::Value>(&body) {
                Ok(json_value) => {
                    if let Some(results_array) = json_value.get("results").and_then(|v| v.as_array()) {
                        let mut pages = Vec::new();
                        for item in results_array {
                            if let (Some(id), Some(title)) = (
                                item.get("id").and_then(|v| v.as_str()),
                                item.get("title").and_then(|v| v.as_str())
                            ) {
                                pages.push(ConfluencePage {
                                    id: id.to_string(),
                                    title: title.to_string(),
                                    page_type: "page".to_string(),
                                    space: ConfluenceSpace {
                                        key: config.space.clone(),
                                    },
                                    body: None,
                                    version: None,
                                });
                            }
                        }
                        info!("Manual parsing found {} child pages", pages.len());
                        Ok(pages)
                    } else {
                        Ok(Vec::new()) // No results found
                    }
                }
                Err(json_err) => {
                    Err(WithReturnCode::new(Error::msg(format!("Failed to parse message pages response as JSON: {} | Response body: {}", json_err, body)), 1))
                }
            }
        }
    }
}

fn read_page_content(config: &ConfluenceConfig, page_id: &str) -> FnResult<String> {
    // Get page content with body expansion
    let base_url = if config.base_url.ends_with("/wiki") {
        config.base_url.clone()
    } else {
        format!("{}/wiki", config.base_url)
    };
    
    let url = format!("{}/rest/api/content/{}", base_url, page_id);
    let query_params = "?expand=body.storage"; // Expand body to get content
    let full_url = format!("{}{}", url, query_params);
    
    info!("Reading page content from: {}", full_url);
    
    let auth_header = create_basic_auth_header(&config.email, &config.api_token);
    
    let request = HttpRequest::new(&full_url)
        .with_method("GET")
        .with_header("Authorization", &auth_header)
        .with_header("Accept", "application/json")
        .with_header("Content-Type", "application/json");

    let response = http::request::<()>(&request, None)
        .map_err(|e| WithReturnCode::new(Error::msg(format!("HTTP request failed: {:?}", e)), 1))?;

    info!("Read page content response status: {}", response.status());

    if response.status() >= 400 {
        let response_body = response.body();
        let body = String::from_utf8_lossy(&response_body);
        return Err(WithReturnCode::new(Error::msg(format!("Failed to read page content. Status: {} - Response: {}", response.status(), body)), 1));
    }

    let response_body = response.body();
    let body = String::from_utf8(response_body)
        .map_err(|e| WithReturnCode::new(Error::msg(format!("Invalid UTF-8 response: {}", e)), 1))?;

    // Extract content from the response
    match serde_json::from_str::<serde_json::Value>(&body) {
        Ok(json_value) => {
            // Navigate to body.storage.value to get the HTML content
            if let Some(content_html) = json_value
                .get("body")
                .and_then(|b| b.get("storage"))
                .and_then(|s| s.get("value"))
                .and_then(|v| v.as_str()) 
            {
                info!("Extracted HTML content: {}", &content_html[..std::cmp::min(100, content_html.len())]);
                // Convert HTML to plain text (simple approach - remove HTML tags)
                let plain_text = strip_html_tags(content_html);
                Ok(plain_text)
            } else {
                info!("No content found in page, response: {}", &body[..std::cmp::min(200, body.len())]);
                Ok(String::new()) // Return empty string if no content
            }
        }
        Err(parse_err) => {
            Err(WithReturnCode::new(Error::msg(format!("Failed to parse page content response: {} | Response body: {}", parse_err, body)), 1))
        }
    }
}

fn delete_page(config: &ConfluenceConfig, page_id: &str) -> FnResult<()> {
    let base_url = if config.base_url.ends_with("/wiki") {
        config.base_url.clone()
    } else {
        format!("{}/wiki", config.base_url)
    };
    
    let url = format!("{}/rest/api/content/{}", base_url, page_id);
    
    info!("Deleting page: {}", url);
    
    let auth_header = create_basic_auth_header(&config.email, &config.api_token);
    
    let request = HttpRequest::new(&url)
        .with_method("DELETE")
        .with_header("Authorization", &auth_header)
        .with_header("Accept", "application/json")
        .with_header("Content-Type", "application/json");

    let response = http::request::<()>(&request, None)
        .map_err(|e| WithReturnCode::new(Error::msg(format!("HTTP request failed: {:?}", e)), 1))?;

    info!("Delete page response status: {}", response.status());

    // 204 No Content is the expected success status for DELETE
    if response.status() >= 400 {
        let response_body = response.body();
        let body = String::from_utf8_lossy(&response_body);
        return Err(WithReturnCode::new(Error::msg(format!("Failed to delete page. Status: {} - Response: {}", response.status(), body)), 1));
    }

    Ok(())
}

fn strip_html_tags(html: &str) -> String {
    // Simple HTML tag removal - in a real implementation you might use a proper HTML parser
    let mut result = String::new();
    let mut inside_tag = false;
    
    for char in html.chars() {
        match char {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ => {
                if !inside_tag {
                    result.push(char);
                }
            }
        }
    }
    
    // Clean up extra whitespace and decode basic HTML entities
    result
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .trim()
        .to_string()
}