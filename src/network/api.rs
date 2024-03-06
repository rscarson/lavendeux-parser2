use super::request;
use crate::{error::ErrorDetails, Error};
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
        let endpoint = endpoint.unwrap_or_default().trim_start_matches('/');
        let target = format!("{}/{}", &self.base_url, endpoint);
        if let Some(auth_key) = &self.auth_key {
            headers.insert("Authorization".to_string(), format!("Bearer {}", auth_key));
        }

        request(&target, body, headers)
    }
}

impl TryFrom<Value> for ApiDefinition {
    type Error = ErrorDetails;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
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
            .ok_or(ErrorDetails::ValueFormat {
                expected_format: "<base_url: string> | {<base_url: string>, <description: string>, <examples: string>, <auth_key: string>, <headers: object>}".to_string(),
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
                .get(&("auth_key".into())).map(|v| v.to_string()),

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

impl From<ApiDefinition> for Value {
    fn from(val: ApiDefinition) -> Self {
        let mut obj = Object::new(Default::default());
        obj.insert("base_url".into(), Value::from(val.base_url))
            .ok();
        obj.insert("description".into(), Value::from(val.description))
            .ok();
        obj.insert("examples".into(), Value::from(val.examples))
            .ok();

        if let Some(auth_key) = val.auth_key {
            obj.insert("auth_key".into(), Value::from(auth_key)).ok();
        }

        obj.insert(
            "additional_headers".into(),
            Value::try_from(
                val.additional_headers
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
