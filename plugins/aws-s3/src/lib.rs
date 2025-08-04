use extism_pdk::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use hmac::{Hmac, Mac};
use hex;
use wasi::{clock_time_get, CLOCKID_REALTIME};

type HmacSha256 = Hmac<Sha256>;

#[derive(Serialize, Deserialize)]
struct PluginInput {
    action: String,
    params: S3Config,
}

#[derive(Serialize, Deserialize)]
struct S3Config {
    agent_id: String,
    access_key: String,
    secret_key: String,
    region: String,
    bucket: String,
    timestamp: Option<String>, // Optional timestamp override
    message: Option<String>, // For send action
}

#[derive(Serialize, Deserialize)]
struct S3Object {
    key: String,
    last_modified: String,
    size: i64,
    etag: String,
}

#[derive(Serialize, Deserialize)]
struct S3ListResponse {
    objects: Vec<S3Object>,
    is_truncated: bool,
    next_marker: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct PluginResponse {
    success: bool,
    status: String,
    messages: Option<Vec<String>>,
}

#[plugin_fn]
pub fn c4(input: String) -> FnResult<String> {
    // Debug: log the input we received
    info!("Received input: '{}'", input);
    info!("Input length: {}", input.len());
    
    if input.is_empty() {
        return Err(WithReturnCode::new(Error::msg("Empty input received"), 1));
    }
    
    let plugin_input: PluginInput = serde_json::from_str(&input)
        .map_err(|e| WithReturnCode::new(Error::msg(format!("Invalid input format: {} | Input was: '{}'", e, input)), 1))?;

    match plugin_input.action.as_str() {
        "receive" => handle_receive_action(plugin_input.params),
        "send" => handle_send_action(plugin_input.params),
        _ => Err(WithReturnCode::new(Error::msg(format!("Unknown action: {}", plugin_input.action)), 1)),
    }
}

fn handle_send_action(config: S3Config) -> FnResult<String> {
    let message = config.message.as_ref()
        .ok_or_else(|| WithReturnCode::new(Error::msg("Missing 'message' field for send action"), 1))?;

    // Create timestamp for AWS signature and filename
    let timestamp = config.timestamp.clone()
        .unwrap_or_else(|| get_current_timestamp_from_wasi());
    let date = &timestamp[0..8];

    // Generate nanosecond timestamp for filename
    let filename_timestamp = get_nanosecond_timestamp();
    let filename = format!("{}.txt", filename_timestamp);
    
    // AWS S3 endpoint for uploading file
    let host = format!("{}.s3.{}.amazonaws.com", config.bucket, config.region);
    let path = format!("/{}/{}", config.agent_id, filename);
    let url = format!("https://{}{}", host, path);
    
    info!("Uploading file to: s3://{}/{}/{}", config.bucket, config.agent_id, filename);

    // Convert message to bytes (UTF-8)
    let message_bytes = message.as_bytes();
    let payload_hash = format!("{:x}", Sha256::digest(message_bytes));

    // Create canonical request for PUT operation
    let canonical_request = create_canonical_request_for_put(&host, &path, &timestamp, &payload_hash);
    
    // Create string to sign
    let string_to_sign = create_string_to_sign(&timestamp, &config.region, &canonical_request);
    
    // Calculate signature
    let signature = calculate_signature(&config.secret_key, &date, &config.region, &string_to_sign);
    
    // Create authorization header
    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={}/{}/{}/s3/aws4_request,SignedHeaders=host;x-amz-content-sha256;x-amz-date,Signature={}",
        config.access_key, date, config.region, signature
    );

    // Make HTTP PUT request to upload file
    let request = HttpRequest::new(&url)
        .with_method("PUT")
        .with_header("Host", &host)
        .with_header("X-Amz-Date", &timestamp)
        .with_header("X-Amz-Content-Sha256", &payload_hash)
        .with_header("Authorization", &authorization);

    info!("Making PUT request to: {}", url);

    let response = http::request(&request, Some(message_bytes))
        .map_err(|e| WithReturnCode::new(Error::msg(format!("HTTP request failed: {:?}", e)), 1))?;

    info!("PUT response status: {}", response.status());

    // Check for HTTP error status
    if response.status() >= 400 {
        let response_body = response.body();
        let body = String::from_utf8_lossy(&response_body);
        let error_response = PluginResponse {
            success: false,
            status: format!("Failed to upload file. S3 error: {}", body),
            messages: None,
        };
        return serde_json::to_string(&error_response)
            .map_err(|e| WithReturnCode::new(Error::msg(format!("Failed to serialize response: {}", e)), 1));
    }

    // Success response
    let response = PluginResponse {
        success: true,
        status: format!("Successfully uploaded message to s3://{}/{}/{}", config.bucket, config.agent_id, filename),
        messages: None,
    };

    serde_json::to_string(&response)
        .map_err(|e| WithReturnCode::new(Error::msg(format!("Failed to serialize response: {}", e)), 1))
}

fn get_nanosecond_timestamp() -> u64 {
    unsafe {
        match clock_time_get(CLOCKID_REALTIME, 0) {
            Ok(nanoseconds) => nanoseconds,
            Err(_) => {
                // Fallback: use seconds * 1_000_000_000 + some pseudo-random component
                let seconds = get_current_timestamp_from_wasi();
                // Extract timestamp and convert to nanoseconds with some randomness
                1751313600000000000 + (seconds.len() as u64 * 123456789) // Simple fallback
            }
        }
    }
}

fn create_canonical_request_for_put(host: &str, path: &str, timestamp: &str, payload_hash: &str) -> String {
    let canonical_uri = path;
    let canonical_querystring = ""; // No query parameters for PUT
    let canonical_headers = format!("host:{}\nx-amz-content-sha256:{}\nx-amz-date:{}\n", host, payload_hash, timestamp);
    let signed_headers = "host;x-amz-content-sha256;x-amz-date";

    format!(
        "PUT\n{}\n{}\n{}\n{}\n{}",
        canonical_uri, canonical_querystring, canonical_headers, signed_headers, payload_hash
    )
}

fn handle_receive_action(config: S3Config) -> FnResult<String> {

    // AWS S3 endpoint with agent_id as prefix parameter
    let host = format!("{}.s3.{}.amazonaws.com", config.bucket, config.region);
    let path = "/"; // Root path
    let prefix = format!("{}/", config.agent_id); // Use agent_id as prefix
    
    // URL encode the prefix value for AWS signature calculation
    let encoded_prefix = url_encode(&prefix);
    let query_string = format!("prefix={}", encoded_prefix);
    let url = format!("https://{}{}?{}", host, path, query_string);
    
    info!("Listing files in S3 with prefix: s3://{}/{}/", config.bucket, config.agent_id);

    // Create timestamp for AWS signature
    let timestamp = config.timestamp.clone()
        .unwrap_or_else(|| get_current_timestamp_from_wasi());
    let date = &timestamp[0..8];

    // SHA256 hash of empty payload (for GET requests)
    let payload_hash = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

    // Create canonical request
    let canonical_request = create_canonical_request(&host, &path, &query_string, &timestamp, payload_hash);
    
    // Create string to sign
    let string_to_sign = create_string_to_sign(&timestamp, &config.region, &canonical_request);
    
    // Calculate signature
    let signature = calculate_signature(&config.secret_key, date, &config.region, &string_to_sign);
    
    // Create authorization header
    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={}/{}/{}/s3/aws4_request,SignedHeaders=host;x-amz-content-sha256;x-amz-date,Signature={}",
        config.access_key, date, config.region, signature
    );

    // Make HTTP request with all required headers
    let request = HttpRequest::new(&url)
        .with_method("GET")
        .with_header("Host", &host)
        .with_header("X-Amz-Date", &timestamp)
        .with_header("X-Amz-Content-Sha256", payload_hash)
        .with_header("Authorization", &authorization);

    info!("Making request to: {}", url);
    info!("Host header: {}", host);
    info!("X-Amz-Date header: {}", timestamp);
    info!("X-Amz-Content-Sha256 header: {}", payload_hash);
    info!("Authorization header: {}", authorization);

    let response = http::request::<()>(&request, None)
        .map_err(|e| WithReturnCode::new(Error::msg(format!("HTTP request failed: {:?}", e)), 1))?;

    info!("Response status: {}", response.status());
    info!("Response headers: {:?}", response.headers());
    
    // Check if we got a response body even if status is 0
    let response_body = response.body();
    let body_preview = String::from_utf8_lossy(&response_body[..std::cmp::min(1000, response_body.len())]);
    info!("Response body preview (first 1000 chars): {}", body_preview);

    // Status 0 might still have a valid response in some Extism versions
    if response.status() == 0 && response_body.is_empty() {
        return Err(WithReturnCode::new(
            Error::msg("HTTP request failed - status 0 with empty body indicates network/connection failure. Check --allow-host and network connectivity."),
            1,
        ));
    }

    // Check for HTTP error status (but allow 0 if we have response body)
    if response.status() >= 400 {
        let body = String::from_utf8_lossy(&response_body);
        return Err(WithReturnCode::new(
            Error::msg(format!("S3 request failed with status: {} - Response: {}", response.status(), body)),
            1,
        ));
    }

    // Parse XML response
    let response_body = response.body();
    let body = String::from_utf8(response_body)
        .map_err(|e| WithReturnCode::new(Error::msg(format!("Invalid UTF-8 response: {}", e)), 1))?;

    let s3_response = parse_s3_response(&body)
        .map_err(|e| WithReturnCode::new(Error::msg(format!("Failed to parse S3 response: {}", e)), 1))?;

    info!("Parsed {} objects from S3 response", s3_response.objects.len());
    for (i, obj) in s3_response.objects.iter().enumerate() {
        info!("Object {}: key='{}', size={}", i, obj.key, obj.size);
    }

    if s3_response.objects.is_empty() {
        let response = PluginResponse {
            success: true,
            status: "No messages found".to_string(),
            messages: None,
        };
        return serde_json::to_string(&response)
            .map_err(|e| WithReturnCode::new(Error::msg(format!("Failed to serialize response: {}", e)), 1));
    }

    // Read contents of each file and delete after reading
    let mut messages = Vec::new();
    let mut failed_files = Vec::new();

    for object in &s3_response.objects {
        info!("Reading file: {}", object.key);
        
        match read_s3_file(&config, &object.key, &timestamp, payload_hash) {
            Ok(content) => {
                messages.push(content);
                
                // Delete the file after successfully reading it
                info!("Deleting file after reading: {}", object.key);
                match delete_s3_file(&config, &object.key, &timestamp, payload_hash) {
                    Ok(_) => {
                        info!("Successfully deleted file: {}", object.key);
                    }
                    Err(e) => {
                        info!("Warning: Failed to delete file {}: {}", object.key, e);
                        // Don't fail the whole operation if delete fails, just log it
                    }
                }
            }
            Err(e) => {
                info!("Failed to read file {}: {}", object.key, e);
                failed_files.push(object.key.clone());
            }
        }
    }

    let response = if messages.is_empty() {
        PluginResponse {
            success: false,
            status: format!("Failed to read any files. Errors occurred with: {}", failed_files.join(", ")),
            messages: None,
        }
    } else if failed_files.is_empty() {
        PluginResponse {
            success: true,
            status: format!("Successfully read {} file(s)", messages.len()),
            messages: Some(messages),
        }
    } else {
        PluginResponse {
            success: true,
            status: format!("Read {} file(s), failed to read {} file(s): {}", 
                messages.len(), failed_files.len(), failed_files.join(", ")),
            messages: Some(messages),
        }
    };

    serde_json::to_string(&response)
        .map_err(|e| WithReturnCode::new(Error::msg(format!("Failed to serialize response: {}", e)), 1))
}

fn delete_s3_file(config: &S3Config, file_key: &str, timestamp: &str, payload_hash: &str) -> Result<(), String> {
    let host = format!("{}.s3.{}.amazonaws.com", config.bucket, config.region);
    let path = format!("/{}", file_key);
    let url = format!("https://{}{}", host, path);
    let date = &timestamp[0..8];

    // Create canonical request for DELETE operation
    let canonical_request = create_canonical_request_for_delete(&host, &path, timestamp, payload_hash);
    
    // Create string to sign
    let string_to_sign = create_string_to_sign(timestamp, &config.region, &canonical_request);
    
    // Calculate signature
    let signature = calculate_signature(&config.secret_key, date, &config.region, &string_to_sign);
    
    // Create authorization header
    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={}/{}/{}/s3/aws4_request,SignedHeaders=host;x-amz-content-sha256;x-amz-date,Signature={}",
        config.access_key, date, config.region, signature
    );

    // Make HTTP DELETE request
    let request = HttpRequest::new(&url)
        .with_method("DELETE")
        .with_header("Host", &host)
        .with_header("X-Amz-Date", timestamp)
        .with_header("X-Amz-Content-Sha256", payload_hash)
        .with_header("Authorization", &authorization);

    let response = http::request::<()>(&request, None)
        .map_err(|e| format!("HTTP DELETE request failed: {:?}", e))?;

    // Check for HTTP error status (204 No Content is success for DELETE)
    if response.status() >= 400 {
        let response_body = response.body();
        let body = String::from_utf8_lossy(&response_body);
        return Err(format!("S3 DELETE failed with status: {} - Response: {}", response.status(), body));
    }

    Ok(())
}

fn create_canonical_request_for_delete(host: &str, path: &str, timestamp: &str, payload_hash: &str) -> String {
    let canonical_uri = path;
    let canonical_querystring = ""; // No query parameters for DELETE
    let canonical_headers = format!("host:{}\nx-amz-content-sha256:{}\nx-amz-date:{}\n", host, payload_hash, timestamp);
    let signed_headers = "host;x-amz-content-sha256;x-amz-date";

    format!(
        "DELETE\n{}\n{}\n{}\n{}\n{}",
        canonical_uri, canonical_querystring, canonical_headers, signed_headers, payload_hash
    )
}

fn read_s3_file(config: &S3Config, file_key: &str, timestamp: &str, payload_hash: &str) -> Result<String, String> {
    let host = format!("{}.s3.{}.amazonaws.com", config.bucket, config.region);
    let path = format!("/{}", file_key);
    let url = format!("https://{}{}", host, path);
    let date = &timestamp[0..8];

    // Create canonical request for individual file
    let canonical_request = create_canonical_request(&host, &path, "", timestamp, payload_hash);
    
    // Create string to sign
    let string_to_sign = create_string_to_sign(timestamp, &config.region, &canonical_request);
    
    // Calculate signature
    let signature = calculate_signature(&config.secret_key, date, &config.region, &string_to_sign);
    
    // Create authorization header
    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={}/{}/{}/s3/aws4_request,SignedHeaders=host;x-amz-content-sha256;x-amz-date,Signature={}",
        config.access_key, date, config.region, signature
    );

    // Make HTTP request to get file contents
    let request = HttpRequest::new(&url)
        .with_method("GET")
        .with_header("Host", &host)
        .with_header("X-Amz-Date", timestamp)
        .with_header("X-Amz-Content-Sha256", payload_hash)
        .with_header("Authorization", &authorization);

    let response = http::request::<()>(&request, None)
        .map_err(|e| format!("HTTP request failed: {:?}", e))?;

    info!("File request status: {}", response.status());

    // Check for HTTP error status (but allow 0 status like we do for list operations)
    if response.status() >= 400 {
        let response_body = response.body();
        let body = String::from_utf8_lossy(&response_body);
        return Err(format!("S3 request failed with status: {} - Response: {}", response.status(), body));
    }

    // Get response body and debug its content
    let response_body = response.body();
    info!("File response body length: {} bytes", response_body.len());
    
    if response_body.is_empty() {
        return Err("Empty response from S3".to_string());
    }

    // Try to decode the content, handling different encodings
    let content = decode_file_content(&response_body)
        .map_err(|e| format!("Failed to decode file content: {}", e))?;

    Ok(content)
}

fn decode_file_content(bytes: &[u8]) -> Result<String, String> {
    if bytes.is_empty() {
        return Ok(String::new());
    }

    // Check for UTF-16 LE BOM (FF FE)
    if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
        info!("Detected UTF-16 LE encoding with BOM");
        return decode_utf16_le(&bytes[2..]); // Skip BOM
    }

    // Check for UTF-16 BE BOM (FE FF)
    if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
        info!("Detected UTF-16 BE encoding with BOM");
        return decode_utf16_be(&bytes[2..]); // Skip BOM
    }

    // Check for UTF-8 BOM (EF BB BF)
    if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
        info!("Detected UTF-8 encoding with BOM");
        return String::from_utf8(bytes[3..].to_vec()) // Skip BOM
            .map_err(|e| format!("Invalid UTF-8 after BOM: {}", e));
    }

    // Try UTF-8 without BOM
    match String::from_utf8(bytes.to_vec()) {
        Ok(content) => {
            info!("Successfully decoded as UTF-8");
            Ok(content)
        }
        Err(_) => {
            // If UTF-8 fails, try UTF-16 LE without BOM (common for Windows files)
            info!("UTF-8 failed, trying UTF-16 LE without BOM");
            decode_utf16_le(bytes)
        }
    }
}

fn decode_utf16_le(bytes: &[u8]) -> Result<String, String> {
    if bytes.len() % 2 != 0 {
        return Err("Invalid UTF-16 LE: odd number of bytes".to_string());
    }

    let mut utf16_chars = Vec::new();
    for chunk in bytes.chunks_exact(2) {
        let code_unit = u16::from_le_bytes([chunk[0], chunk[1]]);
        utf16_chars.push(code_unit);
    }

    String::from_utf16(&utf16_chars)
        .map_err(|e| format!("Invalid UTF-16 LE sequence: {}", e))
}

fn decode_utf16_be(bytes: &[u8]) -> Result<String, String> {
    if bytes.len() % 2 != 0 {
        return Err("Invalid UTF-16 BE: odd number of bytes".to_string());
    }

    let mut utf16_chars = Vec::new();
    for chunk in bytes.chunks_exact(2) {
        let code_unit = u16::from_be_bytes([chunk[0], chunk[1]]);
        utf16_chars.push(code_unit);
    }

    String::from_utf16(&utf16_chars)
        .map_err(|e| format!("Invalid UTF-16 BE sequence: {}", e))
}

fn url_encode(input: &str) -> String {
    // AWS-compliant URL encoding for query parameters
    // Based on RFC 3986 with AWS-specific requirements
    input.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            c => format!("%{:02X}", c as u8),
        })
        .collect()
}

fn get_current_timestamp_from_wasi() -> String {
    unsafe {
        match clock_time_get(CLOCKID_REALTIME, 0) {
            Ok(now_time) => {
                // Convert nanoseconds to seconds
                let seconds = now_time / 1_000_000_000;
                let timestamp = format_timestamp_from_unix(seconds);
                
                // Verify the timestamp is reasonable
                if seconds > 1577836800 && seconds < 1893456000 {
                    return timestamp;
                } else {
                    return timestamp; // Use it anyway, might be valid
                }
            }
            Err(_) => {
                // Return a reasonable fallback
                let fallback = "20250630T010000Z";
                fallback.to_string()
            }
        }
    }
}





fn format_timestamp_from_unix(unix_seconds: u64) -> String {
    // Convert Unix timestamp to AWS ISO 8601 format: YYYYMMDDTHHMMSSZ
    const SECONDS_PER_MINUTE: u64 = 60;
    const SECONDS_PER_HOUR: u64 = 3600;
    const SECONDS_PER_DAY: u64 = 86400;
    const DAYS_PER_YEAR: u64 = 365;
    const DAYS_PER_LEAP_YEAR: u64 = 366;
    
    // Unix epoch starts at 1970-01-01
    let mut remaining_seconds = unix_seconds;
    let mut year = 1970u64;
    
    // Calculate year
    loop {
        let days_in_year = if is_leap_year(year) { DAYS_PER_LEAP_YEAR } else { DAYS_PER_YEAR };
        let seconds_in_year = days_in_year * SECONDS_PER_DAY;
        
        if remaining_seconds >= seconds_in_year {
            remaining_seconds -= seconds_in_year;
            year += 1;
        } else {
            break;
        }
    }
    
    // Calculate day of year
    let remaining_days = remaining_seconds / SECONDS_PER_DAY;
    remaining_seconds %= SECONDS_PER_DAY;
    
    // Convert day of year to month and day
    let (month, day) = day_of_year_to_month_day(remaining_days + 1, is_leap_year(year));
    
    // Calculate hours, minutes, seconds
    let hours = remaining_seconds / SECONDS_PER_HOUR;
    remaining_seconds %= SECONDS_PER_HOUR;
    let minutes = remaining_seconds / SECONDS_PER_MINUTE;
    let seconds = remaining_seconds % SECONDS_PER_MINUTE;
    
    format!("{:04}{:02}{:02}T{:02}{:02}{:02}Z", year, month, day, hours, minutes, seconds)
}

fn is_leap_year(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn day_of_year_to_month_day(day_of_year: u64, is_leap: bool) -> (u64, u64) {
    let days_in_months = if is_leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    
    let mut remaining_days = day_of_year;
    for (month_index, &days_in_month) in days_in_months.iter().enumerate() {
        if remaining_days <= days_in_month {
            return (month_index as u64 + 1, remaining_days);
        }
        remaining_days -= days_in_month;
    }
    
    // Fallback (shouldn't happen with valid input)
    (12, 31)
}



fn create_canonical_request(host: &str, path: &str, query_string: &str, timestamp: &str, payload_hash: &str) -> String {
    let canonical_uri = path; // "/"
    
    // AWS requires query parameters to be sorted by parameter name
    // For S3 ListObjects, we need to properly format the query string
    let canonical_querystring = if query_string.is_empty() {
        "".to_string()
    } else {
        // Sort query parameters (though we only have one, this is good practice)
        let mut params: Vec<&str> = query_string.split('&').collect();
        params.sort();
        params.join("&")
    };
    
    let canonical_headers = format!("host:{}\nx-amz-content-sha256:{}\nx-amz-date:{}\n", host, payload_hash, timestamp);
    let signed_headers = "host;x-amz-content-sha256;x-amz-date";

    format!(
        "GET\n{}\n{}\n{}\n{}\n{}",
        canonical_uri, canonical_querystring, canonical_headers, signed_headers, payload_hash
    )
}

fn create_string_to_sign(timestamp: &str, region: &str, canonical_request: &str) -> String {
    let algorithm = "AWS4-HMAC-SHA256";
    let date = &timestamp[0..8];
    let credential_scope = format!("{}/{}/s3/aws4_request", date, region);
    let hashed_canonical_request = format!("{:x}", Sha256::digest(canonical_request.as_bytes()));

    format!(
        "{}\n{}\n{}\n{}",
        algorithm, timestamp, credential_scope, hashed_canonical_request
    )
}

fn calculate_signature(secret_key: &str, date: &str, region: &str, string_to_sign: &str) -> String {
    let k_secret = format!("AWS4{}", secret_key);
    let k_date = hmac_sha256(k_secret.as_bytes(), date.as_bytes());
    let k_region = hmac_sha256(&k_date, region.as_bytes());
    let k_service = hmac_sha256(&k_region, b"s3");
    let k_signing = hmac_sha256(&k_service, b"aws4_request");
    let signature = hmac_sha256(&k_signing, string_to_sign.as_bytes());
    
    hex::encode(signature)
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).unwrap();
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

fn parse_s3_response(xml: &str) -> Result<S3ListResponse, String> {
    // Simple XML parsing - in a real implementation you'd use a proper XML parser
    let mut objects = Vec::new();
    
    // Look for <Contents> elements
    let mut start = 0;
    while let Some(contents_start) = xml[start..].find("<Contents>") {
        let contents_start = start + contents_start;
        if let Some(contents_end) = xml[contents_start..].find("</Contents>") {
            let contents_end = contents_start + contents_end + 11;
            let contents_xml = &xml[contents_start..contents_end];
            
            // Extract key
            let key = extract_xml_value(contents_xml, "Key").unwrap_or_default();
            let last_modified = extract_xml_value(contents_xml, "LastModified").unwrap_or_default();
            let size = extract_xml_value(contents_xml, "Size")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let etag = extract_xml_value(contents_xml, "ETag").unwrap_or_default();
            
            objects.push(S3Object {
                key,
                last_modified,
                size,
                etag,
            });
            
            start = contents_end;
        } else {
            break;
        }
    }
    
    let is_truncated = xml.contains("<IsTruncated>true</IsTruncated>");
    let next_marker = extract_xml_value(xml, "NextMarker");
    
    Ok(S3ListResponse {
        objects,
        is_truncated,
        next_marker,
    })
}

fn extract_xml_value(xml: &str, tag: &str) -> Option<String> {
    let start_tag = format!("<{}>", tag);
    let end_tag = format!("</{}>", tag);
    
    if let Some(start) = xml.find(&start_tag) {
        let start = start + start_tag.len();
        if let Some(end) = xml[start..].find(&end_tag) {
            return Some(xml[start..start + end].to_string());
        }
    }
    None
}