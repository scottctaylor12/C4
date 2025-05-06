use extism_pdk::{*};
use json;

////
// DEFINITIONS
////

#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Receive,
    Send,
    Custom(String), // For handling unknown/custom APIs
}

#[derive(serde::Serialize)]
pub struct Output {
    pub success: bool,
    pub message: String,
    pub tasks: Option<Vec<String>>,
}

#[derive(serde::Deserialize)]
pub struct Input {
    pub action: Action,
    pub params: json::Value,
}

#[derive(serde::Deserialize)]
pub struct ReceiveParams {
    pub api_key: String,
    pub id: String,
}

pub struct SendParams {
    pub api_key: String,
    pub id: String,
    pub message: String,
}

#[derive(serde::Deserialize)]
pub struct Gist {
    pub url: String,
    pub forks_url: String,
    pub commits_url: String,
    pub id: String,
    pub node_id: String,
    pub git_pull_url: String,
    pub git_push_url: String,
    pub html_url: String,
    pub files: json::Value,
    pub public: bool,
    pub created_at: String,
    pub updated_at: String,
    pub description: String,
    pub comments: i32,
    pub user: Option<String>,
    pub comments_url: String,
    pub owner: json::Value,
    pub truncated: bool,
}