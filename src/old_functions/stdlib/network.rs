use crate::{
    error::{ErrorDetails, WrapExternalError},
    get_argument, get_optional_argument, oops, optional_argument, required_argument,
    static_function,
    std_functions::Function,
    State,
};
use polyvalue::{
    types::{Object, Str},
    Value, ValueTrait, ValueType,
};
use serde_json::json;
use std::collections::HashMap;

pub fn register_all(map: &mut HashMap<String, Function>) {
    #[cfg(feature = "network-functions")]
    static_function!(
        registry = map,
        name = "resolve",
        description = "Resolves a hostname to an IP address",
        category = "network",
        arguments = [required_argument!("hostname", ValueType::String)],
        returns = ValueType::String,
        handler = |_: &mut State, arguments, token, _| {
            let hostname = get_argument!("hostname", arguments)
                .as_a::<Str>()
                .with_context(token)?
                .inner()
                .clone();
            crate::network_utils::resolve(&hostname).with_context(token)
        }
    );

    #[cfg(feature = "network-functions")]
    static_function!(
        registry = map,
        name = "get",
        description = "Performs an HTTP GET request",
        category = "network",
        arguments = [
            required_argument!("url", ValueType::String),
            optional_argument!("headers", ValueType::Object)
        ],
        returns = ValueType::String,
        handler = |_: &mut State, arguments, token, _| {
            let url = get_argument!("url", arguments)
                .as_a::<Str>()
                .with_context(token)?
                .inner()
                .clone();
            let headers = get_argument!("headers", arguments)
                .as_a::<Object>()
                .with_context(token)?
                .inner()
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone().as_a::<Str>().unwrap().inner().clone(),
                        v.clone().as_a::<Str>().unwrap().inner().clone(),
                    )
                })
                .collect::<HashMap<String, String>>();
            crate::network_utils::request(&url, None, headers).with_context(token)
        }
    );

    #[cfg(feature = "network-functions")]
    static_function!(
        registry = map,
        name = "post",
        description = "Performs an HTTP POST request",
        category = "network",
        arguments = [
            required_argument!("url", ValueType::String),
            required_argument!("body", ValueType::String),
            optional_argument!("headers", ValueType::Object)
        ],
        returns = ValueType::String,
        handler = |_: &mut State, arguments, token, _| {
            let url = get_argument!("url", arguments)
                .as_a::<Str>()
                .with_context(token)?
                .inner()
                .clone();
            let body = get_argument!("body", arguments)
                .as_a::<Str>()
                .with_context(token)?
                .inner()
                .clone();
            let headers = get_argument!("headers", arguments)
                .as_a::<Object>()
                .with_context(token)?
                .inner()
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone().as_a::<Str>().unwrap().inner().clone(),
                        v.clone().as_a::<Str>().unwrap().inner().clone(),
                    )
                })
                .collect::<HashMap<String, String>>();
            crate::network_utils::request(&url, Some(body), headers).with_context(token)
        }
    );

    #[cfg(feature = "network-functions")]
    static_function!(
        registry = map,
        name = "add_api",
        description = "Registers an API. Accepts a string, or an object with the properties [ url, headers, description, examples, auth_key]",
        category = "network",
        arguments = [
            required_argument!("name", ValueType::String),
            required_argument!("endpoint", ValueType::Any)
        ],
        returns = ValueType::String,
        handler = |state: &mut State, arguments, token, _| {
            let name = get_argument!("name", arguments)
                .as_a::<Str>().with_context(token)?
                .inner()
                .clone();
            let endpoint = get_argument!("endpoint", arguments);

            crate::network_utils::ApiManager::set(
                state,
                &name,
                crate::network_utils::ApiDefinition::from_value(endpoint).ok_or(ErrorDetails::ValueFormat {
                    expected_format: "<url: string> | {<url: string>, <description: string>, <examples: string>, <auth_key: string>, <headers: object>}".to_string(),
                }).with_context(token)?,
            ).with_context(token)?;
            Ok(Value::from(name))
        }
    );

    #[cfg(feature = "network-functions")]
    static_function!(
        registry = map,
        name = "del_api",
        description = "Unregisters an API",
        category = "network",
        arguments = [required_argument!("name", ValueType::String)],
        returns = ValueType::String,
        handler = |state: &mut State, arguments, token, _| {
            let name = get_argument!("name", arguments)
                .as_a::<Str>()
                .with_context(token)?
                .inner()
                .clone();
            crate::network_utils::ApiManager::delete(state, &name).with_context(token)?;
            Ok(Value::from(name))
        }
    );

    #[cfg(feature = "network-functions")]
    static_function!(
        registry = map,
        name = "list_api",
        description = "Returns a list of registered APIs",
        category = "network",
        arguments = [],
        returns = ValueType::Object,
        handler = |state: &mut State, _arguments, _token, _| {
            let apis = crate::network_utils::ApiManager::list(state)?;
            let apis = apis
                .iter()
                .map(|a| Value::from(a.as_str()))
                .collect::<Vec<_>>();
            Ok(Value::from(apis))
        }
    );

    #[cfg(feature = "network-functions")]
    static_function!(
        registry = map,
        name = "get_api",
        description = "Calls an API endpoint using GET",
        category = "network",
        arguments = [
            required_argument!("name", ValueType::String),
            optional_argument!("endpoint", ValueType::String)
        ],
        returns = ValueType::String,
        handler = |state: &mut State, arguments, token, _| {
            let name = get_argument!("name", arguments)
                .as_a::<Str>()
                .with_context(token)?
                .inner()
                .clone();
            let endpoint = get_optional_argument!("endpoint", arguments)
                .and_then(|v| v.as_a::<Str>().ok())
                .map(|s| s.inner().clone());

            let api = crate::network_utils::ApiManager::get(state, &name)
                .ok_or(ErrorDetails::ValueFormat {
                expected_format: format!(
                    "requested API is not defined. You can set it with add_api('{name}', 'endpoint')"
                ),
            }).with_context(token)?;

            let result = api
                .call(endpoint.as_deref(), None, Default::default())
                .with_context(token)?;
            Ok(result)
        }
    );

    #[cfg(feature = "network-functions")]
    static_function!(
        registry = map,
        name = "post_api",
        description = "Calls an API endpoint using POST",
        category = "network",
        arguments = [
            required_argument!("name", ValueType::String),
            optional_argument!("endpoint", ValueType::String),
            optional_argument!("body", ValueType::String)
        ],
        returns = ValueType::String,
        handler = |state: &mut State, arguments, token, _| {
            let name = get_argument!("name", arguments)
                .as_a::<Str>()
                .with_context(token)?
                .inner()
                .clone();
            let endpoint = get_optional_argument!("endpoint", arguments)
                .and_then(|v| v.as_a::<Str>().ok())
                .map(|s| s.inner().clone());
            let body = get_optional_argument!("body", arguments)
                .unwrap_or(Str::default().into())
                .as_a::<Str>()
                .with_context(token)?
                .inner()
                .clone();
            let api = crate::network_utils::ApiManager::get(state, &name)
                .ok_or(ErrorDetails::ValueFormat {
                expected_format: format!(
                    "requested API is not defined. You can set it with add_api('{name}', 'endpoint')"
                ),
            }).with_context(token)?;

            let result = api
                .call(endpoint.as_deref(), Some(body), Default::default())
                .with_context(token)?;
            Ok(result)
        }
    );

    // Uses network_utils::ApiManager::add_key_for
    #[cfg(feature = "network-functions")]
    static_function!(
        registry = map,
        name = "api_key",
        description = "Adds an API key for an API",
        category = "network",
        arguments = [
            required_argument!("name", ValueType::String),
            required_argument!("key", ValueType::String)
        ],
        returns = ValueType::String,
        handler = |state: &mut State, arguments, token, _| {
            let name = get_argument!("name", arguments)
                .as_a::<Str>()
                .with_context(token)?
                .inner()
                .clone();
            let key = get_argument!("key", arguments)
                .as_a::<Str>()
                .with_context(token)?
                .inner()
                .clone();
            crate::network_utils::ApiManager::add_key_for(state, &name, &key);
            Ok(Value::from(name))
        }
    );

    #[cfg(feature = "network-functions")]
    static_function!(
        registry = map,
        name = "chatgpt",
        description = "Calls the ChatGPT API",
        category = "network",
        arguments = [required_argument!("query", ValueType::String)],
        returns = ValueType::String,
        handler = |state: &mut State, arguments, token, _| {
            let query = get_argument!("query", arguments)
                .as_a::<Str>()
                .with_context(token)?
                .inner()
                .clone();
            let api = crate::network_utils::ApiManager::get(state, "chatgpt")
                .ok_or(ErrorDetails::ValueFormat {
                expected_format:
                    "API chatgpt is not defined. You can set it with add_api('chatgpt', 'endpoint')"
                        .to_string(),
            }).with_context(token)?;
            if api.auth_key.is_none() {
                return oops!(ValueFormat {
                    expected_format: "API key for chatgpt is not set. You can set one with api_key('chatgpt', '<key>')".to_string()
                }, token.clone());
            }

            use serde::{Deserialize, Serialize};
            #[derive(Serialize, Deserialize)]
            struct GPTMsg {
                role: String,
                content: String,
            }
            #[derive(Serialize, Deserialize)]
            struct GPTQuery {
                model: String,
                messages: Vec<GPTMsg>,
            }

            let query = GPTQuery {
                model: "gpt-3.5-turbo".to_string(),
                messages: vec![
                    GPTMsg {
                        role: "system".to_string(),
                        content:
                            "You are a chatbot that must respond in concise, single-line messages."
                                .to_string(),
                    },
                    GPTMsg {
                        role: "user".to_string(),
                        content: query,
                    },
                ],
            };
            let query = serde_json::to_string(&query).with_context(token)?;

            let result = api
                .call(Some(&query), None, Default::default())
                .with_context(token)?
                .as_a::<Str>()
                .with_context(token)?
                .inner()
                .clone();

            let json = json!(result);
            let result = json["choices"][0]["message"]["content"].clone();

            Ok(Value::from(result.to_string()))
        }
    );
}
