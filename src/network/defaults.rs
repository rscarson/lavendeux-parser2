use super::ApiDefinition;
use std::collections::HashMap;

pub fn default_apis() -> HashMap<String, ApiDefinition> {
    let mut apis = HashMap::new();

    apis.insert(
        "httpbin".to_string(),
        ApiDefinition {
            base_url: "https://httpbin.org".to_string(),
            description: "A simple HTTP Request & Response Service.".to_string(),
            examples: "https://httpbin.org".to_string(),
            ..Default::default()
        },
    );

    apis.insert(
        "ipify".to_string(),
        ApiDefinition {
            base_url: "https://api.ipify.org".to_string(),
            description: "A simple public IP address API.".to_string(),
            examples: "https://api.ipify.org".to_string(),
            ..Default::default()
        },
    );

    apis.insert(
        "ipinfo".to_string(),
        ApiDefinition {
            base_url: "https://ipinfo.io".to_string(),
            description: "Find out your public and private IP addresses.".to_string(),
            examples: "https://ipinfo.io".to_string(),
            ..Default::default()
        },
    );

    apis.insert(
        "ipapi".to_string(),
        ApiDefinition {
            base_url: "https://ipapi.co".to_string(),
            description: "IP address location API and geolocation service.".to_string(),
            examples: "https://ipapi.co".to_string(),
            ..Default::default()
        },
    );

    apis.insert(
        "chatgpt".to_string(),
        ApiDefinition {
            base_url: "https://api.openai.com/v1/chat/completions".to_string(),
            description: "Chat with GPT-3.5".to_string(),
            examples: "chatgpt('hello world')".to_string(),
            additional_headers: vec![("Content-Type".to_string(), "application/json".to_string())]
                .into_iter()
                .collect(),
            ..Default::default()
        },
    );

    apis
}
