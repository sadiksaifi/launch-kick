use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "launcher::query")]
    Query { query: String },
    #[serde(rename = "launcher::execute")]
    Execute {
        result_id: String,
        action_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct LauncherResult {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<IconDescriptor>,
    pub actions: Vec<LauncherAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct LauncherAction {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct IconDescriptor {
    pub kind: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "launcher::results")]
    Results {
        query: String,
        results: Vec<LauncherResult>,
    },
    #[serde(rename = "launcher::action::result")]
    ActionResult {
        result_id: String,
        action_id: String,
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

    const MANIFEST: &str = include_str!("../../../ipc/fixtures/manifest.json");

    #[test]
    fn manifest_drives_client_fixture_conformance() {
        for fixture_case in manifest().cases_for_direction("client_to_core") {
            let fixture = fixture_json(&fixture_case.file);
            let message = decode_client_line(&fixture).unwrap();
            let encoded = serde_json::to_string(&message).unwrap();

            assert_json_eq(&encoded, &fixture);
        }
    }

    #[test]
    fn manifest_drives_server_fixture_conformance() {
        for fixture_case in manifest().cases_for_direction("core_to_client") {
            let fixture = fixture_json(&fixture_case.file);
            let message: ServerMessage = serde_json::from_str(&fixture).unwrap();
            let encoded = encode_server_line(&message).unwrap();

            assert_json_line_eq(&encoded, &fixture);
        }
    }

    #[test]
    fn rejects_legacy_input_message() {
        let error = decode_client_line(r#"{"type":"input","text":"1 + 2"}"#).unwrap_err();

        assert!(error.to_string().contains("invalid IPC message"));
    }

    #[test]
    fn rejects_app_specific_client_message_type() {
        let error =
            decode_client_line(r#"{"type":"app::launch","path":"/Applications/Safari.app"}"#)
                .unwrap_err();

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
            decode_client_line(
                r#"{"type":"launcher::execute","result_id":"result","action_id":"open""#
            )
            .is_err()
        );
    }

    #[test]
    fn manifest_lists_existing_fixture_files() {
        for fixture_case in manifest().cases {
            let path = fixtures_dir().join(&fixture_case.file);
            assert!(path.exists(), "missing fixture file {}", fixture_case.file);
        }
    }

    #[test]
    fn every_json_fixture_file_is_listed_in_manifest() {
        let listed = manifest()
            .cases
            .into_iter()
            .map(|fixture_case| fixture_case.file)
            .collect::<std::collections::HashSet<_>>();

        for entry in std::fs::read_dir(fixtures_dir()).unwrap() {
            let path = entry.unwrap().path();
            let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if file_name == "manifest.json" || !file_name.ends_with(".json") {
                continue;
            }

            assert!(
                listed.contains(file_name),
                "unlisted fixture file {file_name}"
            );
        }
    }

    #[test]
    fn manifest_case_metadata_matches_fixture_types() {
        for fixture_case in manifest().cases {
            let json = fixture_json(&fixture_case.file);
            let value: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(
                value["type"].as_str(),
                Some(fixture_case.message_type.as_str())
            );
            assert!(
                matches!(
                    fixture_case.direction.as_str(),
                    "client_to_core" | "core_to_client"
                ),
                "unknown fixture direction {}",
                fixture_case.direction
            );
            assert!(!fixture_case.name.is_empty());
        }
    }

    fn assert_json_line_eq(actual_line: &str, expected_json: &str) {
        assert!(actual_line.ends_with('\n'));
        assert_json_eq(actual_line.trim_end(), expected_json);
    }

    fn assert_json_eq(actual_json: &str, expected_json: &str) {
        let actual: serde_json::Value = serde_json::from_str(actual_json).unwrap();
        let expected: serde_json::Value = serde_json::from_str(expected_json).unwrap();
        assert_eq!(actual, expected);
    }

    fn manifest() -> FixtureManifest {
        serde_json::from_str(MANIFEST).unwrap()
    }

    fn fixture_json(file: &str) -> String {
        std::fs::read_to_string(fixtures_dir().join(file)).unwrap()
    }

    fn fixtures_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../ipc/fixtures")
    }

    #[derive(serde::Deserialize)]
    struct FixtureManifest {
        cases: Vec<FixtureCase>,
    }

    impl FixtureManifest {
        fn cases_for_direction(&self, direction: &str) -> impl Iterator<Item = &FixtureCase> {
            self.cases
                .iter()
                .filter(move |fixture_case| fixture_case.direction == direction)
        }
    }

    #[derive(serde::Deserialize)]
    struct FixtureCase {
        name: String,
        direction: String,
        #[serde(rename = "type")]
        message_type: String,
        file: String,
    }
}
