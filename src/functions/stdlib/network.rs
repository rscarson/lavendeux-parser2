use crate::{
    define_stdfunction,
    error::{ErrorDetails, WrapExternalError, WrapOption},
    functions::std_function::ParserFunction,
    network::{request, resolve, ApiDefinition, ApiRegistry},
    State,
};
use polyvalue::{types::Object, Value};
use serde_json::json;
use std::collections::HashMap;

/**********************************************
 *
 * Network IO
 *
 *********************************************/

define_stdfunction!(
    resolve {
        hostname: Standard::String
    },
    returns = String,
    docs = {
        category: "Network",
        description: "Resolves a hostname to an IP address",
        ext_description: "
            This function uses the system's DNS resolver to resolve a hostname to an IP address.
            If the hostname cannot be resolved, this function will return an error, or time out
        ",
        examples: "#skip
            resolve('example.com')
        "
    },
    handler = (state) {
        let hostname = state.get_variable("hostname").unwrap().to_string();
        Ok(resolve(&hostname).unwrap())
    }
);

define_stdfunction!(
    get {
        url: Standard::String,
        headers: Optional::Object
    },
    returns = String,
    docs = {
        category: "Network",
        description: "Performs an HTTP GET request",
        ext_description: "
            This function performs an HTTP GET request to the specified URL.
            If the request fails, this function will return an error or time out
        ",
        examples: "#skip
            str_out = get('https://jsonplaceholder.typicode.com/users')
            obj_out = get('https://jsonplaceholder.typicode.com/users', {
                'Content-Type': 'application/json'
            })
            assert(str_out is string)
            assert(obj_out is array)
        "
    },
    handler = (state) {
        let url = state.get_variable("url").unwrap().to_string();
        let headers = state.get_variable("headers").unwrap_or(Value::from(Object::default())).as_a::<Object>()?;
        let headers = headers.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect::<HashMap<_, _>>();
        request(&url, None, headers).without_context()
    }
);

define_stdfunction!(
    post {
        url: Standard::String,
        body: Standard::String,
        headers: Optional::Object
    },
    returns = String,
    docs = {
        category: "Network",
        description: "Performs an HTTP POST request",
        ext_description: "
            This function performs an HTTP POST request to the specified URL.
            If the request fails, this function will return an error or time out
        ",
        examples: "#skip
            obj_out = post(
                'https://jsonplaceholder.typicode.com/users', 
                '{\"name\": \"John Doe\"}',
                {'Content-Type': 'application/json'}
            )
        "
    },
    handler = (state) {
        let url = state.get_variable("url").unwrap().to_string();
        let body = state.get_variable("body").unwrap().to_string();
        let headers = state.get_variable("headers").unwrap_or(Value::from(Object::default())).as_a::<Object>()?;
        let headers = headers.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect::<HashMap<_, _>>();
        request(&url, Some(body), headers).without_context()
    }
);

/**********************************************
 *
 * API Registry
 *
 *********************************************/

define_stdfunction!(
    api_add {
        name: Standard::String,
        endpoint: Standard::Any
    },
    returns = String,
    docs = {
        category: "API",
        description: "Registers an API",
        ext_description: "
            This function registers an API with the system. The API can then be used to make requests to the specified endpoint.
            The endpoint can be a string, or an object with the properties [ base_url, headers, description, examples, auth_key]
            Use the 'api_get' and 'api_post' functions to make requests to the registered API
        ",
        examples: "
            api_add('ipify', 'https://api.ipify.org')
            assert( api_list() contains 'ipify' )
        "
    },
    handler = (state) {
        let name = state.get_variable("name").unwrap().to_string();
        let endpoint = state.get_variable("endpoint").unwrap();

        let api = ApiDefinition::try_from(endpoint)?;

        ApiRegistry::new(state).add(state, &name, api);
        Ok(Value::from(name))
    }
);

define_stdfunction!(
    api_rem {name: Standard::String},
    returns = String,
    docs = {
        category: "API",
        description: "Unregisters an API",
        ext_description: "
            This function unregisters an API with the system, and returns its name
            The API can no longer be used to make requests
        ",
        examples: "
            api_rem('ipify')
            assert( !(api_list() contains 'ipify') )
        "
    },
    handler = (state) {
        let name = state.get_variable("name").unwrap().to_string();
        ApiRegistry::new(state).remove(state, &name);
        Ok(Value::from(name))
    }
);

define_stdfunction!(
    api_all {},
    returns = Object,
    docs = {
        category: "API",
        description: "Details all registered APIs",
        ext_description: "
            This function returns an object containing the names and endpoints of all registered APIs
        ",
        examples: "
            api_all()['chatgpt']['base_url']
        "
    },
    handler = (state) {
        Ok(ApiRegistry::raw(state))
    }
);

define_stdfunction!(
    api_list {},
    returns = Object,
    docs = {
        category: "API",
        description: "Lists all registered APIs",
        ext_description: "
            This function returns an array containing the names of all registered APIs
        ",
        examples: "
            assert( api_list() contains 'chatgpt' )
        "
    },
    handler = (state) {
        Ok(ApiRegistry::new(state).all().keys().cloned().map(Value::from).collect::<Vec<_>>().into())
    }
);

define_stdfunction!(
    api_get {
        name: Standard::String,
        path: Optional::String
    },
    returns = String,
    docs = {
        category: "API",
        description: "Performs a GET request to a registered API",
        ext_description: "
            This function performs a GET request to the specified path of a registered API.
            The path is appended to the base URL of the API.
        ",
        examples: "#skip
            api_get('ipify')
            api_get('ipify', '/?format=json')
        "
    },
    handler = (state) {
        let name = state.get_variable("name").unwrap().to_string();
        let path = state.get_variable("path").map(|v| v.to_string());

        let registry = ApiRegistry::new(state);
        let api = registry.get(&name).or_error(ErrorDetails::Custom {
            msg: format!("API '{}' not found", name),
        })?;

        api.call(path.as_deref(), None, Default::default())
    }
);

define_stdfunction!(
    api_post {
        name: Standard::String,
        body: Standard::String,
        path: Optional::String
    },
    returns = String,
    docs = {
        category: "API",
        description: "Performs a POST request to a registered API",
        ext_description: "
            This function performs a POST request to the specified path of a registered API.
            The path is appended to the base URL of the API.
        ",
        examples: "#skip
            api_post('ipify', '{\"name\"=\"john\"}', 'format=json')
        "
    },
    handler = (state) {
        let name = state.get_variable("name").unwrap().to_string();
        let path = state.get_variable("path").map(|v| v.to_string());
        let body = state.get_variable("body").unwrap().to_string();

        let registry = ApiRegistry::new(state);
        let api = registry.get(&name).or_error(ErrorDetails::Custom {
            msg: format!("API '{}' not found", name),
        })?;

        api.call(path.as_deref(), Some(body), Default::default())
    }
);

define_stdfunction!(
    api_key {
        name: Standard::String,
        auth_key: Standard::String
    },
    returns = String,
    docs = {
        category: "API",
        description: "Sets an authentication key for a registered API",
        ext_description: "
            This function sets an authentication key for a registered API.
            The key will be used in the 'Authorization' header of requests to the API.
        ",
        examples: "
            api_key('chatgpt', 'my_super_secret_api_key')
            assert_eq( api_all()['chatgpt']['auth_key'], 'my_super_secret_api_key' )
        "
    },
    handler = (state) {
        let name = state.get_variable("name").unwrap().to_string();
        let auth_key = state.get_variable("auth_key").unwrap().to_string();

        let mut registry = ApiRegistry::new(state);
        let mut api = registry.get(&name).or_error(ErrorDetails::Custom {
            msg: format!("API '{}' not found", name),
        })?.clone();

        api.auth_key = Some(auth_key);
        registry.add(state, &name, api);
        Ok(Value::from(name))
    }
);

define_stdfunction!(
    chatgpt {
        prompt: Standard::String
    },
    returns = String,
    docs = {
        category: "API",
        description: "Performs a request to the ChatGPT API",
        ext_description: "
            This function performs a request to the ChatGPT 3.5 API, using the specified prompt.
        ",
        examples: "#skip
            api_key('chatgpt', 'my_super_secret_api_key')
            chatgpt('What is the meaning of life?')
        "
    },
    handler = (state) {
        let prompt = state.get_variable("prompt").unwrap().to_string();
        let registry = ApiRegistry::new(state);
        let api = registry.get("chatgpt").or_error(ErrorDetails::Custom {
            msg: "API 'chatgpt' not found".to_string(),
        })?;

        if api.auth_key.is_none() {
            return oops!(ValueFormat {
                expected_format: "API key for chatgpt is not set. You can set one with api_key('chatgpt', '<key>')".to_string()
            });
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
                    content: prompt,
                },
            ],
        };
        let query = serde_json::to_string(&query)?;

        let result = api
            .call(Some(&query), None, Default::default())?.to_string();

        let json = json!(result);
        let result = json["choices"][0]["message"]["content"].clone();

        Ok(Value::from(result.to_string()))
    }
);
