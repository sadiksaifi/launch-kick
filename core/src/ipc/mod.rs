use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "app::list")]
    AppList,
    #[serde(rename = "app::launch")]
    AppLaunch { path: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Application {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "app::list")]
    AppList { apps: Vec<Application> },
    #[serde(rename = "app::launch::result")]
    AppLaunchResult {
        path: String,
        ok: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
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

    const CLIENT_APP_LIST: &str = include_str!("../../../ipc/fixtures/client-app-list.json");
    const CLIENT_APP_LAUNCH: &str = include_str!("../../../ipc/fixtures/client-app-launch.json");
    const SERVER_APP_LIST: &str = include_str!("../../../ipc/fixtures/server-app-list.json");
    const SERVER_APP_LAUNCH_SUCCEEDED: &str =
        include_str!("../../../ipc/fixtures/server-app-launch-succeeded.json");
    const SERVER_APP_LAUNCH_FAILED: &str =
        include_str!("../../../ipc/fixtures/server-app-launch-failed.json");

    #[test]
    fn decodes_client_app_list_fixture() {
        let message = decode_client_line(CLIENT_APP_LIST).unwrap();

        assert_eq!(message, ClientMessage::AppList);
    }

    #[test]
    fn decodes_client_app_launch_fixture() {
        let message = decode_client_line(CLIENT_APP_LAUNCH).unwrap();

        assert_eq!(
            message,
            ClientMessage::AppLaunch {
                path: "/Applications/Safari.app".to_string()
            }
        );
    }

    #[test]
    fn encodes_server_app_list_fixture() {
        let line = encode_server_line(&ServerMessage::AppList {
            apps: vec![Application {
                name: "Safari".to_string(),
                path: "/Applications/Safari.app".to_string(),
            }],
        })
        .unwrap();

        assert_json_line_eq(&line, SERVER_APP_LIST);
    }

    #[test]
    fn encodes_server_app_launch_success_fixture() {
        let line = encode_server_line(&ServerMessage::AppLaunchResult {
            path: "/Applications/Safari.app".to_string(),
            ok: true,
            error: None,
        })
        .unwrap();

        assert_json_line_eq(&line, SERVER_APP_LAUNCH_SUCCEEDED);
    }

    #[test]
    fn encodes_server_app_launch_failure_fixture() {
        let line = encode_server_line(&ServerMessage::AppLaunchResult {
            path: "/Applications/Missing.app".to_string(),
            ok: false,
            error: Some("launch failed".to_string()),
        })
        .unwrap();

        assert_json_line_eq(&line, SERVER_APP_LAUNCH_FAILED);
    }

    #[test]
    fn rejects_legacy_input_message() {
        let error = decode_client_line(r#"{"type":"input","text":"1 + 2"}"#).unwrap_err();

        assert!(error.to_string().contains("invalid IPC message"));
    }

    #[test]
    fn rejects_unknown_client_message_type() {
        let error = decode_client_line(r#"{"type":"unknown","text":"1 + 2"}"#).unwrap_err();

        assert!(error.to_string().contains("invalid IPC message"));
    }

    #[test]
    fn rejects_malformed_json() {
        assert!(
            decode_client_line(r#"{"type":"app::launch","path":"/Applications/Safari.app""#)
                .is_err()
        );
    }

    fn assert_json_line_eq(actual_line: &str, expected_json: &str) {
        assert!(actual_line.ends_with('\n'));
        let actual: serde_json::Value = serde_json::from_str(actual_line.trim_end()).unwrap();
        let expected: serde_json::Value = serde_json::from_str(expected_json).unwrap();
        assert_eq!(actual, expected);
    }
}
