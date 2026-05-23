use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "input")]
    Input { text: String },
    #[serde(rename = "app::list")]
    AppList,
    #[serde(rename = "app::launch")]
    AppLaunch { path: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Application {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "result")]
    Result { value: String },
    #[serde(rename = "app::list")]
    AppList { apps: Vec<Application> },
}

#[derive(Debug)]
pub struct IpcError {
    source: serde_json::Error,
}

impl fmt::Display for IpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid IPC message: {}", self.source)
    }
}

impl Error for IpcError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

impl From<serde_json::Error> for IpcError {
    fn from(source: serde_json::Error) -> Self {
        Self { source }
    }
}

pub fn decode_client_line(line: &str) -> Result<ClientMessage, IpcError> {
    serde_json::from_str::<ClientMessage>(line).map_err(Into::into)
}

pub fn encode_server_line(message: &ServerMessage) -> Result<String, IpcError> {
    let mut line = serde_json::to_string(message)?;
    line.push('\n');
    Ok(line)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn decodes_app_list_message() {
        let message = decode_client_line(r#"{"type":"app::list"}"#).unwrap();

        assert_eq!(message, ClientMessage::AppList);
    }

    #[test]
    fn decodes_app_launch_message() {
        let message = decode_client_line(
            r#"{"type":"app::launch","path":"/Applications/Safari.app"}"#,
        )
        .unwrap();

        assert_eq!(
            message,
            ClientMessage::AppLaunch {
                path: "/Applications/Safari.app".to_string()
            }
        );
    }

    #[test]
    fn rejects_unknown_client_message_type() {
        let error = decode_client_line(r#"{"type":"unknown","text":"1 + 2"}"#).unwrap_err();

        assert!(error.to_string().contains("invalid IPC message"));
    }

    #[test]
    fn rejects_malformed_json() {
        assert!(decode_client_line(r#"{"type":"input","text":"1 + 2"#).is_err());
    }

    #[test]
    fn encodes_app_list_message_as_json_line() {
        let line = encode_server_line(&ServerMessage::AppList {
            apps: vec![Application {
                name: "Safari".to_string(),
                path: "/Applications/Safari.app".to_string(),
            }],
        })
        .unwrap();

        assert!(line.ends_with('\n'));
        let value: serde_json::Value = serde_json::from_str(line.trim_end()).unwrap();
        assert_eq!(
            value,
            json!({
                "type": "app::list",
                "apps": [{ "name": "Safari", "path": "/Applications/Safari.app" }]
            })
        );
    }
}
