use super::request;
use crate::{
    error::{ErrorDetails, WrapOption},
    Error,
};
use polyvalue::{types::Object, Value, ValueTrait, ValueType};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct ApiDefinition {
    pub base_url: String,
    pub additional_headers: HashMap<String, String>,
    pub description: String,
    pub examples: String,
    pub auth_key: Option<String>,
}

impl ApiDefinition {
    pub fn call(
        &self,
        endpoint: Option<&str>,
        body: Option<String>,
        mut headers: HashMap<String, String>,
    ) -> Result<Value, Error> {
        let endpoint = endpoint.unwrap_or_default().trim_start_matches("/");
        let target = format!("{}/{}", &self.base_url, endpoint);
        if let Some(auth_key) = &self.auth_key {
            headers.insert("Authorization".to_string(), format!("Bearer {}", auth_key));
        }

        request(&target, body, headers)
    }
}

impl TryFrom<Value> for ApiDefinition {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Error> {
        let value = if value.is_a(ValueType::String) {
            Object::try_from(vec![(
                Value::from("base_url"),
                Value::from(value.to_string()),
            )])?
        } else {
            value.as_a::<Object>()?
        };

        let mut base_url =
        value
            .get(&Value::from("base_url"))
            .or_error(ErrorDetails::ValueFormat {
                expected_format: format!("<base_url: string> | {{<base_url: string>, <description: string>, <examples: string>, <auth_key: string>, <headers: object>}}"),
            })?.to_string();

        base_url = base_url.trim_end_matches('/').to_string();

        Ok(Self {
            base_url,

            description: value
                .get(&("description".into()))
                .unwrap_or(&Value::from(""))
                .to_string(),
            examples: value
                .get(&("examples".into()))
                .unwrap_or(&Value::from(""))
                .to_string(),

            auth_key: value
                .get(&("auth_key".into()))
                .and_then(|v| Some(v.to_string())),

            additional_headers: value
                .get(&("additional_headers".into()))
                .unwrap_or(&Value::from(Object::new(Default::default())))
                .clone()
                .as_a::<Object>()?
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        })
    }
}

impl Into<Value> for ApiDefinition {
    fn into(self) -> Value {
        let mut obj = Object::new(Default::default());
        obj.insert("base_url".into(), Value::from(self.base_url))
            .ok();
        obj.insert("description".into(), Value::from(self.description))
            .ok();
        obj.insert("examples".into(), Value::from(self.examples))
            .ok();

        if let Some(auth_key) = self.auth_key {
            obj.insert("auth_key".into(), Value::from(auth_key)).ok();
        }

        obj.insert(
            "additional_headers".into(),
            Value::try_from(
                self.additional_headers
                    .iter()
                    .map(|(k, v)| (Value::from(k.as_str()), Value::from(v.as_str())))
                    .collect::<Vec<(_, _)>>(),
            )
            .unwrap(),
        )
        .ok();
        Value::from(obj)
    }
}