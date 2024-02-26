use polyvalue::Value;
use std::collections::HashMap;
use std::net::ToSocketAddrs;
use std::str::FromStr;
use std::time::Duration;

use crate::Error;

pub fn resolve(hostname: &str) -> Result<Value, Error> {
    match (hostname, 0).to_socket_addrs() {
        Ok(mut addresses) => {
            let address = addresses.next().unwrap().to_string();
            let suffix = ":".to_string() + address.split(':').last().unwrap_or("80");

            Ok(Value::from(address.replace(&suffix, "")))
        }
        Err(e) => Err(e.into()),
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
) -> Result<Value, Error> {
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
                    Err(e) => Err(e.into()),
                },
                Err(e) => Err(e.into()),
            }
        }
        Err(e) => Err(e.into()),
    }
}
