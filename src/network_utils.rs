use polyvalue::types::{Object, ObjectInner, Str};
use polyvalue::{Value, ValueTrait, ValueType};
use std::collections::HashMap;
use std::net::ToSocketAddrs;
use std::str::FromStr;
use std::time::Duration;

/// Resolve a hostname to an IP address
///
/// # Arguments
/// * `hostname` - Host to resolve
pub fn resolve(hostname: &str) -> Result<Value, std::io::Error> {
    match (hostname, 0).to_socket_addrs() {
        Ok(mut addresses) => {
            let address = addresses.next().unwrap().to_string();
            let suffix = ":".to_string() + address.split(':').last().unwrap_or("80");

            Ok(Value::from(address.replace(&suffix, "")))
        }
        Err(e) => Err(e),
    }
}

fn decode_response(response: &str, headers: &HashMap<String, String>) -> Value {
    let json_decode = headers.get("Content-Type").cloned().unwrap_or_default()
        == "application/json"
        || headers.get("content-type").cloned().unwrap_or_default() == "application/json";
    if json_decode {
        if let Ok(v) = serde_json::Value::from_str(response) {
            if let Ok(v) = Value::try_from(v) {
                return v;
            }
        }
    }

    Value::from(response)
}

/// Fetch from a given URL
///
/// # Arguments
/// * `url` - Target URL
/// * `body` - Body if POST
/// * `headers` - Array of header=value strings
pub fn request(
    url: &str,
    body: Option<String>,
    headers: HashMap<String, String>,
) -> Result<Value, reqwest::Error> {
    match reqwest::blocking::Client::builder()
        .timeout(Duration::from_millis(1500))
        .build()
    {
        Ok(client) => {
            let mut request = match body {
                None => client.get(url),
                Some(s) => client.post(url).body(s),
            };

            for (header, value) in headers.iter() {
                request = request.header(header, value);
            }

            match request.send() {
                Ok(res) => match res.text() {
                    Ok(s) => {
                        let value = decode_response(&s, &headers);
                        Ok(value)
                    }
                    Err(e) => Err(e),
                },
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

#[derive(Debug, Clone)]
pub struct ApiDefinition {
    pub base_url: String,
    pub additional_headers: HashMap<String, String>,
    pub description: String,
    pub examples: String,
    pub auth_key: Option<String>,
}

impl ApiDefinition {
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
    pub fn additional_headers(&self) -> &HashMap<String, String> {
        &self.additional_headers
    }
    pub fn description(&self) -> &str {
        &self.description
    }
    pub fn examples(&self) -> &str {
        &self.examples
    }
    pub fn auth_key(&self) -> &Option<String> {
        &self.auth_key
    }

    pub fn to_polyvalue(&self) -> Result<Value, crate::error::ExternalError> {
        let mut value = Object::try_from(vec![
            (Value::from("url"), Value::from(self.base_url())),
            (
                Value::from("headers"),
                Value::try_from(
                    self.additional_headers()
                        .iter()
                        .map(|(k, v)| (Value::from(k.as_str()), Value::from(v.as_str())))
                        .collect::<Vec<(Value, Value)>>(),
                )?,
            ),
            (Value::from("description"), Value::from(self.description())),
            (Value::from("examples"), Value::from(self.examples())),
        ])?;

        if let Some(auth_key) = self.auth_key() {
            value.insert(Value::from("auth_key"), Value::from(auth_key.as_str()))?;
        }

        Ok(value.into())
    }

    pub fn from_value(value: Value) -> Option<Self> {
        if let Some(mut obj) = value.if_is_a::<Object>(ValueType::Object) {
            let base_url = obj
                .remove(&Value::from("url"))
                .unwrap_or(Value::from(""))
                .as_a::<Str>()
                .ok()?
                .to_string();
            let description = obj
                .remove(&Value::from("description"))
                .unwrap_or(Value::from(""))
                .as_a::<Str>()
                .ok()?
                .to_string();
            let examples = obj
                .remove(&Value::from("examples"))
                .unwrap_or(Value::from(""))
                .as_a::<Str>()
                .ok()?
                .to_string();
            let auth_key = obj
                .remove(&Value::from("auth_key"))
                .map(|v| v.as_a::<Str>().unwrap().to_string());

            let additional_headers = obj
                .remove(&Value::from("headers"))
                .unwrap_or(Value::from(ObjectInner::new()))
                .as_a::<Object>()
                .ok()?
                .inner()
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone().as_a::<Str>().unwrap().to_string(),
                        v.clone().as_a::<Str>().unwrap().to_string(),
                    )
                })
                .collect::<HashMap<String, String>>();

            Some(ApiDefinition {
                base_url,
                additional_headers,
                description,
                examples,
                auth_key,
            })
        } else {
            Some(ApiDefinition {
                base_url: value.to_string(),
                additional_headers: HashMap::new(),
                description: "".to_string(),
                examples: "".to_string(),
                auth_key: None,
            })
        }
    }

    pub fn call(
        &self,
        endpoint: Option<&str>,
        body: Option<String>,
        mut headers: HashMap<String, String>,
    ) -> Result<Value, reqwest::Error> {
        let target = format!("{}{}", self.base_url(), endpoint.unwrap_or_default());
        if let Some(auth_key) = &self.auth_key {
            headers.insert("Authorization".to_string(), format!("Bearer {}", auth_key));
        }

        request(&target, body, headers)
    }
}

macro_rules! define_api {
    ($state:expr, name = $name:literal, url = $url:literal, additional_headers = [$($key:literal:$value:literal),*], description = $description:literal, examples = $examples:literal) => {
        ApiManager::set(
            $state,
            $name,
            ApiDefinition {
                base_url: $url.to_string(),
                additional_headers: HashMap::from([$((
                    $key.to_string(),
                    $value.to_string(),
                )),*]),
                description: $description.to_string(),
                examples: $examples.to_string(),
                auth_key: None,
            },
        )
        .ok();
    };
}

/// Manager that stores API definitions within the program state
///
pub struct ApiManager();
impl ApiManager {
    const STORE_NAME: &'static str = "__api_definitions";
    pub fn retrieve_store(state: &crate::State) -> HashMap<String, ApiDefinition> {
        let store = state
            .get_variable(Self::STORE_NAME)
            .unwrap_or(Object::default().into())
            .as_a::<Object>()
            .unwrap()
            .inner()
            .clone();
        store
            .iter()
            .map(|(k, v)| {
                (
                    k.clone().as_a::<Str>().unwrap().to_string(),
                    ApiDefinition::from_value(v.clone()),
                )
            })
            .filter(|(_, v)| v.is_some())
            .map(|(k, v)| (k, v.unwrap()))
            .collect::<HashMap<String, ApiDefinition>>()
    }

    pub fn store(
        state: &mut crate::State,
        api_definitions: HashMap<String, ApiDefinition>,
    ) -> Result<(), crate::error::ExternalError> {
        let mut store = Object::default();
        for (k, v) in api_definitions.iter() {
            store.insert(Value::from(k.as_str()), v.to_polyvalue()?)?;
        }
        state.set_variable(Self::STORE_NAME, store.into());
        Ok(())
    }

    pub fn get(state: &crate::State, name: &str) -> Option<ApiDefinition> {
        Self::retrieve_store(state).get(name).cloned()
    }

    pub fn set(
        state: &mut crate::State,
        name: &str,
        api_definition: ApiDefinition,
    ) -> Result<(), crate::error::ExternalError> {
        let mut store = Self::retrieve_store(state);
        store.insert(name.to_string(), api_definition);
        Self::store(state, store)
    }

    pub fn delete(state: &mut crate::State, name: &str) -> Result<(), crate::error::ExternalError> {
        let mut store = Self::retrieve_store(state);
        store.remove(name);
        Self::store(state, store)
    }

    pub fn list(state: &crate::State) -> Result<Vec<String>, crate::Error> {
        Ok(Self::retrieve_store(state)
            .keys()
            .map(|k| k.to_string())
            .collect::<Vec<String>>())
    }

    pub fn add_key_for(state: &mut crate::State, name: &str, key: &str) {
        if let Some(mut api_definition) = Self::get(state, name) {
            api_definition.auth_key = Some(key.to_string());
            Self::set(state, name, api_definition).ok();
        }
    }

    pub fn default_apis(state: &mut crate::State) {
        define_api!(
            state,
            name = "httpbin",
            url = "https://httpbin.org",
            additional_headers = [],
            description = "A simple HTTP Request & Response Service.",
            examples = "https://httpbin.org"
        );

        define_api!(
            state,
            name = "ipify",
            url = "https://api.ipify.org",
            additional_headers = [],
            description = "A simple public IP address API.",
            examples = "https://api.ipify.org"
        );

        define_api!(
            state,
            name = "ipinfo",
            url = "https://ipinfo.io",
            additional_headers = [],
            description = "Find out your public and private IP addresses.",
            examples = "https://ipinfo.io"
        );

        define_api!(
            state,
            name = "ipapi",
            url = "https://ipapi.co",
            additional_headers = [],
            description = "IP address location API and geolocation service.",
            examples = "https://ipapi.co"
        );

        define_api!(
            state,
            name = "chatgpt",
            url = "https://api.openai.com/v1/chat/completions",
            additional_headers = [
                "Content-Type": "application/json"
            ],
            description = "Chat with GPT-3.5",
            examples = "chatgpt('hello world')"
        );
    }
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_api_definition() {
        let api = ApiDefinition {
            base_url: "https://example.com".to_string(),
            additional_headers: HashMap::from([("key".to_string(), "value".to_string())]),
            description: "A test API".to_string(),
            examples: "https://example.com".to_string(),
            auth_key: Some("test_key".to_string()),
        };

        let value = api.to_polyvalue().unwrap();
        let api2 = ApiDefinition::from_value(value).unwrap();

        assert_eq!(api.base_url, api2.base_url);
        assert_eq!(api.additional_headers, api2.additional_headers);
        assert_eq!(api.description, api2.description);
        assert_eq!(api.examples, api2.examples);
        assert_eq!(api.auth_key, api2.auth_key);

        let value = Value::from("https://example.com");
        let api2 = ApiDefinition::from_value(value).unwrap();
        assert_eq!(api2.base_url, "https://example.com");

        // mock server with mockito
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body("hello world")
            .create();

        let url = format!("http://{}", server.host_with_port());
        let result = ApiDefinition::from_value(url.into())
            .unwrap()
            .call(None, None, Default::default())
            .unwrap();
        assert_eq!(result.to_string(), "hello world");

        mock.assert();

        let mock = server
            .mock("GET", "/json")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body("{\"hello\": \"world\"}")
            .create();

        // create headers hashmap with content-type json
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let url = format!("http://{}", server.host_with_port());
        let result = ApiDefinition::from_value(url.into())
            .unwrap()
            .call(Some("/json"), None, headers)
            .unwrap()
            .as_a::<polyvalue::types::Object>()
            .unwrap();
        assert_eq!(result.get(&"hello".into()).unwrap().to_string(), "world");

        mock.assert();
    }
}
